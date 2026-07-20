# Tasks: Dual-Screen Presenter View

**Input**: Design documents from `/specs/012-presenter-view/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/session-state-format.md, quickstart.md

**Tests**: Included and REQUIRED, not optional — constitution Principle VII: "A feature is not done until its tests exist at the correct layer." Every implementation task below has a matching test task at the layer plan.md's Constitution Check names.

**Organization**: Tasks are grouped by user story (spec.md priorities P1/P2/P3) so each can be implemented, tested, and demoed independently, per the Suggested Order in `.claude/plans/2026-07-19-fable-ux-audit.md`.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: Which user story this task belongs to (US1/US2/US3)

## Path Conventions

Existing 4-crate Cargo workspace at the repository root — no new crate. Paths below are relative to the repo root.

---

## Phase 1: Setup (governance)

**Purpose**: Record the two constitution-mandated decisions before any code lands (plan.md Constitution Check).

- [X] T001 [P] Write ADR-014 recording the ADR-004 scope extension (the `notes` verb and `--fullscreen` flag) and the 2026-07-19 user request that satisfies Principle II's scope-addition gate, in `.claude/adrs/adr-014-dual-screen-presenter-view-scope.md` (mirror the format of `.claude/adrs/adr-004-presenter-first-rewrite.md`)
- [X] T002 [P] Write ADR-015 recording the session-state file contract decision (separate per-deck file vs. a `resume.json` extension, location, schema, atomicity, staleness threshold, FNV-1a 64 choice) per research.md §1 and contracts/session-state-format.md, in `.claude/adrs/adr-015-session-state-file-contract.md`

**Checkpoint**: Both ADRs merged — implementation may begin.

---

## Phase 2: Foundational (blocking prerequisites)

**Purpose**: The session-state file module and the cross-crate types every user story depends on. No user story can be implemented before this phase completes.

- [X] T003 [P] Implement FNV-1a 64-bit hashing and the session file's path resolution (`$XDG_STATE_HOME/fireside/sessions/<hex>.json`, falling back to `~/.local/state`, mirroring `resume_path()`) in new file `crates/fireside-cli/src/session.rs`; add `mod session;` to `crates/fireside-cli/src/main.rs`
- [X] T004 Implement `SessionRecord` read/write in `crates/fireside-cli/src/session.rs`: atomic write (temp file + rename in the same directory), a `write(path, node_id, reveal_step, reveal_total, elapsed_secs)` function called every tick, a `read(path) -> SessionStatus`-shaped read that treats missing file / unparseable JSON / wrong `schema` / heartbeat >2s stale identically as "not running" per contracts/session-state-format.md, and a `delete(path)` function for clean-exit cleanup (depends on T003)
- [X] T005 [P] Unit tests for `crates/fireside-cli/src/session.rs` covering: write-then-read round-trips node id/reveal/elapsed correctly; a missing file reads as not-running; a corrupt/truncated file reads as not-running without panicking; two sequential writes to the same path never leave a partially-written file readable (mirror `resume.rs`'s test style and use `tempfile::tempdir()`) (depends on T004)
- [X] T006 [P] Add `SessionSnapshot`, `SessionStatus`, `SessionSource` (data-model.md), and `SessionTick`/`SessionTickSink` (data-model.md) types to `crates/fireside-tui/src/lib.rs`, each with a `///` doc comment describing the caller-owns-I/O contract (Principle IV)
- [X] T007 Add the `tick_sink: SessionTickSink<'_>` parameter to `present_authoring`/`present_impl`/`event_loop` in `crates/fireside-tui/src/lib.rs`, called once per event-loop iteration (unconditionally, not only on position change) with the current node id, `Session::reveal_progress()` (or `(0, 0)` when `None`), and `App::elapsed()` (depends on T006)
- [X] T008 Update the single call site in `crates/fireside-cli/src/main.rs::present()` to pass a tick-sink closure that calls `session::write(...)` with the resume `key`'s path (depends on T004, T007)
- [X] T009 [P] Generalize `exit_on_not_a_tty` in `crates/fireside-cli/src/main.rs` from `Result<PresentSummary, TuiError> -> Result<PresentSummary>` to a generic `fn exit_on_not_a_tty<T>(result: Result<T, TuiError>) -> Result<T>`, updating its two existing call sites (`present`, `demo`)

**Checkpoint**: The presenter now writes a live session-state file on every tick and deletes it on request; `fireside present`/`demo`/shorthand behavior is otherwise unchanged (verify `cargo test --workspace` still passes before moving on).

---

## Phase 3: User Story 1 - Follow my own notes while the deck is on the projector (Priority: P1) 🎯 MVP

**Goal**: `fireside notes <deck>` shows the current slide's title, notes, next title/branch options, reveal progress, and elapsed time, tracking a running presenter within ~500ms.

**Independent Test**: Per quickstart.md steps 1–5 — start presenter + follower on the same deck, navigate/reveal/branch/reach-the-end in the presenter, watch the follower update.

### Tests for User Story 1

- [X] T010 [P] [US1] TestBackend scenario in `crates/fireside-tui/src/render/tests.rs`: given a `SessionStatus::Running` snapshot pointing at a node with `speaker_notes`, the follower screen renders the node's title and notes text
- [X] T011 [P] [US1] TestBackend scenario in `crates/fireside-tui/src/render/tests.rs`: a node with no `speaker_notes` renders the plain "No notes for this slide" line, not a blank panel (FR-012)
- [X] T012 [P] [US1] TestBackend scenario in `crates/fireside-tui/src/render/tests.rs`: a node with a `next` edge renders the next node's title; a terminal node with no branch renders "This is the last slide" instead of an empty field (FR-013)
- [X] T013 [P] [US1] TestBackend scenario in `crates/fireside-tui/src/render/tests.rs`: a node with a `branch-point` renders its options' labels and keys instead of a single next-title line
- [X] T014 [P] [US1] TestBackend scenario in `crates/fireside-tui/src/render/tests.rs`: `reveal_total > 0` renders `"{step}/{total} revealed"`; `reveal_total == 0` omits the reveal line entirely

### Implementation for User Story 1

- [X] T015 [US1] Create `Follower` state, `FollowerMsg` (`Terminal`, `Reload`, `SessionUpdate`), and its `update()` (the sole mutation point, Principle IV) in new file `crates/fireside-tui/src/follower.rs`, deriving the "current/next/branch/reveal/waiting" view per data-model.md's Follower section — resolve `graph.node(&snapshot.node_id)` and treat a lookup miss as the "waiting for presenter…" case (FR-007), never `unwrap()`/`expect()`
- [X] T016 [US1] Create the follower's rendering in new file `crates/fireside-tui/src/render/notes.rs`: current title + notes (or the no-notes line), next title or branch options, reveal progress line when present, elapsed timer (reuse `format_present_summary`'s `mm:ss` shape), the "waiting for presenter…" state, and the "Presenter not running — start \"fireside <deck>\" in another window" state (FR-004 exact wording) — all styling through `theme::Tokens` (Principle IV), a footer teaching `q` quit (Principle II) (depends on T015)
- [X] T017 [US1] Wire the follower's draw function into `crates/fireside-tui/src/render/mod.rs`'s dispatch (depends on T016)
- [X] T018 [US1] Add `pub fn follow(graph: Graph, deck_source: ReloadSource<'_>, session_source: SessionSource<'_>) -> Result<(), TuiError>` to `crates/fireside-tui/src/lib.rs`: the non-tty guard (reusing `TuiError::NotATty`, per research.md §5), terminal init/restore (mirroring `present_impl`), and its own small event loop polling both sources at the existing 250ms cadence and dispatching `FollowerMsg`s into `Follower::update` (depends on T015, T017)
- [X] T019 [US1] Add the `notes <deck>` subcommand to the `Command` enum and its handler in `crates/fireside-cli/src/main.rs`: `load()` the deck (reusing the existing friendly-error path), construct a `watch::Watcher` for the deck-reload source, construct a `session::read`-backed closure for the session source keyed by the same canonicalized path `resume::resume_key` would use, call `fireside_tui::follow(...)`, and route its `Result` through the now-generic `exit_on_not_a_tty` (depends on T004, T009, T018)
- [X] T020 [P] [US1] Add an `App::with_fullscreen()` builder (mirroring `without_sink()`) to `crates/fireside-tui/src/app.rs` that sets `view_override = Some(ViewMode::Fullscreen)`, and thread a `fullscreen: bool` parameter through `present_authoring`/`present_impl` in `crates/fireside-tui/src/lib.rs`
- [X] T021 [US1] Add a `--fullscreen` flag to the `present` subcommand and the shorthand `Cli.file` form in `crates/fireside-cli/src/main.rs`, passed through to `present_authoring` (depends on T020)
- [X] T022 [P] [US1] `cli_e2e.rs` test in `crates/fireside-cli/tests/cli_e2e.rs`: `fireside notes <missing-file>` gives the same friendly missing-deck message `present` already gives, not an anyhow chain

**Checkpoint**: `fireside notes <deck>` tracks a running presenter live — the MVP is demoable per quickstart.md steps 1–5. Run `cargo test --workspace` before continuing.

---

## Phase 4: User Story 2 - Know immediately if I've lost the connection to my presentation (Priority: P2)

**Goal**: The follower shows a plain "not running" state — never a stale frozen frame — within ~2s of the presenter never having started, exiting cleanly, or crashing.

**Independent Test**: Per quickstart.md steps 6–8 — kill the presenter mid-talk, restart it, quit it cleanly; confirm the follower's state at each step.

### Tests for User Story 2

- [X] T023 [P] [US2] Unit test in `crates/fireside-cli/src/session.rs`: a heartbeat older than 2 seconds reads as not-running even though the file parses successfully (staleness, not just corruption, per contracts/session-state-format.md)
- [X] T024 [P] [US2] TestBackend scenario in `crates/fireside-tui/src/render/tests.rs`: `SessionStatus::NotRunning` from the very first poll (no presenter has ever started) renders the plain not-running message, not an error (spec edge case)
- [X] T025 [P] [US2] TestBackend scenario in `crates/fireside-tui/src/render/tests.rs`: a follower that was tracking `SessionStatus::Running` and then receives `SessionStatus::NotRunning` on a later poll switches its rendered state completely — no leftover slide content bleeding through
- [X] T026 [P] [US2] `cli_e2e.rs` test in `crates/fireside-cli/tests/cli_e2e.rs`: `fireside notes <deck>` with piped stdio gives the same one-line non-tty message the presenter gives (mirrors `present_without_a_tty_gives_a_plain_message`), not a raw panic

### Implementation for User Story 2

- [X] T027 [US2] In `crates/fireside-cli/src/main.rs::present()`, call `session::delete(...)` for the deck's session-file path after `present_authoring` returns `Ok` (clean quit) — mirrors the existing `store.clear(key)` resume cleanup on a terminal node, but unconditional on any clean exit (depends on T004, T008)
- [X] T028 [US2] Extend `scripts/smoke.sh` with a two-tmux-pane scenario (presenter in one pane, `fireside notes` in the other, per W4-DS-5): assert the follower's pane shows tracking output after presenter navigation, assert `capture-pane` shows the not-running message within ~2s of `tmux kill-session`/`kill -9` on the presenter pane, and assert the same after a clean `q` quit — wire into `rust.yml`'s existing `Smoke (tmux)` job and `scripts/verify.sh` alongside CH-2's existing coverage (depends on T019, T027)

**Checkpoint**: The follower never shows stale-but-unmarked information. Run `cargo test --workspace` and `scripts/smoke.sh` before continuing.

---

## Phase 5: User Story 3 - Trust the notes even if I edit the deck mid-talk (Priority: P3)

**Goal**: The follower picks up a live deck edit without restarting, and never crashes on a reload-timing mismatch with the presenter.

**Independent Test**: Per quickstart.md step 9 — quick-edit a slide's notes in the presenter, save, confirm the follower picks it up live.

### Tests for User Story 3

- [X] T029 [P] [US3] TestBackend scenario in `crates/fireside-tui/src/render/tests.rs`: a `FollowerMsg::Reload(Ok(graph))` with different `speaker_notes` text for the current node updates the rendered notes without any other observable state change
- [X] T030 [P] [US3] TestBackend scenario in `crates/fireside-tui/src/render/tests.rs`: a `SessionStatus::Running` snapshot naming a `node_id` absent from the follower's currently-loaded graph (reload skew) renders "waiting for presenter…", not a panic or error (confirms T015's `graph.node(...)` miss-handling under the scenario it exists for)

### Implementation for User Story 3

- [X] T031 [US3] In the `notes` subcommand handler in `crates/fireside-cli/src/main.rs`, construct a second `watch::Watcher` for the deck file (independent of the session-file poll) and pass its `poll()` as `follow()`'s `deck_source`, so a quick-edit save on the presenter side is picked up by the follower the same way `present()` already picks up external edits (depends on T019)

**Checkpoint**: All three user stories are independently functional and demoable per quickstart.md end-to-end.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T032 [P] Add a "Presenting with two screens" section to `docs/src/content/docs/guides/presenting.md` (drag the deck terminal to the projector, OS-fullscreen or `--fullscreen`, `fireside notes` on the laptop), and update the existing speaker-notes-panel (`s` key) caveat to say it's the single-screen/rehearsal path while `fireside notes` is the on-stage path (W4-DS-6)
- [X] T033 [P] Add a `fireside notes <deck>` entry (flags, exit codes) to `docs/src/content/docs/reference/cli.md`, matching the existing per-verb table format
- [X] T034 [P] Add a one-line pointer to the two-screen workflow in `docs/src/content/docs/guides/quickstart.md`
- [X] T035 Run `scripts/verify.sh` (full CI-mirroring suite) and `graphify update .`
- [X] T036 Tick the W4-DS-1..6 Progress Log line in `.claude/plans/2026-07-19-fable-ux-audit.md` with status and date

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — the two ADRs can be written immediately and in parallel.
- **Foundational (Phase 2)**: Depends on Phase 1 (ADR-015 documents the decisions T003/T004 implement) — BLOCKS all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational. No dependency on US2/US3.
- **User Story 2 (Phase 4)**: Depends on Foundational; T027/T028 additionally depend on US1's `notes` subcommand (T019) existing to smoke-test against. Independently testable once US1 is demoable.
- **User Story 3 (Phase 5)**: Depends on Foundational and US1's `notes` subcommand (T019, T031's watcher wiring point).
- **Polish (Phase 6)**: Depends on all three user stories being complete.

### Parallel Opportunities

- T001/T002 (the two ADRs) in parallel.
- T003, T006, T009 in parallel (independent files/concerns within Foundational); T004 depends on T003, T005 depends on T004, T007 depends on T006, T008 depends on T004+T007.
- All of T010–T014 (US1 tests) in parallel with each other.
- T020 in parallel with T015–T019 (independent concern: fullscreen launch vs. the follower itself); T021 depends on T020.
- T022 in parallel with T015–T021 (different file).
- All of T023–T026 (US2 tests) in parallel with each other.
- All of T029–T030 (US3 tests) in parallel with each other.
- T032/T033/T034 (docs) in parallel with each other.

---

## Parallel Example: User Story 1

```bash
# Tests, all different assertions in the same TestBackend suite file — run/write together:
Task: "TestBackend scenario: notes render in render/tests.rs"
Task: "TestBackend scenario: no-notes line in render/tests.rs"
Task: "TestBackend scenario: next-title / last-slide in render/tests.rs"
Task: "TestBackend scenario: branch options in render/tests.rs"
Task: "TestBackend scenario: reveal progress in render/tests.rs"

# Independent implementation slices:
Task: "App::with_fullscreen() builder in app.rs"          # T020
Task: "cli_e2e missing-file test for notes"                # T022
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 (both ADRs) → Phase 2 (Foundational) → Phase 3 (US1).
2. **STOP and VALIDATE**: run quickstart.md steps 1–5 in two real terminals.
3. That alone is a demoable dual-screen presenter view — US2/US3 harden edge cases on top of it.

### Incremental Delivery

1. Setup + Foundational → the presenter silently starts writing session state; no user-visible change yet.
2. Add US1 → `fireside notes` exists and tracks a live presenter → demo the MVP.
3. Add US2 → the follower is now trustworthy when the presenter dies, not just when it's alive.
4. Add US3 → live-edit workflows (rehearsal, quick-edit) no longer show stale or broken notes.
5. Polish → docs, full verify suite, graph refresh, plan checkbox.

## Notes

- No task touches `fireside-core` or `fireside-engine` — confirmed in plan.md's Constitution Check; `Session::reveal_progress()` already provides everything the tick sink needs.
- Every TUI-visible task (T015–T021, T027, T031) needs a tmux smoke pass per constitution Principle VII's fourth bullet before its phase is called done — T028 is the scripted version for US2's specific failure modes (kill/restart/clean-quit); US1/US3's own happy-path tracking should also be hand-verified in tmux per quickstart.md before merging, even though only T028 adds a *scripted* smoke case.
- Commit after each task or logical group, per repository convention; do not batch unrelated tasks into one commit.
