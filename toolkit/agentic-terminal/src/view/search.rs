//! Main search + preview surface.

use crossterm::event::{Event, KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::ViewCtx;
use crate::runtime::WorkerBusyError;
use crate::view::palette::CommandPaletteView;
use crate::view::{inner_rect, point_in_rect, View, ViewResult};

#[derive(Default)]
pub struct SearchView;

impl View for SearchView {
    fn title(&self) -> &'static str {
        "search"
    }

    fn on_event(&mut self, event: &Event, ctx: &mut ViewCtx<'_>) -> ViewResult {
        if ctx.command_palette_visible {
            return ViewResult::Consumed;
        }

        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => ViewResult::Quit,
                KeyCode::Char('/') => ViewResult::Push(Box::new(CommandPaletteView::default())),
                KeyCode::Char('?') => {
                    if let Ok(()) = ctx.shared.background.send_line("help".to_string()) {
                        ctx.shared.push_message("Agent command submitted");
                    } else {
                        ctx.shared.push_message("system busy, retry");
                    }
                    ViewResult::Consumed
                }
                KeyCode::Char(c) => {
                    ctx.shared.search_query.push(c);
                    ctx.shared
                        .navigator
                        .set_query(&ctx.shared.search_query);
                    ViewResult::Consumed
                }
                KeyCode::Backspace => {
                    let _ = ctx.shared.search_query.pop();
                    ctx.shared
                        .navigator
                        .set_query(&ctx.shared.search_query);
                    ViewResult::Consumed
                }
                KeyCode::Up => {
                    ctx.shared.navigator.previous();
                    ViewResult::Consumed
                }
                KeyCode::Down => {
                    ctx.shared.navigator.next();
                    ViewResult::Consumed
                }
                KeyCode::PageUp => {
                    let size = (ctx.shared.list_height as usize).saturating_sub(2).max(5);
                    ctx.shared.navigator.page_up(size);
                    ViewResult::Consumed
                }
                KeyCode::PageDown => {
                    let size = (ctx.shared.list_height as usize).saturating_sub(2).max(5);
                    ctx.shared.navigator.page_down(size);
                    ViewResult::Consumed
                }
                KeyCode::Enter => {
                    if let Some(tool) = ctx.shared.navigator.selected_tool() {
                        let tool_name = tool.name.clone();
                        match ctx.shared.background.send_open_tool(tool_name.clone()) {
                            Ok(()) => ctx.shared.push_message(format!(
                                "Submitted tool lookup: {tool_name}"
                            )),
                            Err(WorkerBusyError(())) => {
                                ctx.shared.push_message("system busy, retry");
                            }
                        }
                    }
                    ViewResult::Consumed
                }
                _ => ViewResult::Consumed,
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => {
                    ctx.shared.navigator.previous();
                    ViewResult::Consumed
                }
                MouseEventKind::ScrollDown => {
                    ctx.shared.navigator.next();
                    ViewResult::Consumed
                }
                MouseEventKind::Down(_) => {
                    let content_area = inner_rect(ctx.shared.list_area);
                    if point_in_rect(content_area, mouse.column, mouse.row) {
                        let clicked_row = (mouse.row - content_area.y) as usize;
                        let list_offset = ctx.shared.navigator.list_state.offset();
                        let index = list_offset.saturating_add(clicked_row);
                        // `list_state.offset()` reflects the scrollbar position from the last draw;
                        // nucleo-driven match refreshes between render and poll can stale for a frame.
                        if index < ctx.shared.navigator.filtered_len() {
                            ctx.shared.navigator.list_state.select(Some(index));
                        }
                    }
                    ViewResult::Consumed
                }
                _ => ViewResult::Consumed,
            },
            _ => ViewResult::Consumed,
        }
    }

    fn render(&mut self, area: Rect, frame: &mut Frame<'_>, ctx: &mut ViewCtx<'_>) {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(4),
            ])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(vertical[1]);

        ctx.shared.list_height = horizontal[0].height;
        ctx.shared.list_area = horizontal[0];
        ctx.shared.input_area = vertical[0];

        let input_title = "Search mode ( / opens palette )";
        let input_text = format!("Search: {}", ctx.shared.search_query);

        frame.render_widget(
            Paragraph::new(input_text.as_str())
                .block(Block::default().borders(Borders::ALL).title(input_title)),
            vertical[0],
        );

        let query_empty = ctx.shared.search_query.trim().is_empty();

        let items: Vec<ListItem> = {
            let nav = &ctx.shared.navigator;
            nav.matches()
                .iter()
                .filter_map(|m| {
                    let tool = nav.tool_at_idx(m.tool_idx)?;
                    let line = if query_empty {
                        tool.name.clone()
                    } else {
                        format!("{:<24} score={}", tool.name, m.score)
                    };
                    Some(ListItem::new(line))
                })
                .collect()
        };

        frame.render_stateful_widget(
            List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Tools"))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
            horizontal[0],
            &mut ctx.shared.navigator.list_state,
        );

        let preview_text = if let Some(tool) = ctx.shared.navigator.selected_tool() {
            let package = tool.package.clone().unwrap_or_else(|| "n/a".into());
            let score = ctx.shared.navigator.selected_score().unwrap_or(0);
            format!(
                "{}\n\nPackage: {}\nScore: {}\n\nExamples:\n{}",
                tool.description,
                package,
                score,
                tool.examples.join("\n"),
            )
        } else {
            "No selection".to_string()
        };

        frame.render_widget(
            Paragraph::new(preview_text)
                .block(Block::default().borders(Borders::ALL).title("Preview")),
            horizontal[1],
        );

        let mut recent_tail: Vec<String> = ctx.shared.messages.iter().rev().take(3).cloned().collect();
        recent_tail.reverse();

        let latest_block = if recent_tail.is_empty() {
            "No messages".to_string()
        } else {
            recent_tail.join(" | ")
        };

        let busy_star = if ctx.shared.background.pending_commands() > 0 {
            "*"
        } else {
            ""
        };

        frame.render_widget(
            Paragraph::new(format!(
                "q=quit | ↑↓ PgUp/PgDn | wheel/click | Enter=open\n{busy_star}{latest_block}",
            ))
            .block(Block::default().borders(Borders::ALL).title("Status")),
            vertical[2],
        );
    }
}
