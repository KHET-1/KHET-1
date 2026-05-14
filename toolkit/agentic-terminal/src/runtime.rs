//! Tokio-backed worker thread and channel wiring.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, TryRecvError};
use parking_lot::Mutex;
use tokio::sync::mpsc;

use crate::agent::builtin::register_builtins;
use crate::agent::{AgentCtx, AgentEvent, AgentOutcome, AgentRegistry, AgentResponder};
use crate::journal::{InMemoryJournal, Journal};
use crate::manifest::default_manifest;
use crate::types::{AgentId, ToolManifest};

/// Commands handled by the background worker runtime.
#[derive(Debug, Clone)]
pub enum WorkerCommand {
    Line(String),
    OpenToolLookup(String),
}

enum BridgeSignal {
    Work(WorkerCommand),
    Stop,
}

/// Shared registry/manifest journal state guarded for access from the foreground and worker.
#[derive(Clone)]
pub struct BackgroundRuntimeSyncState {
    pub(crate) registry: Arc<Mutex<AgentRegistry>>,
    pub(crate) journal: Arc<Mutex<InMemoryJournal>>,
    pub(crate) manifest: Arc<Mutex<ToolManifest>>,
}

impl BackgroundRuntimeSyncState {
    #[must_use]
    pub fn new() -> Self {
        let mut registry = AgentRegistry::new();
        register_builtins(&mut registry);
        Self {
            registry: Arc::new(Mutex::new(registry)),
            journal: Arc::new(Mutex::new(InMemoryJournal::new())),
            manifest: Arc::new(Mutex::new(default_manifest())),
        }
    }

    #[must_use]
    pub fn manifest_clone(&self) -> ToolManifest {
        self.manifest.lock().clone()
    }

    #[must_use]
    pub fn tools_clone(&self) -> Vec<crate::types::Tool> {
        self.manifest.lock().tools.clone()
    }
}

impl Default for BackgroundRuntimeSyncState {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridges crossbeam ingress with a Tokio `current_thread` runtime.
pub struct BackgroundRuntime {
    worker_join: Mutex<Option<JoinHandle<()>>>,
    work_tx: Sender<WorkerCommand>,
    shutdown_tx: Sender<()>,
    event_rx: Receiver<AgentEvent>,
    pending_commands: Arc<AtomicUsize>,
    sync: BackgroundRuntimeSyncState,
}

impl Default for BackgroundRuntime {
    fn default() -> Self {
        Self::with_state(BackgroundRuntimeSyncState::new())
    }
}

impl Drop for BackgroundRuntime {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.try_send(());

        let join = self.worker_join.lock().take();
        if let Some(handle) = join {
            let deadline = std::time::Instant::now() + Duration::from_secs(2);
            while !handle.is_finished() && std::time::Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
            }
            if handle.is_finished() {
                let _res = handle.join();
            } else {
                eprintln!("agentic-terminal: worker thread stalled during shutdown");
            }
        }
    }
}

impl BackgroundRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_state(sync_state: BackgroundRuntimeSyncState) -> Self {
        let (shutdown_tx, shutdown_rx) = crossbeam_channel::bounded::<()>(1);
        let (work_tx, work_rx) = crossbeam_channel::bounded::<WorkerCommand>(128);
        let (event_tx, event_rx) = crossbeam_channel::bounded::<AgentEvent>(128);
        let pending_commands = Arc::new(AtomicUsize::new(0));

        let responder = AgentResponder::new(event_tx);

        let sync_for_worker = BackgroundRuntimeSyncState {
            registry: Arc::clone(&sync_state.registry),
            journal: Arc::clone(&sync_state.journal),
            manifest: Arc::clone(&sync_state.manifest),
        };

        let pending_for_worker = Arc::clone(&pending_commands);
        let join = spawn_worker(
            shutdown_rx,
            work_rx,
            responder,
            sync_for_worker,
            pending_for_worker,
        );

        Self {
            worker_join: Mutex::new(Some(join)),
            work_tx,
            shutdown_tx,
            event_rx,
            pending_commands,
            sync: sync_state,
        }
    }

    #[must_use]
    pub fn sync(&self) -> &BackgroundRuntimeSyncState {
        &self.sync
    }

    #[must_use]
    pub fn pending_commands(&self) -> usize {
        self.pending_commands.load(Ordering::Relaxed)
    }

    /// Non-blocking dequeue of outbound agent/UI events (`try_recv` semantics).
    pub fn recv_event_try(&self) -> Result<AgentEvent, TryRecvError> {
        self.event_rx.try_recv()
    }

    pub fn send_line(&self, line: impl Into<String>) -> Result<(), WorkerBusyError> {
        self.enqueue(WorkerCommand::Line(line.into()))
    }

    pub fn send_open_tool(&self, tool_name: String) -> Result<(), WorkerBusyError> {
        self.enqueue(WorkerCommand::OpenToolLookup(tool_name))
    }

    fn enqueue(&self, cmd: WorkerCommand) -> Result<(), WorkerBusyError> {
        self.work_tx.try_send(cmd).map_err(|_| WorkerBusyError(()))?;
        self.pending_commands.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkerBusyError(pub ());

fn spawn_worker(
    shutdown_rx: Receiver<()>,
    work_rx: Receiver<WorkerCommand>,
    responder: AgentResponder,
    sync: BackgroundRuntimeSyncState,
    pending_commands: Arc<AtomicUsize>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("spawn background Tokio runtime");
        let (signal_tx, mut signal_rx) = mpsc::channel::<BridgeSignal>(128);

        let signal_sender = signal_tx.clone();
        let bridge = thread::spawn(move || loop {
            crossbeam_channel::select! {
                recv(shutdown_rx) -> _ => {
                    let _ = signal_sender.blocking_send(BridgeSignal::Stop);
                    break;
                },
                recv(work_rx) -> msg => match msg {
                    Ok(cmd) => {
                        if signal_sender.blocking_send(BridgeSignal::Work(cmd)).is_err() {
                            break;
                        }
                    },
                    Err(_) => break,
                },
            }
        });

        rt.block_on(async move {
            while let Some(signal) = signal_rx.recv().await {
                match signal {
                    BridgeSignal::Stop => break,
                    BridgeSignal::Work(command) => {
                        let responder_cmd = responder.clone();
                        let registry = Arc::clone(&sync.registry);
                        let journal = Arc::clone(&sync.journal);
                        let manifest = Arc::clone(&sync.manifest);
                        let pending_job = Arc::clone(&pending_commands);

                        tokio::task::spawn(async move {
                            let _job_guard = JobAck::new(&pending_job);
                            let _res = tokio::task::spawn_blocking(move || {
                                process_worker_command(
                                    command,
                                    responder_cmd,
                                    registry,
                                    journal,
                                    manifest,
                                );
                            })
                            .await;
                        });
                    }
                }
            }
            drop(signal_rx);
        });

        let _ignored = bridge.join();
    })
}

struct JobAck<'a>(&'a AtomicUsize);

impl Drop for JobAck<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

impl<'a> JobAck<'a> {
    fn new(inner: &'a AtomicUsize) -> Self {
        Self(inner)
    }
}

#[allow(clippy::needless_pass_by_value)]
fn process_worker_command(
    cmd: WorkerCommand,
    responder: AgentResponder,
    registry: Arc<Mutex<AgentRegistry>>,
    journal: Arc<Mutex<InMemoryJournal>>,
    manifest: Arc<Mutex<ToolManifest>>,
) {
    let outbound = responder.clone();
    let composed = match &cmd {
        WorkerCommand::Line(s) => s.clone(),
        WorkerCommand::OpenToolLookup(tool) => format!("search {tool}"),
    };

    let mut registry_guard = registry.lock();
    let mut journal_guard = journal.lock();
    let mut manifest_guard = manifest.lock();

    let mut ctx = AgentCtx {
        responder,
        journal: &mut *journal_guard as &mut dyn Journal,
        manifest: &mut manifest_guard,
    };

    match registry_guard.dispatch(composed.trim(), &mut ctx) {
        Ok(AgentOutcome::Ok | AgentOutcome::Shutdown) => {}
        Err(msg) | Ok(AgentOutcome::Error(msg)) => {
            outbound.send(AgentEvent::Error {
                agent: AgentId("system".into()),
                text: msg,
            });
        }
    }
}
