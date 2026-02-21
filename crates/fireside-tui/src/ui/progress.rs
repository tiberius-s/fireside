//! Progress bar widget showing node position and elapsed time.
//!
//! Renders a footer bar with navigation hints on each side, a row of coloured
//! segment dots indicating the current position through the presentation, and
//! an optional elapsed-time counter.

use fireside_engine::PresentationSession;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::theme::Theme;

/// Maximum number of visible segment dots regardless of total node count.
const MAX_SEGMENTS: usize = 30;

/// Render the progress bar in the footer area.
///
/// Layout (left → right):
/// ```text
///  [k] prev   ░ ░ █ ░ ░ ░ ░   3 / 7   [j] next
/// ```
/// Each `█` / `░` is one segment coloured in `border_active` / `border_inactive`.
/// When there are more nodes than `MAX_SEGMENTS`, multiple nodes share a segment.
pub fn render_progress_bar(
    frame: &mut Frame,
    area: Rect,
    session: &PresentationSession,
    elapsed_secs: u64,
    show_timer: bool,
    theme: &Theme,
) {
    let current = session.current_node_index();
    let total = session.graph.nodes.len();
    let current_node = &session.graph.nodes[current];

    // ── Marker suffixes for branch / override nodes ───────────────────────
    let branch_marker = if current_node.branch_point().is_some() {
        " ⎇"
    } else {
        ""
    };
    let traversal_marker = if current_node.next_override().is_some() {
        " ↪"
    } else {
        ""
    };

    // ── Fixed spans ───────────────────────────────────────────────────────
    let (left_hint, right_hint) = footer_hints(area.width, current_node.branch_point().is_some());

    let position_str = format!(
        " {}/{}{}{} ",
        current + 1,
        total,
        branch_marker,
        traversal_marker
    );

    let timer_str = if show_timer {
        let minutes = elapsed_secs / 60;
        let seconds = elapsed_secs % 60;
        format!(" {:02}:{:02} ", minutes, seconds)
    } else {
        String::new()
    };

    // ── Segment calculation ───────────────────────────────────────────────
    // Available width for the segment dots sits between the fixed elements.
    let fixed_chars = left_hint.chars().count()
        + right_hint.chars().count()
        + position_str.chars().count()
        + timer_str.chars().count();
    let area_width = area.width as usize;
    let segment_space = area_width.saturating_sub(fixed_chars);

    // Each visible segment occupies 1 char; gaps between segments are 1 space.
    // n segments need n + (n-1) = 2n-1 chars → n = (space+1)/2.
    let segment_count = if total == 0 {
        0
    } else {
        let n = segment_space.div_ceil(2);
        n.max(1).min(total).min(MAX_SEGMENTS)
    };

    // Which segment corresponds to the current node?
    let active_seg = if total <= 1 || segment_count == 0 {
        0
    } else {
        (current * segment_count) / total.max(1)
    };

    // ── Build the line ────────────────────────────────────────────────────
    let hint_style = Style::default().fg(theme.footer);
    let pos_style = Style::default()
        .fg(theme.on_surface)
        .add_modifier(Modifier::BOLD);

    let mut spans: Vec<Span> = Vec::with_capacity(4 + segment_count * 2);
    spans.push(Span::styled(left_hint, hint_style));

    for i in 0..segment_count {
        if i > 0 {
            spans.push(Span::styled(" ", Style::default().bg(theme.toolbar_bg)));
        }
        let seg_style = if i == active_seg {
            Style::default().fg(theme.border_active)
        } else if any_node_in_bucket_is_branch(i, total, segment_count, session) {
            Style::default().fg(theme.heading_h3)
        } else {
            Style::default().fg(theme.border_inactive)
        };
        spans.push(Span::styled("█", seg_style));
    }

    spans.push(Span::styled(position_str, pos_style));
    if !timer_str.is_empty() {
        spans.push(Span::styled(timer_str, hint_style));
    }
    spans.push(Span::styled(right_hint, hint_style));

    let line = Line::from(spans);
    frame.render_widget(
        Paragraph::new(line).block(Block::default().style(Style::default().bg(theme.toolbar_bg))),
        area,
    );
}

fn footer_hints(width: u16, is_branch: bool) -> (String, String) {
    if width <= 80 {
        return (
            " ← ? ".to_string(),
            if is_branch {
                " ⎇ ".to_string()
            } else {
                " → e ".to_string()
            },
        );
    }

    if width < 120 {
        return (
            " [←] prev ".to_string(),
            if is_branch {
                " ⎇ BRANCH ".to_string()
            } else {
                " [→] next  [?] help ".to_string()
            },
        );
    }

    (
        " [←] prev ".to_string(),
        if is_branch {
            " ⎇ BRANCH  ·  [?] help  ·  [e] edit ".to_string()
        } else {
            " next [→]  ·  [?] help  ·  [e] edit ".to_string()
        },
    )
}

fn any_node_in_bucket_is_branch(
    seg: usize,
    total: usize,
    count: usize,
    session: &PresentationSession,
) -> bool {
    if total == 0 || count == 0 {
        return false;
    }
    let start = (seg * total) / count;
    let end = ((seg + 1) * total).div_ceil(count).min(total);
    (start..end).any(|idx| {
        session
            .graph
            .nodes
            .get(idx)
            .is_some_and(|node| node.branch_point().is_some())
    })
}
