//! Built-in [`crate::agent::Agent`] implementations.

mod help;
mod quit;
mod refresh;
mod search;
mod verify;

pub use help::HelpAgent;
pub use quit::QuitAgent;
pub use refresh::RefreshAgent;
pub use search::SearchAgent;
pub use verify::VerifyAgent;

use crate::agent::AgentRegistry;

pub fn register_builtins(reg: &mut AgentRegistry) {
    reg.register(HelpAgent);
    reg.register(RefreshAgent);
    reg.register(VerifyAgent);
    reg.register(SearchAgent);
    reg.register(QuitAgent);
}
