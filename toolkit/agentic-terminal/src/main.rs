use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nucleo::{
    pattern::{CaseMatching, Normalization},
    Config, Injector, Match, Matcher, Nucleo, Utf32String,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::{
    collections::{HashMap, VecDeque},
    fmt, io, panic,
    sync::{mpsc, Arc},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::runtime::Builder;
use tokio::sync::{mpsc as tokio_mpsc, oneshot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub examples: Vec<String>,
    pub package: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hash(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationReport {
    pub subject: String,
    pub root_hash: Option<Hash>,
    pub signature: Option<Signature>,
    pub passed: bool,
    pub details: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolManifest {
    pub schema_version: u32,
    pub generated_at_epoch_secs: u64,
    pub tool_count: usize,
    pub merkle_root: Hash,
}

impl ToolManifest {
    fn from_tools(tools: &[Tool]) -> Self {
        let generated_at_epoch_secs = now_epoch_secs();
        let merkle_root = Hash(format!("tools:{}:{}", tools.len(), generated_at_epoch_secs));
        Self {
            schema_version: 1,
            generated_at_epoch_secs,
            tool_count: tools.len(),
            merkle_root,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JournalEvent {
    CommandSubmitted {
        timestamp_epoch_secs: u64,
        agent_id: AgentId,
        raw_command: String,
        args: Vec<String>,
    },
    AgentCompleted {
        timestamp_epoch_secs: u64,
        agent_id: AgentId,
        summary: String,
    },
    VerificationProduced {
        timestamp_epoch_secs: u64,
        report: VerificationReport,
    },
    Note {
        timestamp_epoch_secs: u64,
        message: String,
    },
}

pub trait Journal {
    fn append(&mut self, event: JournalEvent);
    fn recent(&self, limit: usize) -> Vec<JournalEvent>;
}

#[derive(Default)]
pub struct MemoryJournal {
    events: Vec<JournalEvent>,
}

impl Journal for MemoryJournal {
    fn append(&mut self, event: JournalEvent) {
        self.events.push(event);
    }

    fn recent(&self, limit: usize) -> Vec<JournalEvent> {
        self.events
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
}

pub trait StateStore {
    fn load_manifest(&self) -> Option<ToolManifest>;
    fn save_manifest(&mut self, manifest: ToolManifest);
}

#[derive(Default)]
pub struct InMemoryStateStore {
    manifest: Option<ToolManifest>,
}

impl StateStore for InMemoryStateStore {
    fn load_manifest(&self) -> Option<ToolManifest> {
        self.manifest.clone()
    }

    fn save_manifest(&mut self, manifest: ToolManifest) {
        self.manifest = Some(manifest);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedCommand {
    pub command: String,
    pub args: Vec<String>,
}

pub fn parse_agent_command(input: &str) -> Result<ParsedCommand, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Empty command".into());
    }

    let mut parts = trimmed.split_whitespace();
    let command = parts
        .next()
        .expect("split_whitespace has at least one item after trim")
        .to_ascii_lowercase();
    let args = parts.map(str::to_string).collect::<Vec<_>>();

    Ok(ParsedCommand { command, args })
}

#[derive(Clone, Debug)]
pub struct AgentInvocation {
    pub agent_id: AgentId,
    pub args: Vec<String>,
    pub raw_command: String,
}

pub struct AgentExecutionContext<'a> {
    pub tools: &'a [Tool],
    pub manifest: &'a ToolManifest,
}

#[derive(Default)]
pub struct AgentOutput {
    pub messages: Vec<String>,
    pub tools_update: Option<Vec<Tool>>,
    pub verification_report: Option<VerificationReport>,
    pub request_quit: bool,
    pub journal_events: Vec<JournalEvent>,
}

pub trait Agent: Send + Sync {
    fn identity(&self) -> AgentId;
    fn aliases(&self) -> &'static [&'static str];
    fn execute(&self, invocation: &AgentInvocation, ctx: &AgentExecutionContext<'_>)
        -> AgentOutput;
}

#[derive(Clone, Default)]
pub struct AgentRegistry {
    agents: HashMap<AgentId, Arc<dyn Agent>>,
    alias_index: HashMap<String, AgentId>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_builtin_agents() -> Self {
        let mut registry = Self::new();
        registry.register(Arc::new(RefreshAgent));
        registry.register(Arc::new(VerifyAgent));
        registry.register(Arc::new(SearchAgent));
        registry.register(Arc::new(HelpAgent));
        registry.register(Arc::new(QuitAgent));
        registry
    }

    pub fn register(&mut self, agent: Arc<dyn Agent>) {
        let id = agent.identity();
        for alias in agent.aliases() {
            self.alias_index
                .insert(alias.to_ascii_lowercase(), id.clone());
        }
        self.alias_index
            .insert(id.0.to_ascii_lowercase(), id.clone());
        self.agents.insert(id, agent);
    }

    pub fn resolve_alias(&self, alias: &str) -> Option<AgentId> {
        self.alias_index.get(&alias.to_ascii_lowercase()).cloned()
    }

    pub fn get(&self, id: &AgentId) -> Option<Arc<dyn Agent>> {
        self.agents.get(id).cloned()
    }
}

struct RefreshAgent;
struct VerifyAgent;
struct SearchAgent;
struct HelpAgent;
struct QuitAgent;

impl Agent for RefreshAgent {
    fn identity(&self) -> AgentId {
        AgentId::new("refresh")
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["refresh"]
    }

    fn execute(
        &self,
        invocation: &AgentInvocation,
        _ctx: &AgentExecutionContext<'_>,
    ) -> AgentOutput {
        AgentOutput {
            messages: vec!["Tool metadata refreshed".into()],
            tools_update: Some(default_tools()),
            verification_report: None,
            request_quit: false,
            journal_events: vec![JournalEvent::AgentCompleted {
                timestamp_epoch_secs: now_epoch_secs(),
                agent_id: invocation.agent_id.clone(),
                summary: "refresh completed".into(),
            }],
        }
    }
}

impl Agent for VerifyAgent {
    fn identity(&self) -> AgentId {
        AgentId::new("verify")
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["verify", "attest"]
    }

    fn execute(
        &self,
        invocation: &AgentInvocation,
        ctx: &AgentExecutionContext<'_>,
    ) -> AgentOutput {
        let report = VerificationReport {
            subject: "boot-chain-and-manifest".into(),
            root_hash: Some(ctx.manifest.merkle_root.clone()),
            signature: Some(Signature("local-dev-signature-placeholder".into())),
            passed: true,
            details: format!(
                "placeholder verification complete for {} tools",
                ctx.tools.len()
            ),
        };
        AgentOutput {
            messages: vec!["verification completed".into()],
            tools_update: None,
            verification_report: Some(report.clone()),
            request_quit: false,
            journal_events: vec![
                JournalEvent::AgentCompleted {
                    timestamp_epoch_secs: now_epoch_secs(),
                    agent_id: invocation.agent_id.clone(),
                    summary: "verify completed".into(),
                },
                JournalEvent::VerificationProduced {
                    timestamp_epoch_secs: now_epoch_secs(),
                    report,
                },
            ],
        }
    }
}

impl Agent for SearchAgent {
    fn identity(&self) -> AgentId {
        AgentId::new("search")
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["search", "find"]
    }

    fn execute(
        &self,
        invocation: &AgentInvocation,
        ctx: &AgentExecutionContext<'_>,
    ) -> AgentOutput {
        let query = invocation.args.join(" ").to_ascii_lowercase();
        if query.is_empty() {
            return AgentOutput {
                messages: vec!["usage: search <term>".into()],
                tools_update: None,
                verification_report: None,
                request_quit: false,
                journal_events: vec![],
            };
        }

        let results = ctx
            .tools
            .iter()
            .filter(|tool| {
                tool.name.to_ascii_lowercase().contains(&query)
                    || tool.description.to_ascii_lowercase().contains(&query)
            })
            .map(|tool| tool.name.clone())
            .collect::<Vec<_>>();

        let summary = if results.is_empty() {
            format!("no agent-side tool matches for '{}'", query)
        } else {
            format!("agent-side matches for '{}': {}", query, results.join(", "))
        };

        AgentOutput {
            messages: vec![summary],
            tools_update: None,
            verification_report: None,
            request_quit: false,
            journal_events: vec![JournalEvent::AgentCompleted {
                timestamp_epoch_secs: now_epoch_secs(),
                agent_id: invocation.agent_id.clone(),
                summary: "search completed".into(),
            }],
        }
    }
}

impl Agent for HelpAgent {
    fn identity(&self) -> AgentId {
        AgentId::new("help")
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["help", "?"]
    }

    fn execute(
        &self,
        invocation: &AgentInvocation,
        _ctx: &AgentExecutionContext<'_>,
    ) -> AgentOutput {
        AgentOutput {
            messages: vec!["commands: refresh | verify | search <term> | help | quit".into()],
            tools_update: None,
            verification_report: None,
            request_quit: false,
            journal_events: vec![JournalEvent::AgentCompleted {
                timestamp_epoch_secs: now_epoch_secs(),
                agent_id: invocation.agent_id.clone(),
                summary: "help completed".into(),
            }],
        }
    }
}

impl Agent for QuitAgent {
    fn identity(&self) -> AgentId {
        AgentId::new("quit")
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["quit", "exit"]
    }

    fn execute(
        &self,
        invocation: &AgentInvocation,
        _ctx: &AgentExecutionContext<'_>,
    ) -> AgentOutput {
        AgentOutput {
            messages: vec!["quitting...".into()],
            tools_update: None,
            verification_report: None,
            request_quit: true,
            journal_events: vec![JournalEvent::AgentCompleted {
                timestamp_epoch_secs: now_epoch_secs(),
                agent_id: invocation.agent_id.clone(),
                summary: "quit requested".into(),
            }],
        }
    }
}

#[derive(Debug)]
pub enum BackgroundMessage {
    ToolsUpdated(Vec<Tool>),
    AgentResponse(String),
    VerificationResult(VerificationReport),
    QuitRequested,
    Journal(JournalEvent),
}

#[derive(Debug)]
pub enum WorkRequest {
    Invoke(AgentInvocation),
}

#[derive(Debug)]
pub enum ControlRequest {
    Ping,
}

pub struct BackgroundRuntime {
    work_tx: tokio_mpsc::Sender<WorkRequest>,
    control_tx: tokio_mpsc::Sender<ControlRequest>,
    result_rx: mpsc::Receiver<BackgroundMessage>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    worker_join: Option<thread::JoinHandle<()>>,
}

impl BackgroundRuntime {
    pub fn new(initial_tools: Vec<Tool>, registry: AgentRegistry) -> Self {
        let (work_tx, mut work_rx) = tokio_mpsc::channel::<WorkRequest>(256);
        let (control_tx, mut control_rx) = tokio_mpsc::channel::<ControlRequest>(64);
        let (result_tx, result_rx) = mpsc::channel::<BackgroundMessage>();
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        let worker_join = thread::spawn(move || {
            let runtime = Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to build tokio runtime");
            runtime.block_on(async move {
                let mut tools = initial_tools;
                let mut manifest = ToolManifest::from_tools(&tools);

                loop {
                    tokio::select! {
                        _ = &mut shutdown_rx => {
                            break;
                        }
                        Some(control) = control_rx.recv() => match control {
                            ControlRequest::Ping => {
                                let _ = result_tx.send(BackgroundMessage::AgentResponse("runtime:ok".into()));
                            }
                        },
                        Some(work) = work_rx.recv() => {
                            match work {
                                WorkRequest::Invoke(invocation) => {
                                    let Some(agent) = registry.get(&invocation.agent_id) else {
                                        let _ = result_tx.send(BackgroundMessage::AgentResponse(format!(
                                            "agent not registered: {}",
                                            invocation.agent_id
                                        )));
                                        continue;
                                    };

                                    let ctx = AgentExecutionContext {
                                        tools: &tools,
                                        manifest: &manifest,
                                    };
                                    let output = agent.execute(&invocation, &ctx);

                                    for journal_event in output.journal_events {
                                        let _ = result_tx.send(BackgroundMessage::Journal(journal_event));
                                    }
                                    for message in output.messages {
                                        let _ = result_tx.send(BackgroundMessage::AgentResponse(message));
                                    }
                                    if let Some(report) = output.verification_report {
                                        let _ = result_tx.send(BackgroundMessage::VerificationResult(report));
                                    }
                                    if let Some(updated_tools) = output.tools_update {
                                        manifest = ToolManifest::from_tools(&updated_tools);
                                        tools = updated_tools.clone();
                                        let _ = result_tx.send(BackgroundMessage::ToolsUpdated(updated_tools));
                                    }
                                    if output.request_quit {
                                        let _ = result_tx.send(BackgroundMessage::QuitRequested);
                                    }
                                }
                            }
                        }
                        else => {
                            break;
                        }
                    }
                }
            });
        });

        Self {
            work_tx,
            control_tx,
            result_rx,
            shutdown_tx: Some(shutdown_tx),
            worker_join: Some(worker_join),
        }
    }

    pub fn try_send_work(&self, req: WorkRequest) -> Result<(), String> {
        self.work_tx
            .try_send(req)
            .map_err(|err| format!("failed to dispatch work: {}", err))
    }

    pub fn try_send_control(&self, req: ControlRequest) -> Result<(), String> {
        self.control_tx
            .try_send(req)
            .map_err(|err| format!("failed to dispatch control: {}", err))
    }

    pub fn try_recv(&self) -> Option<BackgroundMessage> {
        self.result_rx.try_recv().ok()
    }

    pub fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.worker_join.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for BackgroundRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[derive(Clone, Debug)]
struct ToolCandidate {
    tool_index: usize,
    searchable: String,
}

pub struct ToolNavigator {
    pub tools: Vec<Tool>,
    pub matches: Vec<Match>,
    pub list_state: ListState,
    nucleo: Nucleo<ToolCandidate>,
    injector: Injector<ToolCandidate>,
    last_query: String,
    score_matcher: Matcher,
}

impl ToolNavigator {
    pub fn new(tools: Vec<Tool>) -> Self {
        let notify = Arc::new(|| {});
        let nucleo = Nucleo::new(Config::DEFAULT, notify, None, 1);
        let injector = nucleo.injector();
        let mut nav = Self {
            tools,
            matches: vec![],
            list_state: ListState::default(),
            nucleo,
            injector,
            last_query: String::new(),
            score_matcher: Matcher::new(Config::DEFAULT),
        };
        nav.rebuild_index();
        nav.set_query("");
        nav
    }

    pub fn set_tools(&mut self, tools: Vec<Tool>) {
        self.tools = tools;
        self.rebuild_index();
        let query = self.last_query.clone();
        self.set_query(&query);
    }

    pub fn set_query(&mut self, query: &str) {
        let append = query.starts_with(&self.last_query) && query.len() >= self.last_query.len();
        self.nucleo
            .pattern
            .reparse(0, query, CaseMatching::Smart, Normalization::Smart, append);
        self.last_query = query.to_string();

        self.tick_until_idle(16);
        self.refresh_matches();
    }

    pub fn tick(&mut self) -> bool {
        let status = self.nucleo.tick(4);
        if status.changed {
            self.refresh_matches();
        }
        status.changed || status.running
    }

    pub fn selected_tool(&self) -> Option<&Tool> {
        self.list_state
            .selected()
            .and_then(|i| self.matches.get(i))
            .and_then(|m| self.tools.get(m.idx as usize))
    }

    pub fn selected_score(&self) -> Option<u32> {
        self.list_state
            .selected()
            .and_then(|i| self.matches.get(i))
            .map(|m| m.score)
    }

    pub fn next(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % self.matches.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.matches.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn page_up(&mut self, page_size: usize) {
        if self.matches.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let next = current.saturating_sub(page_size);
        self.list_state.select(Some(next));
    }

    pub fn page_down(&mut self, page_size: usize) {
        if self.matches.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let max = self.matches.len().saturating_sub(1);
        let next = (current + page_size).min(max);
        self.list_state.select(Some(next));
    }

    fn rebuild_index(&mut self) {
        self.nucleo.restart(true);
        self.injector = self.nucleo.injector();
        for (tool_index, tool) in self.tools.iter().enumerate() {
            let candidate = ToolCandidate {
                tool_index,
                searchable: tool.name.clone(),
            };
            self.injector.push(candidate, |item, cols| {
                cols[0] = Utf32String::from(item.searchable.as_str())
            });
        }
    }

    fn tick_until_idle(&mut self, max_ticks: usize) {
        for _ in 0..max_ticks {
            let status = self.nucleo.tick(4);
            if status.changed {
                self.refresh_matches();
            }
            if !status.running {
                break;
            }
        }
    }

    fn refresh_matches(&mut self) {
        self.matches.clear();
        let snapshot = self.nucleo.snapshot();
        let pattern = snapshot.pattern();
        for item in snapshot.matched_items(..) {
            let score = pattern
                .score(item.matcher_columns, &mut self.score_matcher)
                .unwrap_or(0);
            self.matches.push(Match {
                score,
                idx: item.data.tool_index as u32,
            });
        }

        if self.matches.is_empty() {
            self.list_state.select(None);
            return;
        }

        let selected = self.list_state.selected().unwrap_or(0);
        let clamped = selected.min(self.matches.len() - 1);
        self.list_state.select(Some(clamped));
    }
}

pub struct AppState {
    pub list_height: u16,
    pub list_area: Rect,
    pub messages: VecDeque<String>,
    pub should_quit: bool,
}

impl AppState {
    const MAX_MESSAGES: usize = 30;

    fn new() -> Self {
        Self {
            list_height: 20,
            list_area: Rect::default(),
            messages: VecDeque::new(),
            should_quit: false,
        }
    }

    pub fn push_message(&mut self, message: impl Into<String>) {
        self.messages.push_back(message.into());
        while self.messages.len() > Self::MAX_MESSAGES {
            let _ = self.messages.pop_front();
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
        self.last_resize = Instant::now();
        self.resize_pending = true;
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

pub struct AppContext<'a> {
    app_state: &'a mut AppState,
    navigator: &'a mut ToolNavigator,
    runtime: &'a BackgroundRuntime,
    registry: &'a AgentRegistry,
    journal: &'a mut dyn Journal,
}

impl<'a> AppContext<'a> {
    fn push_message(&mut self, message: impl Into<String>) {
        self.app_state.push_message(message.into());
    }

    fn submit_invocation(&mut self, invocation: AgentInvocation) {
        self.journal.append(JournalEvent::CommandSubmitted {
            timestamp_epoch_secs: now_epoch_secs(),
            agent_id: invocation.agent_id.clone(),
            raw_command: invocation.raw_command.clone(),
            args: invocation.args.clone(),
        });

        match self.runtime.try_send_work(WorkRequest::Invoke(invocation)) {
            Ok(()) => self.push_message("agent command submitted"),
            Err(err) => self.push_message(err),
        }
    }
}

pub enum ViewSignal {
    None,
    Push(Box<dyn View>),
    Pop,
    Quit,
}

pub trait View {
    fn title(&self) -> &'static str;
    fn input_text(&self) -> String;
    fn on_key(&mut self, key: KeyEvent, ctx: &mut AppContext<'_>) -> ViewSignal;
    fn on_mouse(&mut self, _mouse: MouseEvent, _ctx: &mut AppContext<'_>) -> ViewSignal {
        ViewSignal::None
    }
}

#[derive(Default)]
pub struct SearchView {
    query: String,
}

impl View for SearchView {
    fn title(&self) -> &'static str {
        "Search ( / or ? => agent command view )"
    }

    fn input_text(&self) -> String {
        format!("Search: {}", self.query)
    }

    fn on_key(&mut self, key: KeyEvent, ctx: &mut AppContext<'_>) -> ViewSignal {
        match key.code {
            KeyCode::Char('q') => ViewSignal::Quit,
            KeyCode::Char('/') | KeyCode::Char('?') => {
                ViewSignal::Push(Box::new(AgentCommandView::default()))
            }
            KeyCode::Char(c) => {
                self.query.push(c);
                ctx.navigator.set_query(&self.query);
                ViewSignal::None
            }
            KeyCode::Backspace => {
                let _ = self.query.pop();
                ctx.navigator.set_query(&self.query);
                ViewSignal::None
            }
            KeyCode::Up => {
                ctx.navigator.previous();
                ViewSignal::None
            }
            KeyCode::Down => {
                ctx.navigator.next();
                ViewSignal::None
            }
            KeyCode::PageUp => {
                let page_size = (ctx.app_state.list_height as usize)
                    .saturating_sub(2)
                    .max(5);
                ctx.navigator.page_up(page_size);
                ViewSignal::None
            }
            KeyCode::PageDown => {
                let page_size = (ctx.app_state.list_height as usize)
                    .saturating_sub(2)
                    .max(5);
                ctx.navigator.page_down(page_size);
                ViewSignal::None
            }
            KeyCode::Enter => {
                if let Some(tool) = ctx.navigator.selected_tool() {
                    let Some(agent_id) = ctx.registry.resolve_alias("search") else {
                        ctx.push_message("search agent missing from registry");
                        return ViewSignal::None;
                    };
                    let invocation = AgentInvocation {
                        agent_id,
                        args: vec![tool.name.clone()],
                        raw_command: format!("search {}", tool.name),
                    };
                    ctx.submit_invocation(invocation);
                }
                ViewSignal::None
            }
            _ => ViewSignal::None,
        }
    }

    fn on_mouse(&mut self, mouse: MouseEvent, ctx: &mut AppContext<'_>) -> ViewSignal {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                ctx.navigator.previous();
                ViewSignal::None
            }
            MouseEventKind::ScrollDown => {
                ctx.navigator.next();
                ViewSignal::None
            }
            MouseEventKind::Down(_) => {
                let content_area = inner_rect(ctx.app_state.list_area);
                if point_in_rect(content_area, mouse.column, mouse.row) {
                    let clicked_row = (mouse.row - content_area.y) as usize;
                    let list_offset = ctx.navigator.list_state.offset();
                    let index = list_offset.saturating_add(clicked_row);
                    if index < ctx.navigator.matches.len() {
                        ctx.navigator.list_state.select(Some(index));
                    }
                }
                ViewSignal::None
            }
            _ => ViewSignal::None,
        }
    }
}

#[derive(Default)]
pub struct AgentCommandView {
    input: String,
}

impl View for AgentCommandView {
    fn title(&self) -> &'static str {
        "Agent command (Enter submit, Esc cancel)"
    }

    fn input_text(&self) -> String {
        format!("> {}", self.input)
    }

    fn on_key(&mut self, key: KeyEvent, ctx: &mut AppContext<'_>) -> ViewSignal {
        match key.code {
            KeyCode::Esc => ViewSignal::Pop,
            KeyCode::Backspace => {
                let _ = self.input.pop();
                ViewSignal::None
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                ViewSignal::None
            }
            KeyCode::Enter => {
                let raw = self.input.trim().to_string();
                if raw.is_empty() {
                    return ViewSignal::Pop;
                }
                match parse_agent_command(&raw) {
                    Ok(parsed) => {
                        let Some(agent_id) = ctx.registry.resolve_alias(&parsed.command) else {
                            ctx.push_message(format!("unknown command: {}", parsed.command));
                            return ViewSignal::Pop;
                        };
                        let invocation = AgentInvocation {
                            agent_id,
                            args: parsed.args,
                            raw_command: raw,
                        };
                        ctx.submit_invocation(invocation);
                    }
                    Err(err) => ctx.push_message(err),
                }
                ViewSignal::Pop
            }
            _ => ViewSignal::None,
        }
    }
}

pub struct App {
    state: AppState,
    navigator: ToolNavigator,
    views: Vec<Box<dyn View>>,
    runtime: BackgroundRuntime,
    registry: AgentRegistry,
    journal: Box<dyn Journal>,
    state_store: Box<dyn StateStore>,
    resize_debouncer: ResizeDebouncer,
    dirty: bool,
}

impl App {
    fn new() -> Self {
        let tools = default_tools();
        let registry = AgentRegistry::with_builtin_agents();
        let mut state = AppState::new();
        state.push_message("ready");
        let mut state_store: Box<dyn StateStore> = Box::new(InMemoryStateStore::default());
        state_store.save_manifest(ToolManifest::from_tools(&tools));

        let runtime = BackgroundRuntime::new(tools.clone(), registry.clone());
        let _ = runtime.try_send_control(ControlRequest::Ping);

        Self {
            state,
            navigator: ToolNavigator::new(tools),
            views: vec![Box::new(SearchView::default())],
            runtime,
            registry,
            journal: Box::new(MemoryJournal::default()),
            state_store,
            resize_debouncer: ResizeDebouncer::new(120),
            dirty: true,
        }
    }

    fn top_view(&self) -> &dyn View {
        self.views
            .last()
            .expect("view stack must always contain at least one view")
            .as_ref()
    }

    fn handle_key(&mut self, key: KeyEvent) {
        self.with_top_view(|view, ctx| view.on_key(key, ctx));
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        self.with_top_view(|view, ctx| view.on_mouse(mouse, ctx));
    }

    fn with_top_view<F>(&mut self, f: F)
    where
        F: FnOnce(&mut dyn View, &mut AppContext<'_>) -> ViewSignal,
    {
        let mut current = self
            .views
            .pop()
            .expect("view stack must always contain at least one view");
        let signal = {
            let mut ctx = AppContext {
                app_state: &mut self.state,
                navigator: &mut self.navigator,
                runtime: &self.runtime,
                registry: &self.registry,
                journal: self.journal.as_mut(),
            };
            f(current.as_mut(), &mut ctx)
        };
        self.apply_view_signal(current, signal);
        self.dirty = true;
    }

    fn apply_view_signal(&mut self, current: Box<dyn View>, signal: ViewSignal) {
        match signal {
            ViewSignal::None => self.views.push(current),
            ViewSignal::Push(next) => {
                self.views.push(current);
                self.views.push(next);
            }
            ViewSignal::Pop => {
                if self.views.is_empty() {
                    self.state.should_quit = true;
                }
            }
            ViewSignal::Quit => {
                self.state.should_quit = true;
            }
        }
    }

    fn handle_background_messages(&mut self) {
        while let Some(msg) = self.runtime.try_recv() {
            match msg {
                BackgroundMessage::ToolsUpdated(tools) => {
                    self.navigator.set_tools(tools.clone());
                    self.state_store
                        .save_manifest(ToolManifest::from_tools(&tools));
                    self.state.push_message("tools updated");
                }
                BackgroundMessage::AgentResponse(text) => {
                    self.state.push_message(format!("agent: {}", text));
                }
                BackgroundMessage::VerificationResult(report) => {
                    self.state.push_message(format!(
                        "verification: passed={} root={}",
                        report.passed,
                        report
                            .root_hash
                            .as_ref()
                            .map(|h| h.0.clone())
                            .unwrap_or_else(|| "none".into())
                    ));
                }
                BackgroundMessage::QuitRequested => {
                    self.state.push_message("quit requested by agent");
                    self.state.should_quit = true;
                }
                BackgroundMessage::Journal(event) => {
                    self.journal.append(event);
                }
            }
            self.dirty = true;
        }
    }

    fn tick(&mut self) {
        if self.navigator.tick() {
            self.dirty = true;
        }
        if self.resize_debouncer.should_redraw() {
            self.dirty = true;
        }
        self.handle_background_messages();
    }

    fn shutdown(&mut self) {
        self.runtime.shutdown();
    }
}

fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    terminal.draw(|f| {
        let size = f.size();

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(4),
            ])
            .split(size);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(vertical[1]);

        app.state.list_height = horizontal[0].height;
        app.state.list_area = horizontal[0];

        let top_view = app.top_view();
        let input = Paragraph::new(top_view.input_text()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(top_view.title()),
        );
        f.render_widget(input, vertical[0]);

        let list_items = app
            .navigator
            .matches
            .iter()
            .filter_map(|m| {
                app.navigator
                    .tools
                    .get(m.idx as usize)
                    .map(|tool| (tool, m.score))
            })
            .map(|(tool, score)| ListItem::new(format!("{:<22} score={}", tool.name, score)))
            .collect::<Vec<_>>();

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL).title("Tools"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        f.render_stateful_widget(list, horizontal[0], &mut app.navigator.list_state);

        let preview_text = if let Some(tool) = app.navigator.selected_tool() {
            format!(
                "{}\n\npackage: {}\nscore: {}\n\nexamples:\n{}",
                tool.description,
                tool.package.as_deref().unwrap_or("n/a"),
                app.navigator.selected_score().unwrap_or(0),
                tool.examples.join("\n")
            )
        } else {
            "No tool selected".into()
        };
        let preview = Paragraph::new(preview_text)
            .block(Block::default().borders(Borders::ALL).title("Preview"));
        f.render_widget(preview, horizontal[1]);

        let latest = app
            .state
            .messages
            .back()
            .cloned()
            .unwrap_or_else(|| "no messages".into());
        let status = Paragraph::new(format!(
            "q quit | / command view | arrows/page | mouse wheel/click\n{}",
            latest
        ))
        .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(status, vertical[2]);
    })?;
    Ok(())
}

fn cleanup_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
}

fn install_panic_hook() {
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        cleanup_terminal();
        previous_hook(panic_info);
    }));
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
        cleanup_terminal();
    }
}

pub fn point_in_rect(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

pub fn inner_rect(rect: Rect) -> Rect {
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
            description: "JSON processor".into(),
            examples: vec![
                "jq '.items[]' x.json".into(),
                "cat x.json | jq '.a.b'".into(),
            ],
            package: Some("jq".into()),
        },
    ]
}

fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    install_panic_hook();
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    while !app.state.should_quit {
        if app.dirty {
            draw(&mut terminal, &mut app)?;
            app.dirty = false;
        }

        if event::poll(Duration::from_millis(30))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => app.handle_key(key),
                Event::Mouse(mouse) => app.handle_mouse(mouse),
                Event::Resize(_, _) => app.resize_debouncer.on_resize(),
                _ => {}
            }
        }
        app.tick();
    }

    app.shutdown();
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_agent_command_parses_command_and_args() {
        let parsed = parse_agent_command("search ripgrep fast").expect("command should parse");
        assert_eq!(parsed.command, "search");
        assert_eq!(parsed.args, vec!["ripgrep", "fast"]);
    }

    #[test]
    fn parse_agent_command_rejects_empty_input() {
        let err = parse_agent_command("  ").expect_err("empty command should fail");
        assert!(err.contains("Empty"));
    }

    #[test]
    fn tool_navigator_filters_and_selects() {
        let mut nav = ToolNavigator::new(default_tools());
        nav.set_query("rg");
        assert!(!nav.matches.is_empty());
        let has_expected_match = nav.matches.iter().any(|m| {
            nav.tools
                .get(m.idx as usize)
                .map(|tool| tool.name.eq_ignore_ascii_case("ripgrep"))
                .unwrap_or(false)
        });
        assert!(has_expected_match, "ripgrep should appear in results");
        assert!(nav.selected_tool().is_some(), "selection should exist");
    }

    #[test]
    fn point_in_rect_is_inclusive_exclusive() {
        let rect = Rect::new(10, 10, 4, 3);
        assert!(point_in_rect(rect, 10, 10));
        assert!(point_in_rect(rect, 13, 12));
        assert!(!point_in_rect(rect, 14, 12));
        assert!(!point_in_rect(rect, 13, 13));
    }

    #[test]
    fn inner_rect_trims_borders() {
        let inner = inner_rect(Rect::new(2, 3, 10, 8));
        assert_eq!(inner, Rect::new(3, 4, 8, 6));
    }

    #[test]
    fn app_state_push_message_keeps_bounded_history() {
        let mut app_state = AppState::new();
        for i in 0..50 {
            app_state.push_message(format!("m{i}"));
        }
        assert_eq!(app_state.messages.len(), AppState::MAX_MESSAGES);
        assert_eq!(
            app_state
                .messages
                .front()
                .expect("history should retain latest window"),
            "m20"
        );
        assert_eq!(
            app_state
                .messages
                .back()
                .expect("latest entry should be present"),
            "m49"
        );
    }
}
