//! Full-screen overlays drawn on top of the presenting view: the quick-edit
//! modal and the help screen.

use ratatui::Frame;
use ratatui::layout::Rect;
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
    tokens: &Tokens,
) {
    let content_lines: u16 = fields
        .iter()
        .map(|f| 1 + f.buffer.len() as u16 + 1)
        .sum::<u16>()
        + 1;
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
        ("q", "quit"),
    ];
    // Wide enough for the longest row so nothing clips, capped by the
    // terminal itself inside `overlay_rect`.
    let content_width = KEYS
        .iter()
        .map(|(_, what)| 1 + KEY_COL + what.chars().count())
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

    let mut lines: Vec<Line<'static>> = KEYS
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
    lines.push(Line::default());
    lines.push(Line::styled(
        " press any key to close".to_owned(),
        tokens.muted,
    ));
    frame.render_widget(Paragraph::new(Text::from(lines)), inner);
}
