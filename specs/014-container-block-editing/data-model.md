# Phase 1 Data Model: Container Block Editing

No on-disk or wire-format entities change (Constitution Principle I is
unaffected — see research.md Decision 2). This document covers the
**runtime** entities inside `fireside-tui::editor` whose *shape* already
exists but whose *behavior* this feature extends, plus the one entity
whose computed structure changes.

## `BlockPath` (existing, reused as-is)

```text
type BlockPath = Vec<usize>   // fireside-engine::authoring
```

Already generalizes to any depth: `[i]` addresses a top-level block,
`[i, j]` addresses the `j`-th child of the container at top-level index
`i`, and so on. This feature does not change the type — it changes which
code paths in `fireside-tui` are willing to *construct* and *consume* a
path longer than one element.

- Constructed today only with length 1, by `select_adjacent_block` (Tab
  cycling) and `hit::canvas_hit` (click resolution).
- Consumed depth-agnostically already by `fireside-engine::authoring::apply`
  and `editor::forms::open` (research.md Decision 2) — no change needed
  on the consuming side.

## `Selection::Block(NodeId, BlockPath)` (existing variant, extended range)

`fireside-tui::editor::mod.rs`'s `Selection` enum already carries a full
`BlockPath`, not just a top-level index — so `Selection::Block` can
already *represent* a selected container child. The gap is entirely in
what *produces* such a selection (Tab cycling, click) and what *reads*
one back correctly (the render-side `path.len() == 1` gate in
`draw_selection_marker`, research.md Decision 1 item 3).

**State transitions added by this feature**:

- Tab/Shift+Tab from a selected container descends into its first/last
  child instead of stopping; Tab from a container's last child (or
  Shift+Tab from its first) moves to the container's own next/previous
  top-level sibling, not back into the container — cycling treats a
  container's children as sitting "between" the container and its next
  top-level sibling, consistent with how an author reads the canvas
  top-to-bottom.
- Deleting a selected child (User Story 2) moves selection to the nearest
  remaining sibling, or to the (now-empty) container itself if it was the
  last child — never to a stale path.
- Deleting or undoing-away a container while one of its children is
  selected falls back to the nearest valid top-level selection, per the
  spec's Edge Cases.

## `CanvasLayout::block_extents` (existing struct field, recursive shape)

```text
pub(crate) struct CanvasLayout {
    pub(crate) inner: Rect,
    pub(crate) block_extents: Vec<(usize, usize)>,   // today: depth-1 only
    pub(crate) scroll: u16,
}
```

**Change**: `block_extents` (or a sibling field alongside it — exact
shape is an implementation decision for `/speckit-tasks`, not frozen
here) must also expose each container's children's own `[start, end)`
sub-ranges *within* the container's own `[start, end)` range, computed
the same way the top-level ranges already are (by re-rendering an
increasing prefix and diffing cumulative line counts — the same
disagreement-proof technique `block_extents` already uses, per its own
doc comment, just applied one level deeper for a selected/hovered
container). Both `hit::canvas_hit`/`hit::resolve_drop_slot` (resolving a
click/drag) and `render/editor/canvas.rs` (drawing the glow) must read
this same computed geometry, preserving the existing "drawing and
hit-testing can never disagree" guarantee `CanvasLayout` was built to
provide (spec 013 contract, unchanged principle).

## `ChildSummary` (existing struct, gains an implicit index)

```text
pub(crate) struct ChildSummary {
    pub(crate) label: String,
}
```

Already produced one per child, in child order, by
`forms::child_summary`. This feature does not need to change the struct
itself — the row's position in the `Vec<ChildSummary>` *is* its child
index, which combined with the container form's own `path: BlockPath` is
enough to construct the full nested `BlockPath` (`path` + `[row_index]`)
needed to open that child's form. The change is in how the form's rows
are made selectable (a new `Target`-producing hit region over each row,
contracts/hit-testing.md addendum), not in this data shape.

## No new entities

This feature introduces no new persistent or long-lived struct — it
extends the reach of `BlockPath`/`Selection`/`CanvasLayout`, which already
exist and already have the right shape, into code paths that currently
stop one level too early.
