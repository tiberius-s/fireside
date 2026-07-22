//! Full-screen overlays drawn on top of the presenting view: the quick-edit
//! modal and the help screen.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph};
use unicode_width::UnicodeWidthChar;

use crate::editor::forms::{EditableField, EditableKind};
use crate::theme::Tokens;

use super::content::indicator;
use super::{MEASURE, overlay_rect};

/// The column budget available for a field's text once the modal's fixed
/// outer width (`MEASURE`, clamped to the terminal), its border, and the
/// "  " indent every row uses are accounted for — the width `edit_layout`
/// wraps to, and the same value `hits::edit_field_hit` must reproduce so a
/// click always lands on the row it looks like it's over.
pub(super) fn edit_text_width(area_width: u16) -> usize {
    let rect_width = MEASURE.min(area_width.saturating_sub(2));
    rect_width.saturating_sub(4).max(1) as usize // 2 border cols + 2-space indent
}

/// One row of the quick-edit modal's content: non-interactive chrome, or a
/// slice of a field's word-wrapped buffer row that a mouse click can land
/// on. Built once by `edit_layout` and shared by `draw_edit` (which turns
/// it into styled `Line`s) and `hits::edit_field_hit` (which turns a
/// click's row/col into a buffer position) — the same "one pure layout, two
/// consumers" convention `map`/`branch_option_hit` already use, so a click
/// can never disagree with what's on screen.
pub(super) enum EditRow {
    Banner,
    Blank,
    Label {
        field: usize,
    },
    /// `content` is one visual line of `field`'s `buffer_row`; `seg_start`
    /// is that line's first character's column in the *unwrapped* buffer
    /// row, so a click on it (or the live cursor) maps back to a real edit
    /// position.
    Text {
        field: usize,
        buffer_row: usize,
        seg_start: usize,
        content: String,
    },
    Footer,
}

/// Flattens every field's buffer into wrapped, clickable rows plus the
/// modal's fixed chrome (banner, labels, spacers, footer) — the full
/// content `draw_edit` renders and `hits::edit_field_hit` hit-tests, in
/// the exact order both walk it in.
pub(super) fn edit_layout(
    fields: &[EditableField],
    sink_available: bool,
    text_width: usize,
) -> Vec<EditRow> {
    let mut rows = Vec::new();
    if !sink_available {
        rows.push(EditRow::Banner);
        rows.push(EditRow::Blank);
    }
    for (field, f) in fields.iter().enumerate() {
        rows.push(EditRow::Label { field });
        for (buffer_row, text) in f.buffer.iter().enumerate() {
            for (content, seg_start) in wrap_row(text, text_width) {
                rows.push(EditRow::Text {
                    field,
                    buffer_row,
                    seg_start,
                    content,
                });
            }
        }
        rows.push(EditRow::Blank);
    }
    rows.push(EditRow::Footer);
    rows
}

/// Word-wraps one buffer row's text to `width` columns for display, never
/// breaking a word (a single word wider than `width` hard-breaks as a last
/// resort — the same policy `footer::flash_lines` uses). Returns each
/// visual segment paired with the buffer column — a character index into
/// the *original*, unwrapped row — its first character sits at, so a click
/// or the live cursor can be mapped back to a real edit position even
/// though the row it addresses no longer corresponds 1:1 to a screen line.
fn wrap_row(text: &str, width: usize) -> Vec<(String, usize)> {
    let width = width.max(1);
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return vec![(String::new(), 0)];
    }
    let mut segs: Vec<(String, usize)> = Vec::new();
    let mut current = String::new();
    let mut current_start = 0usize;
    let mut used = 0usize;

    let mut i = 0usize;
    while i < chars.len() {
        let is_space = chars[i] == ' ';
        let start_i = i;
        while i < chars.len() && (chars[i] == ' ') == is_space {
            i += 1;
        }
        let token = &chars[start_i..i];
        let token_w: usize = token.iter().filter_map(|c| c.width()).sum();

        if !is_space && token_w > width {
            // A single word wider than the whole width: hard-break it.
            for (k, &ch) in token.iter().enumerate() {
                let cw = ch.width().unwrap_or(0);
                if used + cw > width && !current.is_empty() {
                    segs.push((std::mem::take(&mut current), current_start));
                    current_start = start_i + k;
                    used = 0;
                }
                current.push(ch);
                used += cw;
            }
            continue;
        }

        if used + token_w > width && used > 0 {
            segs.push((std::mem::take(&mut current), current_start));
            used = 0;
            current_start = start_i;
            if is_space {
                // The wrap boundary absorbs this space — it never starts a
                // new visual line — but it still occupies a real buffer
                // column, so the *next* token's start (set on the next
                // iteration) is what `current_start` ends up as.
                continue;
            }
        }
        for &ch in token {
            current.push(ch);
            used += ch.width().unwrap_or(0);
        }
    }
    segs.push((current, current_start));
    segs
}

/// Which wrapped segment (and column within it) buffer column `col` falls
/// in — a column that lands on a dropped wrap-space (absorbed by
/// `wrap_row`, never rendered) snaps to the start of the following
/// segment, matching where a click there would visually land.
fn locate_in_wrap(segs: &[(String, usize)], col: usize) -> (usize, usize) {
    for (idx, (text, start)) in segs.iter().enumerate() {
        let len = text.chars().count();
        if col < *start {
            return (idx, 0);
        }
        if col < start + len {
            return (idx, col - start);
        }
    }
    let last = segs.len() - 1;
    (last, segs[last].0.chars().count())
}

/// Where the focused field's cursor lands once its row is wrapped: the
/// buffer row, the wrapped segment's start column, and the local column
/// within that segment. `None` if `focused` is out of range (shouldn't
/// happen with a non-empty `fields`, but keeps this total rather than
/// panicking on a stale index).
fn cursor_position(
    fields: &[EditableField],
    focused: usize,
    text_width: usize,
) -> Option<(usize, usize, usize)> {
    let f = fields.get(focused)?;
    let (row, col) = f.cursor;
    let text = f.buffer.get(row)?;
    let segs = wrap_row(text, text_width);
    let (seg_idx, local) = locate_in_wrap(&segs, col);
    Some((row, segs[seg_idx].1, local))
}

/// The flat index into `rows` the focused field's cursor sits on — `0` if
/// it can't be found (degenerate/empty `fields`).
fn cursor_row_index(
    rows: &[EditRow],
    fields: &[EditableField],
    focused: usize,
    text_width: usize,
) -> usize {
    let Some((row, seg_start, _)) = cursor_position(fields, focused, text_width) else {
        return 0;
    };
    rows.iter()
        .position(|r| {
            matches!(r, EditRow::Text { field, buffer_row, seg_start: s, .. }
                if *field == focused && *buffer_row == row && *s == seg_start)
        })
        .unwrap_or(0)
}

/// How many leading rows to skip so the focused field's cursor stays
/// inside a `visible`-row window — shared by `draw_edit` (which slices
/// `rows` before rendering) and `hits::edit_field_hit` (which adds this
/// same offset back to translate a click's screen row into `rows`), so
/// scrolled content never drifts between what's drawn and what a click
/// resolves to. A modal short enough to show everything never scrolls
/// (`0`); once it doesn't fit, this follows the cursor — typing or
/// navigating past the bottom (or top) edge scrolls exactly enough to keep
/// it in view, the same "auto-follow" contract every other in-app scroll
/// (`render::content::draw_content`, the map) already gives a presenter.
pub(super) fn edit_scroll(
    rows: &[EditRow],
    fields: &[EditableField],
    focused: usize,
    text_width: usize,
    visible: usize,
) -> usize {
    let max_scroll = rows.len().saturating_sub(visible);
    if max_scroll == 0 {
        return 0;
    }
    let cursor_idx = cursor_row_index(rows, fields, focused, text_width);
    cursor_idx
        .saturating_sub(visible.saturating_sub(1))
        .min(max_scroll)
}

/// The quick-edit modal: one editable field per heading/text/list block
/// found on the current node, each shown word-wrapped (so nothing is
/// cropped off-screen) with a visible cursor on the focused field.
/// Content-only per ADR-005/ADR-016 — no structural edits.
pub(super) fn draw_edit(
    frame: &mut Frame,
    area: Rect,
    fields: &[EditableField],
    focused: usize,
    sink_available: bool,
    tokens: &Tokens,
) {
    let text_width = edit_text_width(area.width);
    let rows = edit_layout(fields, sink_available, text_width);
    // P2-4: sink-less presentations (the demo deck) can still preview
    // edits, but the presenter learns up front that Ctrl+S can't save,
    // rather than finding out only after typing.
    let rect = overlay_rect(area, MEASURE, rows.len() as u16 + 4);
    frame.render_widget(Clear, rect);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(tokens.border)
        .title(Span::styled(
            " Quick edit ".to_owned(),
            tokens.accent.add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    // Where the focused field's cursor lands once its row is wrapped —
    // computed once, then matched against each `EditRow::Text` below by
    // its (buffer_row, seg_start), the pair that uniquely names a segment.
    let cursor_target = cursor_position(fields, focused, text_width);

    let lines: Vec<Line<'static>> = rows
        .iter()
        .map(|row| match row {
            EditRow::Banner => Line::styled(
                " Demo deck — edits preview but can't be saved".to_owned(),
                tokens.muted.add_modifier(Modifier::ITALIC),
            ),
            EditRow::Blank => Line::default(),
            EditRow::Label { field } => {
                let label = match fields[*field].kind {
                    EditableKind::Heading(level) => format!("Heading (level {level})"),
                    EditableKind::Text => "Text".to_owned(),
                    EditableKind::List { ordered: true } => "Ordered list".to_owned(),
                    EditableKind::List { ordered: false } => "List".to_owned(),
                };
                let style = if *field == focused {
                    tokens.selected.add_modifier(Modifier::BOLD)
                } else {
                    tokens.muted
                };
                Line::styled(format!(" {label}"), style)
            }
            EditRow::Text {
                field,
                buffer_row,
                seg_start,
                content,
            } => {
                let cursor_here = *field == focused
                    && cursor_target.is_some_and(|(r, s, _)| r == *buffer_row && s == *seg_start);
                let local = cursor_target.map_or(0, |(_, _, local)| local);
                edit_line(content, cursor_here, local, tokens)
            }
            EditRow::Footer => Line::styled(" Ctrl+S save  ·  Esc cancel".to_owned(), tokens.muted),
        })
        .collect();

    let visible = inner.height as usize;
    let scroll = edit_scroll(&rows, fields, focused, text_width, visible);
    let shown: Vec<Line<'static>> = lines.into_iter().skip(scroll).take(visible).collect();
    frame.render_widget(Paragraph::new(Text::from(shown)), inner);

    if scroll > 0 {
        indicator(frame, inner, 0, "▲", tokens);
    }
    if scroll < rows.len().saturating_sub(visible) {
        indicator(
            frame,
            inner,
            inner.height.saturating_sub(1),
            "▼ more",
            tokens,
        );
    }
}

/// One line of quick-edit buffer text, with a reversed-block cursor cell
/// when this is the focused line.
fn edit_line(text: &str, cursor_here: bool, col: usize, tokens: &Tokens) -> Line<'static> {
    if !cursor_here {
        return Line::styled(format!("  {text}"), tokens.text);
    }
    let chars: Vec<char> = text.chars().collect();
    let before: String = chars[..col.min(chars.len())].iter().collect();
    let at = chars.get(col).copied().unwrap_or(' ');
    let after: String = chars
        .get(col + 1..)
        .map_or(String::new(), |s| s.iter().collect());
    Line::from(vec![
        Span::raw("  "),
        Span::styled(before, tokens.text),
        Span::styled(at.to_string(), tokens.text.add_modifier(Modifier::REVERSED)),
        Span::styled(after, tokens.text),
    ])
}

/// Width of the left-hand key column in the help overlay, matching the
/// `{key:<KEY_COL$}` padding used when the rows are laid out below.
const KEY_COL: usize = 18;

/// `q` quit and the close hint, pinned as the overlay's fixed footer row
/// (P2-2) — a height-constrained terminal (44×14 and below) must never lose
/// these two, so they live outside the droppable key list entirely.
const HELP_FOOTER: &str = "q quit  ·  any key closes";

pub(super) fn draw_help(frame: &mut Frame, area: Rect, tokens: &Tokens) {
    const KEYS: &[(&str, &str)] = &[
        ("Space / → / Enter", "next slide"),
        ("← / Backspace", "previous slide"),
        ("↑ / ↓", "pick a choice · scroll"),
        ("1–9 or a letter", "take a choice directly"),
        ("m", "map — see and jump anywhere"),
        ("click", "select a map row or branch option"),
        ("f", "fullscreen on/off"),
        ("s", "speaker notes"),
        ("e", "quick-edit this slide's text"),
        ("t", "elapsed timer"),
    ];
    // Wide enough for the longest row so nothing clips, capped by the
    // terminal itself inside `overlay_rect`.
    let content_width = KEYS
        .iter()
        .map(|(_, what)| 1 + KEY_COL + what.chars().count())
        .chain(std::iter::once(1 + HELP_FOOTER.chars().count()))
        .max()
        .unwrap_or(0) as u16;
    let rect = overlay_rect(area, content_width + 2, KEYS.len() as u16 + 4);
    frame.render_widget(Clear, rect);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(tokens.border)
        .title(Span::styled(
            " Keys ".to_owned(),
            tokens.accent.add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    if inner.height == 0 {
        return;
    }

    // The footer always gets its row; if the remaining rows can't fit
    // every key, drop from the middle first — the first and last taught
    // keys (advance/back and the "e" edit hint sitting above the footer)
    // stay visible over the ones a presenter reaches for less often.
    let [list_area, footer_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(inner);
    let list_height = list_area.height as usize;
    let shown: Vec<&(&str, &str)> = if KEYS.len() <= list_height || list_height == 0 {
        KEYS.iter().collect()
    } else {
        let front = list_height.div_ceil(2);
        let back = list_height - front;
        KEYS[..front]
            .iter()
            .chain(KEYS[KEYS.len() - back..].iter())
            .collect()
    };
    let lines: Vec<Line<'static>> = shown
        .iter()
        .map(|(key, what)| {
            Line::from(vec![
                Span::styled(
                    format!(" {key:<KEY_COL$}"),
                    tokens.text.add_modifier(Modifier::BOLD),
                ),
                Span::styled((*what).to_owned(), tokens.muted),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(Text::from(lines)), list_area);
    frame.render_widget(
        Paragraph::new(Line::styled(format!(" {HELP_FOOTER}"), tokens.muted)),
        footer_area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_row_is_not_wrapped() {
        let segs = wrap_row("hi", 40);
        assert_eq!(segs, vec![("hi".to_owned(), 0)]);
    }

    #[test]
    fn long_row_wraps_at_word_boundaries_never_mid_word() {
        let text = "Every edge is explicit. No implicit sequential fallback.";
        let segs = wrap_row(text, 20);
        assert!(segs.len() > 1, "must wrap: {segs:?}");
        for word in text.split(' ') {
            assert!(
                segs.iter().any(|(seg, _)| seg.contains(word)),
                "{word:?} must survive whole somewhere in {segs:?}"
            );
        }
        for (seg, _) in &segs {
            let w: usize = seg.chars().filter_map(UnicodeWidthChar::width).sum();
            assert!(w <= 20, "segment exceeds the wrap width: {seg:?}");
        }
    }

    #[test]
    fn a_word_wider_than_the_whole_width_hard_breaks() {
        let segs = wrap_row("supercalifragilisticexpialidocious", 10);
        assert!(segs.len() > 1, "must break: {segs:?}");
        for (seg, _) in &segs {
            assert!(seg.chars().count() <= 10, "segment exceeds width: {seg:?}");
        }
    }

    /// Every character offset in the original row — including ones that
    /// land on a dropped wrap-space — must resolve to *some* segment/col,
    /// and re-deriving the buffer column from `(segment, local col)` must
    /// round-trip for indices that actually render (round-tripping a
    /// dropped space isn't meaningful — it snaps to the next segment's
    /// start instead).
    #[test]
    fn locate_in_wrap_places_every_column_including_dropped_wrap_spaces() {
        let text = "one two three four five";
        let segs = wrap_row(text, 8);
        assert!(segs.len() > 1, "must wrap: {segs:?}");
        for col in 0..=text.chars().count() {
            let (seg_idx, local) = locate_in_wrap(&segs, col);
            assert!(seg_idx < segs.len());
            assert!(local <= segs[seg_idx].0.chars().count());
        }
        // A column that lands exactly on a rendered character round-trips.
        for (seg_idx, (seg, start)) in segs.iter().enumerate() {
            for local in 0..seg.chars().count() {
                assert_eq!(locate_in_wrap(&segs, start + local), (seg_idx, local));
            }
        }
    }
}
