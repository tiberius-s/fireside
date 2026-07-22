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

use fireside_core::{ContainerLayout, Graph, Node};
use fireside_engine::authoring::{BlockKind, BlockPath, OutlineRow, outline_order};

use crate::render::content::{NodeLines, SlideView, content_inner, node_lines};
use crate::render::{Surface, blocks, surface};
use crate::theme::Tokens;

use super::forms::{self, FormState};
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

/// One of a selected block's contextual chips. `Edit`, `AddBelow`, and
/// `Delete` are produced by `hit()` as of US1/US2 (T034, T042, T043) —
/// `MoveUp`/`MoveDown`/`Reveal` wait for US3. `#[allow(dead_code)]`:
/// forward-declared API surface per `contracts/hit-testing.md`, not dead
/// code to clean up.
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

/// One chip inside the currently open form (spec 013, US1-US2). `Done`
/// and `Cancel` are common to every form; `ConvertToTextArt` is the
/// picture form's shortcut (T031), `GenerateFromPhrase` the text-art
/// form's CLI-injected callback trigger (T032), `CycleLayout` the
/// container form's layout picker (T033), and `PaletteCard` one of the
/// add-block palette's eight cards (T042).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FormChipKind {
    Done,
    Cancel,
    ConvertToTextArt,
    GenerateFromPhrase,
    CycleLayout,
    PaletteCard(BlockKind),
}

/// Which of an open form's text fields a click landed in — coarse-grained
/// (the whole field's rendered extent, not a character column): clicking
/// anywhere in a field focuses it, exactly like the block/insertion-slot
/// convention elsewhere in this module. Precise cursor placement stays a
/// keyboard-only refinement for forms (unlike the presenter's quick-edit
/// modal, which does place the cursor precisely on click).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldSlot {
    Only,
    Language,
    Source,
    Src,
    Alt,
    Art,
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
    BlockChip(String, BlockPath, BlockAction),
    /// Forward-declared; T045 produces this.
    #[allow(dead_code)]
    InsertionSlot(String, BlockPath, usize),
    /// Forward-declared; T051 produces this.
    #[allow(dead_code)]
    GoesToChip(String),
    FormChip(FormChipKind),
    /// A click inside one of the open form's text fields — focuses it
    /// (T034).
    FormField(FieldSlot),
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
/// never disagree with what's on screen. The one-row blank separator
/// `render_blocks` inserts before every block after the first is
/// deliberately *excluded* from both neighbors' ranges: it is the
/// insertion-slot gap `canvas_hit`/`resolve_drop_slot` resolve to
/// `Target::InsertionSlot` instead (spec 013 US2, T045) — a block's own
/// extent is exactly its rendered content, never the gap above it.
/// Nested (`Container`) children are out of scope for the canvas; only
/// top-level blocks are addressed here (container children are reached
/// through the container form's breadcrumb, T033).
fn block_extents(
    node: &Node,
    width: u16,
    tokens: &Tokens,
    reveal_level: u32,
) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(node.content.len());
    let mut prev_cumulative = 0usize;
    for i in 0..node.content.len() {
        let cumulative =
            blocks::render_blocks(&node.content[..=i], width, tokens, reveal_level).len();
        let start = if i == 0 { 0 } else { prev_cumulative + 1 };
        out.push((start, cumulative));
        prev_cumulative = cumulative;
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

/// The canvas's card-inner rect, each top-level block's line-range extent
/// within it, and the current (clamped) scroll offset — the geometry
/// `canvas_hit` resolves clicks against and `render::editor::canvas` reuses
/// verbatim to draw the selection marker (spec 013, T034), so the two can
/// never disagree about which rows belong to which block.
pub(crate) struct CanvasLayout {
    pub(crate) inner: Rect,
    pub(crate) block_extents: Vec<(usize, usize)>,
    pub(crate) scroll: u16,
}

pub(crate) fn canvas_layout(app: &EditorApp, canvas: Rect) -> Option<CanvasLayout> {
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
    let max = total.saturating_sub(inner.height);
    let scroll = app.scroll().min(max);
    let block_extents = block_extents(node, surf.width, &tokens, u32::MAX);
    Some(CanvasLayout {
        inner,
        block_extents,
        scroll,
    })
}

fn canvas_hit(app: &EditorApp, canvas: Rect, col: u16, row: u16) -> Option<Target> {
    if !rect_contains(canvas, col, row) {
        return None;
    }
    let node = selected_node(app)?;
    if node.content.is_empty() {
        // Empty slide (spec 013 T046): the whole pane is the "add your
        // first block" target, equivalent to inserting at position 0 —
        // a zero-content card has no non-empty `inner` rect to test
        // against otherwise (`content_inner` sizes to content height).
        return Some(Target::InsertionSlot(node.id.clone(), Vec::new(), 0));
    }
    let CanvasLayout {
        inner,
        block_extents: extents,
        scroll,
    } = canvas_layout(app, canvas)?;
    if !rect_contains(inner, col, row) {
        return None;
    }
    let clicked_line = scroll as usize + (row - inner.y) as usize;
    for (i, &(start, _)) in extents.iter().enumerate().skip(1) {
        if clicked_line == start - 1 {
            return Some(Target::InsertionSlot(node.id.clone(), Vec::new(), i));
        }
    }
    let block_index = extents
        .iter()
        .position(|&(start, end)| clicked_line >= start && clicked_line < end)?;
    Some(Target::Block(node.id.clone(), vec![block_index]))
}

/// Resolves where a block drag currently hovering `(col, row)` on
/// `node_id`'s canvas would drop, in the *pre-removal* index space (see
/// `editor::EditorApp::on_release` for the conversion `Op::MoveBlock`'s
/// `to` parameter needs) — `None` when the pointer isn't over `node_id`'s
/// canvas at all. A row on a gap gives that gap's slot directly, exactly
/// like a plain click would (`canvas_hit`); a row within a block's own
/// extent resolves to "insert before" or "insert after" that block by
/// whichever half of its rendered rows the pointer is nearer — this
/// halfway split exists only for drag resolution (`hit()` itself always
/// resolves a block's full extent to `Target::Block`, per
/// `contracts/hit-testing.md`'s test contract) and is what lets a block
/// become the deck's first or last without a dedicated gap row to target.
#[must_use]
pub(crate) fn resolve_drop_slot(
    app: &EditorApp,
    node_id: &str,
    canvas: Rect,
    col: u16,
    row: u16,
) -> Option<usize> {
    let node = selected_node(app)?;
    if node.id != node_id {
        return None;
    }
    if node.content.is_empty() {
        return rect_contains(canvas, col, row).then_some(0);
    }
    let CanvasLayout {
        inner,
        block_extents: extents,
        scroll,
    } = canvas_layout(app, canvas)?;
    if !rect_contains(inner, col, row) {
        return None;
    }
    let clicked_line = scroll as usize + (row - inner.y) as usize;
    for (i, &(start, _)) in extents.iter().enumerate().skip(1) {
        if clicked_line == start - 1 {
            return Some(i);
        }
    }
    let idx = extents
        .iter()
        .position(|&(s, e)| clicked_line >= s && clicked_line < e)
        .unwrap_or(extents.len() - 1);
    let (s, e) = extents[idx];
    let mid = s + (e - s) / 2;
    Some(if clicked_line < mid { idx } else { idx + 1 })
}

/// The selected block's contextual chips, drawn in the hint line (spec
/// 013, T034/T042/T043): "at rest, ~7 controls" (spec FR-030) rules out a
/// permanent floating action bar on the canvas, so a block's actions take
/// over the hint line exactly the way a flash message would — visible
/// only while something is selected, gone the moment it isn't.
/// `[ ✎ Edit ]` only appears when the block has a form (a `Divider` has
/// nothing to edit); `[ + Add below ]` and `[ Delete ]` are always
/// available for any selected block.
pub(crate) const BLOCK_EDIT_CHIP: &str = " [ \u{270e} Edit ]";
pub(crate) const BLOCK_ADD_BELOW_CHIP: &str = " [ + Add below ]";
pub(crate) const BLOCK_DELETE_CHIP: &str = " [ Delete ]";

/// Whether `block` has an edit form at all — a `Divider` has nothing to
/// edit, so a selected divider offers no `[ Edit ]` chip.
fn has_form(node: &str, path: &BlockPath, node_ref: &Node) -> bool {
    forms::block_at(&node_ref.content, path)
        .is_some_and(|block| forms::open(node, path.clone(), block).is_some())
}

/// Whether the currently selected block has an edit form — shared by
/// `selected_block_chips` (does `[ Edit ]` appear?) and
/// `render::editor::mod::draw_hint`, so the two can never disagree about
/// whether a divider gets an `[ Edit ]` label it wouldn't actually respond
/// to.
#[must_use]
pub(crate) fn selection_has_form(app: &EditorApp) -> bool {
    let Selection::Block(node, path) = app.selection() else {
        return false;
    };
    app.working_graph()
        .node(node)
        .is_some_and(|node_ref| has_form(node, path, node_ref))
}

/// The currently selected block's contextual chips, in on-screen order —
/// shared by `hint_hit` (does a click resolve to one?) and
/// `render::editor::mod::draw_hint` (drawing), so the two can never
/// disagree. Empty when nothing is selected.
#[must_use]
pub(crate) fn selected_block_chips(app: &EditorApp) -> Vec<(BlockAction, &'static str)> {
    if !matches!(app.selection(), Selection::Block(..)) {
        return Vec::new();
    }
    let mut chips = Vec::new();
    if selection_has_form(app) {
        chips.push((BlockAction::Edit, BLOCK_EDIT_CHIP));
    }
    chips.push((BlockAction::AddBelow, BLOCK_ADD_BELOW_CHIP));
    chips.push((BlockAction::Delete, BLOCK_DELETE_CHIP));
    chips
}

/// Column rects for `chips` within the hint line, left-to-right in the
/// order given — the same "one pure layout, two consumers" convention
/// `toolbar_chip_rects` already keeps for the toolbar.
#[must_use]
pub(crate) fn block_chip_rects(
    hint: Rect,
    chips: &[(BlockAction, &'static str)],
) -> Vec<(BlockAction, Rect)> {
    let mut x = hint.x;
    let mut out = Vec::with_capacity(chips.len());
    for (action, label) in chips {
        let w = (label.chars().count() as u16).min(hint.width.saturating_sub(x - hint.x));
        out.push((
            *action,
            Rect {
                x,
                y: hint.y,
                width: w,
                height: 1,
            },
        ));
        x += w;
    }
    out
}

fn hint_hit(app: &EditorApp, hint: Rect, col: u16, row: u16) -> Option<Target> {
    let Selection::Block(node, path) = app.selection() else {
        return None;
    };
    let chips = selected_block_chips(app);
    let (action, _) = block_chip_rects(hint, &chips)
        .into_iter()
        .find(|(_, r)| rect_contains(*r, col, row))?;
    Some(Target::BlockChip(node.clone(), path.clone(), action))
}

// ─── Open-form layout (spec 013, US1/T034) ─────────────────────────────────

/// One field's rendered extent inside an open form: the label row plus
/// every buffer row, one rect (a click anywhere on it focuses the field —
/// see [`FieldSlot`]'s doc).
#[derive(Debug, Clone, Copy)]
pub(crate) struct FormFieldLayout {
    pub(crate) slot: FieldSlot,
    pub(crate) label: &'static str,
    pub(crate) rect: Rect,
}

/// The open form's full geometry — shared by `render::editor::forms`
/// (drawing) and this module (hit-testing), the same "one pure layout, two
/// consumers" convention every other editor pane already keeps.
#[derive(Debug, Clone)]
pub(crate) struct FormLayout {
    pub(crate) overlay: Rect,
    pub(crate) title: &'static str,
    pub(crate) fields: Vec<FormFieldLayout>,
    pub(crate) hint_lines: Vec<String>,
    pub(crate) hint_rect: Rect,
    pub(crate) children_lines: Vec<String>,
    pub(crate) children_rect: Rect,
    pub(crate) chips: Vec<(FormChipKind, &'static str, Rect)>,
}

fn form_title(form: &FormState) -> &'static str {
    match form {
        FormState::Heading { .. } => " Edit heading ",
        FormState::Text { .. } => " Edit text ",
        FormState::Code { .. } => " Edit code ",
        FormState::List { .. } => " Edit list ",
        FormState::Picture { .. } => " Edit picture ",
        FormState::TextArt { .. } => " Edit text art ",
        FormState::Container { .. } => " Edit layout ",
        FormState::AddPalette { .. } => " Add a block ",
    }
}

fn form_sections(form: &FormState) -> Vec<(FieldSlot, &'static str, u16)> {
    let n = |len: usize| (len as u16).max(1);
    match form {
        FormState::Heading { field, .. } => {
            vec![(FieldSlot::Only, "Heading text", n(field.buffer.len()))]
        }
        FormState::Text { field, .. } => vec![(FieldSlot::Only, "Text", n(field.buffer.len()))],
        FormState::List { field, .. } => {
            vec![(FieldSlot::Only, "One item per line", n(field.buffer.len()))]
        }
        FormState::Code {
            language, source, ..
        } => vec![
            (FieldSlot::Language, "Language", n(language.buffer.len())),
            (FieldSlot::Source, "Code", n(source.buffer.len())),
        ],
        FormState::Picture { src, alt, .. } => vec![
            (FieldSlot::Src, "Image path", n(src.buffer.len())),
            (FieldSlot::Alt, "Description", n(alt.buffer.len())),
        ],
        FormState::TextArt { art, alt, .. } => vec![
            (FieldSlot::Art, "Art", n(art.buffer.len())),
            (FieldSlot::Alt, "Description", n(alt.buffer.len())),
        ],
        FormState::Container { .. } | FormState::AddPalette { .. } => Vec::new(),
    }
}

fn form_hints(form: &FormState) -> Vec<String> {
    match form {
        FormState::Picture { .. } => vec![
            "Pictures render as their description in the terminal \u{2014} the image itself never displays.".to_owned(),
        ],
        FormState::TextArt { .. } if form.art_too_wide() => vec![format!(
            "This art is wider than {} columns \u{2014} shorten it or generate a new one.",
            forms::MAX_ART_WIDTH
        )],
        _ => Vec::new(),
    }
}

/// Every add-block palette card (spec 013 T042/FR-006/FR-007): a plain
/// name plus a one-line description, in the order shown. The divider
/// kind is named "Line" here (not the raw kind string, which the
/// vocabulary gate denies) and the container kind "Columns / box /
/// stack" — the same plain names `.claude/plans/2026-07-19-wysiwyg-editor-plan.md`
/// specifies.
const PALETTE_CARDS: [(BlockKind, &str); 8] = [
    (
        BlockKind::Heading,
        "Heading \u{2014} a big title or section heading",
    ),
    (BlockKind::Text, "Text \u{2014} a paragraph of prose"),
    (
        BlockKind::Code,
        "Code \u{2014} a code sample with syntax highlighting",
    ),
    (BlockKind::List, "List \u{2014} a bulleted or numbered list"),
    (
        BlockKind::Image,
        "Picture \u{2014} an image placeholder with a caption",
    ),
    (BlockKind::Divider, "Line \u{2014} a plain horizontal rule"),
    (
        BlockKind::Container,
        "Columns / box / stack \u{2014} group blocks side-by-side, centered, or stacked",
    ),
    (
        BlockKind::AsciiArt,
        "Text art \u{2014} a banner made of characters",
    ),
];

fn form_chip_defs(form: &FormState) -> Vec<(FormChipKind, &'static str)> {
    if matches!(form, FormState::AddPalette { .. }) {
        // Unreachable via `form_layout` (which early-returns to
        // `add_palette_layout` for this variant) — kept only so this
        // match stays exhaustive over every `FormState` variant.
        return Vec::new();
    }
    let mut chips = match form {
        FormState::Picture { .. } => {
            vec![(FormChipKind::ConvertToTextArt, "[ Convert to text art ]")]
        }
        FormState::TextArt { .. } => vec![(
            FormChipKind::GenerateFromPhrase,
            "[ Generate from a phrase\u{2026} ]",
        )],
        FormState::Container { layout, .. } => vec![(
            FormChipKind::CycleLayout,
            match layout {
                ContainerLayout::Stack => "[ Layout: Stack \u{25be} ]",
                ContainerLayout::Columns => "[ Layout: Columns \u{25be} ]",
                ContainerLayout::Center => "[ Layout: Centered \u{25be} ]",
            },
        )],
        _ => Vec::new(),
    };
    chips.push((FormChipKind::Done, "[ Done ]"));
    chips.push((FormChipKind::Cancel, "[ Cancel ]"));
    chips
}

/// The add-block palette's own layout (spec 013 T042): a vertical list of
/// the 8 kind cards plus `[ Cancel ]` — distinct from the generic
/// field/hint/chip-row shape every block-edit form shares, since 8
/// plain-language cards don't fit one horizontal chip row.
fn add_palette_layout(area: Rect) -> FormLayout {
    let content_lines: u16 = 1 + PALETTE_CARDS.len() as u16 + 1 + 1;
    let overlay = form_overlay(area, content_lines);
    let inner = Rect {
        x: overlay.x + 1,
        y: overlay.y + 1,
        width: overlay.width.saturating_sub(2),
        height: overlay.height.saturating_sub(2),
    };
    let bottom = inner.y.saturating_add(inner.height);
    let mut y = inner.y.saturating_add(1);
    let mut chips = Vec::new();
    for (kind, label) in PALETTE_CARDS {
        let rect = Rect {
            x: inner.x,
            y: y.min(bottom.saturating_sub(1)),
            width: inner.width,
            height: 1,
        };
        chips.push((FormChipKind::PaletteCard(kind), label, rect));
        y = y.saturating_add(1);
    }
    y = y.saturating_add(1);
    let cancel_rect = Rect {
        x: inner.x,
        y: y.min(bottom.saturating_sub(1)),
        width: inner.width,
        height: 1,
    };
    chips.push((FormChipKind::Cancel, "[ Cancel ]", cancel_rect));
    FormLayout {
        overlay,
        title: " Add a block ",
        fields: Vec::new(),
        hint_lines: Vec::new(),
        hint_rect: Rect::new(inner.x, bottom, inner.width, 0),
        children_lines: Vec::new(),
        children_rect: Rect::new(inner.x, bottom, inner.width, 0),
        chips,
    }
}

/// A centered overlay sized to fit `content_lines` of content (plus its
/// border), clamped to `area` — the same shape `render::overlay_rect`
/// gives the presenter's own overlays, reproduced here since that helper
/// is private to the `render` module tree.
fn form_overlay(area: Rect, content_lines: u16) -> Rect {
    let w = 76u16.min(area.width.saturating_sub(4)).max(20);
    let h = (content_lines + 2).min(area.height.saturating_sub(2));
    Rect {
        x: area.x + area.width.saturating_sub(w) / 2,
        y: area.y + area.height.saturating_sub(h) / 2,
        width: w,
        height: h,
    }
}

/// The open form's full layout: field rects, hint lines, and chip rects,
/// computed purely from `form` and the frame `area` — reused verbatim by
/// `render::editor::forms::draw` and this module's `form_hit`.
pub(crate) fn form_layout(form: &FormState, area: Rect) -> FormLayout {
    if matches!(form, FormState::AddPalette { .. }) {
        return add_palette_layout(area);
    }
    let sections = form_sections(form);
    let hint_lines = form_hints(form);
    let children_lines: Vec<String> = match form {
        FormState::Container { children, .. } => children.iter().map(|c| c.label.clone()).collect(),
        _ => Vec::new(),
    };

    let mut content_lines: u16 = 1; // leading blank under the title
    for (_, _, n) in &sections {
        content_lines += 1 + n + 1; // label + text rows + trailing blank
    }
    if !children_lines.is_empty() {
        content_lines += children_lines.len() as u16 + 1;
    }
    content_lines += hint_lines.len() as u16;
    content_lines += 1; // chip row

    let overlay = form_overlay(area, content_lines);
    let inner = Rect {
        x: overlay.x + 1,
        y: overlay.y + 1,
        width: overlay.width.saturating_sub(2),
        height: overlay.height.saturating_sub(2),
    };
    let bottom = inner.y.saturating_add(inner.height);

    let mut y = inner.y.saturating_add(1);
    let mut fields = Vec::new();
    for (slot, label, n) in &sections {
        let label_y = y.min(bottom);
        let text_h = (*n).min(bottom.saturating_sub(label_y.saturating_add(1)));
        let rect = Rect {
            x: inner.x,
            y: label_y,
            width: inner.width,
            height: (1 + text_h).min(bottom.saturating_sub(label_y)),
        };
        fields.push(FormFieldLayout {
            slot: *slot,
            label,
            rect,
        });
        y = label_y
            .saturating_add(1)
            .saturating_add(*n)
            .saturating_add(1);
    }

    let children_rect = if children_lines.is_empty() {
        Rect::new(inner.x, y.min(bottom), inner.width, 0)
    } else {
        let rect = Rect {
            x: inner.x,
            y: y.min(bottom),
            width: inner.width,
            height: (children_lines.len() as u16).min(bottom.saturating_sub(y.min(bottom))),
        };
        y = y
            .saturating_add(children_lines.len() as u16)
            .saturating_add(1);
        rect
    };

    let hint_rect = Rect {
        x: inner.x,
        y: y.min(bottom),
        width: inner.width,
        height: (hint_lines.len() as u16).min(bottom.saturating_sub(y.min(bottom))),
    };
    y = y.saturating_add(hint_lines.len() as u16);

    let chip_row = Rect {
        x: inner.x,
        y: y.min(bottom.saturating_sub(1).max(inner.y)),
        width: inner.width,
        height: 1,
    };
    let mut chips = Vec::new();
    let mut cx = chip_row.x;
    for (action, label) in form_chip_defs(form) {
        let w = (label.chars().count() as u16).min(chip_row.width);
        chips.push((
            action,
            label,
            Rect {
                x: cx.min(chip_row.right().saturating_sub(w)),
                y: chip_row.y,
                width: w,
                height: 1,
            },
        ));
        cx += w + 1;
    }

    FormLayout {
        overlay,
        title: form_title(form),
        fields,
        hint_lines,
        hint_rect,
        children_lines,
        children_rect,
        chips,
    }
}

/// Resolves a click against the currently open form's layout — fully
/// modal while a form is open (per `contracts/hit-testing.md`: "a form,
/// when open, captures the area it occupies"): every other region (toolbar,
/// outline, canvas, status) is unreachable until the form closes.
fn form_hit(app: &EditorApp, area: Rect, col: u16, row: u16) -> Option<Target> {
    let form = app.open_form()?;
    let layout = form_layout(form, area);
    if let Some((action, _, _)) = layout
        .chips
        .iter()
        .find(|(_, _, r)| rect_contains(*r, col, row))
    {
        return Some(Target::FormChip(*action));
    }
    layout
        .fields
        .iter()
        .find(|f| rect_contains(f.rect, col, row))
        .map(|f| Target::FormField(f.slot))
}

fn status_hit(app: &EditorApp, status: Rect, col: u16, row: u16) -> Option<Target> {
    if !rect_contains(status, col, row) || app.status().is_empty() {
        return None;
    }
    Some(Target::StatusBanner)
}

/// Which interactive region (if any) sits at `(col, row)` of the just-drawn
/// frame. Priority order (top-most drawn wins), per the contract: an open
/// form captures the whole frame first; otherwise toolbar chips > canvas
/// (block/insertion-slot) > outline rows > the selected block's hint-line
/// chip > status banner > `None`.
#[must_use]
pub(crate) fn hit(app: &EditorApp, area: Rect, col: u16, row: u16) -> Option<Target> {
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        return None;
    }
    if app.open_form().is_some() {
        return form_hit(app, area, col, row);
    }
    let areas = editor_areas(area);
    toolbar_hit(areas.toolbar, col, row)
        .or_else(|| canvas_hit(app, areas.canvas, col, row))
        .or_else(|| outline_hit(app, areas.outline, col, row))
        .or_else(|| hint_hit(app, areas.hint, col, row))
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

    // ─── InsertionSlot / drag resolution (spec 013 US2, T045) ────────────

    #[test]
    fn the_gap_between_two_blocks_resolves_to_an_insertion_slot() {
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

        // The row between block 0 and block 1 is the gap `render_blocks`
        // joins them with — neither block's own extent, per
        // `block_extents`'s doc.
        let gap_row = extents[1].0 - 1;
        let row = inner.y + gap_row as u16;
        assert_eq!(
            hit(&app, area(), inner.x, row),
            Some(Target::InsertionSlot("a".to_owned(), Vec::new(), 1))
        );
    }

    #[test]
    fn empty_slide_resolves_the_whole_card_to_slot_zero() {
        const EMPTY: &str = r#"{"nodes":[{"id":"a","title":"Blank","content":[]}]}"#;
        let app = EditorApp::new(Graph::from_json(EMPTY).expect("fixture parses"));
        let areas = editor_areas(area());
        let target = hit(&app, area(), areas.canvas.x + 5, areas.canvas.y + 2);
        assert_eq!(
            target,
            Some(Target::InsertionSlot("a".to_owned(), Vec::new(), 0))
        );
    }

    #[test]
    fn resolve_drop_slot_snaps_to_the_nearer_half_of_a_block() {
        // A body long enough to wrap across several rows at this width, so
        // the block has a genuine top/bottom half to snap to.
        const TALL: &str = r#"{"nodes":[{"id":"a","title":"Welcome","content":[
            {"kind":"divider"},
            {"kind":"text","body":"one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty"}
        ]}]}"#;
        let app = EditorApp::new(Graph::from_json(TALL).expect("fixture parses"));
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

        let (start, end) = extents[1]; // block 1, the wrapped text block
        assert!(end - start > 1, "fixture must wrap across multiple rows");
        // A pointer in the top half of block 1 resolves to "insert before
        // block 1" (slot 1); the bottom half resolves to "after" (slot 2).
        assert_eq!(
            resolve_drop_slot(&app, "a", areas.canvas, inner.x, inner.y + start as u16),
            Some(1)
        );
        assert_eq!(
            resolve_drop_slot(&app, "a", areas.canvas, inner.x, inner.y + (end - 1) as u16),
            Some(2)
        );
    }

    #[test]
    fn resolve_drop_slot_is_none_off_the_matching_nodes_canvas() {
        let app = app();
        let areas = editor_areas(area());
        assert_eq!(
            resolve_drop_slot(
                &app,
                "does-not-exist",
                areas.canvas,
                areas.canvas.x,
                areas.canvas.y
            ),
            None
        );
    }

    #[test]
    fn add_below_chip_and_delete_chip_resolve_once_a_block_is_selected() {
        let mut app = app();
        app.selection = Selection::Block("a".to_owned(), vec![1]);
        let chips = selected_block_chips(&app);
        assert!(chips.iter().any(|(a, _)| *a == BlockAction::AddBelow));
        assert!(chips.iter().any(|(a, _)| *a == BlockAction::Delete));
    }
}
