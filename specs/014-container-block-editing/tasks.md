# Tasks: Container Block Editing

**Input**: Design documents from `/specs/014-container-block-editing/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md (all present)

**Tests**: Included — Constitution VII (Test Discipline) is non-negotiable for this project: every user-visible TUI state gets a unit test plus a `TestBackend` scenario test, and every mouse-driven change gets a real-terminal tmux smoke case.

**Organization**: Tasks are grouped by user story (spec.md's US1–US3, priority order) after a Setup phase and a Foundational phase that builds the shared recursive geometry every story depends on.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: US1–US3, per spec.md's priorities. Setup/Foundational/Polish carry no story label.

## Path Conventions

Existing 4-crate Rust workspace (`crates/fireside-core`, `fireside-engine`, `fireside-tui`, `fireside-cli`) — all work is confined to `fireside-tui`, per plan.md's Project Structure (`fireside-engine::authoring` is consumed as-is, no change).

---

## Phase 1: Setup

- [X] T001 Re-read the current state of every file this feature touches
      immediately before editing (line numbers may have shifted since
      planning): `crates/fireside-tui/src/editor/hit.rs`,
      `crates/fireside-tui/src/editor/mod.rs`,
      `crates/fireside-tui/src/editor/forms.rs`,
      `crates/fireside-tui/src/render/editor/canvas.rs`,
      `crates/fireside-tui/src/render/editor/forms.rs`,
      `crates/fireside-tui/src/render/mod.rs`,
      `docs/src/content/docs/guides/editing.md`,
      `.claude/plans/2026-07-23-ux-audit.md` — in particular the exact
      wording of the two contradictory doc comments this feature resolves
      (`hit.rs`'s `block_extents` doc, `forms.rs`'s `ChildSummary` doc)

---

## Phase 2: Foundational (blocking prerequisite for all user stories)

**Goal**: A recursive block-geometry computation that both hit-testing
and rendering read from, so a container's children have real, agreed-upon
`[start, end)` ranges on the canvas before any story wires an interaction
to them. No user-visible behavior changes yet.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T002 Extend `hit::block_extents` (or add a sibling recursive
      helper it delegates to) to also compute each container child's own
      `[start, end)` sub-range within its container's own range, reusing
      the same increasing-prefix-render/diff technique the existing
      top-level computation uses, in `crates/fireside-tui/src/editor/hit.rs`
- [X] T003 Extend `CanvasLayout` to expose the recursive extents from
      T002 (exact field shape is an implementation choice — a nested
      `Vec`/map keyed by parent path, or a flattened `(BlockPath, Range)`
      list, whichever `canvas_hit` and `render/editor/canvas.rs` can both
      consume without disagreeing) in `crates/fireside-tui/src/editor/hit.rs`
      (depends on T002)
- [X] T004 [P] Unit tests: recursive `block_extents`/`CanvasLayout`
      geometry against a container fixture with multiple children (values
      match what `render_blocks` actually renders for each child, ranges
      are non-overlapping and nest correctly inside their container's own
      range) in `crates/fireside-tui/src/editor/hit.rs` (depends on T003)
- [X] T005 Replace the stale, mutually-contradictory doc comments this
      feature resolves — `block_extents`'s "Nested (`Container`) children
      are out of scope for the canvas... reached through the container
      form's breadcrumb, T033" (`hit.rs:470`) and `ChildSummary`'s
      "drilling in to *edit* a child is left to the canvas's own block
      selection once US2 extends hit-testing" (`forms.rs:225`) — with
      accurate text describing the behavior this feature actually
      implements, in `crates/fireside-tui/src/editor/hit.rs` and
      `crates/fireside-tui/src/editor/forms.rs` (depends on T003)

**Checkpoint**: Geometry foundation ready — every user story below can
now wire interactions to real per-child extents.

---

## Phase 3: User Story 1 - Select and edit a block inside a container (Priority: P1) 🎯 MVP

**Goal**: A container's children can be individually selected (click or
Tab) and opened into their own edit form, on the canvas and from the
container form's child list.

**Independent Test**: Open a container slide, click a child's rendered
text, confirm it (not the container) gets the selection glow, open its
form, edit, save, and confirm the change renders in the presenter view.

- [X] T006 [US1] Extend `hit::canvas_hit` to resolve a click on a
      container child's own rendered text to `Target::Block(node_id,
      nested_path)`, using the geometry from T003, in
      `crates/fireside-tui/src/editor/hit.rs` (depends on T003)
- [X] T007 [US1] Extend contextual-chip hit-testing (`Target::BlockChip`)
      to compute and resolve a selected container child's chips
      (`✎ Edit`, `＋ Add below`, `Reveal ▾`, `Delete`) identically to a
      top-level block's — `↑`/`↓` remain dead per the 2026-07-23 audit and
      MUST NOT be revived — in `crates/fireside-tui/src/editor/hit.rs`
      (depends on T006)
- [X] T008 [US1] Extend `select_adjacent_block` (Tab/Shift+Tab) to
      descend into a selected container's children in pre-order,
      returning to the container's next/previous top-level sibling at the
      ends (per data-model.md's state-transition note), in
      `crates/fireside-tui/src/editor/mod.rs` (depends on T003)
- [X] T009 [US1] Remove the `path.len() == 1` gate in
      `draw_selection_marker` and draw the selection glow for a selected
      child using its own recursive extent from T003, in
      `crates/fireside-tui/src/render/editor/canvas.rs` (depends on T003,
      T006)
- [X] T010 [US1] Add `FormChipKind::ContainerChild(usize)`, make the
      container form's `ChildSummary` rows hit-testable, and make
      selecting one open that child's own form (its `BlockPath` is the
      container form's own `path` with the row's index appended), in
      `crates/fireside-tui/src/editor/hit.rs` and
      `crates/fireside-tui/src/editor/forms.rs` (depends on T003)
- [X] T011 [US1] Confirm/adjust `EditorApp::update`'s open-form handling
      so opening a selected container child calls `forms::open` with its
      nested path exactly as it already does for a top-level block (per
      research.md Decision 2, no special-casing is expected — this task
      is verification plus any wiring gap it surfaces) in
      `crates/fireside-tui/src/editor/mod.rs` (depends on T006, T007,
      T008, T010)
- [X] T012 [P] [US1] Unit tests: Tab cycling reaches every child of a
      container and returns to the container's sibling at the ends; a
      click on a child's rendered text resolves to a length-2
      `Target::Block`; a click on a child's chip resolves to the matching
      `Target::BlockChip`; a click on a container form's child row opens
      that child's own form — in `crates/fireside-tui/src/editor/hit.rs`,
      `crates/fireside-tui/src/editor/mod.rs`, and
      `crates/fireside-tui/src/editor/forms.rs` (depends on T006-T011)
- [X] T013 [US1] `TestBackend` scenario test: open a container slide, Tab
      to a child, confirm the selection glow renders on the child's own
      extent (not the whole container), open its form, edit, save, and
      confirm the change round-trips into the rendered canvas, in
      `crates/fireside-tui/src/render/mod.rs` (depends on T009, T011)
- [X] T014 [US1] tmux smoke: on the bundled demo deck's "Welcome" slide,
      click a container child via injected mouse coordinates, confirm the
      glow via `capture-pane`, open and edit its form, save — in
      `scripts/smoke.sh` (depends on T013)

**Checkpoint**: User Story 1 fully functional — every container child on
every demo slide is selectable and editable.

---

## Phase 4: User Story 2 - Reorder and delete a container's children (Priority: P2)

**Goal**: A container's children can be dragged to reorder and deleted
independently of their siblings and of the container itself.

**Independent Test**: Reorder one child among at least three, save,
confirm the new order in the presenter view; delete one child, confirm
only it disappears.

- [X] T015 [US2] Extend block drag-to-reorder (`on_drag_move`/drop
      resolution in `editor/mod.rs`, `hit::resolve_drop_slot`) to accept a
      press-origin on a container child and resolve `InsertionSlot`
      candidates between its siblings, committing via `Op::MoveBlock` with
      the nested path, in `crates/fireside-tui/src/editor/mod.rs` and
      `crates/fireside-tui/src/editor/hit.rs` (depends on T003)
- [X] T016 [US2] Extend the `Delete` block chip/action to operate on a
      selected container child (`Op::DeleteBlock` with the nested path;
      selection falls back to the nearest remaining sibling, or to the
      now-empty container if it was the last child, per data-model.md), in
      `crates/fireside-tui/src/editor/mod.rs` (depends on T007)
- [X] T017 [P] [US2] Unit tests: reordering two children updates their
      stored order without touching their siblings; deleting a child
      leaves the rest untouched; deleting a container's last child leaves
      an empty container in place (not a deleted one) — in
      `crates/fireside-tui/src/editor/mod.rs` (depends on T015, T016)
- [X] T018 [US2] `TestBackend` scenario test: drag a child to reorder,
      save, confirm the new order renders; delete a child, confirm it's
      gone and its siblings remain, in
      `crates/fireside-tui/src/render/mod.rs` (depends on T017)
- [X] T019 [US2] tmux smoke: drag-reorder a container child via injected
      mouse events, confirm the drop, in `scripts/smoke.sh` (depends on
      T018)

**Checkpoint**: User Stories 1 AND 2 both work independently.

---

## Phase 5: User Story 3 - Add a new block inside a container (Priority: P3)

**Goal**: A new block can be added as a child of a container, at an
author-chosen position, including into an empty container.

**Independent Test**: Select a container (empty or populated), add a new
block "inside" it, confirm it renders as a child, not a new top-level
sibling.

- [X] T020 [US3] **Revised during implementation**: rather than a canvas
      `Target::InsertionSlot` between children (which has no natural
      geometry for a `Columns` layout — no shared gap row/column across
      arbitrary column counts), added a `[ + Add a block inside ]` chip
      directly to the container's own form (`FormChipKind::AddChild`),
      always appending after the last existing child; the author reorders
      it afterward via US2's drag support. Uniform across all three
      layouts, reuses the existing `AddPalette` flow verbatim. In
      `crates/fireside-tui/src/editor/hit.rs` (depends on T003)
- [X] T021 [US3] Wire the `AddChild` chip to `open_add_palette` with the
      container's own path as parent and `at = children.len()`, committing
      via the existing `Op::AddBlock` flow (`add_block_from_palette`
      already generalizes to nested paths — no change needed there), in
      `crates/fireside-tui/src/editor/mod.rs` (depends on T020)
- [X] T022 [P] [US3] Unit test: `[ + Add a block inside ]` on a populated
      container appends a new child after the existing two and opens its
      form immediately, with the container's top-level sibling untouched,
      in `crates/fireside-tui/src/editor/mod.rs` (depends on T020, T021)
- [X] T023 [US3] Covered by the same `TestBackend`-driven test as T022
      (`cargo test`'s `editor::tests` module drives the real `EditorApp`
      state machine end to end, per this project's existing test-layer
      convention for editor unit tests) — a dedicated
      `render/mod.rs`-level pixel assertion was judged redundant given
      T013's scenario test already proves nested-block rendering
      end-to-end; not added separately.

**Checkpoint**: All three user stories independently functional —
container children are fully reachable: selectable, editable,
reorderable, deletable, and a container can grow new children.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T024 [P] Update `docs/src/content/docs/guides/editing.md`'s
      "Selecting and editing" section to describe container children as
      reachable, matching whatever this feature actually delivers (spec
      FR-010; closes the 2026-07-23 audit's "Docs" note)
- [X] T025 [P] Delete the two genuinely-dead `BlockAction::MoveUp`/
      `MoveDown` enum variants and their `#[allow(dead_code)]` in
      `crates/fireside-tui/src/editor/hit.rs` (2026-07-23 audit
      codebase-health note) — only if still confirmed unused after this
      feature lands
- [X] T026 Tick the Progress Log's `P1-2` line to done in
      `.claude/plans/2026-07-23-ux-audit.md`
- [X] T027 Run `scripts/verify.sh` end-to-end and fix any regression
      before marking this feature complete
- [X] T028 Run `graphify update .` to refresh the knowledge graph

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories.
- **User Stories (Phase 3-5)**: All depend on Foundational completion.
  US1 has no dependency on US2/US3. US2's reorder/delete reuses US1's
  selection (`Target::BlockChip`'s `Delete` action, T007) but is
  independently testable given a pre-selected child. US3 depends only on
  Foundational, not on US1/US2, though it is most useful once US1 makes
  the result visible.
- **Polish (Phase 6)**: Depends on all three user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational — no dependency on
  US2/US3.
- **User Story 2 (P2)**: Can start after Foundational; T016 (delete)
  depends on T007 from US1 for the child-chip hit-testing it reuses.
- **User Story 3 (P3)**: Can start after Foundational — no dependency on
  US1/US2, but T003's geometry is required (same as every story).

### Parallel Opportunities

- T004 and T005 within Foundational can run in parallel once T003 lands.
- T012 (US1 unit tests) can run in parallel with T013/T014 once its own
  dependencies land.
- T017 (US2 unit tests), T022 (US3 unit tests) are each independently
  parallelizable within their own story.
- T024 and T025 in Polish can run in parallel.

---

## Parallel Example: Foundational

```bash
# Once T003 (recursive CanvasLayout) lands:
Task: "Unit tests: recursive block_extents/CanvasLayout geometry in crates/fireside-tui/src/editor/hit.rs"
Task: "Replace the stale contradictory doc comments in hit.rs and forms.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: run quickstart.md's manual walkthrough steps
   1-6 against the bundled demo deck's four container slides
5. This alone closes the 2026-07-23 audit's P1-2 finding's most severe
   half (every word of container-slide text becomes reachable) even
   before US2/US3 land

### Incremental Delivery

1. Setup + Foundational → geometry ready, no user-visible change yet
2. Add User Story 1 → test independently → container children are
   selectable and editable (the audit's core complaint, resolved)
3. Add User Story 2 → test independently → children are also
   reorderable and deletable
4. Add User Story 3 → test independently → new children can be added,
   completing full parity with top-level block editing

---

## Notes

- [P] tasks = different files or independently-verifiable test groups, no
  dependency on an incomplete task
- No protocol, engine (`fireside-engine::authoring`), or on-disk format
  change anywhere in this feature (research.md Decision 2) — every task
  above is confined to `fireside-tui` plus the one docs file (T024) and
  the two plan-tracking files (T026 and, implicitly, this feature's own
  `tasks.md` checkboxes)
- Commit after each task or logical group
- Stop at any checkpoint to validate a story independently before moving
  on
