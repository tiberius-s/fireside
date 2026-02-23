//! Breadcrumb navigation trail for presenter mode.

use fireside_engine::PresentationSession;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::theme::Theme;

/// Render a breadcrumb trail from navigation history.
pub fn render_breadcrumb(
    frame: &mut Frame,
    area: Rect,
    session: &PresentationSession,
    nav_path: &[(usize, bool)],
    current_index: usize,
    theme: &Theme,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let mut path = Vec::new();
    for (idx, branch_step) in nav_path.iter().copied() {
        if path.last().is_none_or(|(last_idx, _)| *last_idx != idx) {
            path.push((idx, branch_step));
        }
    }
    if path.last().map(|(idx, _)| *idx) != Some(current_index) {
        path.push((current_index, false));
    }

    let mut spans = vec![Span::styled(
        " path: ",
        Style::default().fg(theme.toolbar_fg),
    )];

    let mut used = 6usize;
    let max_width = usize::from(area.width);

    let mut rev_segments = Vec::new();
    for (i, (idx, branch_step)) in path.iter().enumerate().rev() {
        let label = node_short_label(session, *idx, 12);
        let seg_len = label.chars().count() + if i == 0 { 0 } else { 3 };
        if used + seg_len > max_width && !rev_segments.is_empty() {
            break;
        }
        rev_segments.push((i, *idx, *branch_step, label));
        used += seg_len;
    }
    rev_segments.reverse();

    if rev_segments.len() < path.len() {
        spans.push(Span::styled("… ", Style::default().fg(theme.footer)));
    }

    for (pos, (_orig_pos, idx, _branch_step, label)) in rev_segments.iter().enumerate() {
        if pos > 0 {
            let prev_is_branch = rev_segments[pos].2;
            spans.push(Span::styled(
                if prev_is_branch { "⎇ " } else { "→ " },
                if prev_is_branch {
                    Style::default().fg(theme.heading_h3)
                } else {
                    Style::default().fg(theme.footer)
                },
            ));
        }

        let style = if *idx == current_index {
            Style::default()
                .fg(theme.heading_h1)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.foreground)
        };
        spans.push(Span::styled(label.clone(), style));
        if pos + 1 < rev_segments.len() {
            spans.push(Span::raw(" "));
        }
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
