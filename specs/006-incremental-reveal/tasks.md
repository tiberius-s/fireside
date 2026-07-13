# Tasks: Incremental reveal

**Input**: Design documents from `/specs/006-incremental-reveal/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md, ADR-009

**Tests**: Included — Test Discipline (constitution VII) requires unit
tests at the model/engine layer, scenario tests for user-visible TUI
state, and a tmux smoke test for this presenter-facing keypress change.

**Organization**: Tasks are grouped by user story per spec.md priorities
(US1 P1, US2 P2, US3 P3), preceded by a Foundational phase since the wire
field, core model, and engine state-machine changes are shared
prerequisites every story depends on.

## Phase 1: Setup

- [X] T001 Re-read the current state of every file this feature touches
      immediately before editing (line numbers may have shifted since
      planning): `protocol/main.tsp`, `crates/fireside-core/src/model/mod.rs`,
      `crates/fireside-engine/src/session.rs`,
      `crates/fireside-engine/src/validation.rs`,
      `crates/fireside-tui/src/app.rs`,
      `crates/fireside-tui/src/render/blocks.rs`,
      `crates/fireside-tui/src/render/mod.rs`, `protocol/validate.mjs`

## Phase 2: Foundational (blocking prerequisite for all user stories)

**Goal**: The `reveal` field exists on the wire, in the Rust model, and
`Session` tracks/consumes reveal state. No story is independently testable
until this phase is done.

- [X] T002 In `protocol/main.tsp`, add `model Revealable { @minValue(0) reveal?: int32; }` (per contracts/reveal-field.md) near the Content Blocks section, above `union ContentBlock`
- [X] T003 In `protocol/main.tsp`, spread `...Revealable;` into all seven block models (`HeadingBlock`, `TextBlock`, `CodeBlock`, `ListBlock`, `ImageBlock`, `DividerBlock`, `ContainerBlock`)
- [X] T004 In `protocol/main.tsp`, add `v0_1_2: "0.1.2"` to the `Versions` enum and update the "## Protocol Version" doc banner per data-model.md
- [X] T005 In `protocol/main.tsp`, rewrite `TraversalOps.next()`'s doc comment per contracts/next-operation.md's revised algorithm (reveal step precedes branch-point/next-target)
- [X] T006 Run `cd protocol && npm run build`; commit the regenerated `protocol/tsp-output/` output (per constitution Operational Constraints)
- [X] T007 In `crates/fireside-core/src/model/mod.rs`, add `reveal: Option<u32>` (with `#[serde(skip_serializing_if = "Option::is_none")]` and a doc comment) to all seven `ContentBlock` variants
- [X] T008 In `crates/fireside-core/src/model/mod.rs`, add `ContentBlock::reveal(&self) -> Option<u32>` accessor matching over all seven variants
- [X] T009 In `crates/fireside-core/src/model/mod.rs`, add `Node::reveal_levels(&self) -> Vec<u32>` — recursively walks `content` (including `Container::children`), collects `reveal` values `>= 1`, dedups, sorts ascending
- [X] T010 [P] Add unit test `reveal_field_round_trips_and_defaults_to_none` in `crates/fireside-core/src/model/mod.rs`'s test module: a block with `reveal: 2` parses and re-serializes it; a block with no `reveal` key has `reveal() == None` and omits the key on serialize (matching `round_trip_preserves_absent_fields`'s existing style)
- [X] T011 [P] Add unit test `reveal_levels_collects_distinct_positive_values_recursively` in `crates/fireside-core/src/model/mod.rs`: a node with top-level and container-nested blocks at reveal `1`, `1`, `3`, and one block with no reveal (or `reveal: 0`); assert `reveal_levels() == [1, 3]` (deduped, sorted, zero/absent excluded)
- [X] T012 [P] Add unit test `reveal_levels_is_empty_when_no_block_uses_reveal` in `crates/fireside-core/src/model/mod.rs`: a normal node (e.g. built from `HELLO`'s first node) has `reveal_levels().is_empty()`
- [X] T013 In `crates/fireside-engine/src/session.rs`, add `Outcome::Revealed` variant with a doc comment (distinct from `Moved` — node did not change)
- [X] T014 In `crates/fireside-engine/src/session.rs`, add private `reveal_level: u32` field to `Session`, initialized to `0` in `Session::new`
- [X] T015 In `crates/fireside-engine/src/session.rs`, reset `reveal_level = 0` inside `move_to` (covers `next`/`choose`/`goto`) and separately inside `back` (which bypasses `move_to`)
- [X] T016 In `crates/fireside-engine/src/session.rs`, rewrite `Session::next()` per contracts/next-operation.md: check `current().reveal_levels()` for the first value greater than `reveal_level`; if found, set `reveal_level` to it and return `Outcome::Revealed` before any branch-point/next-target check; otherwise fall through to the existing (unchanged) logic
- [X] T017 In `crates/fireside-engine/src/session.rs`, add `Session::reveal_level(&self) -> u32`, `Session::has_pending_reveal(&self) -> bool`, and `Session::reveal_progress(&self) -> Option<(usize, usize)>` (per data-model.md), each with `#[must_use]` and a doc comment
- [X] T018 [P] Add unit test `next_reveals_one_distinct_step_at_a_time_before_moving` in `crates/fireside-engine/src/session.rs`: a node with two content blocks at `reveal: 1` and `reveal: 2` and a `next` target; assert the first two `next()` calls return `Revealed` (without changing `current().id`) and the third returns `Moved`
- [X] T019 [P] Add unit test `next_skips_gaps_in_reveal_numbering_without_a_dead_step` in `crates/fireside-engine/src/session.rs`: blocks at `reveal: 1` and `reveal: 5`; assert exactly two `Revealed` outcomes total, not five
- [X] T020 [P] Add unit test `next_reveals_before_blocking_on_branch_point` in `crates/fireside-engine/src/session.rs`: a node with one `reveal: 1` block and a branch-point; assert `next()` returns `Revealed` first, then `BlockedByBranch` only after
- [X] T021 [P] Add unit test `next_reveals_before_reporting_end_of_path` in `crates/fireside-engine/src/session.rs`: a terminal node (no traversal) with one `reveal: 1` block; assert `next()` returns `Revealed` then `EndOfPath`
- [X] T022 [P] Add unit test `reveal_resets_on_every_node_entry_including_back` in `crates/fireside-engine/src/session.rs`: fully reveal a node, `next()` past it, `back()` to it; assert `has_pending_reveal()` is true again and `reveal_progress()` shows zero revealed
- [X] T023 [P] Add unit test `reveal_progress_is_none_for_ordinary_nodes` in `crates/fireside-engine/src/session.rs`: using `HELLO`'s entry node (no reveal marks), assert `reveal_progress().is_none()` and `has_pending_reveal()` is `false`

**Checkpoint**: `reveal` field exists on the wire and in the model;
`Session::next()` consumes reveal steps correctly; `cargo test -p
fireside-core -p fireside-engine` passes. No UI changes yet — nothing is
visibly different in the TUI until Phase 3.

---

## Phase 3: User Story 1 - Presenter reveals bullets one at a time (Priority: P1)

**Goal**: Reveal-marked content is hidden/shown structurally in the TUI,
gated correctly against branch-point keys, and the footer shows progress.

**Independent Test**: Per spec.md US1 — a node with reveal-marked bullets;
pressing "next" reveals one group at a time with footer feedback, then
falls through to normal advancement once exhausted.

- [X] T024 [US1] In `crates/fireside-tui/src/render/blocks.rs`, add a `reveal_level: u32` parameter to `render_blocks` and `render_block`; filter out any block whose `reveal().unwrap_or(0) > reveal_level` before rendering it (for `ContentBlock::Container`, filter `children` the same way before computing layout, and thread `reveal_level` into the recursive call)
- [X] T025 [US1] In `crates/fireside-tui/src/render/mod.rs`'s `node_lines`, pass `app.session().reveal_level()` through to `blocks::render_blocks`
- [X] T026 [US1] In `crates/fireside-tui/src/app.rs`, add an `Outcome::Revealed` arm to `App::apply()`: clear `flash`, reset `scroll` to `0`; do not touch `fade_started` or `branch_selected`
- [X] T027 [US1] In `crates/fireside-tui/src/app.rs`, change `on_present_key`'s `at_branch` computation to `self.session.branch_point().is_some() && !self.session.has_pending_reveal()`
- [X] T028 [US1] In `crates/fireside-tui/src/render/mod.rs`'s `draw_footer`, prepend a `"{revealed}/{total} revealed"` badge (accent, bold) followed by the existing separator style, shown only when `session.reveal_progress()` is `Some((revealed, total))` with `revealed < total`; existing hint arrays and their selection logic are otherwise unchanged
- [X] T029 [P] [US1] Add scenario test `reveal_hides_content_until_next_is_pressed_enough_times` in `crates/fireside-tui/src/render/mod.rs`: a node with an always-visible block and two reveal-gated blocks (`reveal: 1`, `reveal: 2`); assert initial screen shows only the always-visible text and a "0/2 revealed" (or equivalent) footer badge; after one Space press, the first reveal text is visible and badge reads progress; after a second Space press, both are visible and the badge is gone
- [X] T030 [P] [US1] Add scenario test `reveal_then_next_advances_normally_once_exhausted` in `crates/fireside-tui/src/render/mod.rs`: a two-node deck where node one has one `reveal: 1` block and a `next` target; assert two Space presses are needed to reach node two (first reveals, second navigates)
- [X] T031 [P] [US1] Add scenario test `branch_keys_are_inert_while_reveal_is_pending` in `crates/fireside-tui/src/render/mod.rs`: a node with a `reveal: 1` block and a branch-point with two options; assert pressing a branch-selection key (e.g. `1`) before revealing does not navigate to that option's target, and Space instead reveals; after revealing, the same key selects the option normally
- [X] T032 [P] [US1] Add scenario test `reveal_marks_do_not_change_a_deck_that_never_uses_them` in `crates/fireside-tui/src/render/mod.rs`: render `HELLO`'s existing content flow before/after this feature's changes (reuse or extend an existing hello.json scenario test) and assert no footer badge appears and content is unchanged — the zero-visual-change regression guarantee (FR-013)
- [X] T033 [US1] Smoke-test in tmux: launch `./target/debug/fireside present <reveal fixture>`, press Space repeatedly, and visually confirm bullets appear one at a time with a footer progress indicator, matching this project's established practice of not trusting `TestBackend` alone for interactive keypress-driven behavior (see `feedback_tmux_smoke_catches_timing_bugs` precedent)

**Checkpoint**: US1 is fully functional and independently demonstrable —
a presenter can reveal a slide's bullets one at a time with clear
footer feedback, verified in a real terminal.

---

## Phase 4: User Story 2 - Reveal composes with side-by-side layouts (Priority: P2)

**Goal**: A hidden column reserves no width; revealing it restores the
normal columns arrangement.

**Independent Test**: Per spec.md US2 — a two-column container with one
column reveal-gated; the visible column uses full width until the hidden
one is revealed.

- [X] T034 [US2] Add scenario test `hidden_column_reserves_no_width_until_revealed` in `crates/fireside-tui/src/render/mod.rs`: a `container { layout: "columns" }` with two children, the second `reveal: 1`; assert the first render shows the first column's content using the space a single (not half) column would use, and the second column's content is entirely absent from the screen; after revealing, assert both columns' content is present side by side
- [X] T035 [US2] Manually verify Scenario 4 from quickstart.md in tmux at 80×24 (visual confirmation that the reveal-gated column doesn't leave a visible gap before it appears)

**Checkpoint**: US2 composes correctly with the existing `columns` layout
— confirmed both by an automated width assertion and a real-terminal
visual check.

---

## Phase 5: User Story 3 - Author is warned about a reveal mistake (Priority: P3)

**Goal**: A new symmetric `reveal-masked-by-container` warning fires in
both validators when a child's reveal value is unreachable.

**Independent Test**: Per spec.md US3 — a fixture with a masked child
produces the warning in both validators; a clean fixture produces nothing.

- [X] T036 [P] [US3] In `crates/fireside-engine/src/validation.rs`, add `check_reveal_masked_by_container(graph, diags)`: for every `ContainerBlock` with `reveal = Some(n)`, walk its immediate and nested children, warning (rule id `reveal-masked-by-container`) on any child whose own `reveal = Some(m)` with `m < n`; wire it into `validate()`'s call chain alongside the existing warning rules
- [X] T037 [P] [US3] In `protocol/validate.mjs`, add `checkRevealMaskedByContainer(graph)` mirroring the Rust logic exactly (same rule id, same message tone); wire into `validate()`'s array-spread chain; update `HELP` text to list the new warning
- [X] T038 [P] [US3] Add unit test `reveal_masked_by_container_warns` in `crates/fireside-engine/src/validation.rs`: a container `reveal: 2` with a child `reveal: 1`; assert the warning fires naming the child
- [X] T039 [P] [US3] Add unit test `reveal_not_masked_when_child_reveal_is_greater_or_equal` in `crates/fireside-engine/src/validation.rs`: a container `reveal: 1` with children at `reveal: 1` and `reveal: 2`; assert no warning fires
- [X] T040 [US3] Add two new fixtures under `protocol/fixtures/valid/` (both belong in the `valid/` bucket, since WARNING severity does not flip `has_errors`): `reveal-not-masked.json` (container `reveal: 1`, child `reveal: 2` — expects zero warnings) and `reveal-masked-by-container.json` (container `reveal: 2`, child `reveal: 1` — expects exactly the `reveal-masked-by-container` warning); add both to `protocol/fixtures.expected.json` with their expected rule-id sets
- [X] T041 [US3] Run `cargo test -p fireside-engine --test fixtures` and `npm run test:fixtures --prefix protocol`; confirm both pass with the new fixtures included

**Checkpoint**: US3 is independently verifiable — the new rule fires
identically in both validators, proven via the shared fixture corpus.

---

## Phase 6: Regression & Composition

- [X] T042 Run every pre-existing `fireside-core`, `fireside-engine`, and `fireside-tui` test unmodified; confirm all still pass (proves this feature is additive, not a behavior change for reveal-free content)
- [X] T043 Diff-check `docs/examples/hello.json`'s presentation (tmux capture or existing scenario test) before/after this feature; confirm byte-for-byte identical footer/content behavior (SC-003)

## Phase 7: Polish & Cross-Cutting

- [X] T044 Update `docs/src/content/docs/spec/traversal.md`'s "Operation: Next" section with the reveal-precedes-everything algorithm and the ordinal-over-distinct-values rule (contracts/next-operation.md, contracts/reveal-field.md)
- [X] T045 Update `docs/src/content/docs/spec/appendix-content-blocks.md` with the `reveal` field's semantics and the compatibility/degrade guarantee
- [X] T046 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets`; both must be clean
- [X] T047 Run `npm run check --prefix docs`; must be clean
- [X] T048 Run `graphify update .` to refresh the knowledge graph
- [X] T049 Update `.claude/plans/2026-07-12-strategic-improvement-plan.md`'s Progress Log: mark "P1 incremental reveal" done, with a technical summary matching the style of existing entries
- [X] T050 Update the `project_strategic_plan_2026_07` memory file and `MEMORY.md` index to reflect this feature's completion

---

## Dependencies & Execution Order

- **Phase 1** has no dependencies.
- **Phase 2 (Foundational)**: T002-T006 (protocol) must land in order (same file, sequential edits then a build). T007-T009 (core model) depend on nothing in Phase 2 except conceptually following the protocol shape, but are independent files — can start in parallel with T002-T006. T010-T012 depend on T007-T009. T013-T017 (engine) depend on T007-T009 (need `reveal()`/`reveal_levels()` to exist). T018-T023 depend on T013-T017.
- **Phase 3 (US1)**: depends entirely on Phase 2 being complete (needs `Session::reveal_level`/`has_pending_reveal`/`Outcome::Revealed`). T024-T028 are sequential (same files, dependent logic). T029-T032 depend on T024-T028; independent of each other — marked [P]. T033 depends on T024-T032 all landing.
- **Phase 4 (US2)**: depends on T024 (the container-child filtering logic) from Phase 3 already existing — not on the rest of Phase 3's UI polish. Can start once T024-T025 land.
- **Phase 5 (US3)**: depends only on Phase 2 (T007-T009, the `reveal` field existing in the model) — independent of Phases 3 and 4 entirely. Could be executed in parallel with Phase 3/4 by a different task run. T036-T037 [P] (different files); T038-T039 [P] (same file, different tests, safe to write together); T040 depends on T036-T037; T041 depends on T040.
- **Phase 6**: depends on all of Phases 2-5.
- **Phase 7**: depends on Phase 6.

## Implementation Strategy

**MVP scope**: Phase 1 + Phase 2 + Phase 3 (US1) delivers the entire
user-visible value named by the plan as "the single most-expected
presenter feature Fireside lacks." Phase 4 (US2) hardens composition with
an existing layout mode. Phase 5 (US3) is an authoring-safety nicety.
Phase 6 is the regression proof this project's "whole stage at a time"
convention requires. Phase 7 is mandatory wrap-up (docs, verification,
progress tracking).
