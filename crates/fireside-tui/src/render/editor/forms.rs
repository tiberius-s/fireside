//! Drawing the currently open block-edit form (spec 013, US1/T034): a
//! centered overlay, geometry courtesy of `editor::hit::form_layout` (the
//! same "one pure layout, two consumers" convention every other editor
//! pane keeps), styled here and nowhere else.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph};

use crate::editor::forms::{EditableField, FormState};
use crate::editor::hit;
use crate::theme::Tokens;

pub(super) fn draw(frame: &mut Frame, area: Rect, form: &FormState, tokens: &Tokens) {
    let layout = hit::form_layout(form, area);
    frame.render_widget(Clear, layout.overlay);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(tokens.border)
        .title(Span::styled(
            layout.title,
            tokens.accent.add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(block, layout.overlay);

    for field_layout in &layout.fields {
        draw_field(frame, field_layout, form, tokens);
    }
    if !layout.children_lines.is_empty() {
        let lines: Vec<Line<'static>> = layout
            .children_lines
            .iter()
            .map(|l| Line::styled(format!("  \u{2022} {l}"), tokens.muted))
            .collect();
        frame.render_widget(Paragraph::new(lines), layout.children_rect);
    }
    if let Some(parent) = form.parent_container_path() {
        let _ = parent;
        frame.render_widget(
            Paragraph::new(Line::styled(
                "  This block is part of a layout block above.",
                tokens.muted,
            )),
            Rect {
                height: 1,
                ..layout.hint_rect
            },
        );
    } else if !layout.hint_lines.is_empty() {
        let lines: Vec<Line<'static>> = layout
            .hint_lines
            .iter()
            .map(|l| Line::styled(format!("  {l}"), tokens.warning))
            .collect();
        frame.render_widget(Paragraph::new(lines), layout.hint_rect);
    }

    for (action, label, rect) in &layout.chips {
        let disabled = matches!(action, hit::FormChipKind::Done) && !form.can_commit();
        let style = if disabled {
            tokens.muted
        } else {
            tokens.affordance
        };
        frame.render_widget(Paragraph::new(Span::styled(*label, style)), *rect);
    }
}

fn field_for(form: &FormState, slot: hit::FieldSlot) -> (&EditableField, bool) {
    use hit::FieldSlot;
    match (form, slot) {
        (
            FormState::Heading { field, .. }
            | FormState::Text { field, .. }
            | FormState::List { field, .. },
            _,
        ) => (field, true),
        (
            FormState::Code {
                language, focus, ..
            },
            FieldSlot::Language,
        ) => (
            language,
            matches!(focus, crate::editor::forms::CodeFocus::Language),
        ),
        (FormState::Code { source, focus, .. }, FieldSlot::Source) => (
            source,
            matches!(focus, crate::editor::forms::CodeFocus::Source),
        ),
        (FormState::Picture { src, focus, .. }, FieldSlot::Src) => (
            src,
            matches!(focus, crate::editor::forms::PictureFocus::Src),
        ),
        (FormState::Picture { alt, focus, .. }, FieldSlot::Alt) => (
            alt,
            matches!(focus, crate::editor::forms::PictureFocus::Alt),
        ),
        (FormState::TextArt { art, focus, .. }, FieldSlot::Art) => (
            art,
            matches!(focus, crate::editor::forms::TextArtFocus::Art),
        ),
        (FormState::TextArt { alt, focus, .. }, FieldSlot::Alt) => (
            alt,
            matches!(focus, crate::editor::forms::TextArtFocus::Alt),
        ),
        _ => unreachable!("form_layout never emits a FormFieldLayout the form itself doesn't have"),
    }
}

fn draw_field(
    frame: &mut Frame,
    field_layout: &hit::FormFieldLayout,
    form: &FormState,
    tokens: &Tokens,
) {
    let (field, focused) = field_for(form, field_layout.slot);
    let rect = field_layout.rect;
    let label_style = if focused {
        tokens.selected.add_modifier(Modifier::BOLD)
    } else {
        tokens.muted
    };
    let mut lines = vec![Line::styled(
        format!(" {}", field_layout.label),
        label_style,
    )];
    for (row, text) in field.buffer.iter().enumerate() {
        let cursor_here = focused && field.cursor.0 == row;
        lines.push(field_line(text, cursor_here, field.cursor.1, tokens));
    }
    frame.render_widget(Paragraph::new(lines), rect);
}

/// One line of a form field's buffer, with a reversed-cell cursor when this
/// is the focused row — the same technique the presenter's quick-edit
/// modal uses (`render::overlays::edit_line`), reproduced here since that
/// helper is private to the `render` module tree.
fn field_line(text: &str, cursor_here: bool, col: usize, tokens: &Tokens) -> Line<'static> {
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
