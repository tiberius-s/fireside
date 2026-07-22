//! The canvas pane: the selected slide, rendered through the exact same
//! `content::draw_content` path the presenter uses (spec 013's WYSIWYG
//! guarantee, spec SC-008) — chrome-free until selection glow and hover
//! cues land (US1/US2). Every block always renders regardless of reveal
//! step: the editor shows the whole slide at once, badges (not omission)
//! are how later stories (US3, T053) will mark staged content.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::editor::hit;
use crate::editor::{EditorApp, Selection};
use crate::render::content::{SlideView, draw_content};
use crate::theme::Tokens;

pub(super) fn draw(frame: &mut Frame, area: Rect, app: &EditorApp, tokens: &Tokens) {
    let Some(node) = hit::selected_node(app) else {
        frame.render_widget(
            Paragraph::new(Span::styled("This deck has no slides yet.", tokens.muted)),
            area,
        );
        return;
    };
    let view_mode = node.resolved_view_mode(app.working_graph().defaults.as_ref());
    let view = SlideView {
        node,
        reveal_level: u32::MAX,
        has_pending_reveal: false,
        branch_selected: 0,
        fading: false,
        scroll: app.scroll(),
        view_mode,
        history_titles: Vec::new(),
    };
    draw_content(frame, area, &view, tokens);
    draw_selection_marker(frame, area, app, tokens);
}

/// A `▎` marker in the card's left gutter across a selected top-level
/// block's full rendered extent (spec 013 US1 acceptance scenario 1: "the
/// block shows a clear selected state") — an overlay drawn *after*
/// `draw_content`, never a parameter threaded into it, so the presenter's
/// own rendering stays byte-identical (spec SC-008). Nested blocks
/// (`path.len() > 1`) aren't marked yet — canvas hit-testing itself only
/// addresses top-level blocks until US2 extends it (see `hit::canvas_hit`).
fn draw_selection_marker(frame: &mut Frame, canvas: Rect, app: &EditorApp, tokens: &Tokens) {
    let Some(node) = hit::selected_node(app) else {
        return;
    };
    let index = match app.selection() {
        Selection::Block(id, path) if id == &node.id && path.len() == 1 => path[0],
        _ => return,
    };
    let Some(hit::CanvasLayout {
        inner,
        block_extents,
        scroll,
    }) = hit::canvas_layout(app, canvas)
    else {
        return;
    };
    if inner.x <= canvas.x {
        return; // no gutter column available to mark
    }
    let Some(&(start, end)) = block_extents.get(index) else {
        return;
    };
    let gutter_x = inner.x - 1;
    let scroll = scroll as usize;
    for line in start.max(scroll)..end {
        let row = inner.y + (line - scroll) as u16;
        if row >= inner.y + inner.height {
            break;
        }
        frame.render_widget(
            Paragraph::new(Span::styled("\u{258e}", tokens.selection)),
            Rect {
                x: gutter_x,
                y: row,
                width: 1,
                height: 1,
            },
        );
    }
}
