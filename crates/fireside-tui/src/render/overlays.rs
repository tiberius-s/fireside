//! Full-screen overlays drawn on top of the presenting view: the quick-edit
//! modal and the help screen.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph};

use crate::app::{EditableField, EditableKind};
use crate::theme::Tokens;

use super::{MEASURE, overlay_rect};

/// The quick-edit modal: one editable field per heading/text block found on
/// the current node, each shown with its buffer and a visible cursor on the
/// focused field. Content-only per ADR-005 — no structural edits.
pub(super) fn draw_edit(
    frame: &mut Frame,
    area: Rect,
    fields: &[EditableField],
    focused: usize,
    sink_available: bool,
    tokens: &Tokens,
) {
    // P2-4: sink-less presentations (the demo deck) can still preview
    // edits, but the presenter learns up front that Ctrl+S can't save,
    // rather than finding out only after typing.
    let banner_lines: u16 = u16::from(!sink_available);
    let content_lines: u16 = fields
        .iter()
        .map(|f| 1 + f.buffer.len() as u16 + 1)
        .sum::<u16>()
        + 1
        + banner_lines;
    let rect = overlay_rect(area, MEASURE, content_lines + 4);
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

    let mut lines: Vec<Line<'static>> = Vec::new();
    if !sink_available {
        lines.push(Line::styled(
            " Demo deck — edits preview but can't be saved".to_owned(),
            tokens.muted.add_modifier(Modifier::ITALIC),
        ));
        lines.push(Line::default());
    }
    for (i, field) in fields.iter().enumerate() {
        let label = match field.kind {
            EditableKind::Heading(level) => format!("Heading (level {level})"),
            EditableKind::Text => "Text".to_owned(),
        };
        let label_style = if i == focused {
            tokens.selected.add_modifier(Modifier::BOLD)
        } else {
            tokens.muted
        };
        lines.push(Line::styled(format!(" {label}"), label_style));
        for (row, text) in field.buffer.iter().enumerate() {
            lines.push(edit_line(
                text,
                i == focused && row == field.cursor.0,
                field.cursor.1,
                tokens,
            ));
        }
        lines.push(Line::default());
    }
    lines.push(Line::styled(
        " Ctrl+S save  ·  Esc cancel".to_owned(),
        tokens.muted,
    ));
    frame.render_widget(Paragraph::new(Text::from(lines)), inner);
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
