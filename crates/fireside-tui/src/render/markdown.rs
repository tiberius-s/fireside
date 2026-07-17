//! Minimal inline-Markdown rendering for prose text.
//!
//! The spec says `text.body` "may contain inline Markdown formatting" but
//! does not pin a subset, so this engine supports `**bold**`, `*italic*`,
//! `` `code` ``, and `[label](url)` links (contracts/link-syntax.md),
//! rendering unmatched markers literally. Output is width-wrapped styled
//! lines, because ratatui's `Paragraph` wrapping cannot be measured ahead
//! of layout.

use std::cell::RefCell;

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

thread_local! {
    /// Per-frame link registry: index -> URL. `parse_inline` assigns an
    /// index to each link it finds (via [`Tokens::link`]'s style-encoded
    /// index) and registers the URL here; `render::apply_hyperlinks`
    /// recovers it once per frame, after every widget has drawn. Not
    /// threaded explicitly through `wrap_styled`'s many callers (headings,
    /// text, lists, containers, code captions) because a URL has no effect
    /// on layout/wrapping and doesn't belong in that signature. Reset at
    /// the start of every `render::draw` call ([`reset_links`]) so indices
    /// never leak between frames.
    static LINKS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Clears the per-frame link registry. Called once at the start of
/// `render::draw`, before any content renders.
pub(crate) fn reset_links() {
    LINKS.with(|links| links.borrow_mut().clear());
}

/// The URL registered at `index`, if any — consumed by
/// `render::apply_hyperlinks`.
#[must_use]
pub(crate) fn link_url(index: usize) -> Option<String> {
    LINKS.with(|links| links.borrow().get(index).cloned())
}

fn register_link(url: &str) -> usize {
    LINKS.with(|links| {
        let mut links = links.borrow_mut();
        links.push(url.to_owned());
        links.len() - 1
    })
}

/// Tries to parse a `[label](url)` link starting at `chars[i] == '['`.
/// Returns `(label, url, index one past the closing paren)` on success —
/// `None` (an unmatched `[`) is rendered literally, matching how an
/// unmatched `**`/`` ` `` already behaves.
fn parse_link(chars: &[char], i: usize) -> Option<(String, String, usize)> {
    let close_bracket = (i + 1..chars.len()).find(|&j| chars[j] == ']')?;
    if chars.get(close_bracket + 1) != Some(&'(') {
        return None;
    }
    let close_paren = (close_bracket + 2..chars.len()).find(|&j| chars[j] == ')')?;
    let label: String = chars[i + 1..close_bracket].iter().collect();
    let url: String = chars[close_bracket + 2..close_paren].iter().collect();
    Some((label, url, close_paren + 1))
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
        if chars[i] == '[' {
            match parse_link(&chars, i) {
                Some((label, url, end)) => {
                    push_plain(&mut plain, &mut out);
                    let index = register_link(&url);
                    out.push(Fragment {
                        text: label,
                        style: tokens.link(index),
                    });
                    i = end;
                }
                None => {
                    plain.push('[');
                    i += 1;
                }
            }
            continue;
        }
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

/// Greedy word-wrap over styled fragments. A word ends only at a space in
/// the source, so a style change mid-word (`**bold**,` or `(\`m\`)`) never
/// inserts one.
fn wrap_fragments(fragments: &[Fragment], width: u16) -> Vec<Line<'static>> {
    let width = width as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut used = 0usize;

    for word in words(fragments) {
        let w: usize = word.iter().map(|p| p.text.width()).sum();
        // Hard-break words wider than the whole line, keeping each
        // character's style.
        if w > width {
            if used > 0 {
                lines.push(Line::from(std::mem::take(&mut current)));
                used = 0;
            }
            for piece in word {
                for ch in piece.text.chars() {
                    let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                    if used + cw > width {
                        lines.push(Line::from(std::mem::take(&mut current)));
                        used = 0;
                    }
                    match current.last_mut() {
                        Some(span) if span.style == piece.style => {
                            span.content.to_mut().push(ch);
                        }
                        _ => current.push(Span::styled(ch.to_string(), piece.style)),
                    }
                    used += cw;
                }
            }
            continue;
        }
        let need = if used == 0 { w } else { w + 1 };
        if used + need > width && used > 0 {
            lines.push(Line::from(std::mem::take(&mut current)));
            used = 0;
        }
        if used > 0 {
            current.push(Span::raw(" ".to_owned()));
            used += 1;
        }
        for piece in word {
            current.push(Span::styled(piece.text, piece.style));
        }
        used += w;
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(Line::from(current));
    }
    lines
}

/// Split fragments into words. A fragment boundary is not a word boundary:
/// only a space in the source text ends a word, so one word may carry
/// several styles.
fn words(fragments: &[Fragment]) -> Vec<Vec<Fragment>> {
    let mut words: Vec<Vec<Fragment>> = Vec::new();
    let mut word: Vec<Fragment> = Vec::new();
    for frag in fragments {
        for (i, piece) in frag.text.split(' ').enumerate() {
            if i > 0 && !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
            if !piece.is_empty() {
                word.push(Fragment {
                    text: piece.to_owned(),
                    style: frag.style,
                });
            }
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
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
    fn punctuation_hugs_styled_spans() {
        assert_eq!(
            render("**bold**, *italic*, and `code` flow", 40),
            ["bold, italic, and code flow"]
        );
        assert_eq!(render("(`m`) opens", 40), ["(m) opens"]);
    }

    #[test]
    fn unmatched_markers_render_literally() {
        assert_eq!(render("2 * 3 = 6", 40), ["2 * 3 = 6"]);
        assert_eq!(render("*open", 40), ["*open"]);
    }

    #[test]
    fn link_marker_is_parsed_alongside_existing_inline_styles() {
        reset_links();
        let tokens = Tokens::default();
        let lines = wrap_styled(
            "[label](https://example.com) and **bold**",
            60,
            Style::new(),
            &tokens,
        );
        let spans: Vec<_> = lines.iter().flat_map(|l| l.spans.iter()).collect();

        let link_span = spans
            .iter()
            .find(|s| s.content.as_ref() == "label")
            .expect("link label span present");
        let index = Tokens::link_index(link_span.style).expect("link style carries an index");
        assert_eq!(link_url(index).as_deref(), Some("https://example.com"));

        let bold_span = spans
            .iter()
            .find(|s| s.content.as_ref() == "bold")
            .expect("bold span present");
        assert!(bold_span.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn unmatched_link_brackets_render_literally() {
        assert_eq!(render("[oops(missing paren", 40), ["[oops(missing paren"]);
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
