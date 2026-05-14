//! Domain types: tools, modes, layers. UI state stays in `crate::app::App`.

pub type ToolId = u32;

#[derive(Debug, Clone)]
pub struct Tool {
    pub id: ToolId,
    pub name: String,
    pub description: String,
}

impl Tool {
    pub fn fuzzy_line(&self) -> String {
        format!("{} {}", self.name, self.description)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Search,
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLayer {
    /// Tool list + preview; search query or navigation keys.
    Navigator,
    /// Agent input line focused (still shows navigator underneath).
    AgentPrompt,
}
