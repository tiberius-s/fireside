//! List block renderer.
//!
//! Renders a [`ContentBlock::List`] into ratatui [`Line`]s with
//! depth-sensitive bullet glyphs (`•`, `◦`, `▪`) and recursive nesting
//! for child items.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use fireside_core::model::content::ListItem;

use crate::design::tokens::DesignTokens;

/// Render an ordered or unordered list at the given nesting depth.
///
/// Call with `depth = 0` for top-level lists; the function recurses for
/// children.
pub(super) fn render_list<'a>(
    ordered: bool,
    items: &'a [ListItem],
    tokens: &DesignTokens,
    depth: usize,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let style = Style::default().fg(tokens.body);

    let bullet = match depth {
        0 => "•",
        1 => "◦",
        _ => "▪",
    };

    for (i, item) in items.iter().enumerate() {
        let guide = if depth == 0 {
            String::new()
        } else {
            "│ ".repeat(depth)
        };

        let marker = if ordered {
            format!("{guide}{}. ", i + 1)
        } else {
            format!("{guide}{bullet} ")
        };

        lines.push(Line::from(vec![
            Span::styled(marker, style.add_modifier(Modifier::DIM)),
            Span::styled(item.text.clone(), style),
        ]));

        if !item.children.is_empty() {
            lines.extend(render_list(false, &item.children, tokens, depth + 1));
        }
    }

    lines
}
