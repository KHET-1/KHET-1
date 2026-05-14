//! Agentic Terminal OS — core navigator + agent harness prototype.
//!
//! Run: `cargo run -p agentos` from the repository root (or `cargo run` inside `agentos/`).

mod agent_harness;
mod app;
mod events;
mod model;
mod tools;
mod ui;
mod worker;

fn main() -> std::io::Result<()> {
    app::run()
}
