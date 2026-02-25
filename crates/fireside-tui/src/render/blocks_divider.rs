//! Divider block renderer.
//!
//! Renders a [`ContentBlock::Divider`] as a full-width horizontal rule
//! using the `─` box-drawing character styled in the inactive border colour.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::design::tokens::DesignTokens;

/// Render a horizontal rule spanning the full content width.
pub(super) fn render_divider(width: u16, tokens: &DesignTokens) -> Vec<Line<'static>> {
    let style = Style::default().fg(tokens.border_inactive);
    vec![Line::from(Span::styled(
        "─".repeat(width.max(1) as usize),
        style,
    ))]
}
