//! Terminal lifecycle: alternate screen guards and cooperative panic teardown.

use std::io::{self};
use std::panic;
use std::sync::Once;

use crossterm::{
    execute,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};

static CLEANUP_GUARD: Once = Once::new();

fn cleanup_terminal_assets() {
    let _ = disable_raw_mode();
    let _ = execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    );
}

/// Restores tty state for panics originating after alternate screen takeover.
///
/// Intended to execute before Crossterm is constructed in [`crate::main`].
pub fn install_panic_hook() {
    CLEANUP_GUARD.call_once(|| {
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            cleanup_terminal_assets();
            default_hook(info);
        }));
    });
}

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;
        Ok(Self {})
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        cleanup_terminal_assets();
    }
}
