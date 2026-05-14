use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, MouseEventKind},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, DisableMouseCapture, EnableMouseCapture,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use nucleo::{Config, Matcher, Utf32String};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::{
    collections::VecDeque,
    io,
    time::{Duration, Instant},
};

// ==================== Types ====================

#[derive(Clone, Debug)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub examples: Vec<String>,
    pub package: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    Search,
    Agent,
}

#[derive(Debug)]
pub enum AgentCommand {
    RefreshMetadata,
    VerifyBootChain,
    SearchTools(String),
    Help,
    Quit,
}

#[derive(Debug)]
pub enum BackgroundMessage {
    ToolsUpdated(Vec<Tool>),
    AgentResponse(String),
    VerificationResult(String),
}

pub struct AppState {
    pub mode: InputMode,
    pub search_query: String,
    pub agent_input: String,
    pub list_height: u16,
    pub list_area: Rect,
    pub messages: VecDeque<String>,
    pub should_quit: bool,
}

impl AppState {
    const MAX_MESSAGES: usize = 20;

    fn push_message<S: Into<String>>(&mut self, message: S) {
        self.messages.push_back(message.into());
        while self.messages.len() > Self::MAX_MESSAGES {
            let _ = self.messages.pop_front();
        }
    }
}

pub struct ToolNavigator {
    pub tools: Vec<Tool>,
    pub filtered: Vec<usize>,
    pub scores: Vec<u32>,
    pub matcher: Matcher,
    pub list_state: ListState,
    haystacks: Vec<Utf32String>,
}

impl ToolNavigator {
    pub fn new(tools: Vec<Tool>) -> Self {
        let config = Config::DEFAULT.ignore_case(true);
        let mut nav = Self {
            haystacks: tools
                .iter()
                .map(|t| Utf32String::from(t.name.as_str()))
                .collect(),
            tools,
            filtered: vec![],
            scores: vec![],
            matcher: Matcher::new(config),
            list_state: ListState::default(),
        };
        nav.update_filter();
        nav
    }

    pub fn set_query(&mut self, query: &str) {
        if query.trim().is_empty() {
            self.update_filter();
            return;
        }

        let query_str = Utf32String::from(query);
        let matches = self
            .matcher
            .match_list(self.haystacks.iter().cloned(), &query_str);

        self.filtered = matches.iter().map(|(idx, _)| *idx).collect();
        self.scores = matches.iter().map(|(_, score)| *score).collect();

        if !self.filtered.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    pub fn selected_tool(&self) -> Option<&Tool> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered.get(i))
            .map(|&idx| &self.tools[idx])
    }

    pub fn next(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % self.filtered.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn page_up(&mut self, page_size: usize) {
        if self.filtered.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new = current.saturating_sub(page_size);
        self.list_state.select(Some(new));
    }

    pub fn page_down(&mut self, page_size: usize) {
        if self.filtered.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let max = self.filtered.len().saturating_sub(1);
        let new = (current + page_size).min(max);
        self.list_state.select(Some(new));
    }

    pub fn selected_score(&self) -> Option<u32> {
        self.list_state
            .selected()
            .and_then(|i| self.scores.get(i))
            .copied()
    }

    fn update_filter(&mut self) {
        self.filtered = (0..self.tools.len()).collect();
        self.scores = vec![0; self.tools.len()];
        if !self.filtered.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }
}

pub struct ResizeDebouncer {
    last_resize: Instant,
    debounce_duration: Duration,
    resize_pending: bool,
}

impl ResizeDebouncer {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            last_resize: Instant::now(),
            debounce_duration: Duration::from_millis(debounce_ms),
            resize_pending: false,
        }
    }

    pub fn on_resize(&mut self) {
        self.resize_pending = true;
        self.last_resize = Instant::now();
    }

    pub fn should_redraw(&mut self) -> bool {
        if self.resize_pending && self.last_resize.elapsed() >= self.debounce_duration {
            self.resize_pending = false;
            true
        } else {
            false
        }
    }
}

pub struct BackgroundRuntime {
    cmd_tx: crossbeam::channel::Sender<AgentCommand>,
    result_rx: crossbeam::channel::Receiver<BackgroundMessage>,
}

impl BackgroundRuntime {
    pub fn new() -> Self {
        use crossbeam::channel;
        let (cmd_tx, cmd_rx) = channel::bounded::<AgentCommand>(128);
        let (result_tx, result_rx) = channel::bounded::<BackgroundMessage>(128);

        std::thread::spawn(move || {
            while let Ok(cmd) = cmd_rx.recv() {
                match cmd {
                    AgentCommand::RefreshMetadata => {
                        let _ = result_tx.send(BackgroundMessage::ToolsUpdated(default_tools()));
                    }
                    AgentCommand::VerifyBootChain => {
                        let _ = result_tx.send(BackgroundMessage::VerificationResult(
                            "Boot chain verification placeholder: chain=unverified".into(),
                        ));
                    }
                    AgentCommand::SearchTools(query) => {
                        let _ = result_tx.send(BackgroundMessage::AgentResponse(format!(
                            "Agent search accepted: {}",
                            query
                        )));
                    }
                    AgentCommand::Help => {
                        let _ = result_tx.send(BackgroundMessage::AgentResponse(
                            "Commands: refresh | verify | search <term> | help | quit".into(),
                        ));
                    }
                    AgentCommand::Quit => break,
                }
            }
        });

        Self { cmd_tx, result_rx }
    }

    pub fn send_command(&self, cmd: AgentCommand) -> Result<(), String> {
        self.cmd_tx
            .send(cmd)
            .map_err(|_| "Background runtime is unavailable".to_string())
    }

    pub fn try_recv(&self) -> Option<BackgroundMessage> {
        self.result_rx.try_recv().ok()
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}

// ==================== Helpers ====================

pub fn default_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "ripgrep".into(),
            description: "Ultra-fast text search tool".into(),
            examples: vec!["rg pattern .".into(), "rg -i pattern".into()],
            package: Some("ripgrep".into()),
        },
        Tool {
            name: "fd".into(),
            description: "Fast alternative to find".into(),
            examples: vec!["fd pattern".into(), "fd -e rs".into()],
            package: Some("fd".into()),
        },
        Tool {
            name: "bat".into(),
            description: "Cat clone with syntax highlighting".into(),
            examples: vec!["bat Cargo.toml".into(), "bat -n src/main.rs".into()],
            package: Some("bat".into()),
        },
        Tool {
            name: "jq".into(),
            description: "Command-line JSON processor".into(),
            examples: vec![
                "jq '.items[]' file.json".into(),
                "cat x.json | jq '.a.b'".into(),
            ],
            package: Some("jq".into()),
        },
    ]
}

pub fn parse_agent_command(input: &str) -> Result<AgentCommand, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Empty command".into());
    }

    let mut parts = trimmed.split_whitespace();
    let command = parts
        .next()
        .expect("split_whitespace() yields at least one part")
        .to_ascii_lowercase();

    match command.as_str() {
        "refresh" => Ok(AgentCommand::RefreshMetadata),
        "verify" => Ok(AgentCommand::VerifyBootChain),
        "search" => {
            let query = parts.collect::<Vec<_>>().join(" ");
            if query.is_empty() {
                Err("Usage: search <term>".into())
            } else {
                Ok(AgentCommand::SearchTools(query))
            }
        }
        "help" | "?" => Ok(AgentCommand::Help),
        "quit" | "exit" => Ok(AgentCommand::Quit),
        _ => Err(format!("Unknown command: {}", trimmed)),
    }
}

fn point_in_rect(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

fn inner_rect(rect: Rect) -> Rect {
    if rect.width <= 2 || rect.height <= 2 {
        return Rect::default();
    }
    Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: rect.width - 2,
        height: rect.height - 2,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState {
        mode: InputMode::Search,
        search_query: String::new(),
        agent_input: String::new(),
        list_height: 20,
        list_area: Rect::default(),
        messages: VecDeque::new(),
        should_quit: false,
    };
    app.push_message("Ready. / enters agent mode.");

    let mut navigator = ToolNavigator::new(default_tools());
    let background = BackgroundRuntime::new();
    let mut resize_debouncer = ResizeDebouncer::new(120);
    let mut dirty = true;

    while !app.should_quit {
        if dirty {
            terminal.draw(|f| {
                let size = f.size();

                let vertical = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(8),
                        Constraint::Length(4),
                    ])
                    .split(size);

                let horizontal = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
                    .split(vertical[1]);

                app.list_height = horizontal[0].height;
                app.list_area = horizontal[0];

                let input_title = match app.mode {
                    InputMode::Search => "Search mode ( / or ? => agent )",
                    InputMode::Agent => "Agent mode (Esc to cancel)",
                };
                let input_text = match app.mode {
                    InputMode::Search => format!("Search: {}", app.search_query),
                    InputMode::Agent => format!("> {}", app.agent_input),
                };
                let input = Paragraph::new(input_text)
                    .block(Block::default().borders(Borders::ALL).title(input_title));
                f.render_widget(input, vertical[0]);

                let items: Vec<ListItem> = navigator
                    .filtered
                    .iter()
                    .enumerate()
                    .map(|(pos, &tool_idx)| {
                        let score = navigator.scores.get(pos).copied().unwrap_or(0);
                        let tool = &navigator.tools[tool_idx];
                        let line = if app.search_query.trim().is_empty() {
                            tool.name.clone()
                        } else {
                            format!("{:<24} score={}", tool.name, score)
                        };
                        ListItem::new(line)
                    })
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("Tools"))
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
                f.render_stateful_widget(list, horizontal[0], &mut navigator.list_state);

                let preview_text = if let Some(tool) = navigator.selected_tool() {
                    let package = tool.package.as_deref().unwrap_or("n/a");
                    let selected_score = navigator.selected_score().unwrap_or(0);
                    format!(
                        "{}\n\nPackage: {}\nScore: {}\n\nExamples:\n{}",
                        tool.description,
                        package,
                        selected_score,
                        tool.examples.join("\n")
                    )
                } else {
                    "No selection".to_string()
                };
                let preview = Paragraph::new(preview_text)
                    .block(Block::default().borders(Borders::ALL).title("Preview"));
                f.render_widget(preview, horizontal[1]);

                let latest = app
                    .messages
                    .back()
                    .cloned()
                    .unwrap_or_else(|| "No messages".to_string());
                let status = Paragraph::new(format!(
                    "q=quit | ↑↓ PgUp/PgDn | wheel/click | Enter=open\n{}",
                    latest
                ))
                .block(Block::default().borders(Borders::ALL).title("Status"));
                f.render_widget(status, vertical[2]);
            })?;
            dirty = false;
        }

        if event::poll(Duration::from_millis(30))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match app.mode {
                    InputMode::Search => match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('/') | KeyCode::Char('?') => {
                            app.mode = InputMode::Agent;
                            app.agent_input.clear();
                            dirty = true;
                        }
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            navigator.set_query(&app.search_query);
                            dirty = true;
                        }
                        KeyCode::Backspace => {
                            let _ = app.search_query.pop();
                            navigator.set_query(&app.search_query);
                            dirty = true;
                        }
                        KeyCode::Up => {
                            navigator.previous();
                            dirty = true;
                        }
                        KeyCode::Down => {
                            navigator.next();
                            dirty = true;
                        }
                        KeyCode::PageUp => {
                            let size = (app.list_height as usize).saturating_sub(2).max(5);
                            navigator.page_up(size);
                            dirty = true;
                        }
                        KeyCode::PageDown => {
                            let size = (app.list_height as usize).saturating_sub(2).max(5);
                            navigator.page_down(size);
                            dirty = true;
                        }
                        KeyCode::Enter => {
                            if let Some(tool) = navigator.selected_tool() {
                                let tool_name = tool.name.clone();
                                if let Err(err) = background
                                    .send_command(AgentCommand::SearchTools(tool_name.clone()))
                                {
                                    app.push_message(err);
                                } else {
                                    app.push_message(format!(
                                        "Submitted tool lookup: {}",
                                        tool_name
                                    ));
                                }
                            }
                            dirty = true;
                        }
                        _ => {}
                    },
                    InputMode::Agent => match key.code {
                        KeyCode::Esc => {
                            app.mode = InputMode::Search;
                            app.agent_input.clear();
                            dirty = true;
                        }
                        KeyCode::Enter => {
                            if !app.agent_input.trim().is_empty() {
                                match parse_agent_command(&app.agent_input) {
                                    Ok(AgentCommand::Quit) => {
                                        let _ = background.send_command(AgentCommand::Quit);
                                        app.should_quit = true;
                                    }
                                    Ok(cmd) => {
                                        if let Err(err) = background.send_command(cmd) {
                                            app.push_message(err);
                                        } else {
                                            app.push_message("Agent command submitted");
                                        }
                                    }
                                    Err(err) => app.push_message(err),
                                }
                                app.agent_input.clear();
                            }
                            app.mode = InputMode::Search;
                            dirty = true;
                        }
                        KeyCode::Char(c) => {
                            app.agent_input.push(c);
                            dirty = true;
                        }
                        KeyCode::Backspace => {
                            let _ = app.agent_input.pop();
                            dirty = true;
                        }
                        _ => {}
                    },
                },
                Event::Mouse(mouse) => {
                    if app.mode == InputMode::Search {
                        match mouse.kind {
                            MouseEventKind::ScrollUp => {
                                navigator.previous();
                                dirty = true;
                            }
                            MouseEventKind::ScrollDown => {
                                navigator.next();
                                dirty = true;
                            }
                            MouseEventKind::Down(_) => {
                                let content_area = inner_rect(app.list_area);
                                if point_in_rect(content_area, mouse.column, mouse.row) {
                                    let clicked_row = (mouse.row - content_area.y) as usize;
                                    let list_offset = navigator.list_state.offset();
                                    let index = list_offset.saturating_add(clicked_row);
                                    if index < navigator.filtered.len() {
                                        navigator.list_state.select(Some(index));
                                        dirty = true;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::Resize(_, _) => {
                    resize_debouncer.on_resize();
                }
                _ => {}
            }
        }

        if resize_debouncer.should_redraw() {
            dirty = true;
        }

        while let Some(msg) = background.try_recv() {
            match msg {
                BackgroundMessage::ToolsUpdated(tools) => {
                    navigator = ToolNavigator::new(tools);
                    navigator.set_query(&app.search_query);
                    app.push_message("Tool metadata refreshed");
                }
                BackgroundMessage::AgentResponse(text) => {
                    app.push_message(format!("Agent: {}", text))
                }
                BackgroundMessage::VerificationResult(result) => {
                    app.push_message(format!("Verification: {}", result))
                }
            }
            dirty = true;
        }
    }

    Ok(())
}
