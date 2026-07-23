//! The canvas pane: the selected slide, rendered through the exact same
//! `content::draw_content` path the presenter uses (spec 013's WYSIWYG
//! guarantee, spec SC-008) — chrome-free until selection glow and hover
//! cues land (US1/US2). Every block always renders regardless of reveal
//! step: the editor shows the whole slide at once, badges (not omission)
//! are how later stories (US3, T053) will mark staged content.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::editor::hit;
use crate::editor::{DragState, EditorApp, Selection};
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
    let is_empty = node.content.is_empty();
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
    draw_drag_ghost(frame, area, app, tokens);
    draw_insertion_indicator(frame, area, app, tokens);
    if is_empty {
        draw_empty_slide_target(frame, area, app, tokens);
    }
}

/// The empty-slide state (spec 013 T046): "a slide with no blocks at all"
/// shows one clear, large, clickable target rather than empty space
/// (spec Edge Cases) — a click anywhere on the card resolves to
/// `Target::InsertionSlot(.., 0)` (`hit::canvas_hit`), opening the
/// add-block palette exactly like any other insertion slot.
fn draw_empty_slide_target(frame: &mut Frame, canvas: Rect, app: &EditorApp, tokens: &Tokens) {
    let Some(node) = hit::selected_node(app) else {
        return;
    };
    if canvas.height == 0 {
        return;
    }
    let hovered = matches!(
        app.hover(),
        Some(hit::Target::InsertionSlot(id, path, 0)) if id == &node.id && path.is_empty()
    );
    let style = if hovered {
        tokens.selection
    } else {
        tokens.affordance
    };
    let rect = Rect {
        x: canvas.x,
        y: canvas.y + canvas.height / 2,
        width: canvas.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Span::styled("+ Add your first block", style)).alignment(Alignment::Center),
        rect,
    );
}

/// The dimmed "lifted" block while a drag is in progress (design brief:
/// "the block lifts — rendered as a dimmed ghost"). Rather than
/// re-rendering the block's content at the pointer's position, this dims
/// it in place — a lighter-weight but still clearly visible "something is
/// happening here" cue (spec FR-032) that can't disagree with the
/// content `draw_content` already drew, since it only restyles cells
/// already on screen.
fn draw_drag_ghost(frame: &mut Frame, canvas: Rect, app: &EditorApp, tokens: &Tokens) {
    let (node_id, path) = match app.drag() {
        DragState::Lifting { node, path } | DragState::Over { node, path, .. } => {
            (node.clone(), path.clone())
        }
        DragState::Idle | DragState::OutlineLifting { .. } | DragState::OutlineOver { .. } => {
            return;
        }
    };
    let Some(node) = hit::selected_node(app) else {
        return;
    };
    if node.id != node_id || path.len() != 1 {
        return;
    }
    let Some(hit::CanvasLayout {
        inner,
        block_extents,
        scroll,
    }) = hit::canvas_layout(app, canvas)
    else {
        return;
    };
    let Some(&(start, end)) = block_extents.get(path[0]) else {
        return;
    };
    let scroll = scroll as usize;
    let top = start.max(scroll);
    if top >= end {
        return;
    }
    let first_row = inner.y + (top - scroll) as u16;
    let bottom = inner.y + inner.height;
    if first_row >= bottom {
        return;
    }
    let visible_rows = ((end - top) as u16).min(bottom - first_row);
    frame.buffer_mut().set_style(
        Rect {
            x: inner.x,
            y: first_row,
            width: inner.width,
            height: visible_rows,
        },
        tokens.ghost,
    );
}

/// The drop-position indicator (design brief: "a bold insertion line
/// snaps between blocks"): drawn at the resolved drop slot while dragging
/// (`tokens.drop_target`), or at a hovered gap otherwise
/// (`tokens.affordance`, the same hover-cue treatment every other
/// affordance gets). Only interior gaps have a dedicated row to draw on
/// (see `hit::block_extents`'s doc) — dragging to the very first or last
/// position still works (`hit::resolve_drop_slot` resolves it), it just
/// has no separate row to highlight, so `draw_drag_ghost`'s dimming is
/// the only visible cue for that edge case.
fn draw_insertion_indicator(frame: &mut Frame, canvas: Rect, app: &EditorApp, tokens: &Tokens) {
    let Some(node) = hit::selected_node(app) else {
        return;
    };
    let (slot, dragging) = match app.drag() {
        DragState::Over { node: n, to, .. } if n == &node.id => (Some(*to), true),
        _ => (
            match app.hover() {
                Some(hit::Target::InsertionSlot(id, path, at))
                    if id == &node.id && path.is_empty() =>
                {
                    Some(*at)
                }
                _ => None,
            },
            false,
        ),
    };
    let Some(slot) = slot else { return };
    let Some(hit::CanvasLayout {
        inner,
        block_extents,
        scroll,
    }) = hit::canvas_layout(app, canvas)
    else {
        return;
    };
    if slot == 0 || slot >= block_extents.len() {
        return; // no dedicated gap row for the first/last position (see doc above)
    }
    let row_line = block_extents[slot].0 - 1;
    let scroll = scroll as usize;
    if row_line < scroll {
        return;
    }
    let row = inner.y + (row_line - scroll) as u16;
    if row >= inner.y + inner.height {
        return;
    }
    let style = if dragging {
        tokens.drop_target
    } else {
        tokens.affordance
    };
    let full = "\u{2500}\u{2500} + add a block here \u{2500}\u{2500}";
    let short = "+ add a block here";
    let label = if full.chars().count() as u16 <= inner.width {
        full
    } else {
        short
    };
    frame.render_widget(
        Paragraph::new(Span::styled(label, style)).alignment(Alignment::Center),
        Rect {
            x: inner.x,
            y: row,
            width: inner.width,
            height: 1,
        },
    );
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
