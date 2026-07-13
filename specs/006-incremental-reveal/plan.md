# Implementation Plan: Incremental reveal

**Branch**: `006-incremental-reveal` | **Date**: 2026-07-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/006-incremental-reveal/spec.md`

## Summary

Add an optional `reveal` field to every `ContentBlock` variant so authors
can mark content to appear progressively as the presenter advances through
a node, instead of all at once. `next()` consumes a node's reveal steps
(computed as the sorted distinct positive `reveal` values used in that
node's content, recursively through containers) before ever checking a
branch point or the node's traversal target — reveal always finishes first.
Reveal state is per-node, transient, and resets on every node entry
(including `back()`). The TUI hides not-yet-revealed blocks structurally
(no layout space reserved) and shows a "N/M revealed" footer badge only
while a reveal is in progress. A new symmetric validator warning,
`reveal-masked-by-container`, catches an authoring mistake where a child's
reveal value is unreachable because its enclosing container reveals later.
Full design already decided in ADR-009
(`.claude/adrs/adr-009-incremental-reveal.md`) — this plan implements it,
it does not re-derive it.

## Technical Context

**Language/Version**: Rust 1.88 (workspace MSRV), 2024 edition; Node.js
(ESM) for `protocol/validate.mjs`; TypeSpec for `protocol/main.tsp`.

**Primary Dependencies**: None new. Uses only what `fireside-core`,
`fireside-engine`, and `fireside-tui` already depend on (`serde`,
`ratatui`, etc. per the existing crate boundary table).

**Storage**: N/A (in-memory session state only, same as all existing
traversal state).

**Testing**: `cargo test --workspace` (unit tests in
`fireside-core`/`fireside-engine`/`fireside-tui`, scenario tests in
`fireside-tui/src/render/mod.rs`'s `TestBackend` suite); `node
protocol/run-fixtures.mjs` for the new validator rule; a tmux real-terminal
smoke test since this changes live keypress behavior in the TUI.

**Target Platform**: Cross-platform terminal (existing Fireside target;
unchanged).

**Project Type**: Rust workspace, 4 crates (existing structure; unchanged).

**Performance Goals**: N/A — reveal-step computation is over content
arrays with realistically single-digit-to-low-double-digit block counts
per node; no measurable performance target beyond "imperceptible," same
bar as all existing render/session operations.

**Constraints**: Must be a 0.1.x-additive protocol change (Principle I) —
old engines ignore the field and degrade to "everything visible," which
this plan's design already guarantees by construction (empty
`reveal_levels()` list when the field is unused).

**Scale/Scope**: Touches `protocol/main.tsp`, `fireside-core::model`,
`fireside-engine::{session,validation}`, `fireside-tui::{app,render}`, and
`protocol/validate.mjs` + fixtures. No new files beyond fixtures and the
usual spec artifacts; no new crates.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Spec Is the Source of Truth** — PASS. The `reveal` field and the
  revised `next()` contract are specified in `main.tsp` and
  `docs/src/content/docs/spec/traversal.md` before any engine code changes,
  per ADR-009. `docs/examples/hello.json` uses no reveal marks and MUST
  continue to parse/validate/present identically (spec's own regression
  bar).
- **II. Presenter-First Experience** — PASS. FR-010/SC-005 require the
  footer to always show reveal progress; FR-007 prevents a presenter from
  accidentally skipping content via a mistimed choice keypress. No new
  product surface beyond `present`/`validate`.
- **III. Crate Boundary Discipline** — PASS, no changes. No new
  dependencies in any crate; the feature is entirely new fields/methods on
  types and control flow already in scope for each crate's existing
  responsibilities (model in `fireside-core`, state machine in
  `fireside-engine`, rendering/keys in `fireside-tui`).
- **IV. Mandatory Code Idioms** — PASS. `next()` continues to return
  `Outcome` (gains one new variant, `Revealed`) — no traversal operation
  becomes a silent no-op, satisfying the existing "every keypress gets
  feedback" idiom rather than creating an exception to it. New public
  methods get `#[must_use]` and doc comments per existing convention.
- **V. Stratified Error Handling** — PASS, unaffected. No new fallible
  operations are introduced (reveal-step computation cannot fail; it
  operates on already-parsed, already-typed content).
- **VI. MSRV 1.88** — PASS, unaffected (no new dependencies).
- **VII. Test Discipline** — PASS, planned: `fireside-core` unit tests for
  `reveal_levels()`; `fireside-engine` unit tests for `Session::next()`'s
  new reveal-consuming behavior and the `reveal-masked-by-container`
  warning; `fireside-tui` scenario tests for footer badge + structural
  hiding (including inside a `columns` container) + branch-key gating; a
  tmux smoke test since this is real, presenter-facing keypress behavior
  change, matching this project's established practice of not trusting
  `TestBackend` alone for interactive-timing-sensitive behavior.

Wire-format ADR gate satisfied by ADR-009
(`.claude/adrs/adr-009-incremental-reveal.md`), written before this spec
per Principle I and the Development Workflow's ADR-before-code rule.

No violations. Complexity Tracking table not needed.

## Project Structure

### Documentation (this feature)

```text
specs/006-incremental-reveal/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md         # Phase 1 output
├── quickstart.md         # Phase 1 output
├── contracts/            # Phase 1 output
│   ├── reveal-field.md
│   └── next-operation.md
└── tasks.md              # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
protocol/
├── main.tsp                          # + Revealable spread model, reveal field on all 7 block models, v0_1_2, next() doc update
├── validate.mjs                      # + checkRevealMaskedByContainer
├── fixtures/{valid,invalid}/*.json   # + reveal fixtures
└── fixtures.expected.json            # + new fixture entries

docs/src/content/docs/spec/
├── traversal.md                      # Operation: Next — reveal-precedes-branch-point + ordinal algorithm
└── appendix-content-blocks.md        # reveal field documentation

crates/fireside-core/src/model/mod.rs # + reveal: Option<u32> on all 7 ContentBlock variants,
                                       #   ContentBlock::reveal(), Node::reveal_levels()

crates/fireside-engine/src/
├── session.rs                        # + Outcome::Revealed, Session::{reveal_level, has_pending_reveal, reveal_progress}, next() rewrite
└── validation.rs                     # + check_reveal_masked_by_container

crates/fireside-tui/src/
├── app.rs                            # + Outcome::Revealed handling in apply(), branch-key gating on has_pending_reveal()
└── render/
    ├── blocks.rs                     # render_blocks/render_block gain reveal_level param, structural hiding
    └── mod.rs                        # footer reveal badge, node_lines() passes reveal_level through
```

**Structure Decision**: No new crates or directories. This feature is a
vertical slice through the existing 4-crate layering (spec → core → engine
→ tui), which is exactly the shape the crate boundary table expects for a
new content/traversal capability — each crate gets only the piece that
belongs to its existing responsibility.

## Complexity Tracking

*No violations — table omitted.*
