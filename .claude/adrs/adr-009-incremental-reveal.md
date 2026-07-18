---
title: 'ADR-009: Incremental reveal via a `reveal` ContentBlock field'
status: 'accepted'
date: '2026-07-12'
deciders: ['@tiberius']
---

# ADR-009: Incremental reveal via a `reveal` ContentBlock field

## Status

Accepted

## Context

The strategic plan (`.claude/plans/2026-07-12-strategic-improvement-plan.md`,
§2 P1 "Incremental reveal") names this "the single most-expected presenter
feature Fireside lacks" and explicitly requires spec-first work because it
changes the `next()` contract (`protocol/main.tsp`'s `TraversalOps.next()`),
not just engine-internal behavior. Two open questions from §4 needed
settling before implementation:

1. **Reveal semantics at branch points** — do reveal steps precede the
   branch menu?
2. What the field's value actually means — a raw counter, or something else?

Read the current reference implementation before deciding anything:
`fireside_engine::Session::next()` (`crates/fireside-engine/src/session.rs`)
checks `branch_point().is_some()` first, then `next_target()`; `App::apply()`
(`crates/fireside-tui/src/app.rs`) turns every `Outcome` into presenter
feedback (a flash, a scroll reset, or a visible slide change) — "every
keypress gets feedback" is an existing, load-bearing convention, not new for
this feature. `ContentBlock` (`crates/fireside-core/src/model/mod.rs`) is a
flat, kind-discriminated enum with no shared base — `main.tsp`'s
`ContentBlock` union has no spread/extends pattern in use anywhere in the
file yet.

Considered two designs for the field's semantics:

- **Raw magnitude comparison**: block visible when `current_step >=
  block.reveal`. Simple, but an author who writes `reveal: 1` then
  `reveal: 3` (skipping 2, e.g. because they reordered blocks and left gaps)
  produces a `next()` press that reveals nothing — a silent dead keypress,
  which violates this project's "every keypress gets feedback" convention
  outright.
- **Ordinal over distinct declared values** (chosen): collect every distinct
  *positive* `reveal` value present in a node's content (recursively,
  including inside containers), sort ascending, and let `next()` advance
  through that sequence one distinct value at a time. Two blocks sharing the
  same `reveal` value reveal together (a natural "fragment group"); gaps in
  the numbering are simply impossible to create by construction, since the
  step sequence is derived from whatever values the author actually used,
  not from the raw integers themselves.

For branch-point ordering: the two behaviors don't compose cleanly if
interleaved (revealing content mid-branch-menu has no clear reference
behavior to point to, unlike the six ambiguities ADR-007 resolved). Decision:
**reveal is entirely a property of the current node's `content` array,
independent of `traversal`. `next()` always exhausts all reveal steps before
ever checking `branch-point` or `next`.** This means a presenter reveals
every bullet on a slide, and only then is offered a choice (or advances, or
hits the end of the path) — matches how every reference presentation tool
with fragments behaves (reveal, then the slide's "real" exit).

## Decision

Add an optional `reveal?: int32` field (`@minValue(0)`) to all seven
`ContentBlock` variants in `main.tsp`, via a shared spread model
(`model Revealable { reveal?: int32 }`, spread with `...Revealable` into
each block model) rather than seven duplicated doc comments. Absent and `0`
are equivalent: visible immediately. This is a new field, so the protocol
version bumps 0.1.1 → 0.1.2 (`Versions` enum gains `v0_1_2`). Old engines
ignore the unknown field per ADR-004's forward-compatibility guarantee and
render everything immediately — the correct, honest degrade the plan
specifies.

`fireside_core::model::ContentBlock` gains a `reveal: Option<u32>` field on
every variant, plus a `reveal()` accessor and `Node::reveal_levels(&self) ->
Vec<u32>` (sorted, deduped, positive-only, walked recursively through
`Container::children`) — a pure structural computation, so it lives in
`fireside-core` alongside `is_terminal()`/`next_target()`, not duplicated in
the engine.

`Session` gains a `reveal_level: u32` field (the currently-reached value,
not a step index), reset to `0` on every node entry — `move_to` already
covers `next`/`choose`/`goto`; `back()` needs the same reset added since it
bypasses `move_to`. `Session::next()` now checks `reveal_levels()` for the
first value greater than `reveal_level` before doing anything else; if
found, it advances `reveal_level` and returns a new `Outcome::Revealed`
(distinct from `Moved` — the node did not change, only its visible content
did, so the UI must not reset fade/branch-selection state as if a real
navigation happened). Only once no more reveal levels remain does `next()`
fall through to today's `branch_point()`/`next_target()` logic, unchanged.
`Session::has_pending_reveal(&self) -> bool` and
`Session::reveal_progress(&self) -> Option<(usize, usize)>` (revealed count,
total count; `None` when the node uses no reveal fields at all, so ordinary
decks show nothing new) expose this for the UI.

`fireside-tui`: `App::on_present_key` gates branch-menu key routing on
`!session.has_pending_reveal()` in addition to the existing
`branch_point().is_some()` check — otherwise a presenter could bypass
pending reveals by choosing early. `App::apply()` gets an `Outcome::Revealed`
arm (clears flash, resets scroll so newly revealed content is in view; no
fade, no branch-selection reset). `blocks::render_blocks`/`render_block`
gain a `reveal_level: u32` parameter; a block whose own `reveal().unwrap_or(0)
> reveal_level` is skipped entirely (not rendered dim — genuinely absent,
so hidden content never reserves layout space, e.g. in a `columns`
container). The footer (`draw_footer`) shows a `"{revealed}/{total} revealed"`
badge only while `has_pending_reveal()` is true; the existing static hint
text ("Space next", branch hints, terminal hints) is otherwise unchanged,
selected via the same `has_pending_reveal()` gate ahead of the existing
branch/terminal/flow three-way split.

One new validator warning, symmetric in both `fireside-engine::validation`
and `protocol/validate.mjs`: `reveal-masked-by-container` — a child block
whose own `reveal` value is less than its container ancestor's `reveal`
value can never actually appear before the container does, so the lower
number is misleading rather than functional. Same tone/pattern as the
existing `empty-traversal`/`unreachable-node` warnings from ADR-007.

## Consequences

### Positive

- Zero new crate dependencies; zero crate-boundary changes — this is a
  content-model and state-machine extension using types already in scope
  everywhere it touches.
- The "distinct ordinal values" design makes dead keypresses structurally
  impossible, keeping faith with the project's existing "every keypress
  gets feedback" invariant instead of introducing an exception to it.
- Old 0.1.0/0.1.1 documents are unaffected (no reveal fields present means
  `reveal_levels()` is always empty, `has_pending_reveal()` is always
  `false`, and `next()`'s new check is a no-op before falling through to
  identical pre-existing behavior) — verified by construction, not just
  by intent, since the check is "any level greater than current" over an
  empty list.

### Negative or Trade-offs

- `next()` now has an extra branch presenters must mentally model: "Space"
  can mean either "reveal more" or "go to the next slide" depending on
  node content, distinguished only by the footer badge. This is the
  standard trade-off every reveal/fragment feature makes (PowerPoint,
  presenterm, patat all share it) — considered acceptable and flagged
  explicitly rather than glossed over.
- `reveal_level` is deliberately *not* part of `history` — going back to a
  node always restarts its reveal at 0, even if the presenter had already
  revealed everything on a first visit. Simpler and more predictable than
  trying to remember per-node reveal progress across an arbitrary path,
  but it does mean re-visiting a slide replays its reveals.
- Container-level reveal masking (child hidden behind a later container
  reveal) is a real authoring footgun the new warning catches, but only
  after the fact (on save/validate), same limitation every other warning
  rule in this project already has.

### Neutral / Follow-up

- Protocol version 0.1.2. `docs/examples/hello.json` is not required to
  bump (same reasoning as ADR-007: older documents remain valid against
  newer engines).
- Regenerate and commit `protocol/tsp-output/` per the constitution's
  Operational Constraints.
- Update `docs/src/content/docs/spec/traversal.md`'s "Operation: Next"
  section with the reveal-precedes-branch-point rule and the
  distinct-ordinal-values algorithm, and
  `appendix-content-blocks.md`/`main.tsp` doc comments for the `reveal`
  field itself.
- Extend `protocol/fixtures/` with cases for the new
  `reveal-masked-by-container` warning, following the existing
  `fixtures.expected.json` pattern from ADR-007's corpus.

### Follow-up (2026-07-18)

Same reversal as ADR-007's follow-up: `hello.json` is no longer treated
as a frozen compat baseline — it's bumped to `"fireside-version":
"0.1.3"` as of the ascii-art feature. This ADR's "not required to bump"
reasoning for 0.1.2/reveal is left as historical record; reveal marks
themselves were **not** retroactively backfilled into `hello.json` in
this pass — flagged as a separate, later decision if wanted, not bundled
into the ascii-art fix. See ADR-012's follow-up for the full rationale.
