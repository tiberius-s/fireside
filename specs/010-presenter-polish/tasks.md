# Tasks: Presenter Polish

**Input**: Design documents from `/specs/010-presenter-polish/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md, constitution 1.2.1

**Tests**: Included — Test Discipline (constitution VII) requires unit
tests at the engine/validator layer, TUI scenario tests for user-visible
state, CLI end-to-end tests for CLI-only-visible behavior, and real-terminal
tmux smoke tests for anything that launches the TUI (exit summary, resume
flash, wizard present-now). US1 additionally requires Rust/Node validator
parity (constitution-adjacent standing rule documented in
`validation.rs`'s own module doc — see `research.md` §8), proven via the
shared fixture corpus.

**Organization**: Tasks are grouped by user story per spec.md priorities
(US1 P1 reserved-key warning, US2 P2 exit summary, US3 P3 resume toast,
US4 P4 wizard momentum, US5 P5 art-text width guard). No Foundational phase
is needed — every story after US1 touches a distinct file set, except US2
and US3 which both touch `fireside-tui/src/lib.rs`'s presenting entry
points; that overlap is a direct US3→US2 task dependency inside Phase 5,
not shared infrastructure every story needs.

**Revision note**: Phase 2 (US1) was expanded after Phase 1 re-reading
surfaced a standing project constraint missed during `/speckit-plan`: every
`fireside-engine` validation rule must also exist in `protocol/validate.mjs`
and be proven in lockstep via the shared fixture corpus
(`protocol/fixtures/`, checked by both `crates/fireside-engine/tests/
fixtures.rs` and `protocol/run-fixtures.mjs`). See `research.md` §8 and
`contracts/validation-reserved-branch-key.md`'s "Rust/Node parity contract"
section. Task IDs below reflect this from the start.

## Phase 1: Setup

- [X] T001 Re-read the current state of every file this feature touches
      immediately before editing (line numbers may have shifted since
      planning): `crates/fireside-engine/src/validation.rs`,
      `crates/fireside-engine/src/lib.rs`, `protocol/validate.mjs`,
      `protocol/fixtures.expected.json`,
      `crates/fireside-tui/src/app.rs`, `crates/fireside-tui/src/lib.rs`,
      `crates/fireside-tui/src/render/tests.rs`,
      `crates/fireside-cli/src/main.rs`, `crates/fireside-cli/src/new.rs`,
      `crates/fireside-cli/src/art.rs`,
      `crates/fireside-cli/tests/cli_e2e.rs`,
      `docs/src/content/docs/spec/validation.md`,
      `docs/src/content/docs/spec/appendix-engine-extensions.md`

---

## Phase 2: User Story 1 - Catch a dead branch key before it ships (Priority: P1) 🎯 MVP

**Goal**: `fireside validate` warns when a branch option's key collides with
a reserved global presenter key — in both the Rust engine and the Node
reference validator, in lockstep.

**Independent Test**: Per spec.md US1 — validate a deck with a branch
option keyed `e`; confirm a `reserved-branch-key` warning naming the key,
node, and label; confirm an all-unreserved-keys deck produces none.

### Tests for User Story 1

- [X] T002 [P] [US1] Add unit test `reserved_branch_key_warns_on_collision`
      in `crates/fireside-engine/src/validation.rs`'s test module: a graph
      with a branch option keyed `"e"`; assert a `Severity::Warning`
      diagnostic with rule `"reserved-branch-key"` naming the node id and
      option label
- [X] T003 [P] [US1] Add unit test
      `reserved_branch_key_silent_for_unreserved_keys` in
      `crates/fireside-engine/src/validation.rs`: a graph with options keyed
      `"1"`, `"y"`, `"x"`; assert no `reserved-branch-key` diagnostic
- [X] T004 [P] [US1] Add unit test
      `reserved_branch_key_fires_once_per_colliding_option` in
      `crates/fireside-engine/src/validation.rs`: a branch point with two
      options, both keyed reserved letters (e.g. `"e"` and `"q"`); assert
      two separate `reserved-branch-key` diagnostics, one per option
      (per contracts/validation-reserved-branch-key.md's non-suppression
      rule)
- [X] T005 [P] [US1] Add unit test
      `reserved_branch_key_ignores_keyless_options` in
      `crates/fireside-engine/src/validation.rs`: a branch option with no
      `key` at all; assert no `reserved-branch-key` diagnostic
- [X] T006 [P] [US1] Add fixture `protocol/fixtures/valid/reserved-branch-key.json`
      (a minimal single-node graph with a branch option keyed `"e"`,
      mirroring `protocol/fixtures/valid/malformed-link-url.json`'s
      minimal style) and its matching entry
      `"valid/reserved-branch-key.json": ["reserved-branch-key"]` in
      `protocol/fixtures.expected.json` — this fixture is what T014 uses to
      prove Rust/Node rule-id parity

### Implementation for User Story 1

- [X] T007 [US1] In `crates/fireside-engine/src/validation.rs`, add
      `pub const RESERVED_PRESENTER_KEYS: [char; 12] = ['e', 'f', 'g', 'h',
      'j', 'k', 'm', 'n', 'p', 'q', 's', 't'];` near the top of the file,
      doc-commented per `contracts/validation-reserved-branch-key.md` and
      `research.md` §1 (naming it as the single source of truth shared with
      `fireside-tui`'s key-dispatch regression test)
- [X] T008 [US1] In `crates/fireside-engine/src/validation.rs`, add
      `fn check_reserved_branch_keys(graph: &Graph, diags: &mut
      Vec<Diagnostic>)` next to `check_branch_options` (~L184): for every
      branch point option with a `key` that appears (as a `char`) in
      `RESERVED_PRESENTER_KEYS`, push a `Diagnostic::new(Severity::Warning,
      "reserved-branch-key", <message naming the key, node id, option
      label, and what the key already does globally>, Some(&node.id))`
- [X] T009 [US1] In `crates/fireside-engine/src/validation.rs`'s `pub fn
      validate` (~L91), add a call to `check_reserved_branch_keys(graph,
      &mut diags);` alongside the other check calls
- [X] T010 [US1] In `crates/fireside-engine/src/lib.rs`, add
      `RESERVED_PRESENTER_KEYS` to the existing `pub use validation::{...}`
      re-export list
- [X] T011 [US1] In `protocol/validate.mjs`, add a `RESERVED_PRESENTER_KEYS`
      array literal (same 12 characters, doc-commented as mirroring
      `fireside-engine`'s constant) and a `checkReservedBranchKeys(graph)`
      function mirroring `checkMalformedLinkUrls`'s shape (~L449) and
      `checkUniqueBranchKeys`'s branch-option iteration (~L175): same
      message shape as T008's Rust diagnostic, `"warning"` severity, rule id
      `"reserved-branch-key"`
- [X] T012 [US1] In `protocol/validate.mjs`'s `export function validate`
      aggregator (~L630), add `...checkReservedBranchKeys(graph),`
      alongside `...checkUniqueBranchKeys(graph)`
- [X] T013 [US1] In `protocol/validate.mjs`'s `HELP` text (~L672, "Rules
      (warnings)" section), add a line for `reserved-branch-key` matching
      the existing table's alignment style
- [X] T014 [US1] Run `node protocol/run-fixtures.mjs` and `cargo test -p
      fireside-engine --test fixtures` — both must pass, proving the T006
      fixture reports exactly `["reserved-branch-key"]` from both
      implementations (Rust/Node parity, research.md §8)
- [X] T015 [US1] Add regression test
      `reserved_presenter_keys_are_all_consumed_globally` in
      `crates/fireside-tui/src/render/tests.rs` (or `app.rs`'s own test
      module if more natural): for every char in
      `fireside_engine::RESERVED_PRESENTER_KEYS`, drive that key through
      `App::update` on a `Screen::Present` app with an open branch point
      whose only option is keyed with that same char, and assert the
      branch was **not** taken (the global action fired instead) — this is
      the cross-crate guard from `contracts/validation-reserved-branch-key.md`
- [X] T016 [US1] Add the `reserved-branch-key` bullet to the "Recommended
      Checks" list in `docs/src/content/docs/spec/validation.md` (~L68,
      alongside `ascii-art-too-wide`/`ascii-art-empty`), naming the twelve
      reserved keys
- [X] T017 [US1] Add one bullet to the "Behavior near the protocol's edges"
      section of `docs/src/content/docs/spec/appendix-engine-extensions.md`,
      following the existing `malformed-link-url` bullet's format exactly
      (per `research.md` §7)
- [X] T018 [US1] Run `cargo test -p fireside-engine -p fireside-tui` and
      `cargo clippy -p fireside-engine -p fireside-tui --all-targets`

**Checkpoint**: `fireside validate` on a deck with a colliding branch key
warns clearly, in both the Rust CLI and `node protocol/validate.mjs`;
`fireside validate docs/examples/hello.json` still reports 0 errors/0
warnings. Independently shippable.

---

## Phase 3: User Story 2 - See how the rehearsal went (Priority: P2)

**Goal**: A graceful `q` (or in-app Ctrl+C) quit prints one summary line
after the TUI closes.

**Independent Test**: Per spec.md US2 — present a deck, view a subset of
slides, quit with `q`; confirm a `Presented N/M slides in MM:SS.` line
after the terminal restores, with correct counts.

### Tests for User Story 2

- [X] T019 [P] [US2] Add unit test(s) for the summary line's formatting
      (e.g. `format_present_summary_pads_seconds`,
      `format_present_summary_handles_first_slide_only`) in
      `crates/fireside-cli/src/main.rs`'s existing `#[cfg(test)] mod
      tests` (~L331): given `seen`, `total`, and a `Duration`, assert the
      exact string `"Presented {seen}/{total} slides in {mm}:{ss}."` with
      zero-padded seconds (per `contracts/present-summary-and-resume-flash.md`)

### Implementation for User Story 2

- [X] T020 [US2] In `crates/fireside-tui/src/lib.rs`, add
      `pub struct PresentSummary { pub seen: usize, pub total: usize, pub
      elapsed: Duration }`, doc-commented per `data-model.md`
- [X] T021 [US2] In `crates/fireside-tui/src/lib.rs`, change
      `present_authoring`'s return type to `Result<PresentSummary,
      TuiError>`: capture `graph.nodes.len()` as `total` before `graph`
      moves into `Session::new`, and after `event_loop` returns `Ok(())`,
      build `PresentSummary { seen: app.session().visited().len(), total,
      elapsed: app.elapsed() }` and return it wrapped in `Ok`
- [X] T022 [US2] In `crates/fireside-tui/src/lib.rs`, update `present` and
      `present_watching`'s return types to `Result<PresentSummary,
      TuiError>` (they already just forward `present_authoring`'s result,
      so no body change beyond the signature)
- [X] T023 [US2] In `crates/fireside-cli/src/main.rs`, add a small pure
      helper `fn format_present_summary(seen: usize, total: usize, elapsed:
      Duration) -> String` producing `"Presented {seen}/{total} slides in
      {mm}:{ss}."` with zero-padded seconds — this is what T019's unit
      tests exercise
- [X] T024 [US2] In `crates/fireside-cli/src/main.rs`'s `present()`
      function (~L226), change the `fireside_tui::present_authoring(...)`
      call site to capture `Ok(summary)`, print
      `format_present_summary(summary.seen, summary.total, summary.elapsed)`
      to stdout after the call returns, then return `Ok(())`; propagate
      `Err` exactly as today (`.context(...)`)
- [X] T025 [US2] In `crates/fireside-cli/src/main.rs`'s `demo()` function
      (~L271), apply the same summary-printing treatment to its
      `fireside_tui::present(graph)` call site
- [X] T026 [US2] Add tmux smoke test coverage for the exit summary (per
      constitution Principle VII and memory
      `feedback_tmux_smoke_catches_timing_bugs`): launch `fireside demo` in
      a detached tmux session, advance a few slides, send `q`, capture-pane
      the shell after the TUI closes, and assert the summary line appears
      with a plausible `N/7` count
- [X] T027 [US2] Run `cargo test -p fireside-tui -p fireside-cli` and
      `cargo clippy -p fireside-tui -p fireside-cli --all-targets`

**Checkpoint**: Every graceful quit from `fireside demo` or `fireside
<deck>` prints an accurate one-line summary. Independently shippable on top
of US1.

---

## Phase 4: User Story 3 - Know at a glance that a session resumed (Priority: P3)

**Goal**: Launching into a resumed session flashes a one-time notice.

**Independent Test**: Per spec.md US3 — quit a deck partway through, relaunch
without `--restart`, confirm the resume flash; relaunch with `--restart`,
confirm no flash; launch a deck with no resume record, confirm no flash.

**Depends on**: T021 (US2's `present_authoring` signature change) — this
story's flash is set inside the same function body; it does not depend on
US2's exit-summary *behavior*, only on `present_authoring` already
returning `PresentSummary` so this story's edit lands in the post-US2
version of the function.

### Tests for User Story 3

- [X] T028 [P] [US3] Add scenario test `resume_flash_shows_on_first_frame`
      in `crates/fireside-tui/src/render/tests.rs`: construct an `App` via
      the same path `present_authoring` uses (or a small test-only
      equivalent) with a resumed session, render the first frame, assert
      the flash text `"Resumed where you left off — --restart starts
      over"` is visible
- [X] T029 [P] [US3] Add scenario test `no_resume_flash_on_fresh_session`
      in `crates/fireside-tui/src/render/tests.rs`: same setup with
      `initial_node: None`; assert no flash is present on the first frame

### Implementation for User Story 3

- [X] T030 [US3] In `crates/fireside-tui/src/app.rs`, widen `set_flash`
      (~L809) from private `fn` to `pub(crate) fn` — no other change to its
      body
- [X] T031 [US3] In `crates/fireside-tui/src/lib.rs`'s `present_authoring`,
      after `Session::new` and the existing `if let Some(id) = initial_node
      { let _ = session.goto(id); }`, capture the `Outcome` instead of
      discarding it; construct `App::new(session)` and, if the captured
      outcome was `Outcome::Moved`, call `app.set_flash("Resumed where you
      left off — --restart starts over", FlashKind::Info)` before entering
      `event_loop`
- [X] T032 [US3] Add tmux smoke test coverage for the resume flash: present
      a scratch deck, advance past the first slide, quit with `q`
      (leaving a resume record), relaunch the same deck in a fresh tmux
      session, capture-pane, and assert the flash text is visible on the
      first frame; relaunch again with `--restart` and assert it is absent
- [X] T033 [US3] Run `cargo test -p fireside-tui` and `cargo clippy -p
      fireside-tui --all-targets`

**Checkpoint**: Resuming a deck now visibly announces itself; a fresh or
`--restart`ed session stays silent. Independently shippable on top of
US1+US2.

---

## Phase 5: User Story 4 - Get from idea to rehearsal in one flow (Priority: P4)

**Goal**: The interactive `fireside new` wizard offers to present the deck
it just created.

**Independent Test**: Per spec.md US4 — run the wizard, answer yes at the
final prompt, confirm the presenter launches on the new deck without a
second command; answer no, confirm the wizard exits normally; confirm the
non-interactive `fireside new <name>` never prompts.

### Tests for User Story 4

- [X] T034 [P] [US4] Add CLI end-to-end test
      `new_non_interactive_never_prompts_to_present` in
      `crates/fireside-cli/tests/cli_e2e.rs`: run `fireside new
      <name>` non-interactively (existing pattern from
      `new_accepts_a_template_and_author_flag_non_interactively`); assert
      stdout does **not** contain `"Present it now"` (FR-010)

### Implementation for User Story 4

- [X] T035 [US4] In `crates/fireside-cli/src/new.rs`, change `new_deck`'s
      signature from `Result<()>` to `Result<Option<PathBuf>>` per
      `contracts/new-wizard-present-now.md`; every existing early return
      becomes `Ok(None)`; the final `Ok(())` at the end becomes the new
      present-now-aware return described in T036
- [X] T036 [US4] In `crates/fireside-cli/src/new.rs`, after the existing
      interactive-only banner-skip note, add: when `name` was originally
      `None` (interactive path), prompt `"Present it now? [Y/n]: "` via the
      existing `prompt_line` helper; `None`/empty/`"y"`/`"yes"`
      (case-insensitive) → return `Ok(Some(path))`; anything else → `Ok(None)`.
      The non-interactive path (`name` was `Some(..)`) always returns
      `Ok(None)` without prompting
- [X] T037 [US4] In `crates/fireside-cli/src/main.rs`'s `New` dispatch arm
      (~L155), change `new::new_deck(name, template, author, banner)` to
      match its `Option<PathBuf>` result: `Some(path) => present(&path,
      false)`, `None => Ok(())`
- [X] T038 [US4] Add tmux smoke test coverage for the wizard's present-now
      prompt: drive `fireside new` interactively via `tmux send-keys`
      through all prompts ending in Enter at `Present it now?`, and assert
      the presenter's first frame appears in the pane
- [X] T039 [US4] Run `cargo test -p fireside-cli` and `cargo clippy -p
      fireside-cli --all-targets`

**Checkpoint**: Interactive `fireside new` can go straight into a
rehearsal; non-interactive `fireside new <name>` is unchanged.
Independently shippable — no dependency on US1-3.

---

## Phase 6: User Story 5 - Know an ASCII banner won't fit before pasting it (Priority: P5)

**Goal**: `fireside art text` warns on stderr when its output exceeds the
76-column authoring threshold.

**Independent Test**: Per spec.md US5 — run `art text` with a long phrase,
confirm a stderr note naming the measured width and unchanged full stdout
output; run with a short phrase, confirm no stderr output.

### Tests for User Story 5

- [X] T040 [P] [US5] Add CLI end-to-end test
      `art_text_warns_on_stderr_when_too_wide` in
      `crates/fireside-cli/tests/cli_e2e.rs`: run `fireside art text` with
      a phrase long enough to exceed 76 columns; assert exit code 0, stdout
      contains the full multi-line banner (non-empty, matches the existing
      `art_text_prints_a_multiline_banner` shape), and stderr is non-empty
- [X] T041 [P] [US5] Add CLI end-to-end test
      `art_text_silent_on_stderr_when_it_fits` in
      `crates/fireside-cli/tests/cli_e2e.rs`: run `fireside art text "Hi"`;
      assert exit code 0 and empty stderr

### Implementation for User Story 5

- [X] T042 [US5] In `crates/fireside-cli/src/art.rs`'s `art_text` (~L32),
      after printing the banner to stdout, measure
      `art.lines().map(str::len).max().unwrap_or(0)` (same measurement
      `new.rs::add_title_banner` uses) and, if it exceeds
      `DEFAULT_ART_WIDTH`, `eprintln!` a note naming the measured width per
      `contracts/art-text-width-guard.md`
- [X] T043 [US5] Run `cargo test -p fireside-cli` and `cargo clippy -p
      fireside-cli --all-targets`

**Checkpoint**: `art text` output is unchanged when it fits and gains a
clear heads-up when it doesn't. Independently shippable — no dependency on
US1-4.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T044 Run `cargo test --workspace` and `cargo clippy --workspace
      --all-targets` (must stay silent per constitution Operational
      Constraints)
- [X] T045 Run `node protocol/validate.mjs docs/examples/hello.json`
      (unaffected by this feature, must stay 0 errors) and `npm run check
      --prefix docs` (validation.md/appendix-engine-extensions.md edits
      must build clean)
- [X] T046 Run `scripts/verify.sh` (mirrors every CI job)
- [X] T047 Run `graphify update .` to refresh the knowledge graph
- [X] T048 Walk through every scenario in `quickstart.md` manually (or via
      the tmux smoke tests already added in T026/T032/T038) as a final
      end-to-end confirmation

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **US1 (Phase 2, P1)**: Depends only on Setup. No dependency on any other
  story.
- **US2 (Phase 3, P2)**: Depends only on Setup. Independent of US1.
- **US3 (Phase 4, P3)**: Depends on Setup **and** on US2's T021 (the
  `present_authoring` signature change) — see the note at the top of
  Phase 4. Independent of US1 and US4/US5.
- **US4 (Phase 5, P4)**: Depends only on Setup. Independent of every other
  story.
- **US5 (Phase 6, P5)**: Depends only on Setup. Independent of every other
  story.
- **Polish (Phase 7)**: Depends on all desired stories being complete.

### Parallel Opportunities

- US1, US2, US4, and US5 can all be implemented in parallel (different
  files, no shared state) once Setup is done.
- US3 must start after US2's T021 lands, but can otherwise proceed in
  parallel with US1/US4/US5.
- Within US1: T002-T006 (tests + fixture) in parallel; T007-T013 sequential
  (shared files/functions: Rust const → Rust check → Rust wiring → Rust
  re-export → JS const+check → JS wiring → JS help text); T014 (parity run)
  depends on T006 and T013; T015 depends on T010 (the re-export); T016-T017
  (docs) in parallel with each other and with T014/T015.
- Within US2: T019 in parallel with T020; T021 depends on T020; T022
  depends on T021; T023 in parallel with T020/T021; T024-T025 depend on
  T021 and T023.
- Within US4: T035 must land before T036; T037 depends on T035's new
  return type.
- Within US5: T040-T041 in parallel; T042 is the only implementation task.

## Implementation Strategy

### MVP First (User Story 1 Only)

Phase 1 → Phase 2 (US1) → validate independently → ship. This alone closes
the exact bug class that shipped in the demo deck (Wave 1's W1-1), in both
the Rust and Node validators.

### Incremental Delivery

Phase 1 → US1 → US2 → US3 (needs US2's T021) → US4 → US5 → Phase 7 Polish,
each phase independently demonstrable, matching the source plan's
suggested priority order (P1 highest leverage, P5 smallest footprint).

## Notes

- Every TUI-visible story (US2, US3, US4) gets a tmux smoke test in
  addition to `TestBackend`/unit coverage — `TestBackend` alone has
  historically missed timing/ordering bugs on this project (memory
  `feedback_tmux_smoke_catches_timing_bugs`).
- `RESERVED_PRESENTER_KEYS` (T007) is the single Rust source of truth for
  the reserved key set — `fireside-tui`'s regression test (T015) must
  import it, never redeclare it. `protocol/validate.mjs`'s copy (T011) is a
  separate, hand-kept-in-sync JS literal (no cross-language import
  mechanism exists) — the fixture in T006, checked by T014, is what
  actually proves the two agree, per
  `contracts/validation-reserved-branch-key.md`.
- Commit after each checkpoint (end of a user-story phase), not after every
  individual task.
