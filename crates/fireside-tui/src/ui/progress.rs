//! Progress bar widget showing node position and elapsed time.

use fireside_engine::PresentationSession;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::design::tokens::DesignTokens;
use crate::theme::Theme;

/// Render the progress bar in the footer area.
pub fn render_progress_bar(
    frame: &mut Frame,
    area: Rect,
    session: &PresentationSession,
    elapsed_secs: u64,
    show_timer: bool,
    theme: &Theme,
) {
    let tokens = DesignTokens::from_theme(theme);
    let current = session.current_node_index();
    let total = session.graph.nodes.len();
    let current_node = &session.graph.nodes[current];

    let style = Style::default().fg(tokens.footer);
    let bold = style.add_modifier(Modifier::BOLD);

    let minutes = elapsed_secs / 60;
    let seconds = elapsed_secs % 60;
    let node_id = current_node
        .id
        .as_deref()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("#{}", current + 1));

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

    let gauge_width = 10usize;
    let filled = if total == 0 {
        0
    } else {
        ((current + 1) * gauge_width) / total.max(1)
    }
    .min(gauge_width);
    let gauge = format!(
        "[{}{}]",
        "▓".repeat(filled),
        "░".repeat(gauge_width.saturating_sub(filled))
    );

    let node_info = format!(
        " {gauge} {node_id} ({}/{}){branch_marker}{traversal_marker} ",
        current + 1,
        total
    );
    let breadcrumbs = build_breadcrumbs(session);
    let time_info = if show_timer {
        format!(" {minutes:02}:{seconds:02} ")
    } else {
        String::new()
    };

    let width = area.width as usize;
    let available_for_middle = width.saturating_sub(node_info.len() + time_info.len());
    let breadcrumb_info = truncate_middle(&breadcrumbs, available_for_middle);
    let padding_len =
        width.saturating_sub(node_info.len() + breadcrumb_info.len() + time_info.len());
    let padding = " ".repeat(padding_len);

    let line = Line::from(vec![
        Span::styled(node_info, bold),
        Span::styled(breadcrumb_info, style),
        Span::styled(padding, style),
        Span::styled(time_info, style),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn build_breadcrumbs(session: &PresentationSession) -> String {
    let mut labels = Vec::new();

    for idx in session
        .traversal
        .history()
        .iter()
        .copied()
        .rev()
        .take(4)
        .rev()
    {
        labels.push(node_label(session, idx));
    }
    labels.push(node_label(session, session.current_node_index()));

    if labels.is_empty() {
        return String::new();
    }

    format!(" {} ", labels.join(" → "))
}

fn node_label(session: &PresentationSession, idx: usize) -> String {
    session
        .graph
        .nodes
        .get(idx)
        .and_then(|node| node.id.as_deref())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("#{}", idx + 1))
}

fn truncate_middle(text: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let count = text.chars().count();
    if count <= max {
        return text.to_string();
    }
    if max <= 1 {
        return "…".to_string();
    }

    let left = (max - 1) / 2;
    let right = max - 1 - left;

    let left_part: String = text.chars().take(left).collect();
    let right_part: String = text
        .chars()
        .rev()
        .take(right)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    format!("{left_part}…{right_part}")
}
