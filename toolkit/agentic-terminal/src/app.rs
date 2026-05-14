//! Application state and main run loop (`run_app()`).

use std::collections::VecDeque;
use std::error::Error;
use std::io::stdout;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use ratatui::backend::CrosstermBackend;
use ratatui::{layout::Rect, Terminal};

use crate::agent::AgentEvent;
use crate::helper_mode::HelperModeConfig;
use crate::navigator::ToolNavigator;
use crate::runtime::BackgroundRuntime;
use crate::state::SessionState;
use crate::term::TerminalGuard;
use crate::view::search::SearchView;
use crate::view::{ViewResult, ViewStack};

/// Routed together with mutable [`AppShared`] so overlays can coexist without conflicting borrows.
pub struct ViewCtx<'a> {
    pub shared: &'a mut AppShared,
    pub command_palette_visible: bool,
}

pub struct ResizeDebouncer {
    last_resize: Instant,
    debounce_duration: Duration,
    resize_pending: bool,
}

impl ResizeDebouncer {
    #[must_use]
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            last_resize: Instant::now(),
            debounce_duration: Duration::from_millis(debounce_ms),
            resize_pending: false,
        }
    }

    #[must_use]
    pub fn is_resize_pending(&self) -> bool {
        self.resize_pending
    }

    pub fn on_resize(&mut self) {
        self.resize_pending = true;
        self.last_resize = Instant::now();
    }

    pub fn drain_redraw(&mut self) -> bool {
        if self.resize_pending && self.last_resize.elapsed() >= self.debounce_duration {
            self.resize_pending = false;
            true
        } else {
            false
        }
    }
}

pub struct AppShared {
    pub search_query: String,
    pub messages: VecDeque<String>,
    pub list_height: u16,
    pub list_area: Rect,
    pub input_area: Rect,
    pub navigator: ToolNavigator,
    pub background: BackgroundRuntime,
    pub resize_debouncer: ResizeDebouncer,
    pub should_quit: bool,
    pub helper_mode: HelperModeConfig,
    pub session_state: SessionState,
}

impl AppShared {
    pub const MAX_MESSAGES: usize = 50;

    #[must_use]
    pub fn new(background: BackgroundRuntime) -> Self {
        let navigator = ToolNavigator::new(background.sync().tools_clone());
        Self {
            search_query: String::new(),
            messages: VecDeque::new(),
            list_height: 20,
            list_area: Rect::default(),
            input_area: Rect::default(),
            navigator,
            background,
            resize_debouncer: ResizeDebouncer::new(120),
            should_quit: false,
            helper_mode: HelperModeConfig::default(),
            session_state: SessionState::new(),
        }
    }

    #[must_use]
    pub fn with_helper_mode(background: BackgroundRuntime, helper_only: bool) -> Self {
        let navigator = ToolNavigator::new(background.sync().tools_clone());
        Self {
            search_query: String::new(),
            messages: VecDeque::new(),
            list_height: 20,
            list_area: Rect::default(),
            input_area: Rect::default(),
            navigator,
            background,
            resize_debouncer: ResizeDebouncer::new(120),
            should_quit: false,
            helper_mode: HelperModeConfig::new(helper_only),
            session_state: SessionState::new(),
        }
    }

    pub fn push_message<S: Into<String>>(&mut self, message: S) {
        self.messages.push_back(message.into());
        while self.messages.len() > Self::MAX_MESSAGES {
            let _ = self.messages.pop_front();
        }
    }
}

fn ingest_agent_mailbox(shared: &mut AppShared) -> bool {
    let mut redraw = false;
    while let Ok(ev) = shared.background.recv_event_try() {
        redraw = true;
        match ev {
            AgentEvent::Status { text, .. } => shared.push_message(format!("Agent: {text}")),
            AgentEvent::ManifestUpdated(manifest) => {
                let tools = manifest.tools.clone();
                shared.navigator = ToolNavigator::new(tools);
                shared.navigator.set_query(&shared.search_query);
                shared.push_message("Tool metadata refreshed");
            }
            AgentEvent::Verification(report) => {
                shared.push_message(format!(
                    "Verification: root={} verified={} links={}",
                    report.root.hex(),
                    report.verified,
                    report.chain.len(),
                ));
            }
            AgentEvent::Error { text, agent } => {
                shared.push_message(format!("{}: {text}", agent.0));
            }
        }
    }
    redraw
}

/// Entry-point for interactive mode: alternate screen terminal + stacked views.
///
/// Caller must invoke [`crate::term::install_panic_hook`] before terminal setup.
pub fn run_app() -> Result<(), Box<dyn Error + Send + Sync>> {
    run_app_with_config(HelperModeConfig::default())
}

/// Entry-point with explicit helper mode configuration.
pub fn run_app_with_config(config: HelperModeConfig) -> Result<(), Box<dyn Error + Send + Sync>> {
    let _guard = TerminalGuard::enter()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let background = BackgroundRuntime::new();
    let mut shared = AppShared::with_helper_mode(background, config.helper_only);
    
    if config.helper_only {
        shared.push_message("Helper mode: navigation only (no agent commands).");
    } else {
        shared.push_message("Ready. / opens the command palette.");
    }

    let mut stack = ViewStack::with_base(Box::<SearchView>::default());
    let mut dirty = true;

    while !shared.should_quit {
        if ingest_agent_mailbox(&mut shared) {
            dirty = true;
        }

        shared.navigator.tick_frame();

        let fast_poll =
            dirty || shared.resize_debouncer.is_resize_pending();

        if event::poll(if fast_poll {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(100)
        })? {
            let ev = event::read()?;
            if let Event::Resize(_, _) = ev {
                shared.resize_debouncer.on_resize();
            }

            let overlay = stack.has_overlay_top();
            let mut view_ctx = ViewCtx {
                shared: &mut shared,
                command_palette_visible: overlay,
            };

            let outcome = match stack.top_mut() {
                Some(v) => v.on_event(&ev, &mut view_ctx),
                None => ViewResult::Consumed,
            };

            match outcome {
                ViewResult::Quit => shared.should_quit = true,
                other => stack.apply_outcome(other),
            }
            dirty = true;
        }

        if shared.resize_debouncer.drain_redraw() {
            dirty = true;
        }

        if dirty {
            let overlay = stack.has_overlay_top();
            let mut view_ctx = ViewCtx {
                shared: &mut shared,
                command_palette_visible: overlay,
            };
            terminal.draw(|frame| {
                stack.render_all(frame.area(), frame, &mut view_ctx);
            })?;
            dirty = false;
        }
    }

    Ok(())
}
