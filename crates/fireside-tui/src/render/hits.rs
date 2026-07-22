//! Mouse hit-testing: which on-screen row a click landed on, recomputed
//! from the exact same pure layout functions `draw` itself uses, so a
//! click can never disagree with what is actually on screen.

use ratatui::layout::Rect;

use crate::app::App;
use crate::editor::forms::EditableField;
use crate::theme::Tokens;

use super::content::{NodeLines, content_inner, node_lines, notes_panel};
use super::overlays::{EditRow, edit_layout, edit_scroll, edit_text_width};
use super::{MEASURE, areas, map, overlay_rect, surface};

/// Whether `(col, row)` falls inside `rect` — small helper since the
/// `ratatui::layout::Rect` version pinned here has no `contains` for a bare
/// coordinate pair.
fn rect_contains(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.right() && row >= rect.y && row < rect.bottom()
}

/// Which branch option (if any) sits at `(col, row)` of the just-drawn
/// frame — recomputes the same pure layout `draw`/`draw_content` use, so a
/// click can never disagree with what is actually on screen. `None` when
/// there is no branch menu, or the click missed every option's row.
#[must_use]
pub fn branch_option_hit(app: &App, frame_area: Rect, col: u16, row: u16) -> Option<usize> {
    let tokens = Tokens::default();
    let (_, mut content, _) = areas(app.view_mode(), frame_area);
    if let Some(notes) = notes_panel(app, content) {
        content.height = content.height.saturating_sub(notes.height);
    }
    let surf = surface(app.view_mode(), content);
    let view = super::content::SlideView::from_app(app);
    let NodeLines { lines, option_rows } = node_lines(&view, surf.width, &tokens);
    if option_rows.is_empty() {
        return None;
    }
    let total = lines.len() as u16;
    let (_, inner) = content_inner(content, &surf, total);
    if !rect_contains(inner, col, row) {
        return None;
    }
    let max = total.saturating_sub(inner.height);
    let scroll = app.scroll().min(max);
    let clicked_line = scroll as usize + (row - inner.y) as usize;
    option_rows.iter().position(|&r| r == clicked_line)
}

/// Which map row (if any) sits at `(col, row)` of the just-drawn frame —
/// delegates to `map::hit_test`, the map screen's own pure geometry.
#[must_use]
pub fn map_row_hit(
    app: &App,
    frame_area: Rect,
    selected: usize,
    col: u16,
    row: u16,
) -> Option<usize> {
    map::hit_test(app, frame_area, selected, col, row)
}

/// Which quick-edit field/buffer-row/column (if any) sits at `(col, row)`
/// of the just-drawn frame — recomputes the exact same pure layout
/// `overlays::draw_edit` uses (`overlays::edit_layout`/`edit_scroll`), so a
/// click can never disagree with what's on screen, scrolled or not.
/// `None` when the click missed every text row (chrome, blank space,
/// outside the modal, or scrolled out of view).
#[must_use]
pub fn edit_field_hit(
    frame_area: Rect,
    fields: &[EditableField],
    focused: usize,
    sink_available: bool,
    col: u16,
    row: u16,
) -> Option<(usize, usize, usize)> {
    let text_width = edit_text_width(frame_area.width);
    let rows = edit_layout(fields, sink_available, text_width);
    let rect = overlay_rect(frame_area, MEASURE, rows.len() as u16 + 4);
    let inner = Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: rect.width.saturating_sub(2),
        height: rect.height.saturating_sub(2),
    };
    if !rect_contains(inner, col, row) {
        return None;
    }
    let scroll = edit_scroll(&rows, fields, focused, text_width, inner.height as usize);
    let idx = scroll + (row - inner.y) as usize;
    match rows.get(idx)? {
        EditRow::Text {
            field,
            buffer_row,
            seg_start,
            content,
        } => {
            let local = (col.saturating_sub(inner.x) as usize).saturating_sub(2);
            let local = local.min(content.chars().count());
            Some((*field, *buffer_row, seg_start + local))
        }
        _ => None,
    }
}
