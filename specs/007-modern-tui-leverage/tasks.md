# Tasks: Modern TUI Leverage

**Input**: Design documents from `/specs/007-modern-tui-leverage/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included — Test Discipline (constitution VII) requires unit
tests at the model/engine layer, scenario tests for user-visible TUI
state, CLI e2e tests for the resume round trip, and a tmux smoke pass for
every presenter-facing change (mouse and OSC 8 in particular cannot be
verified by `TestBackend` alone — see quickstart.md).

**Organization**: Tasks are grouped by user story per spec.md priorities
(US1 P1 mouse, US2 P2 resume, US3 P3 synchronized output, US4 P4 OSC 8
hyperlinks). Unlike `006-incremental-reveal`, these four stories touch
non-overlapping code paths and have **no shared blocking prerequisites** —
research.md and plan.md confirm each is independently implementable and
testable, so there is no Foundational phase; Phase 2 is skipped by design,
not omission.

## Phase 1: Setup

- [X] T001 Re-read the current state of every file this feature touches
      immediately before editing (line numbers may have shifted since
      planning): `crates/fireside-tui/src/app.rs`,
      `crates/fireside-tui/src/lib.rs`, `crates/fireside-tui/src/render/mod.rs`,
      `crates/fireside-tui/src/render/markdown.rs`,
      `crates/fireside-tui/src/render/blocks.rs`,
      `crates/fireside-engine/src/validation.rs`, `crates/fireside-cli/src/main.rs`,
      `protocol/validate.mjs`,
      `docs/src/content/docs/spec/appendix-engine-extensions.md`
- [X] T002 Confirm the baseline is green before any change: `cargo test --workspace` and `cargo clippy --workspace --all-targets`

**Checkpoint**: Baseline confirmed clean. Each user story phase below can
start independently from here — pick any order, though P1→P4 is
recommended since that's presenter-value order.

---

## Phase 2: User Story 1 - Click to navigate (Priority: P1) 🎯 MVP

**Goal**: Mouse clicks on the map screen and branch menu perform the same
navigation/choice a keyboard select would, without changing any existing
keyboard behavior.

**Independent Test**: Per spec.md US1 — open the map, click a
non-current row, confirm the jump; at a branch point, click an option,
confirm the same choice a keypress would produce; confirm reveal-pending
clicks advance reveal instead of choosing early.

### Tests for User Story 1

- [X] T003 [P] [US1] Add scenario test `clicking_a_map_row_navigates_to_that_slide` in `crates/fireside-tui/src/render/mod.rs`: open the map screen, synthesize a `MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), .. }` at the row for a non-current node, drive it through `App::update`, assert the presenter is now on that node and the screen returned to `Present`
- [X] T004 [P] [US1] Add scenario test `clicking_a_branch_option_chooses_it` in `crates/fireside-tui/src/render/mod.rs`: at a fully-revealed branch-point node, synthesize a click on the second option's rendered row, assert the same target node is reached as pressing that option's key would produce
- [X] T005 [P] [US1] Add scenario test `clicking_a_branch_option_row_before_it_is_drawn_is_inert` in `crates/fireside-tui/src/render/mod.rs`: a node with an unrevealed block and a branch-point; a click where the (not-yet-rendered) menu would eventually appear does nothing and does not consume the reveal step — since the branch menu isn't drawn at all until reveal is exhausted (mirrors the keyboard gate), there is nothing there yet to hit-test onto, unlike `branch_keys_are_inert_while_reveal_is_pending` from `006-incremental-reveal`, where a keypress isn't tied to a rendered row
- [X] T006 [P] [US1] Add scenario test `clicking_outside_any_interactive_row_is_inert` in `crates/fireside-tui/src/render/mod.rs`: a click in the map's margin (or on body text during `Present`) produces no state change (spec.md Edge Cases)
- [X] T007 [P] [US1] Add scenario test `keyboard_only_flows_are_unaffected_by_mouse_support` in `crates/fireside-tui/src/render/mod.rs`: drive an existing keyboard-only scenario (e.g. reuse `HELLO`'s flow) end-to-end with zero `Msg::Mouse` events and assert identical behavior to before this feature (regression guarantee, FR-003)

### Implementation for User Story 1

- [X] T008 [US1] In `crates/fireside-tui/src/app.rs`, add a `size: ratatui::layout::Size` (or `Rect`) field to `App`, defaulted to a sane minimum and updated whenever the terminal size is known (first draw + `crossterm::event::Event::Resize`)
- [X] T009 [US1] In `crates/fireside-tui/src/render/mod.rs`, extract pure `fn map_row_rect(area: Rect, row_index: usize) -> Rect` (or equivalent) used by the existing map-screen drawing code, refactoring the current inline layout math into this function without changing rendered output
- [X] T010 [US1] In `crates/fireside-tui/src/render/mod.rs`, extract a pure hit-test/layout function for branch-menu option rows analogous to T009, reusing whatever internal layout the branch-menu renderer already computes
- [X] T011 [US1] In `crates/fireside-tui/src/app.rs`, add `Msg::Mouse(crossterm::event::MouseEvent)` and route `crossterm::event::Event::Mouse(..)` into it from wherever `Msg::Terminal` is currently dispatched
- [X] T012 [US1] In `crates/fireside-tui/src/app.rs`, implement mouse handling: on the map screen, a left-button-down hit on a row calls the same path as `on_map_key`'s `Enter` case (using T009's rect function against `self.size`); on the present screen at a branch point, a hit on an option row calls the same path as `on_branch_key`'s selection case (using T010), respecting the existing `pending_reveal` gate exactly as `on_present_key` already does for keyboard (`app.rs:554-555`)
- [X] T013 [US1] In `crates/fireside-tui/src/lib.rs`, bracket `ratatui::init()`/`ratatui::restore()` (or the surrounding setup/teardown) with `crossterm::execute!(.., EnableMouseCapture)` / `DisableMouseCapture` so mouse events are actually reported
- [X] T014 [US1] Add a one-line mouse-support mention to the help screen content (wherever `Screen::Help`'s text is defined) — e.g. "Click a map row or branch option to select it"
- [X] T015 [US1] Smoke-test in tmux per quickstart.md §1 and the tmux section: inject SGR mouse-click escape sequences via `tmux send-keys -H` against the map screen and a branch menu, `capture-pane`, and visually/textually confirm the same navigation a keypress would produce

**Checkpoint**: Mouse navigation works on map + branch menu; all existing
keyboard scenario tests still pass unmodified; `cargo test -p fireside-tui`
green.

---

## Phase 3: User Story 2 - Resume after a crash or interruption (Priority: P2)

**Goal**: An interrupted (non-terminal) presentation reopens on the same
slide when the same deck content is relaunched; a completed run does not.

**Independent Test**: Per spec.md US2 — present, navigate partway, kill the
process (not `q`), relaunch the same file, confirm same slide; reach the
end normally and relaunch, confirm slide 1; change the file and relaunch,
confirm slide 1; `--restart` always forces slide 1.

### Tests for User Story 2

> **Note on test placement**: `present_authoring` calls `ratatui::init()`
> (raw-mode terminal setup) unconditionally, which errors immediately under
> `assert_cmd`'s piped (non-tty) harness — so the actual kill/relaunch
> interactive behavior cannot be driven from `cli_e2e.rs` at all, discovered
> while implementing this story. Tests below cover the resume-store logic
> directly (`resume.rs`'s own unit tests); the interactive behavior itself
> is proven via the tmux smoke pass (T030), consistent with
> [[feedback-tmux-smoke-catches-timing-bugs]].

- [X] T016 [P] [US2] Add unit test `set_then_load_round_trips_the_node_id` in `crates/fireside-cli/src/resume.rs`: a position set on one `ResumeStore` is visible after reloading the store from the same path — the logic `resume_reopens_on_last_position_after_a_non_clean_exit` depends on; the actual kill/relaunch is proven live in tmux (T030)
- [X] T017 [P] [US2] Add unit test `clear_removes_the_record` in `resume.rs`: after `clear`, a reload sees no record — the logic `resume_does_not_persist_past_a_completed_run` depends on; proven live in tmux (T030)
- [X] T018 [P] [US2] Add unit test `unrelated_fingerprints_do_not_collide` in `resume.rs`: two different fingerprint keys never leak into each other's lookup — covers `resume_is_ignored_when_content_fingerprint_changes` at the storage-key level (a changed fingerprint is definitionally a different, absent key)
- [X] T019 [P] [US2] `resume_falls_back_gracefully_when_saved_node_no_longer_exists` is inherited from `fireside-engine`'s already-tested `Session::goto` guarded-no-op (`goto_unknown_node_is_a_guarded_no_op` in `session.rs`) — `present_authoring`'s `let _ = session.goto(id);` (T024) is a one-line reuse of that exact guarantee, not new logic to re-test at this layer
- [X] T020 [P] [US2] Add unit test `restart_bypasses_the_lookup_without_deleting_the_record` in `resume.rs`: `resolve_initial_node(Some(key), true)` returns `None` while the underlying record is untouched and visible on the next non-restart lookup
- [X] T021 [P] [US2] Add unit tests `no_fingerprint_means_no_initial_node` and `missing_record_for_a_known_fingerprint_means_no_initial_node` in `resume.rs` — `fireside demo` has no `Path` at all, so `resume::fingerprint_key` structurally cannot be called for it; verified by construction (its code path never calls into `resume::`) plus these two lookup-miss cases

### Implementation for User Story 2

- [X] T022 [US2] In `crates/fireside-cli/src/resume.rs` (new module), add resume-state file path resolution (`XDG_STATE_HOME` env var, falling back to `~/.local/state/fireside/resume.json`) per contracts/resume-state-format.md, using only `std::env`/`std::path` (flagged in research.md §2 — no new dependency)
- [X] T023 [US2] In `resume.rs`, add read/write helpers for the resume-state JSON map (`fingerprint → { node_id, updated_at }`) via `serde_json::Value`/`Map` (no new `serde` dependency), tolerating a missing/unparseable file as "no record" per the contract (never an error surfaced to the user)
- [X] T024 [US2] In `crates/fireside-tui/src/lib.rs`, add an `initial_node: Option<&str>` parameter to `present_authoring` (threaded through `present`/`present_watching` as `None`), applied via one `Session::goto` call right after `Session::new`, ignoring its `Outcome` (reuses the existing guarded-no-op fallback, satisfies FR-008 with no new logic)
- [X] T025 [US2] In `lib.rs`, add a `PositionSink` callback parameter to `present_authoring` (same shape/placement convention as `WriteBackSink`), invoked from `event_loop` once at startup and again whenever the current node id changes after any `app.update(..)` call (covers key/mouse/reload-driven moves uniformly, not just `Outcome::Moved`)
- [X] T026 [US2] In `main.rs`'s `present` function, wire `resume.rs`'s helpers to `present_authoring`'s new parameters: look up the current fingerprint's record for the initial node via `resolve_initial_node`, and persist on every position-changed callback invocation
- [X] T027 [US2] In `main.rs`, clear the resume-state entry when the position-changed callback reports a node where `Node::is_terminal()` is true (a dead end — no further `next`/branch target), checked against the graph as loaded at present-startup
- [X] T028 [US2] In `main.rs`, add a `--restart` flag to the `Present` subcommand (per contracts/cli-flags.md) that skips the resume lookup for that invocation only
- [X] T029 [US2] `fireside demo`'s code path (`demo()` in `main.rs`) has no `Path` parameter and calls `fireside_tui::present`, never `present()`'s resume wiring — structurally cannot touch resume state (ties to FR-009 / T021)
- [X] T030 [US2] Smoke-test in tmux per quickstart.md §2: killed a presenting pane mid-deck (SIGKILL, not `q`) after navigating to "features"; a fresh pane on the same file reopened directly on "features" (not "intro"); separately, navigating on to the terminal "thanks" node and killing there showed the resume-state file cleared to `{}`. (`--restart`/demo-never-touches-it are covered by the T020/T021 unit tests above — a second live tmux pass hit an unrelated environment shell-startup hang unrelated to this feature and was not worth fighting further given the logic is already proven both by direct unit test and by the successful kill/relaunch/clear runs.)

**Checkpoint**: Resume works end-to-end via the CLI; `fireside-tui`'s public
API changes are additive and backward-compatible; `cargo test -p
fireside-cli -p fireside-tui` green.

---

## Phase 4: User Story 3 - Flicker-free transitions (Priority: P3)

**Goal**: No visible torn/partial frames during transitions on capable
terminals; zero behavior change on incapable ones.

**Independent Test**: Per spec.md US3 — rapid transitions in a capable
terminal show no tearing; the same sequence in an incapable terminal
behaves exactly as before this feature.

### Tests for User Story 3

- [X] T031 [P] [US3] Add unit test `synchronized_update_commands_are_the_expected_escape_sequences` in `crates/fireside-tui/src/lib.rs`'s test module: pins the exact `\x1b[?2026h`/`\x1b[?2026l` bytes `BeginSynchronizedUpdate`/`EndSynchronizedUpdate` write via `Command::write_ansi` — `event_loop`'s reliance on them being a terminal-ignorable no-op when unsupported is the property that matters (research.md §3); the full "no visible tearing" claim is a real-terminal property proven in tmux instead (T033)

### Implementation for User Story 3

- [X] T032 [US3] In `crates/fireside-tui/src/lib.rs`, bracket the `terminal.draw(|frame| render::draw(frame, app))?;` call with `crossterm::execute!(.., BeginSynchronizedUpdate)` / `EndSynchronizedUpdate` (no capability query — inert by design on unsupported terminals, per research.md §3)
- [X] T033 [US3] Smoke-test in tmux per quickstart.md §3: held Space for 8 rapid transitions (intro → features → choose) in one burst; screen rendered cleanly with no tearing/corruption/hang and landed on the correct slide

**Checkpoint**: Synchronized output wraps every frame; no regression in
existing TUI scenario tests (this change is invisible to `TestBackend`,
which doesn't write real escape sequences, so the unit test in T031 is the
only automated guard — tmux is the real proof).

---

## Phase 5: User Story 4 - Clickable links in slide text (Priority: P4)

**Goal**: `[label](url)` in text-bearing content blocks renders as a
clickable OSC 8 region on capable terminals and plain readable text
otherwise; malformed URLs get a validation warning.

**Independent Test**: Per spec.md US4 — author a deck with a link, validate
it (warning only for a malformed URL), present it in a capable terminal
(clickable, correctly destined) and an incapable one (plain readable
label, no raw escapes).

### Tests for User Story 4

- [X] T034 [P] [US4] Add unit test `link_marker_is_parsed_alongside_existing_inline_styles` in `crates/fireside-tui/src/render/markdown.rs`: `"[label](url) and **bold**"` produces a link fragment plus a bold fragment; assert the label text and captured URL
- [X] T035 [P] [US4] Add unit test `unmatched_link_brackets_render_literally` in `markdown.rs`: `"[oops(missing paren"` renders as literal text, matching the existing "unmatched markers render literally" rule for `**`/`` ` ``
- [X] T036 [P] [US4] Add scenario test `link_cell_carries_osc8_escape_with_forced_width` in `crates/fireside-tui/src/render/mod.rs`: render a text block containing a link, inspect the resulting leading `Cell` for the OSC 8 open/close byte sequence around the label and a `CellDiffOption::ForcedWidth` matching the label's visible width, with the label's trailing cells blanked to a single space (not `Skip` — see the T040 note on why `ForcedWidth` alone suffices)
- [X] T037 [P] [US4] Add unit tests `malformed_link_url_warns` / `well_formed_link_url_does_not_warn` / `text_with_no_links_never_warns` in `crates/fireside-engine/src/validation.rs`: a node with `[label](not a url)` produces the new WARNING rule; well-formed URLs (including `mailto:`) and plain `[bracket]` text with no link syntax produce none
- [X] T038 [P] [US4] Add fixture pair `protocol/fixtures/valid/{malformed,well-formed}-link-url.json` + `fixtures.expected.json` entries, asserting `protocol/validate.mjs` fires the identical rule name as the Rust validator (via the shared fixture corpus mechanism used by `empty-traversal`/`reveal-masked-by-container`) — confirmed to actually catch drift via the same "introduce a deliberate rule-name mismatch, watch it fail, revert" discipline used for those two rules

### Implementation for User Story 4

- [X] T039 [US4] In `crates/fireside-tui/src/render/markdown.rs`, extend `parse_inline` with a `[label](url)` marker pair: on success, registers the URL in a thread-local per-frame registry (`markdown::{reset_links,link_url}`, cleared once per frame by `render::draw`) and produces a `Fragment` styled via a new `Tokens::link(index)` — a real accent+underline look with the link's index smuggled into `underline_color`'s otherwise-unused red channel. **Scope reduction from plan**: the label itself is NOT recursively parsed for nested bold/italic/code (documented in contracts/link-syntax.md and research.md as a deliberate simplification, not an oversight)
- [X] T040 [US4] Added `render::apply_hyperlinks`, called once at the end of `render::draw` over `frame.buffer_mut()`: scans for contiguous runs of `Tokens::link`-styled cells, rewrites each run's leading cell to `OSC-8-open + label + OSC-8-close` with `CellDiffOption::ForcedWidth(label_width)`, and blanks the run's other cells to `" "`. **Correction from the original plan**: `CellDiffOption::Skip` on trailing cells turned out to be unnecessary and was dropped — reading `ratatui-core`'s actual `BufferDiff::next` (0.1.2) showed `ForcedWidth` alone already makes the diff iterator advance past `width - 1` further cells unconditionally; blanking trailing cells to a space is only needed to keep the raw buffer "well-formed" per `Buffer::diff`'s own documented assumption ("no double-width cell is followed by a non-blank cell"), not for the diff/backend path itself. **Dependency note**: this required bumping the transitively-resolved `ratatui-core` from 0.1.0 to 0.1.2 (`CellDiffOption::ForcedWidth` doesn't exist in 0.1.0) — flagged to and approved by the user mid-implementation; pulls in ~7 new compiled transitive crates (strum, hashbrown, critical-section, a `lru` bump, rustix, a `bitflags` bump, line-clipping), still MSRV-1.88-safe, no direct `Cargo.toml` change
- [X] T041 [US4] In `crates/fireside-engine/src/validation.rs`, added `check_malformed_link_urls` (WARNING, rule `malformed-link-url`) with its own minimal, independent `find_links`/`is_well_formed_url` scanner (engine cannot depend on `fireside-tui` per the crate boundary table, so it doesn't reuse the TUI parser — it only needs the URL portion, walking `Text`/`Heading`/`List`/`Container` blocks recursively)
- [X] T042 [US4] Mirrored the rule in `protocol/validate.mjs` (`findLinks`/`isWellFormedUrl`/`checkMalformedLinkUrls`) with the identical rule name, wired into `validate()` and the `--help` rule listing
- [X] T043 [US4] Added `protocol/fixtures/valid/malformed-link-url.json` and `.../well-formed-link-url.json`, updated `fixtures.expected.json`
- [X] T044 [US4] Added the non-normative bullet to `docs/src/content/docs/spec/appendix-engine-extensions.md`'s "Behavior near the protocol's edges" list, extending the existing bold/italic/code bullet rather than adding a new one
- [X] T045 [US4] Smoke-tested in tmux: presented a scratch deck with `[Fireside repo](https://example.com/fireside)` in a text block; `tmux capture-pane -e` + a raw hexdump confirmed the exact byte sequence `ESC[4m ESC[38;5;6m ESC[58;2;1;0;0m ESC]8;;https://example.com/fireside ESC\ Fireside ESC[0m ESC]8;;ESC\` is genuinely emitted to the terminal — underline modifier, accent fg, the link-index marker correctly encoded in `underline_color` as RGB(1,0,0) for index 0, then a well-formed OSC 8 open/label/close. **Discovered nuance**: because "Fireside repo" is two words, `wrap_fragments` puts each word in its own `Span` (separated by a plain-styled space `Span`), so the rendered link becomes two adjacent, separately-OSC8-wrapped regions ("Fireside" and "repo") rather than one continuous one — both carry the same URL via the same registry index, so the link is still fully clickable across its whole visible label, just structurally split at word boundaries. Not tested against an actual OSC-8-incapable terminal emulator (none readily available in this environment) — the plain-text-fallback claim rests on the design property that unrecognized OSC sequences are inert by the terminal spec (same reasoning as synchronized output), not on an observed incapable-terminal run

**Checkpoint**: Links render correctly on both capable and incapable
terminals; validation parity holds across Rust/Node; `cargo test
--workspace` and `node protocol/validate.mjs` both green.

---

## Final Phase: Polish & Cross-Cutting Concerns

- [X] T046 Ran the quickstart.md validation pass across all four sections (mouse nav+choice, resume kill/relaunch/clear, rapid transitions, OSC 8 hyperlink) via the per-story tmux smokes above (T015, T030, T033, T045); `--restart`/demo-never-touches-resume covered by unit test rather than a third live tmux pass (an unrelated environment shell-startup hang made a second resume-specific tmux session impractical mid-session — logic already proven both by direct unit test and by the successful kill/relaunch/clear runs)
- [X] T047 `cargo test --workspace`: 188 tests passing (36+12+15+35+1+89, up from 169 pre-feature). `cargo clippy --workspace --all-targets`: silent
- [X] T048 `node protocol/validate.mjs` run against every fixture in `protocol/fixtures/{valid,invalid}/*.json` (not just the two new ones): rule-ids match `fixtures.expected.json` exactly, including `malformed-link-url` on the new fixtures
- [X] T049 [P] Ran `graphify update .` — AST re-extraction over 211/211 files, no topology changes flagged
- [X] T050 Updated `.claude/plans/2026-07-12-strategic-improvement-plan.md`'s Progress Log: checked off "P2 mouse / synchronized output / resume / OSC 8 hyperlinks" with a full summary; the protocol/workflow-hardening P2 line is untouched (separate, already-complete item)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **User Stories (Phases 2–5)**: All depend only on Setup. Unlike a typical
  feature, there is **no Foundational phase** — the four stories touch
  disjoint code paths (confirmed in plan.md's Project Structure) and can
  proceed in any order, including fully in parallel across developers.
- **Polish (Final Phase)**: Depends on all four stories being complete.

### User Story Dependencies

- **US1 (P1, mouse)**: No dependency on US2/US3/US4.
- **US2 (P2, resume)**: No dependency on US1/US3/US4. Shares `lib.rs`'s
  `event_loop` and `present_authoring` signature with US3 — sequence T024/T025
  before or after T032 to avoid a merge conflict in the same function, but
  neither depends on the other's *behavior*.
- **US3 (P3, sync output)**: No dependency on US1/US2/US4. See US2 note above
  re: `lib.rs` file overlap (not a logical dependency, just a file-conflict
  scheduling note).
- **US4 (P4, links)**: No dependency on US1/US2/US3.

### Within Each User Story

- Tests are written first and must fail before implementation (constitution
  Principle VII / this project's established TDD practice).
- Pure layout/parsing helpers before the stateful code that calls them
  (e.g. T009/T010 before T012; T039 before T040).
- Implementation before the tmux smoke task, which is always last in each
  story (mirrors `006-incremental-reveal`'s pattern).

### Parallel Opportunities

- All Setup tasks can run in sequence quickly (no [P] needed — both are
  read-only checks).
- All test tasks within a story marked [P] can be written in parallel
  (different assertions, same or sibling files, no shared mutable state).
- **All four user story phases can be worked in parallel** by different
  developers once Setup is done — this is the unusual property of this
  feature relative to prior ones in this repo.

---

## Parallel Example: User Story 1

```bash
# Launch all US1 tests together:
Task: "Scenario test clicking_a_map_row_navigates_to_that_slide in crates/fireside-tui/src/render/mod.rs"
Task: "Scenario test clicking_a_branch_option_chooses_it in crates/fireside-tui/src/render/mod.rs"
Task: "Scenario test clicking_a_branch_option_during_pending_reveal_advances_reveal_instead in crates/fireside-tui/src/render/mod.rs"
Task: "Scenario test clicking_outside_any_interactive_row_is_inert in crates/fireside-tui/src/render/mod.rs"
Task: "Scenario test keyboard_only_flows_are_unaffected_by_mouse_support in crates/fireside-tui/src/render/mod.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: User Story 1 (mouse) — the plan's own P1, and the
   most visible "modern TUI" capability.
3. **STOP and VALIDATE**: run US1's tmux smoke (T015) independently.
4. Demo if ready — the other three stories add no risk to this one.

### Incremental Delivery

1. Setup → US1 (mouse) → validate → ship.
2. Add US2 (resume) → validate → ship.
3. Add US3 (synchronized output) → validate → ship.
4. Add US4 (OSC 8 hyperlinks) → validate → ship.
5. Each story is independently valuable and independently testable; order
   beyond P1-first is a scheduling choice, not a correctness requirement.

### Parallel Team Strategy

With multiple developers, after Setup:
- Developer A: US1 (mouse) — `app.rs` + `render/mod.rs`
- Developer B: US2 (resume) — `main.rs` + `lib.rs`
- Developer C: US3 (sync output) — `lib.rs` (coordinate with B on the
  shared `event_loop` function)
- Developer D: US4 (links) — `markdown.rs` + `validation.rs` + `validate.mjs`

---

## Notes

- [P] tasks = different files (or safely-independent regions of a shared
  test file), no dependencies.
- [Story] label maps every task to its user story for traceability.
- This feature's stories are unusually independent for this repo — no
  Foundational phase, by design (see research.md/plan.md).
- Commit after each task or logical group, per this repo's existing
  practice on prior specs (001–006).
- Stop at any checkpoint to validate a story independently before moving on.
