//! Minimal pluggable agent surface: registry + structured outcomes.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum AgentOutcome {
    Message(String),
    /// Reserved for future: request exec / confirm / spawn sub-agent.
    Noop,
}

pub type AgentFn = fn(&str) -> AgentOutcome;

pub struct AgentRegistry {
    commands: HashMap<String, AgentFn>,
}

impl AgentRegistry {
    pub fn with_builtin_agents() -> Self {
        let mut commands: HashMap<String, AgentFn> = HashMap::new();
        commands.insert("help".into(), |args| {
            if args.is_empty() {
                AgentOutcome::Message(
                    "commands: help, echo <text>, quit (alias for app exit TBD)".into(),
                )
            } else {
                AgentOutcome::Message(format!("help: unknown topic {args:?}"))
            }
        });
        commands.insert("echo".into(), |args| {
            AgentOutcome::Message(args.to_string())
        });
        Self { commands }
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
            if let Some(f) = self.commands.get(cmd) {
                f(rest)
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

    #[test]
    fn slash_dispatch() {
        let r = AgentRegistry::with_builtin_agents();
        match r.handle_line("/echo hello") {
            AgentOutcome::Message(s) => assert_eq!(s, "hello"),
            _ => panic!(),
        }
    }
}
