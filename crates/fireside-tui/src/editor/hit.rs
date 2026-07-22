//! Editor hit-testing (spec 013): which on-screen interactive region a
//! click or hover coordinate lands on, recomputed from the exact pure
//! layout the last frame drew — the same "one pure layout, two consumers"
//! contract `render::hits` already keeps for the presenter (see
//! `specs/013-authoring-editor/contracts/hit-testing.md`).
//!
//! `hit()` reads only [`EditorApp`]'s own stored state (`terminal_size`,
//! `working_graph`, `selection`) — never anything the renderer produced —
//! so a click can never disagree with what `render::editor` draws next
//! frame, and there is no render-to-update back-channel (constitution IV).
//! `render::editor` reuses this module's pure layout functions for its own
//! drawing, so the two can never drift apart either.

use ratatui::layout::{Constraint, Layout, Rect};

use fireside_core::{Graph, Node};
use fireside_engine::authoring::{BlockPath, OutlineRow, outline_order};

use crate::render::content::{NodeLines, SlideView, content_inner, node_lines};
use crate::render::{Surface, blocks, surface};
use crate::theme::Tokens;

use super::{EditorApp, Selection};

/// The editor studio's minimum usable size (spec FR-029): below this,
/// `render::editor` draws only the resize guard and `hit()` resolves
/// nothing.
pub(crate) const MIN_WIDTH: u16 = 80;
pub(crate) const MIN_HEIGHT: u16 = 24;

/// One of the toolbar's five chips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolbarAction {
    AddSlide,
    Present,
    Save,
    Undo,
    Help,
}

/// One of a selected block's contextual chips. Not yet produced by `hit()`
/// — the selection chip row lands with US1 (T034). `#[allow(dead_code)]`:
/// this is forward-declared API surface per `contracts/hit-testing.md`,
/// not dead code to clean up — T034 constructs these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum BlockAction {
    Edit,
    AddBelow,
    MoveUp,
    MoveDown,
    Reveal,
    Delete,
}

/// One chip inside the currently open form. Not yet produced by `hit()` —
/// forms land with US1/US3. `#[allow(dead_code)]`: forward-declared per
/// `contracts/hit-testing.md`; T034/T051 construct these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum FormChipKind {
    Done,
    Cancel,
}

/// One interactive region of the editor screen —
/// `contracts/hit-testing.md`'s `Target` enum. Every variant exists now so
/// later stories only ever add *resolution* logic, never redefine the
/// type; the Foundational-phase skeleton only ever *produces*
/// `ToolbarChip`, `OutlineRow`, `OutlineNewSlide`, `Block`, and
/// `StatusBanner` — the rest wait for the drag/form machinery that gives
/// them meaning (US1–US3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Target {
    ToolbarChip(ToolbarAction),
    OutlineRow(String),
    OutlineNewSlide,
    Block(String, BlockPath),
    /// Forward-declared per `contracts/hit-testing.md`; T034 produces this.
    #[allow(dead_code)]
    BlockChip(String, BlockPath, BlockAction),
    /// Forward-declared; T045 produces this.
    #[allow(dead_code)]
    InsertionSlot(String, BlockPath, usize),
    /// Forward-declared; T051 produces this.
    #[allow(dead_code)]
    GoesToChip(String),
    /// Forward-declared; T034/T051 produce this.
    #[allow(dead_code)]
    FormChip(FormChipKind),
    StatusBanner,
}

/// The editor screen's five regions — toolbar, outline, canvas, status,
/// hint — computed from nothing but the terminal size, exactly as
/// `render::editor` lays them out.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EditorAreas {
    pub(crate) toolbar: Rect,
    pub(crate) outline: Rect,
    pub(crate) canvas: Rect,
    pub(crate) status: Rect,
    pub(crate) hint: Rect,
}

/// Splits `area` into the studio's five panes. Pure geometry, shared by
/// `render::editor` (drawing) and this module (hit-testing) — the same
/// "one layout, two consumers" convention `render::areas`/`render::surface`
/// already keep for the presenter.
pub(crate) fn editor_areas(area: Rect) -> EditorAreas {
    let [toolbar, body, status, hint] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    let outline_width = (area.width / 4).clamp(18, 28);
    let [outline, canvas] =
        Layout::horizontal([Constraint::Length(outline_width), Constraint::Fill(1)]).areas(body);
    EditorAreas {
        toolbar,
        outline,
        canvas,
        status,
        hint,
    }
}

/// The toolbar's five chips, in on-screen (left-to-right) order and exact
/// label text — shared by `render::editor`'s drawing and this module's
/// hit-testing so neither can drift from the other.
pub(crate) const TOOLBAR_CHIPS: [(ToolbarAction, &str); 5] = [
    (ToolbarAction::AddSlide, "[ + Slide ]"),
    (ToolbarAction::Present, "[ \u{25b6} Present ]"),
    (ToolbarAction::Save, "[ Save ]"),
    (ToolbarAction::Undo, "[ \u{21b6} Undo ]"),
    (ToolbarAction::Help, "[ ? ]"),
];

/// Column rects for each toolbar chip within `toolbar`, right-aligned in
/// on-screen order with one space between chips.
pub(crate) fn toolbar_chip_rects(toolbar: Rect) -> Vec<(ToolbarAction, Rect)> {
    let widths: Vec<u16> = TOOLBAR_CHIPS
        .iter()
        .map(|(_, label)| label.chars().count() as u16)
        .collect();
    let total = widths.iter().sum::<u16>() + widths.len().saturating_sub(1) as u16;
    let mut x = toolbar.x + toolbar.width.saturating_sub(total);
    let mut out = Vec::with_capacity(TOOLBAR_CHIPS.len());
    for (i, (action, _)) in TOOLBAR_CHIPS.iter().enumerate() {
        let w = widths[i];
        out.push((
            *action,
            Rect {
                x,
                y: toolbar.y,
                width: w.min(toolbar.width),
                height: 1,
            },
        ));
        x += w + 1;
    }
    out
}

/// One line of the outline pane: a slide row, the "not linked yet" divider
/// (shown once, before the first unreachable row), or the permanent
/// "+ new slide" row. Built once by [`outline_lines`] and shared by
/// `render::editor::outline` (which turns it into styled `Line`s) and this
/// module's `outline_hit` (which turns a click's row into a target) — the
/// same "one pure layout, two consumers" convention as everywhere else
/// here, so the divider row can never desync the two.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OutlineLine {
    Row(OutlineRow),
    Divider,
    NewSlide,
}

/// The outline pane's full line sequence for `graph`.
pub(crate) fn outline_lines(graph: &Graph) -> Vec<OutlineLine> {
    let rows = outline_order(graph);
    let mut out = Vec::with_capacity(rows.len() + 2);
    let mut divider_shown = false;
    for row in rows {
        if !row.reachable && !divider_shown {
            out.push(OutlineLine::Divider);
            divider_shown = true;
        }
        out.push(OutlineLine::Row(row));
    }
    out.push(OutlineLine::NewSlide);
    out
}

/// The node the canvas currently shows: the selected slide (or the slide
/// owning the selected block), or the graph's entry node when nothing is
/// selected yet.
pub(crate) fn selected_node(app: &EditorApp) -> Option<&Node> {
    match app.selection() {
        Selection::Slide(id) | Selection::Block(id, _) => app.working_graph().node(id),
        Selection::None => app.working_graph().entry(),
    }
}

/// Each top-level content block's `[start, end)` line range within the
/// node's rendered flow, computed the same way `render::blocks::render_blocks`
/// itself joins blocks (one blank line between each) — so a click can
/// never disagree with what's on screen. Nested (`Container`) children are
/// out of scope for the Foundational-phase skeleton; hit-testing addresses
/// only top-level blocks until the container form (US1, T033) needs to
/// reach inside one.
fn block_extents(
    node: &Node,
    width: u16,
    tokens: &Tokens,
    reveal_level: u32,
) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(node.content.len());
    let mut prev = 0usize;
    for i in 0..node.content.len() {
        let cumulative =
            blocks::render_blocks(&node.content[..=i], width, tokens, reveal_level).len();
        out.push((prev, cumulative));
        prev = cumulative;
    }
    out
}

fn rect_contains(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.right() && row >= rect.y && row < rect.bottom()
}

fn toolbar_hit(toolbar: Rect, col: u16, row: u16) -> Option<Target> {
    if !rect_contains(toolbar, col, row) {
        return None;
    }
    toolbar_chip_rects(toolbar)
        .into_iter()
        .find(|(_, rect)| rect_contains(*rect, col, row))
        .map(|(action, _)| Target::ToolbarChip(action))
}

fn outline_hit(app: &EditorApp, outline: Rect, col: u16, row: u16) -> Option<Target> {
    if !rect_contains(outline, col, row) {
        return None;
    }
    let idx = (row - outline.y) as usize;
    match outline_lines(app.working_graph()).get(idx)? {
        OutlineLine::Row(r) => Some(Target::OutlineRow(r.node_id.clone())),
        OutlineLine::Divider => None,
        OutlineLine::NewSlide => Some(Target::OutlineNewSlide),
    }
}

fn canvas_hit(app: &EditorApp, canvas: Rect, col: u16, row: u16) -> Option<Target> {
    if !rect_contains(canvas, col, row) {
        return None;
    }
    let node = selected_node(app)?;
    let tokens = Tokens::default();
    let view_mode = node.resolved_view_mode(app.working_graph().defaults.as_ref());
    let surf: Surface = surface(view_mode, canvas);
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
    let NodeLines { lines, .. } = node_lines(&view, surf.width, &tokens);
    let total = lines.len() as u16;
    let (_, inner) = content_inner(canvas, &surf, total);
    if !rect_contains(inner, col, row) {
        return None;
    }
    let max = total.saturating_sub(inner.height);
    let scroll = app.scroll().min(max);
    let clicked_line = scroll as usize + (row - inner.y) as usize;
    let extents = block_extents(node, surf.width, &tokens, u32::MAX);
    let block_index = extents
        .iter()
        .position(|&(start, end)| clicked_line >= start && clicked_line < end)?;
    Some(Target::Block(node.id.clone(), vec![block_index]))
}

fn status_hit(app: &EditorApp, status: Rect, col: u16, row: u16) -> Option<Target> {
    if !rect_contains(status, col, row) || app.status().is_empty() {
        return None;
    }
    Some(Target::StatusBanner)
}

/// Which interactive region (if any) sits at `(col, row)` of the just-drawn
/// frame. Priority order (top-most drawn wins), per the contract: toolbar
/// chips > open-form chips (none yet) > canvas (block/insertion-slot) >
/// outline rows > status banner > `None`.
#[must_use]
pub(crate) fn hit(app: &EditorApp, area: Rect, col: u16, row: u16) -> Option<Target> {
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        return None;
    }
    let areas = editor_areas(area);
    toolbar_hit(areas.toolbar, col, row)
        .or_else(|| canvas_hit(app, areas.canvas, col, row))
        .or_else(|| outline_hit(app, areas.outline, col, row))
        .or_else(|| status_hit(app, areas.status, col, row))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::EditorApp;
    use fireside_core::Graph;

    const FIXTURE: &str = r#"{"nodes":[
        {"id":"a","title":"Welcome","traversal":"b","content":[
            {"kind":"heading","level":1,"text":"Hello"},
            {"kind":"text","body":"World"}
        ]},
        {"id":"b","title":"The end","content":[{"kind":"text","body":"Done"}]}
    ]}"#;

    fn app() -> EditorApp {
        EditorApp::new(Graph::from_json(FIXTURE).expect("fixture parses"))
    }

    fn area() -> Rect {
        Rect::new(0, 0, 100, 30)
    }

    #[test]
    fn too_small_a_terminal_resolves_nothing() {
        let app = app();
        assert_eq!(hit(&app, Rect::new(0, 0, 79, 23), 5, 0), None);
    }

    #[test]
    fn toolbar_chip_cells_resolve_to_their_action() {
        let app = app();
        let areas = editor_areas(area());
        for (action, rect) in toolbar_chip_rects(areas.toolbar) {
            let target = hit(&app, area(), rect.x, rect.y);
            assert_eq!(target, Some(Target::ToolbarChip(action)));
        }
    }

    #[test]
    fn outline_row_resolves_to_its_slide() {
        let app = app();
        let areas = editor_areas(area());
        assert_eq!(
            hit(&app, area(), areas.outline.x, areas.outline.y),
            Some(Target::OutlineRow("a".to_owned()))
        );
        assert_eq!(
            hit(&app, area(), areas.outline.x, areas.outline.y + 1),
            Some(Target::OutlineRow("b".to_owned()))
        );
    }

    #[test]
    fn the_row_after_the_last_slide_is_the_new_slide_row() {
        let app = app();
        let areas = editor_areas(area());
        assert_eq!(
            hit(&app, area(), areas.outline.x, areas.outline.y + 2),
            Some(Target::OutlineNewSlide)
        );
    }

    #[test]
    fn a_block_resolves_to_its_full_rendered_extent() {
        let app = app();
        let areas = editor_areas(area());
        let node = app.working_graph().node("a").expect("node a");
        let tokens = Tokens::default();
        let view_mode = node.resolved_view_mode(app.working_graph().defaults.as_ref());
        let surf = surface(view_mode, areas.canvas);
        let extents = block_extents(node, surf.width, &tokens, u32::MAX);
        let view = SlideView {
            node,
            reveal_level: u32::MAX,
            has_pending_reveal: false,
            branch_selected: 0,
            fading: false,
            scroll: 0,
            view_mode,
            history_titles: Vec::new(),
        };
        let NodeLines { lines, .. } = node_lines(&view, surf.width, &tokens);
        let (_, inner) = content_inner(areas.canvas, &surf, lines.len() as u16);

        // Every row across the first block's extent must resolve to the
        // same block index, including rows past its first line — proving
        // hit-testing covers a block's whole rendered extent, not just its
        // top-left cell.
        let (start, end) = extents[0];
        for line in start..end {
            let row = inner.y + line as u16;
            assert_eq!(
                hit(&app, area(), inner.x, row),
                Some(Target::Block("a".to_owned(), vec![0])),
                "line {line} of block 0 did not resolve to block 0"
            );
        }
    }

    #[test]
    fn a_coordinate_outside_every_region_is_none() {
        let app = app();
        assert_eq!(hit(&app, area(), 0, 29), None);
    }
}
