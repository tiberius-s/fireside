//! List block renderer.
//!
//! Renders a [`ContentBlock::List`] into ratatui [`Line`]s with
//! depth-sensitive bullet glyphs (`•`, `◦`, `▪`) and recursive nesting
//! for child items.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use fireside_core::model::content::ListItem;

use crate::design::tokens::DesignTokens;

/// Render an ordered or unordered list at the given nesting depth.
///
/// Call with `depth = 0` for top-level lists; the function recurses for
/// children.  Bullets and numbers are coloured in the accent palette to
/// visually separate them from item text.
pub(super) fn render_list<'a>(
    ordered: bool,
    items: &'a [ListItem],
    tokens: &DesignTokens,
    depth: usize,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let item_style = Style::default().fg(tokens.body);

    // Accent colour for markers decreases with nesting depth.
    let marker_style = match depth {
        0 => Style::default().fg(tokens.heading_h2),
        1 => Style::default().fg(tokens.heading_h3),
        _ => Style::default().fg(tokens.muted),
    };

    let bullet = match depth {
        0 => "•",
        1 => "◦",
        _ => "▪",
    };

    // 2 spaces of base indent + 2 per extra depth level.
    let indent: String = "  ".repeat(depth + 1);

    for (i, item) in items.iter().enumerate() {
        let glyph = if ordered {
            format!("{}. ", i + 1)
        } else {
            format!("{bullet} ")
        };

        lines.push(Line::from(vec![
            Span::raw(indent.clone()),
            Span::styled(glyph, marker_style),
            Span::styled(item.text.clone(), item_style),
        ]));

        if !item.children.is_empty() {
            lines.extend(render_list(false, &item.children, tokens, depth + 1));
        }
    }

    lines
}
