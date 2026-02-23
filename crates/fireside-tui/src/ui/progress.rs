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
///  [←] prev   ○ ○ ● ○ ○ ○ ○   Node 3 / 7   → Next Topic   [→] next
/// ```
/// Each `●` / `○` is one segment coloured in `border_active` / `border_inactive`.
/// When no next node is available, an end marker is shown as `■ END`.
/// When there are more nodes than `MAX_SEGMENTS`, multiple nodes share a segment.
pub fn render_progress_bar(
    frame: &mut Frame,
    area: Rect,
    session: &PresentationSession,
    elapsed_secs: u64,
    show_timer: bool,
    target_duration_secs: Option<u64>,
    theme: &Theme,
) {
    let current = session.current_node_index();
    let total = session.graph.nodes.len();
    let current_node = &session.graph.nodes[current];

    // ── Fixed spans ───────────────────────────────────────────────────────
    let (left_hint, right_hint) = footer_hints(area.width, current_node.branch_point().is_some());

    let position_str = format!(" Node {} / {} ", current + 1, total);

    let next_index = resolve_next_node_index(session, current);
    let next_str = if let Some(index) = next_index {
        let next_node = &session.graph.nodes[index];
        let label = next_node
            .title
            .as_deref()
            .or(next_node.id.as_deref())
            .map_or_else(|| format!("node-{}", index + 1), ToOwned::to_owned);
        format!(" → {label} ")
    } else {
        " ■ END ".to_string()
    };

    let timer_str = if show_timer {
        if let Some(target_secs) = target_duration_secs {
            let elapsed_minutes = elapsed_secs / 60;
            let elapsed_seconds = elapsed_secs % 60;
            let target_minutes = target_secs / 60;
            let target_seconds = target_secs % 60;
            format!(
                " {:02}:{:02} / {:02}:{:02} {} ",
                elapsed_minutes,
                elapsed_seconds,
                target_minutes,
                target_seconds,
                pace_label(elapsed_secs, target_secs)
            )
        } else {
            let minutes = elapsed_secs / 60;
            let seconds = elapsed_secs % 60;
            format!(" {:02}:{:02} ", minutes, seconds)
        }
    } else {
        String::new()
    };

    // ── Segment calculation ───────────────────────────────────────────────
    // Available width for the segment dots sits between the fixed elements.
    let fixed_chars = left_hint.chars().count()
        + right_hint.chars().count()
        + position_str.chars().count()
        + next_str.chars().count()
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
    let next_style = Style::default().fg(theme.toolbar_fg);
    let end_style = Style::default().fg(theme.error);

    let mut spans: Vec<Span> = Vec::with_capacity(4 + segment_count * 2);
    spans.push(Span::styled(left_hint, hint_style));

    for i in 0..segment_count {
        if i > 0 {
            spans.push(Span::styled(" ", Style::default().bg(theme.toolbar_bg)));
        }
        let is_branch_ahead = any_node_in_bucket_is_branch(i, total, segment_count, session);
        let (seg_glyph, seg_style) = if i == active_seg {
            ("●", Style::default().fg(theme.border_active))
        } else if is_branch_ahead {
            ("⎇", Style::default().fg(theme.heading_h3))
        } else {
            ("○", Style::default().fg(theme.border_inactive))
        };
        spans.push(Span::styled(seg_glyph, seg_style));
    }

    spans.push(Span::styled(position_str, pos_style));
    spans.push(Span::styled(
        next_str,
        if next_index.is_some() {
            next_style
        } else {
            end_style
        },
    ));
    if !timer_str.is_empty() {
        let timer_style = if let Some(target_secs) = target_duration_secs {
            Style::default().fg(pace_color(elapsed_secs, target_secs, theme))
        } else {
            hint_style
        };
        spans.push(Span::styled(timer_str, timer_style));
    }
    spans.push(Span::styled(right_hint, hint_style));

    let line = Line::from(spans);
    frame.render_widget(
        Paragraph::new(line).block(Block::default().style(Style::default().bg(theme.toolbar_bg))),
        area,
    );
}

fn pace_color(elapsed_secs: u64, target_secs: u64, theme: &Theme) -> ratatui::style::Color {
    if target_secs == 0 {
        return theme.error;
    }
    let ratio = elapsed_secs as f64 / target_secs as f64;
    if ratio <= 1.0 {
        theme.success
    } else if ratio <= 1.1 {
        theme.heading_h3
    } else {
        theme.error
    }
}

fn pace_label(elapsed_secs: u64, target_secs: u64) -> &'static str {
    if target_secs == 0 {
        return "● Over";
    }
    let ratio = elapsed_secs as f64 / target_secs as f64;
    if ratio <= 1.0 {
        "● On pace"
    } else if ratio <= 1.1 {
        "● Slightly behind"
    } else {
        "● Over"
    }
}

#[cfg(test)]
mod tests {
    use super::{pace_color, pace_label};
    use crate::theme::Theme;

    #[test]
    fn pace_label_thresholds() {
        assert_eq!(pace_label(300, 600), "● On pace");
        assert_eq!(pace_label(630, 600), "● Slightly behind");
        assert_eq!(pace_label(720, 600), "● Over");
    }

    #[test]
    fn pace_color_uses_theme_roles() {
        let theme = Theme::default();
        assert_eq!(pace_color(300, 600, &theme), theme.success);
        assert_eq!(pace_color(630, 600, &theme), theme.heading_h3);
        assert_eq!(pace_color(720, 600, &theme), theme.error);
    }
}

fn resolve_next_node_index(session: &PresentationSession, current: usize) -> Option<usize> {
    let node = &session.graph.nodes[current];

    if let Some(target_id) = node.next_override()
        && let Some(idx) = session.graph.index_of(target_id)
    {
        return Some(idx);
    }

    if let Some(target_id) = node.after_target()
        && let Some(idx) = session.graph.index_of(target_id)
    {
        return Some(idx);
    }

    let sequential = current + 1;
    (sequential < session.graph.nodes.len()).then_some(sequential)
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
