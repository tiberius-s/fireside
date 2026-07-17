# Tasks: Protocol & Workflow Hardening

**Input**: Design documents from `/specs/008-protocol-workflow-hardening/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: This entire feature *is* test/CI hardening — Constitution
Principle VII's layering (unit tests in `fireside-core`/`fireside-engine`,
`TestBackend` scenario tests in `fireside-tui`, e2e tests in
`fireside-cli`) is the deliverable, not an optional add-on. There is no
separate "write tests first" split from "implementation" for US1/US3/US4:
the test *is* the implementation. US2 has a small non-test implementation
component (the new validator rule) that precedes its fixtures.

**Organization**: Tasks are grouped by user story per spec.md priorities
(US1 P1 property tests, US2 P2 conformance corpus, US3 P2 watcher
regression, US4 P3 render-width coverage). Per plan.md's Technical
Context/Scope, these four stories touch non-overlapping code paths with no
shared blocking prerequisites — same pattern as `007-modern-tui-leverage`,
Phase 2 (Foundational) is skipped by design. The one project-wide item
(FR-012, CI configuration) has no user story of its own and lands in the
final Polish phase, since it doesn't gate or get gated by any story.

## Phase 1: Setup

- [X] T001 Re-read the current state of every file this feature touches immediately before editing (line numbers may have shifted since planning): `crates/fireside-core/src/model/mod.rs`, `crates/fireside-core/Cargo.toml`, `crates/fireside-engine/src/session.rs`, `crates/fireside-engine/src/validation.rs`, `crates/fireside-engine/Cargo.toml`, `crates/fireside-engine/tests/fixtures.rs`, `crates/fireside-tui/src/render/blocks.rs`, `crates/fireside-cli/src/main.rs`, `protocol/validate.mjs`, `protocol/fixtures.expected.json`, `protocol/main.tsp`, `.github/workflows/audit.yml`
- [X] T002 Confirm the baseline is green before any change: `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`

**Checkpoint**: Baseline confirmed clean. Each user story phase below can
start independently from here — pick any order, though P1→P2→P2→P3
(spec.md priority order) is recommended.

---

## Phase 2: User Story 1 - Property tests guard the wire format and engine invariants (Priority: P1) 🎯 MVP

**Goal**: A property-based test suite generates many randomized `Graph`
values and randomized navigation-operation sequences to verify two
invariants hand-written examples can't practically cover: serde round-trip
fidelity, and session history/visited-set truthfulness.

**Independent Test**: Per spec.md US1 — run `cargo test -p fireside-core --lib` and `cargo test -p fireside-engine --lib`, confirm the property suites pass; temporarily reintroduce a known bug (e.g. a session op that doesn't push to history), rerun, confirm a reproducible minimal counterexample is reported, then revert.

### Implementation for User Story 1

- [X] T003 [P] [US1] Add `proptest = "1"` to `[dev-dependencies]` in `crates/fireside-core/Cargo.toml`
- [X] T004 [P] [US1] Add `proptest = "1"` to `[dev-dependencies]` in `crates/fireside-engine/Cargo.toml`
- [X] T005 [US1] Verify MSRV per constitution Principle VI and plan.md's Technical Context gate: `cargo +1.88 check --workspace` after T003/T004 land; if it fails, this blocks the story — investigate a lower-pinned `proptest` version or flag to the user per the ADR-008 precedent (do not silently raise workspace MSRV) — verified clean: `proptest 1.11.0` compiles under the pinned 1.88 toolchain
- [X] T006 [US1] In `crates/fireside-core/src/model/mod.rs`, inside the existing `#[cfg(test)]` module, write hand-rolled `proptest::strategy::Strategy` functions per research.md §2: `arbitrary_content_block()` using `prop_recursive` to bound `ContentBlock::Container` nesting depth (cap generation depth low, e.g. 3–4, for fast shrinking — this is independent of the validator's depth-8 limit from US2), `arbitrary_node()`, and `arbitrary_graph()` — implemented in a new `proptest_support` module; deliberately not enforcing unique node ids/resolvable targets since the round-trip property is a pure serde property, documented inline
- [X] T007 [US1] In `crates/fireside-core/src/model/mod.rs`, add a `proptest! { #[test] fn graph_round_trips_through_json(graph in arbitrary_graph()) { ... } }` block: serialize `graph` via its existing `to_json_pretty`/`Serialize` path, deserialize back via `Graph::from_json`, assert equality with the original
- [X] T008 [US1] Manually verify T007 catches regressions (spec Acceptance Scenario 3): temporarily added `#[serde(skip)]` to `Node::title`, reran `cargo test -p fireside-core graph_round_trips_through_json`, confirmed it failed with a shrunk minimal counterexample (`title: Some("")` → `title: None`), then reverted and deleted the generated `proptest-regressions/` seed file (not committed)
- [X] T009 [P] [US1] In `crates/fireside-engine/src/session.rs`, inside a `#[cfg(test)]` module, define a test-only `enum SessionOp { Next, Choose(usize), Goto(String), Back }` (corrected from the plan's `Choose(String)`: `Session::choose` actually takes an option *index*, confirmed by reading `session.rs`) and a `proptest::strategy::Strategy` generating arbitrary sequences; also wrote `arbitrary_graph_and_ops()` scoped to this crate's tests (session-navigable: unique predictable ids `n0..n{n-1}`, targets drawn mostly-valid/occasionally-dangling) — written independently of `fireside-core`'s `arbitrary_graph()` per the crate-boundary note (no shared test-utility crate)
- [X] T010 [US1] In `crates/fireside-engine/src/session.rs`, added `proptest!` test `session_history_and_visited_stay_truthful` combining both invariants in one replay loop (splitting them into two separate proptest blocks would have duplicated the replay harness for no benefit). **Correction found during implementation**: `Session::history()` holds nodes *prior to* the current one (the stack `back()` pops), not including current — so the true invariant is `history() == path[..path.len()-1]` and `current().id == path.last()`, not "last history entry equals current" as tasks.md originally assumed; verified against the existing `next_follows_string_shorthand_and_object_form` test's own assertions before writing the property
- [X] T011 [US1] (merged into T010 — see above) `visited()` subset-of-node-ids check runs inside the same per-op loop
- [X] T012 [US1] Manually verify T010/T011 catch regressions (spec Acceptance Scenario 3): temporarily removed the `self.history.push(...)` line from `Session::move_to`, reran `cargo test -p fireside-engine session_history_and_visited_stay_truthful`, confirmed it failed with a minimal counterexample (`Goto("n0")` on a 1-node graph, expected history `["n0"]`, got `[]`), then reverted and deleted the generated `proptest-regressions/` seed file (not committed)
- [X] T013 [US1] Ran `cargo test -p fireside-core --lib` (16 tests) and `cargo test -p fireside-engine --lib` (36 tests) — all pass in well under 1s combined, within CI budget at proptest's default ~256 cases/property (not raised); confirmed via `cargo tree -p fireside-core -e normal` / `cargo tree -p fireside-engine -e normal` that `proptest` appears in neither crate's production dependency graph

**Checkpoint**: Property tests for serde round-trip and session invariants
are in place and proven to catch regressions; `cargo test -p fireside-core`
and `cargo test -p fireside-engine` green.

---

## Phase 3: User Story 2 - Expanded conformance corpus catches robustness regressions (Priority: P2)

**Goal**: A documented, ADR-recorded container-nesting-depth limit is
enforced identically by both validators and covered by boundary fixtures;
a ~1,000-node deck fixture proves load+validate stays fast, with both
additions consumed identically by the Rust and Node fixture tests.

**Independent Test**: Per spec.md US2 — run `cargo test -p fireside-engine --test fixtures` and `node protocol/run-fixtures.mjs` (or `npm run test:fixtures --prefix protocol`), confirm identical rule-ID results on every fixture including the three new ones, and confirm the 1,000-node fixture's load+validate time is asserted under budget.

### Implementation for User Story 2

- [X] T014 [US2] Write `.claude/adrs/adr-010-container-nesting-depth-limit.md` recording the depth-8 decision and rationale (per research.md §4 and contracts/container-nesting-depth-rule.md), following the existing ADR format used by ADR-007/ADR-009, before any validator code lands (constitution Principle I precedent)
- [X] T015 [US2] Updated `protocol/main.tsp`'s `ContainerBlock` doc comment and `docs/src/content/docs/spec/appendix-engine-guidelines.md`'s "Container Rendering Guidance" section to note the reference implementation's chosen example limit (8) and ADR-010 — no schema/`tsp-output/` regeneration needed since no field changes
- [X] T016 [P] [US2] In `crates/fireside-engine/src/validation.rs`, added `fn check_container_nesting_depth` + `fn container_depth` (data-model.md's formula: `0` for a non-container leaf, `1 + max(child depth)` for a `Container`), pushing an `Error`-severity `container-nesting-depth-exceeded` diagnostic naming the violating node when depth exceeds `MAX_CONTAINER_NESTING_DEPTH = 8`; wired into the `validate()` dispatch list right after `check_branch_options` (grouped with the other Error-severity structural checks)
- [X] T017 [P] [US2] Added the symmetric `containerDepth`/`checkContainerNestingDepth` (same formula, same error severity, same limit of 8) to `protocol/validate.mjs`, wired into `validate()`'s dispatch array and the `--help` rule listing
- [X] T018 [P] [US2] Added fixture `protocol/fixtures/valid/nesting-depth-at-limit.json`: one node with containers nested exactly 8 levels deep around a leaf text block
- [X] T019 [P] [US2] Added fixture `protocol/fixtures/invalid/nesting-depth-exceeds-limit.json`: the same shape, nested 9 levels deep
- [X] T020 [US2] Added entries to `protocol/fixtures.expected.json` for both new fixtures: `"valid/nesting-depth-at-limit.json": []` and `"invalid/nesting-depth-exceeds-limit.json": ["container-nesting-depth-exceeded"]`
- [X] T021 [P] [US2] Generated fixture `protocol/fixtures/valid/large-deck-1000-nodes.json`: 1,000 nodes in a plain linear chain (`node-0..node-999`, each a heading + text block, `traversal` a string shorthand to the next id, last node terminal) — deliberately diagnostic-free (no branches, cycles, or dead ends) so the expected rule-id set is `[]`; generated via a one-off script, committed as a static JSON file like every other fixture
- [X] T022 [US2] Added entry to `protocol/fixtures.expected.json`: `"valid/large-deck-1000-nodes.json": []`
- [X] T023 [US2] Added `large_deck_loads_and_validates_within_budget` to `crates/fireside-engine/tests/fixtures.rs` (reusing its existing `protocol_dir()` helper): loads the fixture, asserts `Graph::from_json` + `validate()` together complete in under 1 second and produce zero diagnostics — actual measured time ~0.01s combined with the rest of the fixture suite
- [X] T024 [US2] Ran `cargo test -p fireside-engine --test fixtures` (2 tests, both pass) and `node protocol/run-fixtures.mjs` (17/17 fixtures match) — identical rule-ID results confirmed; verified the parity check actually catches divergence: temporarily renamed the Node-side rule string to `container-nesting-depth-exceeded-TEMP`, reran `run-fixtures.mjs`, confirmed a clear mismatch report on `invalid/nesting-depth-exceeds-limit.json`, then reverted

**Checkpoint**: Nesting-depth limit is documented and enforced identically
by both validators with boundary fixtures; the 1,000-node fixture proves
and guards load+validate performance; fixture corpus now has 17 entries.

---

## Phase 4: User Story 3 - Watcher survives a half-saved edit without losing state (Priority: P2)

**Goal**: A regression test locks in the watcher's already-correct
behavior (per research.md §5) under a rapid, multi-step invalid-then-valid
write sequence, closing a coverage gap the existing single-malformed-write
tests don't reach.

**Independent Test**: Per spec.md US3 — run `cargo test -p fireside-cli watcher`, confirm the new test passes, driving `Watcher::poll()` through valid → truncated/malformed → still-malformed → valid and asserting no panic and correct recovery at each step.

### Implementation for User Story 3

- [X] T025 [US3] In `crates/fireside-cli/src/main.rs`'s existing test module (right before `write_back_reports_io_failure_without_panicking`), added `watcher_recovers_after_a_rapid_invalid_then_valid_sequence`: a single persistent `Watcher` over a temp file seeded with a valid deck, then in sequence (a) overwrite with truncated JSON and `poll()`, asserting `Some(Err(_))`, (b) overwrite with a *different* malformed payload (`"not json at all"`) and `poll()` again, asserting the same, (c) overwrite with a valid (differently-titled) deck and `poll()`, asserting `Some(Ok(graph))` with the new title — proving recovery doesn't depend on the invalid streak "settling" (spec Edge Cases)
- [X] T026 [US3] Ran `cargo test -p fireside-cli watcher_recovers_after_a_rapid_invalid_then_valid_sequence` — passed against the existing `Watcher::poll` implementation unmodified on the first run, confirming research.md §5's finding; no production code change needed. Full `cargo test -p fireside-cli` (37 tests) and `cargo clippy -p fireside-cli --all-targets -- -D warnings` also clean

**Checkpoint**: Watcher's half-saved-JSON resilience is proven by a
regression test covering a multi-step invalid sequence, not just a single
malformed write.

---

## Phase 5: User Story 4 - Rendering stays correct with wide and multi-codepoint text (Priority: P3)

**Goal**: New scenario-test coverage confirms `fireside-tui`'s existing
`unicode-width`-based measurement is correct for emoji- and CJK-bearing
headings and multi-column layouts.

**Independent Test**: Per spec.md US4 — run `cargo test -p fireside-tui --lib`, confirm the new emoji/CJK scenario tests pass alongside the full existing suite with zero changes to any pre-existing scenario's expected output.

### Implementation for User Story 4

- [X] T027 [P] [US4] In `crates/fireside-tui/src/render/blocks.rs`'s test module, added `heading_with_emoji_and_cjk_measures_by_display_width` (asserts the H1 underline rule length equals `UnicodeWidthStr::width(text)`, computed via the same crate the production code uses rather than a hand-picked number, for a CJK+emoji heading) and `heading_with_cjk_wraps_without_overflowing_narrow_width` (asserts no rendered line exceeds a narrow width for an all-CJK heading)
- [X] T028 [P] [US4] In the same suite, added `columns_with_wide_characters_stay_aligned`: a `columns` container with CJK left content vs. an ASCII string generated to have the identical measured width, asserting the right column's "MARK" marker lands at the same display *column* in both — caught and fixed a bug in the test itself during implementation (`str::find` returns a byte offset, not a display column; CJK characters are 3 bytes each in UTF-8, so byte offsets legitimately differ even when columns align correctly — fixed by measuring `UnicodeWidthStr::width` of the prefix instead of comparing byte offsets directly)
- [X] T029 [US4] Ran `cargo test -p fireside-tui --lib` (92 tests, all pass) and `cargo clippy -p fireside-tui --all-targets -- -D warnings` (silent) — new tests plus the full pre-existing scenario suite pass unchanged

**Checkpoint**: Emoji/CJK render-width correctness is covered by automated
scenario tests, closing the coverage gap identified in research.md §6.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: The one project-wide requirement with no user story of its
own (FR-012, CI configuration), plus the whole-suite gate and bookkeeping
every prior spec in this repo closes with.

- [X] T030 [P] Added a `pull_request` trigger to `.github/workflows/audit.yml` (whole workflow, since both `audit` and `deny` jobs share the file), scoped to the same `paths` filter already used by its `push` trigger — left `.github/workflows/rust.yml`'s `msrv` job unchanged (research.md §8: it already satisfies FR-012's MSRV ask)
- [X] T031 Ran the full quickstart.md whole-suite gate: `cargo test --workspace` (195 tests, all pass), `cargo clippy --workspace --all-targets -- -D warnings` (silent), `cargo fmt --check` (found and fixed formatting drift in the new proptest-strategy code via `cargo fmt`, then reconfirmed clean), `node protocol/validate.mjs docs/examples/hello.json` (unchanged: 0 errors, 0 warnings, 1 pre-existing info note), `npm run check --prefix docs` (0 errors, 0 warnings — pre-existing duplicate-id loader warnings unrelated to this change)
- [X] T032 Ran `graphify update .` — 224/224 files re-extracted, graph rebuilt (2424 nodes, 3459 edges, 219 communities)
- [X] T033 Updated the Progress Log checkbox for "P2 protocol & workflow hardening" in `.claude/plans/2026-07-12-strategic-improvement-plan.md` to `[X]` with a full summary (property tests + the history-invariant correction found during implementation, nesting-depth rule + ADR-010, 1000-node perf fixture, watcher regression test, emoji/CJK coverage + the test-bug-vs-production-bug distinction found while writing it, cargo-deny PR trigger) — this closes out the entire 2026-07-12 strategic plan

**Checkpoint**: All four stories done, CI gap closed, full workspace gate
green, knowledge graph current, strategic plan progress log updated. This
closes out the entire 2026-07-12 strategic plan.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **User Stories (Phases 2–5)**: All depend only on Setup. No Foundational
  phase — the four stories touch non-overlapping files/crates (`fireside-core`
  + `fireside-engine` for US1, `fireside-engine` + `protocol/` for US2,
  `fireside-cli` for US3, `fireside-tui` for US4) with no shared blocking
  prerequisite, confirmed in plan.md's Technical Context/Scope. They may
  proceed in any order or in parallel; P1→P2→P2→P3 (spec.md priority
  order) is recommended for a natural MVP checkpoint sequence.
- **Polish (Phase 6)**: T030 (CI) has no dependency on Phases 2–5 and could
  run any time after Setup. T031–T033 depend on all four story phases
  being complete (the whole-suite gate and progress-log update should
  reflect the finished feature).

### Within Each User Story

- US1: T003/T004 (deps) → T005 (MSRV gate) → T006 (core strategies) → T007
  (round-trip test) → T008 (verify it catches regressions) →
  T009 (engine op-sequence strategy, parallel-safe with T006–T008 since
  different crate) → T010/T011 (invariant tests) → T012 (verify regression
  catch) → T013 (final run).
- US2: T014 (ADR) → T015 (spec doc note) → T016/T017 (rule, parallel across
  Rust/Node) → T018/T019 (boundary fixtures, parallel) → T020 (expected.json
  entries, depends on T018/T019) → T021 (perf fixture) → T022 (expected.json
  entry, depends on T021) → T023 (perf test) → T024 (parity verification).
- US3: T025 → T026 (single linear pair).
- US4: T027/T028 (parallel, different test functions in the same file —
  safe as long as each is added as an independent `#[test]` fn) → T029.

### Parallel Opportunities

- T003/T004 (different `Cargo.toml` files).
- T016/T017 (different files: `validation.rs` vs `validate.mjs`).
- T018/T019 (different fixture files).
- T021 alongside T016–T020 (independent fixture, same directory but no
  file overlap).
- T027/T028 (same file, independent test functions — parallelizable by
  two people, not by two concurrent edits to the same function).
- Across stories: US1, US2, US3, US4 can all proceed in parallel once
  Setup is done, if staffed.

---

## Parallel Example: User Story 1

```bash
# Launch the two dev-dependency edits together:
Task: "Add proptest = \"1\" to crates/fireside-core/Cargo.toml [dev-dependencies]"
Task: "Add proptest = \"1\" to crates/fireside-engine/Cargo.toml [dev-dependencies]"
```

## Parallel Example: User Story 2

```bash
# Launch the two validator-rule implementations together:
Task: "Add check_container_nesting_depth to crates/fireside-engine/src/validation.rs"
Task: "Add container-nesting-depth-exceeded rule to protocol/validate.mjs"

# Then the two boundary fixtures together:
Task: "Add protocol/fixtures/valid/nesting-depth-at-limit.json"
Task: "Add protocol/fixtures/invalid/nesting-depth-exceeds-limit.json"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: User Story 1 (property tests).
3. **STOP and VALIDATE**: `cargo test -p fireside-core -p fireside-engine`
   green, regression-catch verified per T008/T012.
4. This alone closes the plan's single highest-value P2 sub-item (the
   deepest safety net) even if nothing else in this feature lands yet.

### Incremental Delivery

1. Setup → Foundation confirmed clean.
2. US1 (P1) → property tests → validate independently → this is the MVP.
3. US2 (P2) → conformance corpus hardening → validate independently.
4. US3 (P2) → watcher regression test → validate independently.
5. US4 (P3) → render-width coverage → validate independently.
6. Polish → CI gap closed, whole-suite gate, graphify, progress log.

### Parallel Team Strategy

With multiple contributors, once Setup is done: one person takes US1
(`fireside-core`/`fireside-engine`), another takes US2 (also
`fireside-engine` + `protocol/` — coordinate on `validation.rs` if working
alongside US1's session.rs changes in the same crate, though the two touch
different files within it), another takes US3 (`fireside-cli`), another
takes US4 (`fireside-tui`). All four integrate independently; Polish (CI +
final gate) is a single person's pass at the end.

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks.
- [Story] label maps task to specific user story for traceability.
- No wire-format/schema change anywhere in this feature (Constitution
  Principle I) — `tsp-output/` regeneration is NOT required.
- `proptest` must never appear outside a `[dev-dependencies]` block
  (Constitution Principle III) — verify this with `cargo tree -p fireside-core -e normal`
  / `cargo tree -p fireside-engine -e normal` showing no `proptest` entry,
  as a sanity check during T013.
- Commit after each task or logical group, per this repo's established
  practice of landing each spec's work as a coherent, tested unit.
- Stop at any checkpoint to validate a story independently before moving on.
