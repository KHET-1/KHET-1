//! Restore the terminal after a panic so raw mode / alternate screen do not brick the session.

use std::io::stdout;
use std::panic;

use crossterm::cursor::Show;
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

pub fn install() {
    let original = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let mut out = stdout();
        let _ = disable_raw_mode();
        let _ = execute!(out, LeaveAlternateScreen, DisableMouseCapture, Show);
        original(info);
    }));
}
