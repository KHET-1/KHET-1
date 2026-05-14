//! Help command agent.

use crate::agent::{Agent, AgentCtx, AgentEvent, AgentOutcome};

#[derive(Clone, Copy, Debug, Default)]
pub struct HelpAgent;

impl Agent for HelpAgent {
    fn name(&self) -> &'static str {
        "help"
    }

    fn help(&self) -> &'static str {
        "Show available commands."
    }

    fn run(&mut self, _args: &[String], ctx: &mut AgentCtx<'_>) -> AgentOutcome {
        ctx.responder.send(AgentEvent::Status {
            agent: self.identity(),
            text: "Commands: refresh | verify | search <term> | help | quit".into(),
        });
        AgentOutcome::Ok
    }
}
