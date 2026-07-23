# Contract: nested block selection, hit-testing, and form access

Extends `specs/013-authoring-editor/contracts/hit-testing.md`, which this
document does not restate in full — read that first. Nothing here changes
that contract's `Target` enum shape or priority order; it changes which
depths of `BlockPath` the existing variants are allowed to carry, and adds
one narrowly-scoped new hit region for the container form's child list.

## What does not change

- `Target::Block(NodeId, BlockPath)`, `Target::BlockChip(NodeId,
  BlockPath, BlockAction)`, and `Target::InsertionSlot(NodeId, BlockPath,
  usize)` already carry a full `BlockPath`, not a top-level index — spec
  013's contract table already generalizes to nested paths (its own
  wording). No new `Target` variant is needed for canvas selection,
  chips, or insertion slots.
- Priority order (toolbar > open-form chips > canvas regions > outline
  rows > status banner > `None`) is unchanged.
- `hit()`'s purity guarantee (recomputes the same layout the last frame
  drew; no render-to-update back-channel) is unchanged and MUST continue
  to hold for the recursive case.

## What changes: `Target::Block`/`BlockChip`/`InsertionSlot` may now carry
a `BlockPath` of length > 1

Previously only ever constructed with length-1 paths. This contract
requires:

1. `hit::canvas_hit` MUST resolve a click on a container child's own
   rendered text to `Target::Block(node_id, path)` where `path` has the
   child's index appended to its container's path — not to the
   container's own (shorter) path.
2. A selected block's contextual chips (`Target::BlockChip`) MUST be
   computed and hit-testable for a selected *child* exactly as they
   already are for a selected top-level block (same chip set: `✎ Edit`,
   `＋ Add below`, `Reveal ▾`, `Delete`; `↑`/`↓` are dead code already per
   the 2026-07-23 audit and MUST NOT be revived by this feature).
3. `Target::InsertionSlot(node_id, path, at)` MUST be resolvable *between*
   a container's children (path = the container's path, `at` = the
   position within its children) in addition to the existing top-level
   gaps, so User Story 3 (add a block inside a container) has somewhere
   to click.

## New: container form child rows become hit-testable

The container form (`FormState::Container { path, children, .. }`) is
drawn while open, same as any other form — per spec 013's priority order,
open-form chips already sit above canvas regions. This feature adds:

```text
Target::FormChip(FormChipKind::ContainerChild(usize))
```

Row index into `children: Vec<ChildSummary>` (`FormChipKind` is the
existing enum from spec 013's `hit.rs`; this is one new variant on it,
not a new top-level `Target` case). Click → close the container form and
open the child's own form, using the `BlockPath` obtained by appending
the row's index to the container form's own `path` (per data-model.md's
`ChildSummary` note: the row's position **is** its child index).

## Tab/Shift+Tab cycling contract

Not part of `hit()` (that's mouse-only) — `editor::mod.rs`'s
`select_adjacent_block` gets its own, smaller contract:

```text
fn select_adjacent_block(&mut self, backward: bool)
```

Walking order is a pre-order flattening of the selected node's content
tree: each top-level block, and immediately after a `Container` block,
each of its children in order, before moving to the container's next
top-level sibling. `backward` reverses the same order. A container with
no children is a single stop (as it is today) — nothing to descend into.

## Test contract

Extends spec 013's hit-testing test contract with the same
table-driven `(EditorApp fixture, area, col, row) -> expected Target`
shape, plus new required cases: a click on a container child's rendered
text resolves to `Target::Block` with a length-2 path; the same
container's other (unclicked) children and the container's own
non-child chrome do not; a click on a container-child chip resolves to
`Target::BlockChip` with the matching length-2 path; a click between two
children resolves to `Target::InsertionSlot`; a click on a container
form's child row resolves to the new `Target::FormChip`
`ContainerChild(usize)`; Tab from a container's last child moves to the
container's next top-level sibling, not back to the container itself.
