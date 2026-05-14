//! Pluggable agents behind a single registry (`command` → `dyn Agent`).
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::boundary::AgentId;

#[derive(Debug, Clone)]
pub enum AgentOutcome {
    Message(String),
    /// Reserved for future: request exec / confirm / spawn sub-agent.
    Noop,
    /// User (or agent) requested application exit.
    Quit,
}

pub trait Agent: Send + Sync {
    fn id(&self) -> AgentId;
    /// Slash command without leading `/` (e.g. `"help"`).
    fn command(&self) -> &'static str;
    fn handle(&self, args: &str) -> AgentOutcome;
}

struct HelpAgent;

impl Agent for HelpAgent {
    fn id(&self) -> AgentId {
        AgentId::new("builtin.help")
    }

    fn command(&self) -> &'static str {
        "help"
    }

    fn handle(&self, args: &str) -> AgentOutcome {
        if args.is_empty() {
            AgentOutcome::Message(
                "commands: /help, /echo <text>, /quit — Tab: search mode, Ctrl+C: exit".into(),
            )
        } else {
            AgentOutcome::Message(format!("help: unknown topic {args:?}"))
        }
    }
}

struct EchoAgent;

impl Agent for EchoAgent {
    fn id(&self) -> AgentId {
        AgentId::new("builtin.echo")
    }

    fn command(&self) -> &'static str {
        "echo"
    }

    fn handle(&self, args: &str) -> AgentOutcome {
        AgentOutcome::Message(args.to_string())
    }
}

struct QuitAgent;

impl Agent for QuitAgent {
    fn id(&self) -> AgentId {
        AgentId::new("builtin.quit")
    }

    fn command(&self) -> &'static str {
        "quit"
    }

    fn handle(&self, _args: &str) -> AgentOutcome {
        AgentOutcome::Quit
    }
}

pub struct AgentRegistry {
    by_command: HashMap<String, Arc<dyn Agent>>,
}

impl AgentRegistry {
    pub fn with_builtin_agents() -> Self {
        let mut r = Self {
            by_command: HashMap::new(),
        };
        r.register(Arc::new(HelpAgent));
        r.register(Arc::new(EchoAgent));
        r.register(Arc::new(QuitAgent));
        r
    }

    fn register(&mut self, agent: Arc<dyn Agent>) {
        self.by_command.insert(agent.command().to_string(), agent);
    }

    pub fn get_agent(&self, command: &str) -> Option<Arc<dyn Agent>> {
        self.by_command.get(command).cloned()
    }

    /// First token is command name; rest is one string argument body.
    pub fn handle_line(&self, line: &str) -> AgentOutcome {
        let line = line.trim();
        if line.is_empty() {
            return AgentOutcome::Noop;
        }

        let mut parts = line.splitn(2, char::is_whitespace);
        let head = parts.next().unwrap_or("");
        let rest = parts.next().unwrap_or("").trim();

        if let Some(cmd) = head.strip_prefix('/') {
            if let Some(agent) = self.by_command.get(cmd) {
                agent.handle(rest)
            } else {
                AgentOutcome::Message(format!("unknown command /{cmd}"))
            }
        } else {
            AgentOutcome::Message(format!(
                "natural-language agent input not wired yet — try /help ({line})"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary::AgentId;

    #[test]
    fn builtin_agent_ids() {
        let r = AgentRegistry::with_builtin_agents();
        assert_eq!(
            r.get_agent("help").unwrap().id(),
            AgentId::new("builtin.help")
        );
    }

    #[test]
    fn slash_dispatch() {
        let r = AgentRegistry::with_builtin_agents();
        match r.handle_line("/echo hello") {
            AgentOutcome::Message(s) => assert_eq!(s, "hello"),
            _ => panic!(),
        }
    }

    #[test]
    fn quit_agent() {
        let r = AgentRegistry::with_builtin_agents();
        assert!(matches!(r.handle_line("/quit"), AgentOutcome::Quit));
    }
}
