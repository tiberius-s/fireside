//! The footer: contextual key hints, flash messages, and the optional
//! elapsed-time display.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use crate::app::{App, FlashKind};
use crate::theme::Tokens;

/// The separator between footer segments — both key-hint segments and the
/// reveal-progress prefix use it, so its display width is computed once
/// and shared by the fit-measuring and rendering paths.
const SEP: &str = "  ·  ";

/// How many rows the footer needs this frame: 1 normally, or however many
/// rows a showing flash message word-wraps to at `width` columns (P1-6) —
/// never truncated mid-word. Called from `render::draw` before laying out
/// the content area, so a multi-row flash can borrow rows from the bottom
/// of the content area instead of being clipped.
#[must_use]
pub(super) fn footer_rows(app: &App, width: u16) -> u16 {
    match app.flash() {
        Some(flash) => flash_lines(&flash.text, width).len().max(1) as u16,
        None => 1,
    }
}

pub(super) fn draw_footer(frame: &mut Frame, area: Rect, app: &App, tokens: &Tokens) {
    if let Some(flash) = app.flash() {
        let style = match flash.kind {
            FlashKind::Info => tokens.accent,
            FlashKind::Error => tokens.error,
        };
        // P1-6: while a flash is showing, the footer shows *only* the
        // flash — key hints are suppressed for its lifetime, and a flash
        // longer than one row wraps (word-wrapped, never mid-word) onto
        // the extra row(s) `footer_rows` already reserved.
        let lines: Vec<Line<'static>> = flash_lines(&flash.text, area.width)
            .into_iter()
            .map(|line| Line::styled(format!(" {line}"), style.add_modifier(Modifier::BOLD)))
            .collect();
        frame.render_widget(Paragraph::new(Text::from(lines)), area);
        draw_timer(frame, area, app, tokens);
        return;
    }

    let session = app.session();
    let pending_reveal = session.has_pending_reveal();
    let hints: &[(&str, &str)] = if pending_reveal {
        // Reveal always finishes before a branch menu or end-of-path
        // marker can appear, so while it's pending the only advance key
        // is "reveal", regardless of what the node would otherwise do.
        &[
            ("Space", "reveal"),
            ("←", "back"),
            ("m", "map"),
            ("e", "edit"),
            ("?", "help"),
            ("q", "quit"),
        ]
    } else if session.branch_point().is_some() {
        &[
            ("↑↓", "choose"),
            ("Enter", "go"),
            ("←", "back"),
            ("m", "map"),
            ("e", "edit"),
            ("?", "help"),
            ("q", "quit"),
        ]
    } else if session.current().is_terminal() {
        &[
            ("←", "back"),
            ("m", "map"),
            ("e", "edit"),
            ("?", "help"),
            ("q", "quit"),
        ]
    } else {
        &[
            ("Space", "next"),
            ("←", "back"),
            ("m", "map"),
            ("e", "edit"),
            ("?", "help"),
            ("q", "quit"),
        ]
    };

    let reveal_prefix = if pending_reveal {
        session
            .reveal_progress()
            .map(|(revealed, total)| format!("{revealed}/{total} revealed"))
    } else {
        None
    };

    // P1-6: drop lowest-priority segments whole (edit, then map) before
    // ever letting a glyph clip — most terminals still fit everything, so
    // this only bites at 80 cols or narrower.
    let hints = drop_to_fit(hints, reveal_prefix.as_deref(), area.width as usize);

    let mut spans = vec![Span::raw(" ")];
    if let Some(prefix) = &reveal_prefix {
        spans.push(Span::styled(
            prefix.clone(),
            tokens.accent.add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(SEP.to_owned(), tokens.border));
    }
    for (i, (key, action)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(SEP.to_owned(), tokens.border));
        }
        spans.push(Span::styled(
            (*key).to_owned(),
            tokens.text.add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(format!(" {action}"), tokens.muted));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
    draw_timer(frame, area, app, tokens);
}

/// The display width of the footer line built from `reveal_prefix` (if
/// any) and `hints`, joined by [`SEP`] — mirrors `draw_footer`'s span
/// assembly exactly, so fit-checking never drifts from what's rendered.
fn line_width(reveal_prefix: Option<&str>, hints: &[(&str, &str)]) -> usize {
    let sep_w = UnicodeWidthStr::width(SEP);
    let mut w = 1; // leading " "
    if let Some(prefix) = reveal_prefix {
        w += UnicodeWidthStr::width(prefix) + sep_w;
    }
    for (i, (key, action)) in hints.iter().enumerate() {
        if i > 0 {
            w += sep_w;
        }
        w += UnicodeWidthStr::width(*key) + 1 + UnicodeWidthStr::width(*action);
    }
    w
}

/// Drops `e edit` first, then `m map`, if the assembled line still doesn't
/// fit `width` — whole segments, never a partial glyph. Any narrower still
/// falls back to ratatui's own rect-bound clipping (no explicit truncation
/// logic needed here), which by then is a below-minimum-terminal edge case.
fn drop_to_fit<'a>(
    hints: &'a [(&'a str, &'a str)],
    reveal_prefix: Option<&str>,
    width: usize,
) -> Vec<(&'a str, &'a str)> {
    let mut kept: Vec<(&str, &str)> = hints.to_vec();
    for drop_key in ["e", "m"] {
        if line_width(reveal_prefix, &kept) <= width {
            break;
        }
        kept.retain(|(key, _)| *key != drop_key);
    }
    kept
}

/// Word-wraps `text` at `width` columns (reserving 1 column for the
/// footer's leading space), never breaking mid-word — a single word wider
/// than the available width hard-breaks by character as a last resort,
/// mirroring `markdown::wrap_fragments`'s policy. This is plain-text
/// wrapping, not markdown: flash messages carry no inline formatting.
fn flash_lines(text: &str, width: u16) -> Vec<String> {
    let width = (width as usize).saturating_sub(1);
    if width == 0 {
        return vec![text.to_owned()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut used = 0usize;
    for word in text.split(' ') {
        let w = UnicodeWidthStr::width(word);
        if w > width {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                used = 0;
            }
            for ch in word.chars() {
                let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                if used + cw > width && !current.is_empty() {
                    lines.push(std::mem::take(&mut current));
                    used = 0;
                }
                current.push(ch);
                used += cw;
            }
            continue;
        }
        let need = if used == 0 { w } else { w + 1 };
        if used + need > width && !current.is_empty() {
            lines.push(std::mem::take(&mut current));
            used = 0;
        }
        if used > 0 {
            current.push(' ');
            used += 1;
        }
        current.push_str(word);
        used += w;
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }
    lines
}

/// The elapsed timer, right-aligned in the footer when switched on.
fn draw_timer(frame: &mut Frame, area: Rect, app: &App, tokens: &Tokens) {
    if !app.show_timer() {
        return;
    }
    let secs = app.elapsed().as_secs();
    let text = if secs >= 3600 {
        format!(
            "{}:{:02}:{:02} ",
            secs / 3600,
            (secs % 3600) / 60,
            secs % 60
        )
    } else {
        format!("{}:{:02} ", secs / 60, secs % 60)
    };
    frame.render_widget(
        Paragraph::new(Span::styled(text, tokens.muted)).alignment(Alignment::Right),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_flash_stays_on_one_line() {
        assert_eq!(flash_lines("Saved", 40), ["Saved"]);
    }

    #[test]
    fn long_flash_wraps_at_word_boundaries_never_mid_word() {
        let text = "Can't save — Ctrl+S again to overwrite, Esc to discard";
        let lines = flash_lines(text, 40);
        assert!(lines.len() > 1, "must wrap: {lines:?}");
        for word in text.split(' ') {
            assert!(
                lines.iter().any(|l| l.contains(word)),
                "{word:?} must survive whole somewhere in {lines:?}"
            );
        }
        for line in &lines {
            assert!(
                UnicodeWidthStr::width(line.as_str()) <= 39,
                "line exceeds width budget: {line:?}"
            );
        }
    }

    #[test]
    fn footer_rows_is_one_without_a_long_flash() {
        assert_eq!(flash_lines("Saved", 80).len(), 1);
    }

    #[test]
    fn drop_to_fit_keeps_everything_when_it_fits() {
        let hints: &[(&str, &str)] = &[("Space", "next"), ("m", "map"), ("e", "edit")];
        assert_eq!(drop_to_fit(hints, None, 200), hints.to_vec());
    }

    #[test]
    fn drop_to_fit_drops_edit_before_map() {
        let hints: &[(&str, &str)] = &[
            ("Space", "next"),
            ("←", "back"),
            ("m", "map"),
            ("e", "edit"),
            ("?", "help"),
            ("q", "quit"),
        ];
        // Narrow enough to force one drop, wide enough that dropping just
        // "e edit" is sufficient.
        let full = line_width(None, hints);
        let kept = drop_to_fit(hints, None, full - 1);
        assert!(!kept.iter().any(|(k, _)| *k == "e"), "edit dropped first");
        assert!(kept.iter().any(|(k, _)| *k == "m"), "map still present");
    }

    #[test]
    fn drop_to_fit_drops_map_too_when_still_too_narrow() {
        let hints: &[(&str, &str)] = &[
            ("Space", "next"),
            ("←", "back"),
            ("m", "map"),
            ("e", "edit"),
            ("?", "help"),
            ("q", "quit"),
        ];
        let kept = drop_to_fit(hints, None, 10);
        assert!(!kept.iter().any(|(k, _)| *k == "e"));
        assert!(!kept.iter().any(|(k, _)| *k == "m"));
    }
}
