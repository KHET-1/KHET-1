//! Terminate foreground session hints.

use crate::agent::{Agent, AgentCtx, AgentOutcome};

#[derive(Clone, Copy, Debug, Default)]
pub struct QuitAgent;

impl Agent for QuitAgent {
    fn name(&self) -> &'static str {
        "quit"
    }

    fn help(&self) -> &'static str {
        "Shut down this session."
    }

    fn run(&mut self, _args: &[String], _ctx: &mut AgentCtx<'_>) -> AgentOutcome {
        AgentOutcome::Shutdown
    }
}
