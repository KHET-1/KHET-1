//! Layout + render. Updates `LayoutCache` for hit-testing (mouse).

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::LayoutCache;
use crate::model::{InputMode, Tool, ToolId};

pub fn draw(
    frame: &mut Frame<'_>,
    tools: &[Tool],
    filtered_ids: &[ToolId],
    list_state: &mut ratatui::widgets::ListState,
    query: &str,
    agent_buffer: &str,
    mode: InputMode,
    last_agent_message: Option<&str>,
    _event_tail: &[crate::events::AppEvent],
) -> LayoutCache {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    let mode_line = match mode {
        InputMode::Search => "[Search] type to filter — Tab: agent  Esc: clear query  q: quit",
        InputMode::Agent => "[Agent] /help — Tab: search  Enter: submit  Esc: back",
    };
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" agentos — {mode_line} "),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(Block::default().borders(Borders::BOTTOM).title(" harness "));
    frame.render_widget(header, chunks[0]);

    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(chunks[1]);

    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" tools ({}) ", filtered_ids.len()));
    let list_inner = list_block.inner(mid[0]);

    let items: Vec<ListItem> = filtered_ids
        .iter()
        .filter_map(|id| tools.iter().find(|t| t.id == *id))
        .map(|t| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", t.name),
                    Style::default().fg(ratatui::style::Color::Cyan),
                ),
                Span::raw(&t.description),
            ]))
        })
        .collect();

    let list_widget = List::new(items).block(list_block).highlight_style(
        Style::default()
            .bg(ratatui::style::Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_stateful_widget(list_widget, mid[0], list_state);

    let preview_text = preview_for_selection(tools, filtered_ids, list_state.selected());
    let preview = Paragraph::new(preview_text)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" preview / attestation stub "),
        );
    frame.render_widget(preview, mid[1]);

    let status_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(chunks[2]);

    let q = if query.is_empty() {
        "(empty — showing all)".into()
    } else {
        query.to_string()
    };
    let query_line = Paragraph::new(Line::from(vec![
        Span::styled("query: ", Style::default().dim()),
        Span::raw(q),
    ]));
    frame.render_widget(query_line, status_chunks[0]);

    let agent_display = if agent_buffer.is_empty() {
        "(agent buffer empty)".to_string()
    } else {
        agent_buffer.to_string()
    };
    let msg = last_agent_message.unwrap_or("");
    let agent_line = Paragraph::new(Line::from(vec![
        Span::styled("agent: ", Style::default().dim()),
        Span::raw(agent_display),
        Span::styled("  │  ", Style::default().dim()),
        Span::styled(msg, Style::default().fg(ratatui::style::Color::Yellow)),
    ]));
    frame.render_widget(agent_line, status_chunks[1]);

    // Debug strip: last events (future: toggle)
    LayoutCache { list: list_inner }
}

fn preview_for_selection(
    tools: &[Tool],
    filtered_ids: &[ToolId],
    selected: Option<usize>,
) -> String {
    let Some(i) = selected else {
        return "Select a tool (↑/↓ or click).".into();
    };
    let Some(id) = filtered_ids.get(i) else {
        return "Selection out of range.".into();
    };
    let Some(t) = tools.iter().find(|t| t.id == *id) else {
        return "Unknown tool id.".into();
    };

    format!(
        "{}\n{}\n\n—\nFuture: manifest CID, signature, Nix store path, and Merkle anchor for this entry.",
        t.name, t.description
    )
}
