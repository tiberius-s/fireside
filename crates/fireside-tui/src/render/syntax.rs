//! Syntax colors for code blocks.
//!
//! syntect does the parsing; the colors come from [`Tokens`], not from a
//! syntect theme. Mapping scopes onto the ANSI palette keeps code readable
//! on any terminal background and visually consistent with the rest of the
//! presenter — the same reason the theme never hardcodes RGB.

use std::sync::OnceLock;

use ratatui::style::Style;
use ratatui::text::Span;
use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};

use crate::theme::Tokens;

/// The bundled syntax definitions (syntect defaults plus two-face extras),
/// loaded once per process.
fn syntax_set() -> &'static SyntaxSet {
    static SET: OnceLock<SyntaxSet> = OnceLock::new();
    SET.get_or_init(two_face::syntax::extra_newlines)
}

/// Highlight `source` as `language`, one row of styled spans per source
/// line. Returns `None` when the language is unknown or parsing fails —
/// the caller falls back to plain code styling.
#[must_use]
pub fn highlight(
    language: Option<&str>,
    source: &str,
    tokens: &Tokens,
) -> Option<Vec<Vec<Span<'static>>>> {
    let set = syntax_set();
    let syntax = language.and_then(|l| set.find_syntax_by_token(l))?;
    let mut parse = ParseState::new(syntax);
    let mut stack = ScopeStack::new();

    let mut rows = Vec::new();
    for line in source.lines() {
        // The grammars expect a trailing newline on every parsed line.
        let text = format!("{line}\n");
        let ops = parse.parse_line(&text, set).ok()?;

        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut cursor = 0usize;
        for (offset, op) in &ops {
            let end = (*offset).min(line.len());
            if end > cursor {
                spans.push(Span::styled(
                    line[cursor..end].to_owned(),
                    style_for(&stack, tokens),
                ));
                cursor = end;
            }
            stack.apply(op).ok()?;
        }
        if cursor < line.len() {
            spans.push(Span::styled(
                line[cursor..].to_owned(),
                style_for(&stack, tokens),
            ));
        }
        rows.push(spans);
    }
    Some(rows)
}

/// The token style for the innermost recognized scope on the stack.
fn style_for(stack: &ScopeStack, tokens: &Tokens) -> Style {
    for scope in stack.as_slice().iter().rev() {
        let name = scope.build_string();
        if name.starts_with("comment") {
            return tokens.code_comment;
        }
        if name.starts_with("string") {
            return tokens.code_string;
        }
        if name.starts_with("constant") {
            return tokens.code_constant;
        }
        if name.starts_with("keyword") || name.starts_with("storage") {
            return tokens.code_keyword;
        }
        if name.starts_with("entity.name.function") || name.starts_with("support.function") {
            return tokens.code_function;
        }
        if name.starts_with("entity.name")
            || name.starts_with("support.type")
            || name.starts_with("support.class")
        {
            return tokens.code_type;
        }
    }
    tokens.code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_keywords_strings_and_comments_get_distinct_styles() {
        let tokens = Tokens::default();
        let rows = highlight(
            Some("rust"),
            "// hi\nfn main() { let s = \"x\"; }",
            &tokens,
        )
        .expect("rust is a known language");
        assert_eq!(rows.len(), 2);

        let style_of = |row: &[Span<'_>], needle: &str| {
            row.iter()
                .find(|s| s.content.contains(needle))
                .map(|s| s.style)
                .unwrap_or_else(|| panic!("span containing {needle:?}"))
        };
        assert_eq!(style_of(&rows[0], "hi"), tokens.code_comment);
        assert_eq!(style_of(&rows[1], "fn"), tokens.code_keyword);
        assert_eq!(style_of(&rows[1], "x"), tokens.code_string);
    }

    #[test]
    fn unknown_language_returns_none() {
        let tokens = Tokens::default();
        assert!(highlight(Some("not-a-language"), "hi", &tokens).is_none());
        assert!(highlight(None, "hi", &tokens).is_none());
    }

    #[test]
    fn row_text_reassembles_the_source_exactly() {
        let tokens = Tokens::default();
        let source = "fn add(a: u32) -> u32 {\n    a + 1\n}";
        let rows = highlight(Some("rust"), source, &tokens).expect("rust");
        let rebuilt: Vec<String> = rows
            .iter()
            .map(|row| row.iter().map(|s| s.content.as_ref()).collect())
            .collect();
        let expected: Vec<&str> = source.lines().collect();
        assert_eq!(rebuilt, expected);
    }
}
