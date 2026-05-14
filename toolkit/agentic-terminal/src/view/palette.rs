//! Command palette (agent prompt).

use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::ViewCtx;
use crate::view::{View, ViewResult};

#[derive(Default)]
pub struct CommandPaletteView {
    input: String,
}

fn palette_requests_quit(line: &str) -> bool {
    matches!(line.trim().to_ascii_lowercase().as_str(), "quit" | "exit")
}

impl CommandPaletteView {
    fn submit_palette(&mut self, ctx: &mut ViewCtx<'_>) -> ViewResult {
        let trimmed = self.input.trim();
        if trimmed.is_empty() {
            self.input.clear();
            return ViewResult::Pop;
        }
        if palette_requests_quit(trimmed) {
            ctx.shared.should_quit = true;
            self.input.clear();
            return ViewResult::Pop;
        }
        if let Ok(()) = ctx.shared.background.send_line(trimmed.to_string()) {
            ctx.shared.push_message("Agent command submitted");
            self.input.clear();
            ViewResult::Pop
        } else {
            ctx.shared.push_message("system busy, retry");
            ViewResult::Consumed
        }
    }
}

impl View for CommandPaletteView {
    fn title(&self) -> &'static str {
        "command-palette"
    }

    fn layers_as_overlay(&self) -> bool {
        true
    }

    fn on_event(&mut self, event: &Event, ctx: &mut ViewCtx<'_>) -> ViewResult {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Esc => ViewResult::Pop,
                KeyCode::Enter => self.submit_palette(ctx),
                KeyCode::Char(ch) => {
                    self.input.push(ch);
                    ViewResult::Consumed
                }
                KeyCode::Backspace => {
                    let _ = self.input.pop();
                    ViewResult::Consumed
                }
                _ => ViewResult::Consumed,
            },
            _ => ViewResult::Consumed,
        }
    }

    fn render(&mut self, _area: Rect, frame: &mut Frame<'_>, ctx: &mut ViewCtx<'_>) {
        let slice = ctx.shared.input_area;
        if slice.width == 0 || slice.height == 0 {
            return;
        }
        let input_title = "Agent mode (Esc to cancel)";
        let input_text = format!("> {}", self.input);
        let input = Paragraph::new(input_text)
            .block(Block::default().borders(Borders::ALL).title(input_title));
        frame.render_widget(input, slice);
    }
}
