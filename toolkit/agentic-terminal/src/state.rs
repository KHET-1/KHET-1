//! Persistent state abstraction (stub).

use std::error::Error;

pub trait StateStore: Send + Sync {
    fn save_json(
        &mut self,
        _key: &str,
        _value: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;

    fn load_json(
        &mut self,
        _key: &str,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>>;
}

pub struct NullStateStore;

impl StateStore for NullStateStore {
    fn save_json(
        &mut self,
        _key: &str,
        _value: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    fn load_json(
        &mut self,
        _key: &str,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        Ok(None)
    }
}

/// In-memory session state for recent/pinned commands.
#[derive(Clone, Debug, Default)]
pub struct SessionState {
    pub recent_commands: Vec<String>,
    pub pinned_commands: Vec<String>,
}

impl SessionState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            recent_commands: Vec::new(),
            pinned_commands: Vec::new(),
        }
    }

    pub fn add_recent(&mut self, command: String) {
        if !command.is_empty() && !self.recent_commands.contains(&command) {
            self.recent_commands.insert(0, command);
            if self.recent_commands.len() > 20 {
                self.recent_commands.pop();
            }
        }
    }

    pub fn toggle_pin(&mut self, command: String) {
        if let Some(pos) = self.pinned_commands.iter().position(|c| c == &command) {
            self.pinned_commands.remove(pos);
        } else {
            self.pinned_commands.insert(0, command);
            if self.pinned_commands.len() > 10 {
                self.pinned_commands.pop();
            }
        }
    }

    pub fn is_pinned(&self, command: &str) -> bool {
        self.pinned_commands.contains(&command.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_commands_limit_to_20() {
        let mut state = SessionState::new();
        for i in 0..30 {
            state.add_recent(format!("cmd{i}"));
        }
        assert_eq!(state.recent_commands.len(), 20);
    }

    #[test]
    fn pinned_commands_limit_to_10() {
        let mut state = SessionState::new();
        for i in 0..15 {
            state.toggle_pin(format!("cmd{i}"));
        }
        assert_eq!(state.pinned_commands.len(), 10);
    }

    #[test]
    fn no_duplicate_recent_commands() {
        let mut state = SessionState::new();
        state.add_recent("test".into());
        state.add_recent("test".into());
        assert_eq!(state.recent_commands.len(), 1);
    }
}
