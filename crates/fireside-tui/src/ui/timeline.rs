//! Session timeline strip for presenter mode.

use fireside_engine::PresentationSession;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::theme::Theme;

/// Render a compact timeline strip with recently visited nodes.
pub fn render_timeline(
    frame: &mut Frame,
    area: Rect,
    session: &PresentationSession,
    visited_nodes: &[usize],
    current_index: usize,
    theme: &Theme,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let mut sequence = Vec::new();
    for index in visited_nodes.iter().copied() {
        if sequence.last().copied() != Some(index) {
            sequence.push(index);
        }
    }
    if sequence.last().copied() != Some(current_index) {
        sequence.push(current_index);
    }

    let max_items = usize::from((area.width / 8).max(3));
    let start = sequence.len().saturating_sub(max_items);
    let slice = &sequence[start..];

    let mut spans = Vec::new();
    spans.push(Span::styled(
        " timeline: ",
        Style::default().fg(theme.toolbar_fg),
    ));

    for (idx, node_index) in slice.iter().copied().enumerate() {
        if idx > 0 {
            let prev = slice[idx - 1];
            let branch_sep = session
                .graph
                .nodes
                .get(prev)
                .is_some_and(|node| node.branch_point().is_some());
            spans.push(Span::styled(
                if branch_sep { " ⎇ " } else { " · " },
                if branch_sep {
                    Style::default().fg(theme.heading_h3)
                } else {
                    Style::default().fg(theme.footer)
                },
            ));
        }

        let label = node_short_label(session, node_index, 10);
        let style = if node_index == current_index {
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.foreground)
        };
        spans.push(Span::styled(label, style));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .block(Block::default().style(Style::default().bg(theme.toolbar_bg))),
        area,
    );
}

fn node_short_label(session: &PresentationSession, index: usize, max_chars: usize) -> String {
    let label = session
        .graph
        .nodes
        .get(index)
        .and_then(|node| node.id.as_deref().or(node.title.as_deref()))
        .map_or_else(|| format!("#{}", index + 1), ToOwned::to_owned);

    if label.chars().count() <= max_chars {
        label
    } else {
        let mut out = label
            .chars()
            .take(max_chars.saturating_sub(1))
            .collect::<String>();
        out.push('…');
        out
    }
}
