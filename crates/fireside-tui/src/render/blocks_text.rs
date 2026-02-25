//! Text block renderer and plain-text helpers.
//!
//! Renders a [`ContentBlock::Text`] body into word-wrapped ratatui [`Line`]s.
//! Also provides helpers for extracting plain text from styled [`Line`]s and
//! for padding/truncating strings to a fixed column width — used by the
//! container split-horizontal renderer in [`super::blocks`].

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::design::tokens::DesignTokens;

/// Render a body string into word-wrapped styled lines.
pub(super) fn render_text<'a>(text: &'a str, tokens: &DesignTokens, width: u16) -> Vec<Line<'a>> {
    let style = Style::default().fg(tokens.body);
    let wrapped = textwrap::wrap(text, width.max(1) as usize);
    wrapped
        .into_iter()
        .map(|line| Line::from(Span::styled(line.into_owned(), style)))
        .collect()
}

/// Extract a plain-text string from a styled [`Line`] by joining all span
/// contents.
pub(super) fn line_to_plain_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<Vec<_>>()
        .join("")
}

/// Pad or truncate `text` so that it occupies exactly `max_chars` columns.
pub(super) fn fit_to_width(text: &str, max_chars: usize) -> String {
    if text.chars().count() > max_chars {
        return truncate_text(text, max_chars);
    }

    let pad = max_chars.saturating_sub(text.chars().count());
    format!("{text}{}", " ".repeat(pad))
}

/// Truncate `text` to `max_chars` columns, appending `…` if needed.
pub(super) fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let short: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{short}…")
}
