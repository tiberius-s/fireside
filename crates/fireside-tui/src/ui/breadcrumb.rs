//! Navigation Context Bar for presenter mode.
//!
//! Renders a single-row bar that answers three questions a presenter needs:
//! "Where have I been?  Where am I now?  What's coming next?"
//!
//! ```text
//!  ◂ intro · basics  │  3 / 12 — Introduction to Fireside  │  → advanced-topics
//!  └ history (muted)    └ current (foam bold)                  └ lookahead (subtle)
//! ```

use fireside_engine::PresentationSession;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use unicode_width::UnicodeWidthStr;

use crate::theme::Theme;

/// Render the navigation context bar.
///
/// The bar has three visual zones separated by `│` dividers:
/// - **Left** (history): last 1–2 visited nodes in `footer` colour.
/// - **Centre** (position): `N / M — Title` with N/M in `toolbar_fg` and title in `heading_h1` bold.
/// - **Right** (lookahead): next node label, or `⎇ BRANCH` in gold, or `■ end` when at last node.
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

    let total = session.graph.nodes.len();
    let position = format!(" {}/{total} ", current_index + 1);

    // ── Centre zone: position + title ────────────────────────────────────
    let max_title_chars = (area.width as usize)
        .saturating_sub(position.width() + 30)
        .clamp(8, 40);
    let current_title = node_short_label(session, current_index, max_title_chars);

    // ── Left zone: last 1–2 ancestors from nav_path ───────────────────────
    // Collect unique path entries, excluding current.
    let mut ancestors: Vec<usize> = Vec::new();
    for (idx, _) in nav_path.iter().rev() {
        if *idx != current_index && ancestors.last() != Some(idx) {
            ancestors.push(*idx);
            if ancestors.len() == 2 {
                break;
            }
        }
    }
    ancestors.reverse();

    // ── Right zone: peek at what comes next ───────────────────────────────
    let current_node = &session.graph.nodes[current_index];
    let next_info = if current_node.branch_point().is_some() {
        // Branch ahead: warn the presenter in gold.
        NextInfo::Branch
    } else if let Some(target_id) = current_node.next_override() {
        if let Some(idx) = session.graph.index_of(target_id) {
            NextInfo::Node(idx)
        } else {
            NextInfo::End
        }
    } else if let Some(target_id) = current_node.after_target() {
        if let Some(idx) = session.graph.index_of(target_id) {
            NextInfo::Node(idx)
        } else {
            NextInfo::End
        }
    } else if current_index + 1 < total {
        NextInfo::Node(current_index + 1)
    } else {
        NextInfo::End
    };

    // ── Assemble spans ────────────────────────────────────────────────────
    let div = || Span::styled("  │  ", Style::default().fg(theme.footer));

    let mut spans = Vec::new();
    spans.push(Span::raw(" "));

    // Left: history
    if ancestors.is_empty() {
        spans.push(Span::styled("◂", Style::default().fg(theme.footer)));
    } else {
        spans.push(Span::styled("◂ ", Style::default().fg(theme.footer)));
        for (i, &idx) in ancestors.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" · ", Style::default().fg(theme.footer)));
            }
            let label = node_short_label(session, idx, 12);
            spans.push(Span::styled(label, Style::default().fg(theme.footer)));
        }
    }

    spans.push(div());

    // Centre: position + title
    spans.push(Span::styled(
        position,
        Style::default().fg(theme.toolbar_fg),
    ));
    spans.push(Span::styled("— ", Style::default().fg(theme.footer)));
    spans.push(Span::styled(
        current_title,
        Style::default()
            .fg(theme.heading_h1)
            .add_modifier(Modifier::BOLD),
    ));

    spans.push(div());

    // Right: lookahead
    match next_info {
        NextInfo::Node(idx) => {
            let label = node_short_label(session, idx, 16);
            spans.push(Span::styled("→ ", Style::default().fg(theme.toolbar_fg)));
            spans.push(Span::styled(label, Style::default().fg(theme.toolbar_fg)));
        }
        NextInfo::Branch => {
            spans.push(Span::styled(
                "⎇ BRANCH",
                Style::default()
                    .fg(theme.heading_h3)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        NextInfo::End => {
            spans.push(Span::styled("■ end", Style::default().fg(theme.footer)));
        }
    }

    spans.push(Span::raw(" "));

    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .block(Block::default().style(Style::default().bg(theme.toolbar_bg))),
        area,
    );
}

/// What comes after the current node.
enum NextInfo {
    Node(usize),
    Branch,
    End,
}

fn node_short_label(session: &PresentationSession, index: usize, max_chars: usize) -> String {
    let label = session
        .graph
        .nodes
        .get(index)
        .and_then(|node| node.id.as_deref().or(node.title.as_deref()))
        .map_or_else(|| format!("#{}", index + 1), ToOwned::to_owned);

    if label.width() <= max_chars {
        label
    } else {
        let mut width = 0usize;
        let mut out: String = label
            .chars()
            .take_while(|c| {
                let w = unicode_width::UnicodeWidthChar::width(*c).unwrap_or(0);
                if width + w < max_chars {
                    width += w;
                    true
                } else {
                    false
                }
            })
            .collect();
        out.push('…');
        out
    }
}
