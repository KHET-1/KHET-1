//! Refresh builtin tool manifest.

use std::sync::Arc;

use crate::agent::{Agent, AgentCtx, AgentEvent, AgentOutcome};
use crate::manifest::default_manifest;

#[derive(Clone, Copy, Debug, Default)]
pub struct RefreshAgent;

impl Agent for RefreshAgent {
    fn name(&self) -> &'static str {
        "refresh"
    }

    fn help(&self) -> &'static str {
        "Rebuild the default builtin tool manifest."
    }

    fn run(&mut self, _args: &[String], ctx: &mut AgentCtx<'_>) -> AgentOutcome {
        *ctx.manifest = default_manifest();
        ctx.responder.send(AgentEvent::ManifestUpdated(Arc::new(ctx.manifest.clone())));
        ctx.responder.send(AgentEvent::Status {
            agent: self.identity(),
            text: "Tool metadata refreshed".into(),
        });
        AgentOutcome::Ok
    }
}
