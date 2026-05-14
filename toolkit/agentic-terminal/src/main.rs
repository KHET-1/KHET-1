//! Binary entrypoint for the agentic-terminal TUI.

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    agentic_terminal::term::install_panic_hook();
    
    let args: Vec<String> = env::args().collect();
    let helper_only = args.iter().any(|arg| arg == "--helper-only");
    
    let config = if helper_only {
        agentic_terminal::helper_mode::HelperModeConfig::helper_only_mode()
    } else {
        agentic_terminal::helper_mode::HelperModeConfig::default_mode()
    };
    
    agentic_terminal::app::run_app_with_config(config)
}
