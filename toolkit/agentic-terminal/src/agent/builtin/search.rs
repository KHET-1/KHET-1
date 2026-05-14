//! Keyword search routing (placeholder).

use crate::agent::{Agent, AgentCtx, AgentEvent, AgentOutcome};

#[derive(Clone, Copy, Debug, Default)]
pub struct SearchAgent;

impl Agent for SearchAgent {
    fn name(&self) -> &'static str {
        "search"
    }

    fn help(&self) -> &'static str {
        "Echo the forwarded search phrase (semantic search is out of scope)."
    }

    fn run(&mut self, args: &[String], ctx: &mut AgentCtx<'_>) -> AgentOutcome {
        let query = args.join(" ");
        if query.trim().is_empty() {
            return AgentOutcome::Error("Usage: search <term>".into());
        }
        let text = format!("Agent search accepted: {}", query.trim());
        ctx.responder.send(AgentEvent::Status {
            agent: self.identity(),
            text,
        });
        AgentOutcome::Ok
    }
}
