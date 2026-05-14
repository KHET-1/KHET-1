//! Agent harness: registry, context, responder, and background events.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use crossbeam_channel::Sender;

use crate::journal::Journal;
use crate::types::{JournalEvent, ToolManifest, VerificationReport};
pub use crate::types::AgentId;

pub mod builtin;

/// Events emitted asynchronously from agents to the foreground.
#[derive(Clone, Debug)]
pub enum AgentEvent {
    Status { agent: AgentId, text: String },
    ManifestUpdated(Arc<ToolManifest>),
    Verification(VerificationReport),
    Error { agent: AgentId, text: String },
}

/// Routes agent output to the UI thread without blocking (`try_send` only).
#[derive(Clone, Debug)]
pub struct AgentResponder {
    sender: Sender<AgentEvent>,
}

impl AgentResponder {
    #[must_use]
    pub fn new(sender: Sender<AgentEvent>) -> Self {
        Self { sender }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn send(&self, event: AgentEvent) {
        if self.sender.try_send(event).is_err() {
            let _ = self.sender.try_send(AgentEvent::Status {
                agent: AgentId("system".into()),
                text: "system busy, retry".into(),
            });
        }
    }
}

pub struct AgentCtx<'a> {
    pub responder: AgentResponder,
    pub journal: &'a mut dyn Journal,
    pub manifest: &'a mut ToolManifest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentOutcome {
    Ok,
    Error(String),
    Shutdown,
}

pub trait Agent: Send + 'static {
    fn name(&self) -> &'static str;

    fn help(&self) -> &'static str;

    fn identity(&self) -> AgentId {
        AgentId(self.name().to_string())
    }

    fn run(&mut self, args: &[String], ctx: &mut AgentCtx<'_>) -> AgentOutcome;
}

pub struct AgentRegistry {
    agents: HashMap<&'static str, Box<dyn Agent>>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register<A: Agent>(&mut self, agent: A) {
        let name = agent.name();
        self.agents.insert(name, Box::new(agent));
    }

    /// Parses `/`-style shell lines and dispatches the first registered command.
    pub fn dispatch(&mut self, line: &str, ctx: &mut AgentCtx<'_>) -> Result<AgentOutcome, String> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Err("Empty command".into());
        }
        let mut parts = trimmed.split_whitespace().map(String::from).collect::<Vec<_>>();
        let cmd_key = parts[0].to_ascii_lowercase();
        let cmd_normalized: String = match cmd_key.as_str() {
            "?" => "help".into(),
            "exit" => "quit".into(),
            other => other.to_string(),
        };
        parts[0].clone_from(&cmd_normalized);

        let Some(agent) = self.agents.get_mut(cmd_normalized.as_str()) else {
            return Err(format!("Unknown command: {trimmed}"));
        };

        let id = agent.identity();
        let tail = if parts.len() > 1 {
            parts[1..].to_vec()
        } else {
            Vec::new()
        };

        ctx.journal.append(JournalEvent::AgentInvoked {
            agent: id.clone(),
            args: parts,
            at: SystemTime::now(),
        });

        let outcome = agent.run(&tail, ctx);

        let (ok, summary) = match &outcome {
            AgentOutcome::Ok => (true, "ok".to_string()),
            AgentOutcome::Error(s) => (false, s.clone()),
            AgentOutcome::Shutdown => (true, "shutdown".to_string()),
        };

        ctx.journal.append(JournalEvent::AgentResult {
            agent: id,
            ok,
            summary,
            at: SystemTime::now(),
        });

        Ok(outcome)
    }

    pub fn names(&self) -> Vec<&'static str> {
        let mut keys: Vec<_> = self.agents.keys().copied().collect();
        keys.sort_unstable();
        keys
    }
}
