# Tasks: Live Validation While Authoring (`validate --watch`)

**Input**: Design documents from `/specs/001-validate-watch/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli-validate-watch.md, quickstart.md

**Tests**: included — the constitution's Test Discipline principle (VII) requires
tests at the correct layer for every feature; this feature's layer is unit
tests over the new pure functions plus one CLI wiring test.

**Organization**: tasks are grouped by user story (spec.md priorities P1/P2/P3).
Nearly all tasks touch the same file (`crates/fireside-cli/src/main.rs`), so
`[P]` is reserved for tasks in a genuinely separate file with no dependency
on unfinished work.

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

- [X] T001 Run `cargo test --workspace` to confirm a clean baseline before
      changing `crates/fireside-cli/src/main.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: the `--watch` flag and a shared, non-exiting diagnostic
renderer that both the existing one-shot path and the new watch path will
call — this is what keeps non-watch `validate` output unchanged (FR-002,
SC-004), per the Research decision to extract rather than duplicate.

**⚠️ CRITICAL**: no user-story task can begin until this phase is complete.

- [X] T002 Add a `watch: bool` field (with a `#[arg(long)]` clap attribute and
      a doc comment) to the `Command::Validate` variant in
      `crates/fireside-cli/src/main.rs`
- [X] T003 Extract `fn diagnostics_report(path: &Path, diags: &[fireside_engine::Diagnostic]) -> String`
      in `crates/fireside-cli/src/main.rs` from `validate_file`'s existing
      printing logic (the success line, or the per-diagnostic icon lines
      plus the summary line) — the returned string must render byte-for-byte
      what `validate_file` prints today
- [X] T004 Refactor `validate_file` in `crates/fireside-cli/src/main.rs` to
      build its output via `diagnostics_report` and print it, preserving the
      existing exit-code behavior (exit 1 when any diagnostic is an error)
- [X] T005 Update `validate_file`'s signature to `fn validate_file(path: &Path, watch: bool) -> Result<()>`
      and update its call site in `main()`'s match arm in
      `crates/fireside-cli/src/main.rs` to pass the new flag through

**Checkpoint**: `cargo test --workspace` still green, existing
`validate_hello_exits_zero`, `validate_missing_file_fails_with_readable_error`,
and `validate_reports_dangling_targets_in_plain_language` tests in
`cli_e2e.rs` unchanged and passing — non-watch behavior is provably
unaffected before any watch-specific code exists.

---

## Phase 3: User Story 1 - See errors immediately after saving (Priority: P1) 🎯 MVP

**Goal**: `fireside validate --watch deck.json` checks immediately, then
re-checks and prints a fresh result every time the file changes, without the
presenter re-running the command.

**Independent Test**: run `validate --watch` against a valid deck, edit it to
introduce a semantic error and save, confirm the diagnostics appear
unprompted; fix it and save again, confirm the success line reappears.

### Implementation for User Story 1

- [X] T006 [US1] Implement `fn watch_report(path: &Path) -> String` in
      `crates/fireside-cli/src/main.rs`: read the file; on a read error,
      return a one-line "could not read `<path>`: `<err>`" message; on a
      successful read, parse with `Graph::from_json` and on success call
      `validate()` then `diagnostics_report` (parse-failure handling is
      added in Phase 4, User Story 2 — for now a parse error may fall
      through to a generic message)
- [X] T007 [US1] Implement `fn watch_loop(path: &Path) -> Result<()>` in
      `crates/fireside-cli/src/main.rs`: print `watch_report(path)`
      immediately, then loop forever on a 250ms `std::thread::sleep`,
      tracking the last `fingerprint()` value and calling + printing
      `watch_report(path)` again whenever it changes
- [X] T008 [US1] Wire `validate_file`'s `watch` branch in
      `crates/fireside-cli/src/main.rs`: when `true`, call `watch_loop`
      instead of the existing one-shot `load`/`validate`/`diagnostics_report`
      sequence
- [X] T009 [US1] Add unit tests in `crates/fireside-cli/src/main.rs`'s
      `#[cfg(test)]` module: `watch_report` returns the success line for a
      valid deck fixture, and returns the diagnostics text for a deck with a
      dangling traversal target (mirror the existing dangling-target fixture
      used in `cli_e2e.rs`)

**Checkpoint**: User Story 1 is fully functional and independently testable
via `quickstart.md` steps 1–4.

---

## Phase 4: User Story 2 - Get a precise location for JSON syntax errors (Priority: P2)

**Goal**: a JSON syntax mistake shows the exact line, column, and a caret —
not a generic message.

**Independent Test**: save a file with malformed JSON while `validate --watch`
is running; confirm the output matches the caret-block format non-watch
`validate` already produces for the same file.

### Implementation for User Story 2

- [X] T010 [US2] Extend `watch_report` in `crates/fireside-cli/src/main.rs`
      so a `CoreError::Parse(err)` result calls `parse_report(path, &text, &err)`
      and returns its output, replacing the generic fallback from T006
- [X] T011 [US2] Add a unit test in `crates/fireside-cli/src/main.rs`
      asserting `watch_report` on malformed JSON returns a caret-pointed
      report with the correct line/column, matching the same format already
      locked in by the `parse_report_points_at_the_line_with_a_caret` test

**Checkpoint**: User Stories 1 and 2 both independently pass; `quickstart.md`
step 5 confirmed.

---

## Phase 5: User Story 3 - Keep working through transient save states (Priority: P3)

**Goal**: the watch loop never crashes or gets stuck on a mid-save or
missing file; it keeps polling and recovers automatically.

**Independent Test**: delete the watched file while `validate --watch` is
running, confirm a "missing" message appears and the loop keeps running;
recreate the file, confirm it picks the change back up.

### Implementation for User Story 3

- [X] T012 [US3] Harden `watch_loop` in `crates/fireside-cli/src/main.rs` so
      a `fingerprint()` result of `None` (file absent or transiently
      unreadable) is still surfaced via `watch_report` — which already
      returns a one-line message for that case per T006 — rather than being
      skipped or causing a panic, and so the loop never exits on its own
- [X] T013 [US3] Add a unit test in `crates/fireside-cli/src/main.rs`
      simulating a file deleted then recreated between two `watch_report`
      calls (using a `tempfile` fixture), asserting the missing-file message
      appears for the deleted state and the success message reappears after
      recreation, with no panic

**Checkpoint**: all three user stories independently pass; `quickstart.md`
step 6 confirmed.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T014 [P] Add one integration test to
      `crates/fireside-cli/tests/cli_e2e.rs` that spawns
      `fireside validate --watch <file>` and asserts the first success line
      appears within a short timeout, then terminates the process — covers
      the CLI flag wiring (FR-001, FR-003) without asserting on reload
      timing, per the Research decision to avoid timing-sensitive e2e
      assertions
- [X] T015 Run `cargo test --workspace` and
      `cargo clippy --workspace --all-targets` and fix any findings
- [X] T016 Manually walk through `quickstart.md` steps 1–7 in a real
      terminal, confirming the immediate first check, live reload, caret
      errors, missing-file handling, and clean Ctrl-C exit (FR-010)
- [X] T017 [P] Run `graphify update .` to refresh the knowledge graph after
      the code change, per the constitution's Operational Constraints

---

## Dependencies & Execution Order

- **Setup (Phase 1)**: no dependencies.
- **Foundational (Phase 2)**: depends on Setup; blocks every user story.
- **User Story 1 (Phase 3)**: depends on Foundational only. This is the MVP.
- **User Story 2 (Phase 4)**: depends on Foundational and on `watch_report`
  existing (T006, US1) — extends the same function, so it follows US1 in
  practice even though it is conceptually independent.
- **User Story 3 (Phase 5)**: depends on Foundational and on `watch_loop`
  existing (T007, US1) — hardens the same loop, so it follows US1 in
  practice.
- **Polish (Phase 6)**: depends on all desired user stories being complete.

### Parallel Opportunities

- T014 and T017 touch different files than everything else and than each
  other — safe to run in parallel with each other, once Phase 5 is done.
- All other tasks touch `crates/fireside-cli/src/main.rs` and are
  effectively sequential within their phase.

---

## Implementation Strategy

### MVP First (User Story 1 only)

1. Phase 1 (Setup) → Phase 2 (Foundational) → Phase 3 (User Story 1).
2. **Stop and validate**: run `quickstart.md` steps 1–4. This alone ships
   the core authoring loop the strategic plan asked for.

### Incremental Delivery

1. Setup + Foundational → non-watch `validate` provably unchanged.
2. + User Story 1 → live reload with success/diagnostics feedback (MVP).
3. + User Story 2 → precise caret locations for syntax errors.
4. + User Story 3 → robust against deletes/mid-save races.
5. + Polish → CLI wiring test, lint/test pass, manual quickstart walk,
   knowledge graph refresh.
