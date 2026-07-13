# Phase 0 Research: Incremental reveal

No `[NEEDS CLARIFICATION]` markers exist in the Technical Context — every
open question the strategic plan raised for this feature (reveal semantics
at branch points, what the field's value means, whether reveal is
history-aware) was already resolved in ADR-009 before this spec was
written. This document records the research that *fed* those ADR-009
decisions, for traceability, rather than resolving anything new.

## Decision: ordinal-over-distinct-values step semantics

- **Decision**: A node's reveal steps are the sorted, deduped, positive
  `reveal` values actually used in its content (recursively). `next()`
  advances through that sequence one distinct value at a time, not by
  literal integer increments.
- **Rationale**: Read `crates/fireside-tui/src/app.rs`'s module doc before
  deciding anything — "every keypress that cannot act produces a flash
  message — the presenter is never left wondering whether a key worked."
  A raw-magnitude design (`current_step >= block.reveal`) would let an
  author's own numbering gap (`reveal: 1` then `reveal: 3`, no `2` used
  anywhere) silently produce a `next()` press that reveals nothing —
  violating that convention outright, and doing so in a way the author
  might never notice until a live presentation. The ordinal design makes
  a "dead" reveal keypress structurally impossible.
- **Alternatives considered**: Raw magnitude comparison (rejected, above).
  A step index separate from the marker values entirely (e.g. author
  writes `reveal: true` only, engine assigns steps by content order) was
  also considered and rejected — it removes the ability to group multiple
  blocks into one reveal step (a common real need, e.g. a heading + its
  first bullet appearing together), which the chosen design supports for
  free via shared values.

## Decision: reveal precedes branch-point and next-target, unconditionally

- **Decision**: `next()` always fully exhausts a node's reveal steps
  before it will ever check `branch-point` or `traversal.next`.
- **Rationale**: Reveal is scoped to `content` only; `traversal` is a
  structurally separate field on `Node`
  (`crates/fireside-core/src/model/mod.rs`). There is no existing
  reference behavior to preserve here (unlike the six ambiguities
  ADR-007 resolved by reading already-settled code) — this is genuinely
  new behavior, so the decision is made on UX grounds: a presenter should
  finish narrating a slide's revealed content before being asked to make
  a decision or being moved on. This mirrors every reference presentation
  tool with fragments (reveal bullets, then the slide's real exit).
- **Alternatives considered**: Interleaving reveal steps with a
  branch-point (e.g. show the branch menu after some reveals but allow
  further reveals post-menu) was considered and rejected as needlessly
  complex with no clear reference behavior and no requester need — out of
  scope per ADR-009.

## Decision: reveal resets on every node entry, not history-aware

- **Decision**: `reveal_level` lives on `Session`, not in `history`, and
  is reset to `0` on every transition into a node (via `next`, `choose`,
  `goto`, and `back`).
- **Rationale**: `Session::history` (`crates/fireside-engine/src/session.rs`)
  is a stack of `NodeId`s only — adding per-node reveal progress to it
  would mean tracking a `HashMap<NodeId, u32>` that grows for the life of
  the session and must be kept in sync with three different navigation
  entry points (`move_to`, and `back`'s separate bypass of `move_to`).
  Simpler, and arguably more correct for a live presentation: a slide the
  presenter returns to should be re-narrated from the start, not silently
  resume mid-reveal from a memory of an earlier pass.
- **Alternatives considered**: History-aware reveal (per-node progress
  memory) — rejected in ADR-009 explicitly as unnecessary complexity for
  no requested benefit.

## Decision: structural hiding, not dim/invisible styling

- **Decision**: An unrevealed block is omitted entirely from the line flow
  `render_blocks` produces — it never reaches `Line`/`Span` construction.
- **Rationale**: Read `crates/fireside-tui/src/render/blocks.rs`'s
  `columns()` container layout before deciding — it divides available
  width by the *number* of children being laid out. If a hidden block
  were rendered as an empty-but-present slot, a two-column reveal (US2)
  would show one narrow column and one blank column instead of one column
  using the full width, which looks broken, not deliberate — the same
  "looks broken vs. deliberate" bar ADR/feature 005 (ASCII art centering)
  already established for this renderer.
- **Alternatives considered**: Rendering hidden blocks with zero-height or
  dimmed styling was considered and rejected — dimmed styling implies
  "present but de-emphasized" (already a distinct, existing concept in
  this renderer, e.g. `highlight-lines`' unfocused dimming in `code()`),
  which would be a confusing overload of meaning.

## Decision: `reveal-masked-by-container` as a WARNING, not an ERROR

- **Decision**: A child block reveal-marked earlier than its enclosing
  container is a warning, not a validation error.
- **Rationale**: Matches the existing tone and severity precedent set by
  `empty-traversal`/`unreachable-node`/`self-loop` in
  `fireside-engine::validation` (ADR-007) — these are all "probably a
  mistake, but not structurally broken" diagnostics. A masked reveal
  block still renders correctly (it simply appears no earlier than its
  container, which is not incorrect, just possibly not what the author
  intended) — nothing crashes or produces invalid output, so ERROR
  severity would be disproportionate.
- **Alternatives considered**: No new rule at all (rejected — ADR-009
  explicitly calls for it, and it is cheap to add given the existing
  dual-validator warning pattern and fixture-corpus infrastructure from
  ADR-007/feature 004).
