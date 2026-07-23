//! The "Goes to"/"Branches to" strip below the canvas card (spec 013 US3,
//! T051): the selected slide's outgoing wiring in plain words, with a
//! `[ change ]` chip for non-branch slides — reuses `hit::wiring_summary`
//! and `hit::wiring_change_rect` so drawing and hit-testing can never
//! disagree.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::editor::EditorApp;
use crate::editor::hit;
use crate::theme::Tokens;

pub(super) fn draw(frame: &mut Frame, area: Rect, app: &EditorApp, tokens: &Tokens) {
    if area.height == 0 {
        return;
    }
    let Some(node) = hit::selected_node(app) else {
        return;
    };
    let text = hit::wiring_summary(app.working_graph(), node);
    frame.render_widget(
        Paragraph::new(Span::styled(text.clone(), tokens.muted)),
        area,
    );

    if node.branch_point().is_some() {
        for span in hit::wiring_answer_spans(app.working_graph(), node) {
            let hovered =
                app.hover() == Some(&hit::Target::AnswerChip(node.id.clone(), span.index));
            if !hovered {
                continue;
            }
            let rect = Rect {
                x: area.x + span.start,
                y: area.y,
                width: span.len.min(area.width.saturating_sub(span.start)),
                height: 1,
            };
            let text: String = text
                .chars()
                .skip(span.start as usize)
                .take(span.len as usize)
                .collect();
            frame.render_widget(Paragraph::new(Span::styled(text, tokens.selection)), rect);
        }
        return;
    }
    let rect = hit::wiring_change_rect(area, text.chars().count() as u16);
    let hovered = app.hover() == Some(&hit::Target::GoesToChip(node.id.clone()));
    let style = if hovered {
        tokens.selection
    } else {
        tokens.affordance
    };
    frame.render_widget(Paragraph::new(Span::styled("[ change ]", style)), rect);
}
