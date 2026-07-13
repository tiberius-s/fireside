# Tasks: Protocol spec patch 0.1.1

**Input**: Design documents from `/specs/004-spec-patch-0-1-1/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included — this feature's Priority 2 story IS the test corpus, and
the constitution's Test Discipline principle requires engine-semantics
coverage in `validation.rs`.

**Organization**: Tasks are grouped by user story (US1 spec docs, US2
empty-traversal rule, US3 fixture corpus) so each is independently
completable and verifiable, per spec.md's priorities.

## Phase 1: Setup (protocol version bump)

- [X] T001 Add `v0_1_1: "0.1.1"` to the `Versions` enum in `protocol/main.tsp`; update the "## Protocol Version" doc-comment banner near the top of the file to note 0.1.1 alongside 0.1.0
- [X] T002 Run `npm run build` in `protocol/` to regenerate `protocol/tsp-output/`; confirm the generated `Graph.json` schema's `fireside-version` enum now allows both `"0.1.0"` and `"0.1.1"`
- [X] T003 Run `node protocol/validate.mjs docs/examples/hello.json` and `cargo run -p fireside-cli -- validate docs/examples/hello.json` to confirm the canonical example still validates identically (baseline before further changes, re-verified in Polish)

**Checkpoint**: Protocol version enum is additive and regenerated; nothing else depends on this beyond being available.

---

## Phase 2: User Story 1 - Third-party implementer reads an unambiguous spec (Priority: P1)

**Goal**: Every one of the seven audit ambiguities has a plain-language, accurate answer in `protocol/main.tsp` and `docs/src/content/docs/spec/`.

**Independent Test**: Read the spec pages listed below with no access to Rust/Node source; confirm each of US1's seven acceptance scenarios in spec.md is satisfied.

- [X] T004 [P] [US1] In `docs/src/content/docs/spec/validation.md` §4, move "Branch option `key` values that collide within one branch point" out of "Recommended Checks" and into "Required Checks" as a new numbered item, stating it is an Error-severity check (matches existing `unique-branch-keys` rule implementation)
- [X] T005 [P] [US1] In `docs/src/content/docs/spec/validation.md` §4 "Recommended Checks", add a new bullet documenting the `empty-traversal` warning: a `Traversal` object present but setting neither `next` nor `branch-point` is treated as terminal (same as an absent field) but is flagged since it is likely an authoring mistake
- [X] T006 [P] [US1] In `docs/src/content/docs/spec/traversal.md` "Operation: Choose" section, add a sentence requiring implementations to validate that the selected option belongs to the CURRENT node's branch point (not an arbitrary/forged `BranchOption`), citing an index-into-current-options approach as the recommended pattern
- [X] T007 [P] [US1] In `protocol/main.tsp`'s `TraversalOps` interface, update the `choose(option: BranchOption): void` doc comment to note the option MUST be validated as belonging to the current node's branch point
- [X] T008 [P] [US1] In `docs/src/content/docs/spec/appendix-engine-guidelines.md` (Appendix B), add an item documenting ViewMode toggle persistence: the presenter's runtime toggle PERSISTS across node transitions until explicitly toggled again (matches `fireside-tui`'s `view_override` reference behavior)
- [X] T009 [P] [US1] In `docs/src/content/docs/spec/appendix-engine-guidelines.md` (Appendix B), add an item documenting the image width/height overflow rule: engines MUST clamp requested width/height to the available content area; explicitly note this is forward guidance for real image rendering (still deferred) and that the current placeholder-box renderer does not yet interpret these fields at all
- [X] T010 [P] [US1] In `docs/src/content/docs/spec/appendix-engine-guidelines.md` (Appendix B), add an item documenting history growth: engines MAY cap history length for long-running presentations; the reference implementation does not currently cap it
- [X] T011 [P] [US1] In `protocol/main.tsp`'s `ListBlock` model doc comment, state that `items` entries MAY contain inline Markdown, same as `TextBlock.body`
- [X] T012 [P] [US1] In `docs/src/content/docs/spec/appendix-content-blocks.md` (Appendix C), add the same inline-Markdown note for `ListBlock.items` to the Core Blocks table or Rendering Notes section
- [X] T013 [US1] Run `npm run check --prefix docs` to confirm the docs site still type-checks/builds after all prose edits

**Checkpoint**: All seven ambiguities from spec.md's US1 acceptance scenarios have a documented answer; docs site builds clean.

---

## Phase 3: User Story 2 - Presenter gets a diagnostic for empty traversal (Priority: P1)

**Goal**: A node with `"traversal": {}` produces a warning-severity diagnostic from both validators, without changing terminal-node behavior.

**Independent Test**: Validate a document with `"traversal": {}` via both `fireside validate` and `node protocol/validate.mjs`; confirm both warn, and neither reports it for an absent `traversal` field. Per contracts/empty-traversal-rule.md.

- [X] T014 [US2] Add `check_empty_traversal(graph, diags)` function to `crates/fireside-engine/src/validation.rs`, following the existing `check_self_loops`/`check_reachability` style: iterate nodes, skip absent/string traversal, warn (rule `empty-traversal`) when the object form has both `next` and `branch_point` as `None`; wire the call into `validate()`'s check chain
- [X] T015 [US2] Add unit tests for `check_empty_traversal` in `crates/fireside-engine/src/validation.rs`'s existing `#[cfg(test)] mod tests`: (a) `"traversal": {}` warns and names the node, (b) absent `traversal` does not warn, (c) `{"next": "x"}` and a valid branch-point do not warn
- [X] T016 [US2] Add `checkEmptyTraversal(graph)` function to `protocol/validate.mjs`, following the existing `checkSelfLoops`/`checkReachability` style, same rule id `empty-traversal`, matching message tone; wire into the module's `validate()` function; update the `HELP` text's rule list to include `empty-traversal` under warnings
- [X] T017 [US2] Manually verify parity per quickstart.md Scenario 2 and 3: run both validators against a `{}` fixture and an absent-traversal fixture, confirm matching behavior

**Checkpoint**: `empty-traversal` rule exists symmetrically in both validators with equivalent behavior; `cargo test --workspace` passes with new tests.

---

## Phase 4: User Story 3 - Fixture corpus proves Rust/Node parity (Priority: P2)

**Goal**: A shared fixture corpus, consumed by both validator test suites, proves identical rule-id output — not just matching rule-name strings.

**Independent Test**: Run the corpus against both validators independently; confirm each fixture's fired rule-id set matches its documented expectation in both, and the `valid/`/`invalid/` split matches each validator's Error aggregate. Per contracts/fixture-corpus.md.

**Depends on**: Phase 3 (US2) — the `empty-traversal.json` fixture requires the rule to exist first. Other fixtures in this phase have no such dependency and could be built in parallel with Phase 3 if desired.

- [X] T018 [P] [US3] Create `protocol/fixtures/valid/clean.json` — a minimal multi-node graph with zero diagnostics of any kind
- [X] T019 [P] [US3] Create `protocol/fixtures/valid/unreachable-node.json` — isolates the `unreachable-node` warning
- [X] T020 [P] [US3] Create `protocol/fixtures/valid/self-loop.json` — isolates the `self-loop` warning
- [X] T021 [P] [US3] Create `protocol/fixtures/valid/trivial-cycle.json` — isolates the `trivial-cycle` warning
- [X] T022 [P] [US3] Create `protocol/fixtures/valid/dead-end-branch.json` — isolates the `dead-end-branch` info diagnostic
- [X] T023 [US3] Create `protocol/fixtures/valid/empty-traversal.json` — isolates the new `empty-traversal` warning (requires T014/T016 to exist so the expected behavior is real, not aspirational)
- [X] T024 [P] [US3] Create `protocol/fixtures/invalid/duplicate-node-ids.json` — isolates the `unique-node-ids` error
- [X] T025 [P] [US3] Create `protocol/fixtures/invalid/dangling-target.json` — isolates the `valid-traversal-target` error
- [X] T026 [P] [US3] Create `protocol/fixtures/invalid/next-branch-point-conflict.json` — isolates the `next-branch-point-conflict` error
- [X] T027 [P] [US3] Create `protocol/fixtures/invalid/duplicate-branch-keys.json` — isolates the `unique-branch-keys` error (also serves as the doc-fix's proof fixture for T004)
- [X] T028 [US3] Create `protocol/fixtures.expected.json` mapping every fixture path from T018-T027 to its sorted expected rule-id array, per data-model.md's shape
- [X] T029 [US3] Add a fixture-corpus test to `crates/fireside-engine` (either a new `#[test]` in `validation.rs` or a new `tests/fixtures.rs` integration test — whichever reads more naturally alongside existing test placement) that reads `protocol/fixtures.expected.json`, walks `protocol/fixtures/{valid,invalid}/*.json`, runs `validate()` on each via `Graph::from_json`, and asserts the fired rule-id set matches; also asserts `valid/` fixtures have no Errors and `invalid/` fixtures have at least one
- [X] T030 [US3] Create `protocol/run-fixtures.mjs`: a Node script that imports `validate.mjs`'s `validate` function directly, performs the same fixture-walk/comparison as T029, and exits non-zero on any mismatch; wire it as an npm script in `protocol/package.json` (e.g. `"test:fixtures": "node run-fixtures.mjs"`)
- [X] T031 [US3] Run both corpus runners (`cargo test`, `node protocol/run-fixtures.mjs`) and confirm all fixtures pass in both; then temporarily rename one rule string in only one validator to confirm the corpus actually fails on divergence (per quickstart.md Scenario 4), then revert the temporary change

**Checkpoint**: Fixture corpus passes identically in both languages; a deliberate mismatch is provably caught.

---

## Phase 5: Polish & Cross-Cutting

- [X] T032 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets`; both must be clean
- [X] T033 Run the full quickstart.md verification block (protocol build, hello.json validate via both validators, both fixture corpus runners, docs check)
- [X] T034 Update `.claude/plans/2026-07-12-strategic-improvement-plan.md`'s Progress Log: mark "Week 1 spec patch 0.1.1 (7 ambiguities) + validator rules" and "Week 1 shared fixture corpus" as done, with a technical summary matching the style of the existing Stage A-D entries
- [X] T035 Update the `project_strategic_plan_2026_07` memory file and `MEMORY.md` index to reflect Week 1's spec-patch and fixture-corpus items being complete

---

## Dependencies & Execution Order

- **Phase 1 (Setup)** has no dependencies; run first.
- **Phase 2 (US1)** depends only on Phase 1 (needs T001's version banner text to reference "0.1.1" correctly in prose, though this is a soft ordering — T004-T012 could technically be written in parallel with Phase 1). All T004-T012 tasks touch different files/sections and are marked [P].
- **Phase 3 (US2)** depends only on Phase 1. Independent of Phase 2.
- **Phase 4 (US3)** depends on Phase 3 for T023 specifically (the `empty-traversal` fixture needs the rule to exist); T018-T022 and T024-T027 have no such dependency and are marked [P] against each other.
- **Phase 5 (Polish)** depends on all prior phases being complete.

## Parallel Execution Examples

Within Phase 2 (US1), T004-T012 touch entirely different files/sections and can run together:

```text
T004, T005 → docs/.../validation.md (same file, sequential within it)
T006 → docs/.../traversal.md
T007 → protocol/main.tsp (choose doc comment)
T008, T009, T010 → docs/.../appendix-engine-guidelines.md (same file, sequential within it)
T011 → protocol/main.tsp (ListBlock doc comment)
T012 → docs/.../appendix-content-blocks.md
```

Within Phase 4 (US3), T018-T022 and T024-T027 (9 of 10 fixtures) can be authored in parallel; only T023 waits on Phase 3.

## Implementation Strategy

**MVP scope**: Phase 1 + Phase 2 (US1) alone already delivers the highest-priority value — an unambiguous spec — with zero code risk, since every US1 task is prose-only. Phase 3 (US2) is the one behavior change and should land next given equal P1 priority with US1. Phase 4 (US3) is P2 and can follow once US2 exists. Phase 5 is mandatory before calling the feature done, per this project's "whole stage at a time" completion convention.
