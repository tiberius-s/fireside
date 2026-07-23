# Phase 0 Research: Container Block Editing

No `[NEEDS CLARIFICATION]` markers remain in the Technical Context — the
codebase was inspected directly during planning and resolved every open
question a greenfield feature would normally need research for. This
document records those findings as the decision record.

## Decision 1: Where does the depth-1 restriction actually live?

**Decision**: The restriction is confined to three call sites in
`fireside-tui::editor`, all pre-existing and each independently gated at
depth 1:

1. `editor/mod.rs::select_adjacent_block` — Tab/Shift+Tab cycling walks
   `node.content.len()` (top-level count only); never descends into a
   selected container's `children`.
2. `editor/hit.rs::block_extents` — computes each **top-level** block's
   `[start, end)` rendered line range for canvas hit-testing and drag
   drop-slot resolution; its own doc comment says nested children are
   "out of scope for the canvas." `canvas_layout`, `canvas_hit`, and
   `resolve_drop_slot` all consume this function's output, so all three
   inherit the limitation.
3. `render/editor/canvas.rs::draw_selection_marker` — explicitly checks
   `path.len() == 1` before drawing the selection glow; a selection with
   a longer path (which nothing produces today, but would be produced by
   fixing #1/#2) would silently draw no glow at all.

**Rationale**: Confirming the exact call sites (rather than assuming the
whole editor needs restructuring) keeps the feature scoped to extending
three functions plus their render/forms counterparts, not a redesign.

**Alternatives considered**: A generic "flatten the block tree into a
linear addressable list" abstraction was considered, but rejected —
`hit.rs` and `render/editor/canvas.rs` already share geometry via the
`CanvasLayout` struct specifically so drawing and hit-testing can never
disagree (existing spec 013 contract); recursing the *existing* extent
computation preserves that guarantee for free, while a separate
flattening pass would risk the two falling out of sync again.

## Decision 2: Does the engine layer need to change?

**Decision**: No. `fireside-engine::authoring::BlockPath` (`type
BlockPath = Vec<usize>`) is already documented and implemented as
addressing a block "within a node's (possibly nested, via `Container`)
content tree by index path." Every `Op` variant that takes a `path`
(`AddBlock`, `DeleteBlock`, `EditBlock`, `MoveBlock`, `SetRevealStep`)
already recurses through `Container::children` via `children_mut`/
`split_block_path` in `authoring.rs`. `editor::forms::open(node, path,
block)` is also already depth-agnostic — it matches on the `ContentBlock`
kind at `path`, not on `path`'s length, so a container child's own block
kind (heading, text, etc.) already produces the correct form the moment
it's called with a nested path and the matching block reference.

**Rationale**: This was the single highest-risk unknown going into
planning — if the engine's operation semantics didn't already generalize
to nested paths, this would be a much larger feature touching Constitution
Principle I (protocol/spec) territory. Confirming they do generalize
means this feature is TUI-only, matching the spec's own Assumptions
section.

**Alternatives considered**: None needed — this is a factual finding, not
a design choice with tradeoffs.

## Decision 3: Selection glow and drag styling for a nested block

**Decision**: Reuse the existing `Tokens::selection`/`Tokens::affordance`/
`Tokens::drop-target`/`Tokens::ghost` entries added for spec
013-authoring-editor's block editing — no new theme tokens. The rendering
change is to the *geometry* computation (recursing `block_extents` to
also return a child's sub-range within its container's own range), not to
the styling vocabulary.

**Rationale**: Constitution Principle IV requires all styling to flow
through `theme.rs::Tokens`; the existing four entries were sized for
exactly this kind of block-level affordance and need no extension.

**Alternatives considered**: A visually distinct "nested" selection style
(e.g., dimmer glow) was considered to help an author tell child selection
apart from container selection, but rejected for this pass — the
indentation/breadcrumb context (data-model.md) already disambiguates it,
and introducing a new token is unjustified scope for a completion
feature. Can be revisited as a follow-up polish item if real usage shows
confusion.

## Decision 4: Container form's child list interactivity

**Decision**: `editor::forms::FormState::Container`'s `children:
Vec<ChildSummary>` becomes selectable rows (each resolves to a `Target`
that opens that child's own form), following the same
row-selection-then-commit pattern already used by
`FormState::SlidePicker`'s `rows: Vec<PickerRow>` elsewhere in the same
file — not a new interaction pattern.

**Rationale**: Reusing an existing, already-tested interaction pattern
(`SlidePicker` rows) is lower-risk than inventing a new one, and keeps
`forms.rs`'s internal vocabulary consistent.

**Alternatives considered**: Closing the container form entirely and
directly selecting the child on the canvas (relying solely on Tab/click)
was considered simpler, but rejected — the container form's `ChildSummary`
list already exists specifically as a way to see all children at a
glance (useful when some are off-screen or hard to distinguish visually,
e.g. two dividers), so making it inert would waste an existing, useful
piece of UI that the spec's Acceptance Scenario 3 (User Story 1) already
commits to keeping functional.
