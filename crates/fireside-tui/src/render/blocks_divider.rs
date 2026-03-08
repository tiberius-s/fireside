//! Divider block renderer.
//!
//! Renders a [`ContentBlock::Divider`] as a full-width horizontal rule
//! using the `─` box-drawing character styled in the inactive border colour.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::design::tokens::DesignTokens;

/// Render a horizontal rule spanning the full content width.
///
/// A centred `◇` diamond is flanked by `─` fills.  The fills use
/// `tokens.muted` for legibility (contrast > 3:1 on Rose Pine Base);
/// the diamond uses a slightly brighter `tokens.toolbar_fg` to catch the eye.
pub(super) fn render_divider(width: u16, tokens: &DesignTokens) -> Vec<Line<'static>> {
    let rule_style = Style::default().fg(tokens.muted);
    let gem_style = Style::default().fg(tokens.toolbar_fg);
    let w = width.max(9) as usize;
    // Reserve 3 chars for " ◇ " gem segment.
    let half = w.saturating_sub(3) / 2;
    let fill = "─".repeat(half);
    vec![Line::from(vec![
        Span::styled(fill.clone(), rule_style),
        Span::styled(" ◇ ", gem_style),
        Span::styled(fill, rule_style),
    ])]
}
