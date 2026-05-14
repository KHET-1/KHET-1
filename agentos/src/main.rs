//! Agentic Terminal OS — core navigator + agent harness prototype.
//!
//! Run: `cargo run` from the `agentos/` directory.

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
