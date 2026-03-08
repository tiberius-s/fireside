//! Divider block renderer.
//!
//! Renders a [`ContentBlock::Divider`] as a full-width horizontal rule
//! using the `─` box-drawing character styled in the inactive border colour.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::design::tokens::DesignTokens;

/// Render a horizontal rule spanning the full content width.
///
/// A centred `◇` diamond is flanked by `─` fills to produce a clear visual
/// break that stands apart from plain text and code borders.
pub(super) fn render_divider(width: u16, tokens: &DesignTokens) -> Vec<Line<'static>> {
    let style = Style::default().fg(tokens.border_inactive);
    let w = width.max(9) as usize;
    // Reserve 5 chars for " ◇ " (3) plus the two half-dashes on each side.
    let half = w.saturating_sub(3) / 2;
    let rule = format!("{} ◇ {}", "─".repeat(half), "─".repeat(half));
    vec![Line::from(Span::styled(rule, style))]
}
