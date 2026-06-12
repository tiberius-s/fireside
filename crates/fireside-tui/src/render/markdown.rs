//! Minimal inline-Markdown rendering for prose text.
//!
//! The spec says `text.body` "may contain inline Markdown formatting" but
//! does not pin a subset, so this engine supports exactly three spans —
//! `**bold**`, `*italic*`, and `` `code` `` — and renders unmatched markers
//! literally. Output is width-wrapped styled lines, because ratatui's
//! `Paragraph` wrapping cannot be measured ahead of layout.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use crate::theme::Tokens;

/// A styled fragment produced by the parser.
#[derive(Debug, Clone, PartialEq)]
struct Fragment {
    text: String,
    style: Style,
}

/// Parse inline markers in `text`, then wrap to `width` columns.
///
/// Explicit newlines in the source are respected as line breaks. A width of
/// zero yields no lines.
#[must_use]
pub fn wrap_styled(text: &str, width: u16, base: Style, tokens: &Tokens) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let fragments = parse_inline(paragraph, base, tokens);
        lines.extend(wrap_fragments(&fragments, width));
    }
    lines
}

/// Parse one paragraph into styled fragments.
fn parse_inline(text: &str, base: Style, tokens: &Tokens) -> Vec<Fragment> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<Fragment> = Vec::new();
    let mut plain = String::new();
    let mut i = 0;

    let push_plain = |buf: &mut String, out: &mut Vec<Fragment>| {
        if !buf.is_empty() {
            out.push(Fragment {
                text: std::mem::take(buf),
                style: base,
            });
        }
    };

    while i < chars.len() {
        let (marker, style): (&str, Style) = if chars[i..].starts_with(&['*', '*']) {
            ("**", base.add_modifier(Modifier::BOLD))
        } else if chars[i] == '*' {
            ("*", base.add_modifier(Modifier::ITALIC))
        } else if chars[i] == '`' {
            ("`", tokens.code)
        } else {
            plain.push(chars[i]);
            i += 1;
            continue;
        };

        let marker_len = marker.chars().count();
        match find_closer(&chars, i + marker_len, marker) {
            Some(close) => {
                push_plain(&mut plain, &mut out);
                let inner: String = chars[i + marker_len..close].iter().collect();
                out.push(Fragment { text: inner, style });
                i = close + marker_len;
            }
            None => {
                // No closing marker: the characters are literal text.
                plain.push_str(marker);
                i += marker_len;
            }
        }
    }
    push_plain(&mut plain, &mut out);
    out
}

/// Find the index of the next `marker` occurrence at or after `from`,
/// skipping empty spans (so `**` is not read as an empty italic).
fn find_closer(chars: &[char], from: usize, marker: &str) -> Option<usize> {
    let m: Vec<char> = marker.chars().collect();
    let mut i = from;
    while i + m.len() <= chars.len() {
        if chars[i..i + m.len()] == m[..] && i > from {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Greedy word-wrap over styled fragments.
fn wrap_fragments(fragments: &[Fragment], width: u16) -> Vec<Line<'static>> {
    let width = width as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut used = 0usize;

    let mut flush = |current: &mut Vec<Span<'static>>, used: &mut usize| {
        lines.push(Line::from(std::mem::take(current)));
        *used = 0;
    };

    for frag in fragments {
        for word in frag.text.split(' ') {
            if word.is_empty() {
                continue;
            }
            let mut word = word.to_owned();
            let mut w = word.width();
            // Hard-break words wider than the whole line.
            while w > width {
                let cut: String = word.chars().take(width).collect();
                let rest: String = word.chars().skip(width).collect();
                if used > 0 {
                    flush(&mut current, &mut used);
                }
                current.push(Span::styled(cut, frag.style));
                flush(&mut current, &mut used);
                word = rest;
                w = word.width();
            }
            if word.is_empty() {
                continue;
            }
            let need = if used == 0 { w } else { w + 1 };
            if used + need > width && used > 0 {
                flush(&mut current, &mut used);
            }
            if used > 0 {
                current.push(Span::styled(" ".to_owned(), frag.style));
                used += 1;
            }
            used += w;
            current.push(Span::styled(word, frag.style));
        }
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(Line::from(current));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(text: &str, width: u16) -> Vec<String> {
        let tokens = Tokens::default();
        wrap_styled(text, width, Style::new(), &tokens)
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
            .collect()
    }

    #[test]
    fn plain_text_wraps_at_word_boundaries() {
        assert_eq!(render("alpha beta gamma", 11), ["alpha beta", "gamma"]);
    }

    #[test]
    fn bold_italic_and_code_markers_are_stripped() {
        assert_eq!(render("**bold** *it* `code`", 40), ["bold it code"]);
    }

    #[test]
    fn bold_fragment_carries_bold_style() {
        let tokens = Tokens::default();
        let lines = wrap_styled("**hi** there", 40, Style::new(), &tokens);
        let span = &lines[0].spans[0];
        assert_eq!(span.content.as_ref(), "hi");
        assert!(span.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn unmatched_markers_render_literally() {
        assert_eq!(render("2 * 3 = 6", 40), ["2 * 3 = 6"]);
        assert_eq!(render("*open", 40), ["*open"]);
    }

    #[test]
    fn newlines_break_lines() {
        assert_eq!(render("one\ntwo", 40), ["one", "two"]);
    }

    #[test]
    fn long_words_hard_break() {
        assert_eq!(render("abcdefghij", 4), ["abcd", "efgh", "ij"]);
    }
}
