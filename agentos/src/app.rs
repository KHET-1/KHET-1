use std::cell::Cell;
use std::io::{self, stdout};
use std::time::{Duration, Instant};

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Position, Rect};
use ratatui::widgets::ListState;
use ratatui::Terminal;

use crate::agent_harness::{AgentOutcome, AgentRegistry};
use crate::events::{AppEvent, EventLog};
use crate::model::{AppLayer, InputMode, ToolId};
use crate::tools::builtin_tools;
use crate::ui;
use crate::worker::{spawn_filter_worker, FilterJob, FilterResult};

const FILTER_DEBOUNCE: Duration = Duration::from_millis(45);
const RESIZE_DEBOUNCE: Duration = Duration::from_millis(55);

#[derive(Clone, Copy)]
pub struct LayoutCache {
    /// Inner list area (excluding borders) for mouse hit-testing.
    pub list: Rect,
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;

    let mut app = App::new();
    app.terminal_size = terminal.size()?;

    let res = app_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

fn app_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    let tick = Duration::from_millis(32);

    while !app.should_quit {
        // Apply debounced resize
        if let Some((w, h)) = app.resize_pending {
            if app.last_resize.elapsed() >= RESIZE_DEBOUNCE {
                app.terminal_size = ratatui::layout::Size {
                    width: w,
                    height: h,
                };
                terminal.resize(Rect::new(0, 0, w, h))?;
                app.resize_pending = None;
            }
        }

        // Debounced filter flush
        if app.needs_filter_refresh && app.last_input_change.elapsed() >= FILTER_DEBOUNCE {
            app.flush_filter_request();
        }

        // Drain worker results (non-blocking)
        while let Ok(result) = app.worker_rx.try_recv() {
            app.on_filter_result(result);
        }

        let layout_cell: Cell<Option<LayoutCache>> = Cell::new(None);
        terminal.draw(|f| {
            let layout = ui::draw(
                f,
                &app.tools,
                &app.filtered_ids,
                &mut app.list_state,
                &app.query,
                &app.agent_buffer,
                app.mode,
                app.last_agent_message.as_deref(),
                app.event_log.tail(5),
            );
            layout_cell.set(Some(layout));
        })?;
        app.layout = layout_cell.get();

        if event::poll(tick)? {
            match event::read()? {
                Event::Key(key) => app.on_key(key)?,
                Event::Mouse(m) => app.on_mouse(m),
                Event::Resize(w, h) => {
                    app.resize_pending = Some((w, h));
                    app.last_resize = Instant::now();
                }
                _ => {}
            }
        }
    }

    Ok(())
}

pub struct App {
    pub tools: Vec<crate::model::Tool>,
    pub match_lines: Vec<(ToolId, String)>,
    pub filtered_ids: Vec<ToolId>,
    pub selected_id: Option<ToolId>,
    pub list_state: ListState,
    pub query: String,
    pub agent_buffer: String,
    pub mode: InputMode,
    pub layer: AppLayer,
    pub worker_tx: crossbeam_channel::Sender<FilterJob>,
    pub worker_rx: crossbeam_channel::Receiver<FilterResult>,
    /// Monotonic id for the last filter job sent to the worker.
    pub latest_sent_epoch: u64,
    pub needs_filter_refresh: bool,
    pub last_input_change: Instant,
    pub terminal_size: ratatui::layout::Size,
    pub resize_pending: Option<(u16, u16)>,
    pub last_resize: Instant,
    pub event_log: EventLog,
    pub agents: AgentRegistry,
    pub last_agent_message: Option<String>,
    pub should_quit: bool,
    pub layout: Option<LayoutCache>,
}

impl App {
    pub fn new() -> Self {
        let tools = builtin_tools();
        let match_lines: Vec<_> = tools.iter().map(|t| (t.id, t.fuzzy_line())).collect();
        let worker = spawn_filter_worker();
        let initial_ids: Vec<_> = tools.iter().map(|t| t.id).collect();

        let mut list_state = ListState::default();
        if !initial_ids.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            tools,
            match_lines,
            filtered_ids: initial_ids.clone(),
            selected_id: initial_ids.first().copied(),
            list_state,
            query: String::new(),
            agent_buffer: String::new(),
            mode: InputMode::Search,
            layer: AppLayer::Navigator,
            worker_tx: worker.tx,
            worker_rx: worker.rx,
            latest_sent_epoch: 0,
            needs_filter_refresh: false,
            last_input_change: Instant::now(),
            terminal_size: ratatui::layout::Size {
                width: 80,
                height: 24,
            },
            resize_pending: None,
            last_resize: Instant::now(),
            event_log: EventLog::new(512),
            agents: AgentRegistry::with_builtin_agents(),
            last_agent_message: Some(
                "foundation build — worker filter + event log + stable ids".into(),
            ),
            should_quit: false,
            layout: None,
        }
    }

    fn sync_layer_with_mode(&mut self) {
        let new_layer = match self.mode {
            InputMode::Search => AppLayer::Navigator,
            InputMode::Agent => AppLayer::AgentPrompt,
        };
        if new_layer != self.layer {
            let from = layer_name(self.layer);
            let to = layer_name(new_layer);
            self.event_log.push(AppEvent::LayerChanged { from, to });
            self.layer = new_layer;
        }
    }

    fn set_mode(&mut self, to: InputMode) {
        if self.mode != to {
            self.event_log.push(AppEvent::ModeChanged {
                from: self.mode,
                to,
            });
            self.mode = to;
            self.sync_layer_with_mode();
        }
    }

    fn flush_filter_request(&mut self) {
        self.needs_filter_refresh = false;

        if self.query.is_empty() {
            self.filtered_ids = self.tools.iter().map(|t| t.id).collect();
            self.reconcile_selection_after_filter();
            return;
        }

        self.latest_sent_epoch = self.latest_sent_epoch.saturating_add(1);
        let job = FilterJob {
            epoch: self.latest_sent_epoch,
            query: self.query.clone(),
            items: self.match_lines.clone(),
        };

        if self.worker_tx.try_send(job).is_err() {
            self.event_log.push(AppEvent::FilterDispatchBackpressured);
        }
    }

    fn on_filter_result(&mut self, result: FilterResult) {
        if result.epoch != self.latest_sent_epoch {
            self.event_log.push(AppEvent::StaleWorkerResultDropped {
                epoch: result.epoch,
                current_epoch: self.latest_sent_epoch,
            });
            return;
        }

        self.filtered_ids = result.ordered_ids;
        self.reconcile_selection_after_filter();
    }

    fn reconcile_selection_after_filter(&mut self) {
        if self.filtered_ids.is_empty() {
            self.selected_id = None;
            self.list_state.select(None);
            self.event_log.push(AppEvent::SelectionChanged(None));
            return;
        }

        let pos = self
            .selected_id
            .and_then(|id| self.filtered_ids.iter().position(|x| *x == id))
            .unwrap_or(0);

        let new_id = self.filtered_ids[pos];
        if self.selected_id != Some(new_id) {
            self.event_log
                .push(AppEvent::SelectionChanged(Some(new_id)));
        }
        self.selected_id = Some(new_id);
        self.list_state.select(Some(pos));

        // Keep scroll offset sane
        let max_off = self.filtered_ids.len().saturating_sub(1);
        if *self.list_state.offset_mut() > max_off {
            *self.list_state.offset_mut() = max_off;
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.event_log.push(AppEvent::Quit);
                self.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('q') if self.mode == InputMode::Search && self.query.is_empty() => {
                self.event_log.push(AppEvent::Quit);
                self.should_quit = true;
                return Ok(());
            }
            _ => {}
        }

        match self.layer {
            AppLayer::AgentPrompt => self.on_key_agent(key),
            AppLayer::Navigator => match self.mode {
                InputMode::Search => self.on_key_search(key),
                InputMode::Agent => self.on_key_agent(key),
            },
        }
        Ok(())
    }

    fn on_key_search(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                self.set_mode(InputMode::Agent);
            }
            KeyCode::Esc if !self.query.is_empty() => {
                self.query.clear();
                self.event_log.push(AppEvent::QueryChanged(String::new()));
                self.needs_filter_refresh = true;
                self.last_input_change = Instant::now();
            }
            KeyCode::Backspace if !self.query.is_empty() => {
                self.query.pop();
                self.event_log
                    .push(AppEvent::QueryChanged(self.query.clone()));
                self.needs_filter_refresh = true;
                self.last_input_change = Instant::now();
            }
            KeyCode::Down => self.move_selection(1),
            KeyCode::Up => self.move_selection(-1),
            KeyCode::PageDown => self.page_selection(1),
            KeyCode::PageUp => self.page_selection(-1),
            KeyCode::Enter => {
                if let Some(id) = self.selected_id {
                    if let Some(t) = self.tools.iter().find(|t| t.id == id) {
                        self.last_agent_message = Some(format!(
                            "picked {} (exec wiring intentionally absent)",
                            t.name
                        ));
                    }
                }
            }
            KeyCode::Char(c) => {
                self.query.push(c);
                self.event_log
                    .push(AppEvent::QueryChanged(self.query.clone()));
                self.needs_filter_refresh = true;
                self.last_input_change = Instant::now();
            }
            _ => {}
        }
    }

    fn on_key_agent(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                self.set_mode(InputMode::Search);
            }
            KeyCode::Esc => {
                self.set_mode(InputMode::Search);
                self.agent_buffer.clear();
            }
            KeyCode::Enter => {
                let line = std::mem::take(&mut self.agent_buffer);
                self.event_log
                    .push(AppEvent::AgentLineSubmitted(line.clone()));
                let out = self.agents.handle_line(&line);
                self.last_agent_message = match out {
                    AgentOutcome::Message(s) => Some(s),
                    AgentOutcome::Noop => self.last_agent_message.clone(),
                };
            }
            KeyCode::Char(c) => {
                self.agent_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.agent_buffer.pop();
            }
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: isize) {
        let Some(sel) = self.list_state.selected() else {
            return;
        };
        let len = self.filtered_ids.len() as isize;
        if len == 0 {
            return;
        }
        let new = (sel as isize + delta).clamp(0, len - 1) as usize;
        self.list_state.select(Some(new));
        if let Some(id) = self.filtered_ids.get(new) {
            self.selected_id = Some(*id);
            self.event_log.push(AppEvent::SelectionChanged(Some(*id)));
        }
    }

    fn page_selection(&mut self, dir: isize) {
        let page = visible_list_height(self).max(1) as isize;
        self.move_selection(dir * page);
    }

    fn on_mouse(&mut self, m: MouseEvent) {
        let Some(layout) = self.layout else {
            return;
        };

        match m.kind {
            MouseEventKind::ScrollDown => {
                let page = 3;
                let max = self.filtered_ids.len().saturating_sub(1);
                *self.list_state.offset_mut() = (self.list_state.offset() + page).min(max);
            }
            MouseEventKind::ScrollUp => {
                *self.list_state.offset_mut() = self.list_state.offset().saturating_sub(3);
            }
            MouseEventKind::Down(MouseButton::Left)
                if layout.list.contains(Position::new(m.column, m.row)) =>
            {
                let row = m.row.saturating_sub(layout.list.y) as usize;
                let idx = self.list_state.offset().saturating_add(row);
                if idx < self.filtered_ids.len() {
                    self.list_state.select(Some(idx));
                    if let Some(id) = self.filtered_ids.get(idx) {
                        self.selected_id = Some(*id);
                        self.event_log.push(AppEvent::SelectionChanged(Some(*id)));
                    }
                }
            }
            _ => {}
        }
    }
}

fn visible_list_height(app: &App) -> usize {
    app.terminal_size.height.saturating_sub(10).max(3) as usize
}

fn layer_name(l: AppLayer) -> &'static str {
    match l {
        AppLayer::Navigator => "navigator",
        AppLayer::AgentPrompt => "agent_prompt",
    }
}
