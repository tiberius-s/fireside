//! Syntax highlighting for code blocks using syntect + two-face.
//!
//! Converts source code into styled ratatui `Line`s with color information
//! derived from the specified syntax theme.

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::util::LinesWithEndings;

/// Highlight source code and return styled ratatui `Line`s.
///
/// Returns `None` if the language is not recognized by syntect.
///
/// # Arguments
///
/// * `source` - The source code to highlight.
/// * `language` - The language identifier (e.g. `"rust"`, `"python"`).
/// * `theme_name` - The syntect theme name (e.g. `"base16-ocean.dark"`).
#[must_use]
pub fn highlight_code<'a>(
    source: &'a str,
    language: &str,
    theme_name: &str,
) -> Option<Vec<Line<'a>>> {
    let ss = two_face::syntax::extra_newlines();
    let ts: ThemeSet = two_face::theme::extra().into();

    let syntax = ss
        .find_syntax_by_token(language)
        .or_else(|| ss.find_syntax_by_extension(language))?;

    let theme = ts
        .themes
        .get(theme_name)
        .or_else(|| ts.themes.values().next())?;

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut lines = Vec::new();

    for line_str in LinesWithEndings::from(source) {
        match highlighter.highlight_line(line_str, &ss) {
            Ok(ranges) => {
                let spans: Vec<Span<'_>> = ranges
                    .iter()
                    .map(|(style, text)| {
                        let fg =
                            Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                        Span::styled(text.to_string(), Style::default().fg(fg))
                    })
                    .collect();
                lines.push(Line::from(spans));
            }
            Err(_) => {
                // Fallback: render unhighlighted
                lines.push(Line::from(line_str.to_owned()));
            }
        }
    }

    Some(lines)
}

/// List available syntax highlighting languages.
#[must_use]
pub fn available_languages() -> Vec<String> {
    let ss = two_face::syntax::extra_newlines();
    ss.syntaxes().iter().map(|s| s.name.clone()).collect()
}

/// List available syntax highlighting themes.
#[must_use]
pub fn available_themes() -> Vec<String> {
    let ts: ThemeSet = two_face::theme::extra().into();
    ts.themes.keys().cloned().collect()
}
