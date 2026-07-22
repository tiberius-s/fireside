# Tasks: Authoring Editor (`fireside edit`)

**Input**: Design documents from `/specs/013-authoring-editor/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md (all present)

**Tests**: Included — Constitution VII (Test Discipline) is non-negotiable for this project and the design brief explicitly calls for TDD on the engine layer ("the invariants are the crown jewels"). Every phase below carries its layer-appropriate tests per `quickstart.md`.

**Organization**: Tasks are grouped by user story (spec.md's US1–US4, priority order) after a governance Setup phase and an E0 Foundational phase, matching the design brief's wave structure (Setup+Foundational ≈ E0, US1 ≈ core of E1+E2, US2 ≈ rest of E2, US3 ≈ E3, US4 ≈ crash-safety slice of "Never lose work," Polish ≈ E4).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: US1–US4, per spec.md's priorities. Setup/Foundational/Polish carry no story label.

## Path Conventions

Existing 4-crate Rust workspace (`crates/fireside-core`, `fireside-engine`, `fireside-tui`, `fireside-cli`) — see `plan.md`'s Project Structure for the full new-file map this feature adds.

---

## Phase 1: Setup (Governance)

**Purpose**: The two ADRs and the constitution amendment the Constitution Check gate requires, landed before any implementation — same mechanism spec 012 used for ADR-014/015.

- [X] T001 [P] Write ADR-017 (ADR-004 scope extension — `fireside edit`, the 2026-07-19 user request, and the editor-only mouse-first/keyboard-complete interaction-posture inversion) in `.claude/adrs/adr-017-fireside-edit-scope.md`, mirroring `.claude/adrs/adr-014-dual-screen-presenter-view-scope.md`'s format
- [X] T002 [P] Write ADR-018 (`engine::authoring` module charter: `Op`/`AuthoringError` design, full-clone undo over op-inversion, the id-slug/rename algorithm, the outline depth-first ordering algorithm) in `.claude/adrs/adr-018-authoring-transforms-module.md`
- [X] T003 Apply the constitution PATCH amendment bundled with ADR-018 (1.3.0 → 1.3.1): generalize the TEA-invariant wording in Principle IV from "`App::update` ... is the ONLY function that mutates `App` state" to "each TUI application struct has exactly one `update` function that is the sole mutator of its state," note the new `affordance`/`selection`/`drop-target`/`ghost` `theme.rs::Tokens` entries under Principle IV's styling rule, and add a Sync Impact Report entry, in `.specify/memory/constitution.md` (depends on T002)
- [X] T004 Add a stub `Edit { file: PathBuf }` variant to the `Command` enum in `crates/fireside-cli/src/main.rs` (parses, not yet wired to any behavior — later tasks fill it in)

**Checkpoint**: Governance artifacts merged — implementation may begin.

---

## Phase 2: Foundational (E0 — blocking prerequisites for every user story)

**Purpose**: The pure transform layer, the shared rendering seam, the hit-testing skeleton, and the read-only editor shell every user story is built on top of. No mutation-by-the-user yet.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

### `engine::authoring` (pure transform layer — TDD: tests alongside each op group, per `contracts/authoring-ops.md`)

- [X] T005 Define the `Op` enum, `AuthoringError` (thiserror, alongside the existing `EngineError`), `BlockPath`, and `BlockKind`/`BlockContent` types in `crates/fireside-engine/src/authoring.rs`, per `contracts/authoring-ops.md`
- [X] T006 Implement the id/slug algorithm (lowercase; non-alphanumeric runs → single `-`; trim; empty → `"slide"`; dedupe with `-2`, `-3`, … against existing ids) in `crates/fireside-engine/src/authoring.rs` (depends on T005)
- [X] T007 Implement `outline_order(graph: &Graph) -> Vec<OutlineRow>` (depth-first from `graph.entry()`, `next` before branch options in declared order, first-visit wins, unreachable nodes after a divider in declaration order) in `crates/fireside-engine/src/authoring.rs`, per `research.md` §8 (depends on T005)
- [X] T008 Implement the slide ops (`AddSlide`, `DeleteSlide`, `DuplicateSlide`, `RetitleSlide`, `ReorderSlide`, `SetNext`, `ClearNext`, `TurnIntoChoice`, `TurnBackIntoSlide`, `AddAnswer`, `RemoveAnswer`, `RetargetAnswer`) in `crates/fireside-engine/src/authoring.rs` (depends on T006, T007)
- [X] T009 Implement the block ops (`AddBlock`, `DeleteBlock`, `EditBlock`, `MoveBlock`, `SetRevealStep`) in `crates/fireside-engine/src/authoring.rs`, `SetRevealStep` renumbering distinct positive values to stay consecutive per `Node::reveal_levels()`'s existing ordinal semantics (depends on T005)
- [X] T010 Unit tests for every `Op` variant's preconditions/postconditions from `contracts/authoring-ops.md`'s table, in `crates/fireside-engine/src/authoring.rs` (depends on T008, T009)
- [X] T011 Proptest: no sequence of `RetitleSlide`/`DeleteSlide`/`ReorderSlide` ops can ever leave a dangling `next`/target/start-id reference, in `crates/fireside-engine/src/authoring.rs` (depends on T008)
- [X] T012 Proptest: arbitrary `Op` sequences never violate any of the four unrepresentable-by-construction invariants (duplicate id, `next`+`branch_point` conflict, gapped reveal steps, dangling reference — spec SC-007), in `crates/fireside-engine/src/authoring.rs` (depends on T008, T009)

### Shared rendering seam

- [X] T013 Extract a `SlideView` input type from `crates/fireside-tui/src/render/content.rs`'s `draw_content`, behavior-neutral — the presenter's own rendering must be byte-identical before and after
- [X] T014 [P] Add `insta` snapshot pins of the presenter's existing render output across the fixture decks, in `crates/fireside-tui/src/render/tests.rs`, proving the `SlideView` refactor changed no presenter behavior (depends on T013)

### Hit-testing skeleton

- [ ] T015 Define the `Target` enum and a `hit()` skeleton (toolbar/outline/canvas regions only; form and drag targets land with their own stories) in `crates/fireside-tui/src/editor/hit.rs`, generalizing `crates/fireside-tui/src/render/hits.rs`'s pattern per `contracts/hit-testing.md`
- [ ] T016 Table-driven unit tests for the `hit()` skeleton's regions in `crates/fireside-tui/src/editor/hit.rs` (depends on T015)

### Theme tokens

- [ ] T017 [P] Add `affordance`, `selection`, `drop-target`, `ghost` tokens to `crates/fireside-tui/src/theme.rs::Tokens`

### `EditorApp` scaffold + read-only studio

- [ ] T018 Define the `EditorApp` struct (`working_graph`, `saved_graph` marker, `selection`, `drag`, `open_form`, `history`, `terminal_size`, `status`, draft-timer fields) and its sole `update()` in `crates/fireside-tui/src/editor/mod.rs`, per `data-model.md`'s `EditorApp` section (depends on T005, T015, T017)
- [ ] T019 Implement read-only toolbar/outline/canvas/status/hint-line rendering in `crates/fireside-tui/src/render/editor/mod.rs`, `crates/fireside-tui/src/render/editor/canvas.rs`, `crates/fireside-tui/src/render/editor/outline.rs` — canvas renders through the `SlideView` path from T013 (depends on T013, T018)
- [ ] T020 Implement the minimum-geometry guard (below 80×24: single centered message, no overlapping panes; re-checked continuously, not only at open) in `crates/fireside-tui/src/render/editor/mod.rs` (depends on T019)
- [ ] T021 Wire click-to-select (outline row, canvas block), hover cues where motion events are reported, and wheel scrolling in `crates/fireside-tui/src/editor/mod.rs` (depends on T018, T019)
- [ ] T022 Make `event_loop` callable from the editor module (visibility change only) in `crates/fireside-tui/src/lib.rs`, and wire `[ ▶ Present ]` to call it in-process against the already-initialized terminal with a no-op `ReloadSource`, an `Unavailable`-reporting write-back sink, and a no-op position sink, in `crates/fireside-tui/src/editor/mod.rs`, per `research.md` §6 (depends on T018)
- [ ] T023 TestBackend scenario tests for the read-only studio — open, select a slide, select a block, hover (where supported), scroll, present-and-return — driving both `KeyEvent` and synthetic `MouseEvent`s, in `crates/fireside-tui/src/editor/mod.rs` (depends on T019, T020, T021, T022)
- [ ] T024 CLI `edit` subcommand entry point in `crates/fireside-cli/src/edit.rs`: opening-rules chain (non-tty guard, unparseable-deck refusal with the "fix the file first" line, `.md` import hint, create-if-missing reusing `new.rs`/`templates.rs`, open-with-diagnostics-in-status-banner) per `contracts/cli-edit-command.md`, wired to the `Edit` command from T004 (depends on T004, T018)
- [ ] T025 [P] CLI e2e tests for `edit`'s opening rules (non-tty, unparseable, `.md` hint, create-if-missing, diagnostics-don't-block-open) in `crates/fireside-cli/tests/cli_e2e.rs` (depends on T024)
- [ ] T026 tmux smoke: open the editor, confirm the read-only studio renders, click a slide/block via injected mouse coordinates, present-and-return — proves the SGR mouse-injection technique for this feature's smoke suite — in `scripts/smoke.sh` (depends on T023, T024)

**Checkpoint**: Foundation ready — the editor opens, shows a deck read-only, and every user story below can now proceed.

---

## Phase 3: User Story 1 - Edit a slide's content without touching JSON (Priority: P1) 🎯 MVP

**Goal**: Select any block on a slide, edit it through a plain-language form, save, and undo — the smallest slice that proves the core promise.

**Independent Test**: Open a deck with a text and a heading block, edit each via its form, save, present to confirm the change, then undo and confirm the original wording returns.

- [ ] T027 [US1] Promote `EditableField` out of `crates/fireside-tui/src/app.rs` into `crates/fireside-tui/src/editor/forms.rs` (shared location both the presenter's quick-edit and the new editor forms depend on)
- [ ] T028 [US1] Implement the heading/text block edit form (reuses the promoted `EditableField`) in `crates/fireside-tui/src/editor/forms.rs` (depends on T027)
- [ ] T029 [US1] Implement the code block edit form (language picker + multiline source, Tab inserts spaces) in `crates/fireside-tui/src/editor/forms.rs` (depends on T027)
- [ ] T030 [US1] Implement the list block edit form (one item per line, blank lines dropped) in `crates/fireside-tui/src/editor/forms.rs` (depends on T027)
- [ ] T031 [US1] Implement the picture block edit form (path + description fields, placeholder-frame reminder, `[ Convert to text art ]` chip) in `crates/fireside-tui/src/editor/forms.rs` (depends on T027)
- [ ] T032 [US1] Implement the text-art block edit form (paste area + `[ Generate from a phrase… ]` CLI-injected callback + 76-column width check before accepting) in `crates/fireside-tui/src/editor/forms.rs` (depends on T027)
- [ ] T033 [US1] Implement the columns/box/stack (container) block edit form — layout picker + breadcrumb navigation into children — in `crates/fireside-tui/src/editor/forms.rs` (depends on T027)
- [ ] T034 [US1] Wire block selection to its contextual `[ ✎ Edit ]` action opening the block's form, and `[ Done ]`/`[ Cancel ]` committing via `EditBlock` or discarding, in `crates/fireside-tui/src/editor/mod.rs` (depends on T009, T028, T029, T030, T031, T032, T033)
- [ ] T035 [US1] Wire `[ Save ]`/Ctrl+S: atomic write via an injected closure in `crates/fireside-cli/src/edit.rs`, clearing the dirty indicator on success (depends on T024, T034)
- [ ] T036 [US1] Wire `[ ↶ Undo ]`/`u`/`U`: push a full-`Graph`-clone snapshot (with selection) onto `EditorApp::history` on every committed op, cap at 100, clear the redo stack on any new op, in `crates/fireside-tui/src/editor/mod.rs` (depends on T034)
- [ ] T037 [US1] Wire the dirty-state (`●`) indicator against `saved_graph` in `crates/fireside-tui/src/editor/mod.rs` (depends on T034)
- [ ] T038 [US1] TestBackend scenario tests: select → edit → save → undo for each of the 8 block kinds, driving both `KeyEvent` and synthetic `MouseEvent` paths, in `crates/fireside-tui/src/editor/mod.rs` (depends on T034, T035, T036, T037)
- [ ] T039 [US1] Snapshot vocabulary-gate test walking every editor `insta` snapshot fixture against the denylist regex (`\b(node|nodes|graph|traversal|kind|id)\b`, raw kind strings, quoted JSON keys), exempting the preview-fidelity fixture, in `crates/fireside-tui/src/render/tests.rs` (depends on T038)
- [ ] T040 [US1] Property test: the editor canvas's at-rest render buffer equals the presenter's render buffer for the same slide and size, across the fixture decks (spec SC-008), in `crates/fireside-tui/src/render/tests.rs` (depends on T013, T019)
- [ ] T041 [US1] tmux smoke: click a text block, edit via its form with the mouse, save, confirm the file changed on disk; repeat keyboard-only, in `scripts/smoke.sh` (depends on T038)

**Checkpoint**: US1 fully functional and independently testable — content editing without ever seeing JSON.

---

## Phase 4: User Story 2 - Add, remove, and rearrange blocks on a slide (Priority: P2)

**Goal**: Compose a slide's content by adding, deleting, and drag-reordering blocks.

**Independent Test**: Add a new block, delete an existing one, and drag the remaining blocks into a new order — mouse-only — then confirm the presented order/content and that every step is undoable.

- [ ] T042 [US2] Implement the add-block palette (8 plain-language cards, each inserting placeholder content and opening its form immediately) in `crates/fireside-tui/src/render/editor/forms.rs`, wired to `AddBlock` in `crates/fireside-tui/src/editor/mod.rs` (depends on T034)
- [ ] T043 [US2] Wire block delete (`[ Delete ]` chip) to `DeleteBlock` plus a non-blocking undo-toast ("Deleted — Undo") in `crates/fireside-tui/src/editor/mod.rs` (depends on T036)
- [ ] T044 [US2] Implement block drag-and-drop reorder: press-anywhere-on-the-block drag start, dimmed ghost, insertion-line indicator, auto-scroll near canvas edges, Esc cancels and returns the block, release commits `MoveBlock`, in `crates/fireside-tui/src/editor/mod.rs` and `crates/fireside-tui/src/render/editor/canvas.rs` (depends on T009, T015, T034)
- [ ] T045 [US2] Extend `hit()` with `InsertionSlot` targets and drag-target resolution, per `contracts/hit-testing.md`, in `crates/fireside-tui/src/editor/hit.rs` (depends on T015, T044)
- [ ] T046 [US2] Implement the empty-slide state (centered `＋ Add your first block` target) in `crates/fireside-tui/src/render/editor/canvas.rs` (depends on T042)
- [ ] T047 [US2] TestBackend scenario tests: add each block kind via the palette, delete + undo, drag-reorder via synthetic press/move/release `MouseEvent` sequences including Esc-cancel, in `crates/fireside-tui/src/editor/mod.rs` (depends on T042, T043, T044, T045, T046)
- [ ] T048 [US2] tmux smoke: drag-reorder two blocks via injected SGR mouse sequences, confirm the saved file reflects the new order, in `scripts/smoke.sh` (depends on T047)

**Checkpoint**: US1 + US2 both independently functional — full content composition on a slide.

---

## Phase 5: User Story 3 - Restructure the deck: slides, branches, and reveal steps (Priority: P3)

**Goal**: Manage deck structure — slides, wiring, branch points, reveal steps — through named pickers and drag, never typed identifiers.

**Independent Test**: From a 3-slide linear deck, turn one slide into a branch with two named answers, reorder the remaining linear slides by dragging, and set two-step reveal on one slide — all via named pickers/drag — then confirm it presents correctly.

- [ ] T049 [US3] Wire slide create (toolbar chip / outline `＋ new slide` row → title prompt → `AddSlide`), duplicate, and delete (heals wiring, undo-toast) in `crates/fireside-tui/src/editor/mod.rs` (depends on T008, T018)
- [ ] T050 [US3] Implement outline slide drag-reorder within a linear run, plus the refusal-with-explanation toast and "take me there" link when a drag crosses a branch boundary, in `crates/fireside-tui/src/editor/mod.rs` and `crates/fireside-tui/src/editor/hit.rs` (depends on T008, T045)
- [ ] T051 [US3] Implement the "Goes to" strip and its `[ change ]` slide picker (titles with a live mini-preview, plus "a new slide…" and "nothing — an ending"), wired to `SetNext`/`ClearNext`, in `crates/fireside-tui/src/render/editor/forms.rs` and `crates/fireside-tui/src/editor/mod.rs` (depends on T008, T019)
- [ ] T052 [US3] Implement the choice builder — `[ Turn into a choice ]` / `[ Turn back into a normal slide ]`, prompt field, answer rows (label/key/target via the slide picker), inline reserved-branch-key rejection — wired to `TurnIntoChoice`/`TurnBackIntoSlide`/`AddAnswer`/`RemoveAnswer`/`RetargetAnswer`, in `crates/fireside-tui/src/render/editor/forms.rs` and `crates/fireside-tui/src/editor/mod.rs` (depends on T008, T051)
- [ ] T053 [US3] Implement the reveal-step control — `[ Reveal ▾ ]` chip (none → 1 → … → none, auto-compacting), `◇n` badges (edit view only), `[ ▷ preview ]` stepping staging live — wired to `SetRevealStep`, in `crates/fireside-tui/src/editor/mod.rs` and `crates/fireside-tui/src/render/editor/canvas.rs` (depends on T009, T019)
- [ ] T054 [US3] Implement toolbar deck-title click-to-rename and a per-slide `[ Notes ]` chip for speaker notes, in `crates/fireside-tui/src/render/editor/mod.rs` and `crates/fireside-tui/src/editor/mod.rs`
- [ ] T055 [US3] Unit tests for `outline_order` against branch/cycle/unreachable fixtures, confirming the `not linked yet` divider and stable id ordering, in `crates/fireside-engine/src/authoring.rs` (depends on T007)
- [ ] T056 [US3] TestBackend scenario tests: create/duplicate/delete a slide, drag-reorder a linear run, cross-branch-boundary refusal, wire "goes to," build and dissolve a choice, set and preview reveal steps, in `crates/fireside-tui/src/editor/mod.rs` (depends on T049, T050, T051, T052, T053)
- [ ] T057 [US3] Flagship smoke: the two scripted 10-minute walkthroughs (spec SC-001/SC-002) — build a 5-slide deck with one choice and one reveal, present it, save it — once mouse-only (SGR-injected), once keyboard-only, in `scripts/smoke.sh` (depends on T056)

**Checkpoint**: US1 + US2 + US3 independently functional — the full authoring surface, both input modes proven sufficient.

---

## Phase 6: User Story 4 - Never lose work (Priority: P4)

**Goal**: Crash-safety and recovery around every story above — draft autosave/restore, safe quit, deep undo, save-conflict detection.

**Independent Test**: Make several unsaved edits, force-quit, reopen, confirm the draft-restore choice appears with correct timestamps; separately, make 20 sequential edits and confirm every one undoes in order.

- [ ] T058 [US4] Implement the quit-with-unsaved-changes prompt (`[ Save ] [ Discard ] [ Keep editing ]`) in `crates/fireside-tui/src/editor/mod.rs` (depends on T037)
- [ ] T059 [US4] Implement draft sidecar write/read/delete — `fnv1a64` + canonicalized-path keying reused from `crates/fireside-cli/src/session.rs`/`resume.rs`, atomic temp+rename — per `data-model.md`'s Draft sidecar schema, in `crates/fireside-cli/src/edit.rs` (depends on T024)
- [ ] T060 [US4] Wire periodic autosave-to-draft (on a tick while dirty, and on every structural op) in `crates/fireside-tui/src/editor/mod.rs` and `crates/fireside-cli/src/edit.rs` (depends on T059)
- [ ] T061 [US4] Implement the open-time draft-vs-saved-file conflict prompt (`[ Restore draft ] [ Open saved file ]`, both timestamps shown) in `crates/fireside-cli/src/edit.rs` (depends on T059)
- [ ] T062 [US4] Implement the external-change save-conflict guard (fingerprint check at save time, symmetric to quick-edit's existing conflict guard) in `crates/fireside-cli/src/edit.rs` (depends on T035)
- [ ] T063 [US4] Extend undo/redo tests to ≥100 sequential actions, each restoring the exact prior state, in `crates/fireside-engine/src/authoring.rs` and `crates/fireside-tui/src/editor/mod.rs` (depends on T036)
- [ ] T064 [US4] CLI e2e tests for the draft-restore prompt, the save-conflict guard, and atomic-write-survives-interruption, in `crates/fireside-cli/tests/cli_e2e.rs` (depends on T059, T060, T061, T062)
- [ ] T065 [US4] tmux smoke: make unsaved edits, force-kill the process, reopen, confirm the draft-restore prompt with correct timestamps; separately walk the quit-with-unsaved-changes prompt, in `scripts/smoke.sh` (depends on T064)

**Checkpoint**: All four user stories independently functional — the spec's full acceptance bar is achievable.

---

## Phase 7: Polish & Cross-Cutting Concerns (E4)

**Purpose**: Foolproofing polish that spans every story, plus the release-readiness gates every prior wave already had to clear.

- [ ] T066 [P] First-run hint tour (three rotating hint-line messages, dismissed forever after the first save) in `crates/fireside-tui/src/editor/mod.rs`
- [ ] T067 [P] Status-banner jump-to-diagnostic wiring (click a status-line issue → select the offending slide/block), sourced from the existing `fireside_engine::validation::rules()`, in `crates/fireside-tui/src/editor/mod.rs`
- [ ] T068 [P] Drag auto-scroll tuning pass (block drag and outline drag) in `crates/fireside-tui/src/editor/mod.rs`
- [ ] T069 [P] New guide `docs/src/content/docs/guides/editing.md` plus a VHS tape via `scripts/demos.sh`
- [ ] T070 [P] Update `docs/src/content/docs/guides/quickstart.md`, `README.md`, and `docs/src/content/docs/reference/cli.md` for the `edit` verb, plus the bare-invocation teaching line
- [ ] T071 Run `scripts/verify.sh` (full run, not `--skip-slow`) and fix any failures
- [ ] T072 Run `graphify update .`
- [ ] T073 Tick this feature's Progress Log entries in `.claude/plans/2026-07-19-wysiwyg-editor-plan.md` (E0–E4 lines) with completion dates, per this project's established per-wave convention

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — T001/T002 start immediately in parallel; T003 depends on T002; T004 is independent of T001–T003.
- **Foundational (Phase 2)**: Depends on Setup completing (the constitution amendment and `Edit` stub are prerequisites for the code this phase writes) — BLOCKS all user stories.
- **User Stories (Phase 3–6)**: All depend on Foundational (Phase 2) completing. US1 has no dependency on US2–US4. US2 depends on US1's block-selection/edit-form/undo plumbing (T034, T036) already existing — not independently startable before US1, unlike the template's default assumption. US3 depends on Foundational only (T008's slide ops, T019's rendering) and is otherwise independent of US1/US2, though in practice it is sequenced after them per the design brief's wave order. US4 depends on US1's dirty-state/save/undo plumbing (T035, T036, T037) already existing.
- **Polish (Phase 7)**: Depends on all four user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational — no dependency on other stories. This is the MVP slice.
- **US2 (P2)**: Builds directly on US1's selection/edit-form/save/undo machinery (block selection and `EditorApp::history` did not exist before US1) — sequenced after US1, not parallel to it.
- **US3 (P3)**: Structurally independent of US1/US2 (slide ops don't need block-edit forms), but sequenced after them per the design brief's wave order (E3 after E2) since it reuses `hit()` extensions US2 adds (T045).
- **US4 (P4)**: Wraps save/undo/dirty-state machinery US1 builds — sequenced last among the stories.

### Within Each User Story

- Engine ops before the TUI wiring that consumes them.
- Forms/state before the chip/toolbar wiring that opens them.
- Implementation before its TestBackend scenario tests before its tmux smoke.
- Story complete (checkpoint) before moving to the next priority.

### Parallel Opportunities

- T001/T002 (Setup) in parallel.
- T014 and T017 (Foundational) can run in parallel with each other and with the `authoring.rs` sequence (T005–T012), since they touch unrelated files.
- T025 (CLI e2e) can run in parallel with T023 (TUI TestBackend) once both their respective implementation tasks land — different crates, different files.
- Within Polish, T066–T070 are all independent files/concerns and can run fully in parallel; T071–T073 are sequential release gates that depend on everything before them.

---

## Parallel Example: Foundational Phase

```bash
# After T005 lands, these can proceed in parallel (different files):
Task: "Add insta snapshot pins of presenter render output (T014)"
Task: "Add affordance/selection/drop-target/ghost theme tokens (T017)"

# authoring.rs itself (T005-T012) is one file - sequential within it.
```

## Parallel Example: User Story 1

```bash
# After T027 (EditableField promotion) lands, the 6 remaining block-kind
# forms are logically independent but share crates/fireside-tui/src/editor/forms.rs -
# treat as sequential edits to one file, or split into separate files per
# kind at implementation time if true parallelism is wanted.
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 (Setup) and Phase 2 (Foundational) — the ADRs, the constitution amendment, `engine::authoring`, the `SlideView` seam, the `hit()` skeleton, and the read-only studio.
2. Complete Phase 3 (US1).
3. **STOP and VALIDATE**: run `quickstart.md`'s Layers 1–6 against US1's slice; confirm the independent test (select → edit → save → present → undo) end-to-end in a real terminal.
4. This is the first demonstrable "no-JSON content editing" milestone — matches the design brief's E0+E1(+part of E2) as the first PR-able unit.

### Incremental Delivery

1. Setup + Foundational → the editor exists and is explorable.
2. US1 → content editing works → demo-able MVP.
3. US2 → composition (add/delete/reorder blocks) → demo-able.
4. US3 → full structural authoring → the flagship 10-minute smoke test (T057) becomes meaningful and should pass here.
5. US4 → crash-safety net around everything above.
6. Polish → foolproofing, docs, and the full `scripts/verify.sh`/`graphify update .`/Progress-Log release gate.

Each step matches one of the design brief's five waves (E0–E4) and should clear that wave's own definition of done (`scripts/verify.sh` passes, the wave's tmux smoke ran in a real terminal, `graphify update .` ran, the Progress Log line is ticked) before moving to the next.

---

## Notes

- [P] tasks touch different files and have no dependency on an incomplete task; several tasks that are logically independent (e.g. the eight block-kind forms in US1) are still marked sequential because they share one file (`editor/forms.rs`) — split into per-kind files at implementation time if true parallel authorship is wanted.
- Every engine-layer task (`authoring.rs`) follows TDD: write the op, write its unit test, before moving to the next op group — the proptests (T011, T012) are the crown-jewel coverage per the design brief and must not be deferred to Polish.
- Commit after each task or logical group; stop at any checkpoint to validate a story independently before continuing.
- `scripts/verify.sh` mirrors every CI job; run it (at least with `--skip-slow` for the inner loop) before considering any single task's code "done," and the full version before ticking a wave's Progress Log line (T073).
