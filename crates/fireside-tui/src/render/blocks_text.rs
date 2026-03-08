//! Text block renderer.
//!
//! Renders a [`ContentBlock::Text`] body into word-wrapped ratatui [`Line`]s,
//! with a muted `¶` pilcrow prefix on the first line for visual identity.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::design::tokens::DesignTokens;

/// Render a body string into word-wrapped styled lines.
///
/// The first line is prefixed with a muted `¶` pilcrow mark so text blocks
/// have a visual identity that matches the `¶` glyph shown in the editor
/// block list.  Subsequent lines get a matching 2-space indent.
pub(super) fn render_text<'a>(text: &'a str, tokens: &DesignTokens, width: u16) -> Vec<Line<'a>> {
    let style = Style::default().fg(tokens.body);
    let pilcrow_style = Style::default().fg(tokens.muted);
    const INDENT: &str = "  ";
    // "¶ " = 2 display cols; subtract from wrap width so lines align.
    let wrap_width = (width.max(6) as usize)
        .saturating_sub(INDENT.len() + 2)
        .max(1);
    let wrapped = textwrap::wrap(text, wrap_width);
    wrapped
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            let prefix: Span<'static> = if i == 0 {
                Span::styled("¶ ", pilcrow_style)
            } else {
                Span::raw("  ")
            };
            Line::from(vec![
                Span::raw(INDENT),
                prefix,
                Span::styled(line.into_owned(), style),
            ])
        })
        .collect()
}
