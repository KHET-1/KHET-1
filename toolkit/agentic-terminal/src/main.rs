//! Binary entrypoint for the agentic-terminal TUI.

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    agentic_terminal::term::install_panic_hook();
    agentic_terminal::app::run_app()
}
