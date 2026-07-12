# Tasks: Quick-Edit Modal for Text and Heading Blocks

**Input**: Design documents from `/specs/002-quick-edit-modal/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md,
contracts/tui-authoring-api.md, quickstart.md

**Tests**: included — the constitution's Test Discipline principle (VII)
requires tests at the correct layer for every feature. This feature's
layers are: `fireside-tui`'s `TestBackend` scenario suite
(`render/mod.rs`) for `App`/UI behavior, plain unit tests for
`Watcher::write_back` (`fireside-cli/src/main.rs`), and a manual
real-terminal smoke walk (constitution §VII, UI changes).

**Organization**: tasks are grouped by user story (spec.md priorities
P1/P2/P3). Most tasks touch one of three files
(`crates/fireside-tui/src/app.rs`, `crates/fireside-tui/src/lib.rs`,
`crates/fireside-tui/src/render/mod.rs`, `crates/fireside-cli/src/main.rs`);
`[P]` is reserved for tasks in a genuinely separate file with no dependency
on unfinished work in this feature.

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

- [X] T001 Run `cargo test --workspace` to confirm a clean baseline before
      touching `fireside-tui`/`fireside-cli`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: shared plumbing every user story needs — locating editable
blocks, the write-back contract, and the scaffolding needed for the
compiler to accept a new `Screen` variant before any story fills in real
behavior.

**⚠️ CRITICAL**: no user-story task can begin until this phase is complete.

- [X] T002 [P] In `crates/fireside-tui/src/app.rs`, add `pub(crate) struct
      BlockPath(Vec<usize>)`, `pub(crate) enum EditableKind { Heading(u8),
      Text }`, `pub(crate) struct EditableField { path: BlockPath, kind:
      EditableKind, buffer: Vec<String>, cursor: (usize, usize) }`, and
      `pub(crate) fn editable_fields(node: &fireside_core::Node) ->
      Vec<EditableField>` that walks `node.content` depth-first (recursing
      into `ContentBlock::Container`'s `children`) and collects one
      `EditableField` per `ContentBlock::Heading`/`ContentBlock::Text`
      found, in document order, per data-model.md's `BlockPath`/
      `EditableField` definitions
- [X] T003 [P] Add a unit test in `crates/fireside-tui/src/app.rs`'s
      `#[cfg(test)]` module: build a `Node` with a top-level heading, a
      `code` block (must be skipped), and a `container` whose children
      include a nested `text` block; assert `editable_fields` returns
      exactly two fields in document order with the correct `BlockPath`s
      (`[0]` and `[2, <index>]`)
- [X] T004 In `crates/fireside-tui/src/lib.rs`, add `pub enum
      WriteBackError { Unavailable, Conflict, Io(String) }` and `pub type
      WriteBackSink<'a> = &'a mut dyn FnMut(&Graph) ->
      Result<(), WriteBackError>` per contracts/tui-authoring-api.md; add
      `pub fn present_authoring(graph: Graph, source: ReloadSource<'_>,
      sink: WriteBackSink<'_>) -> Result<(), TuiError>`; redefine `present`
      and `present_watching` as thin wrappers calling `present_authoring`
      (the latter with a stub sink that always returns
      `WriteBackError::Unavailable`)
- [X] T005 In `crates/fireside-tui/src/app.rs`, add `Screen::Edit { fields:
      Vec<EditableField>, focused: usize }` to the `Screen` enum and
      `Msg::SaveResult(Result<(), String>)` to the `Msg` enum; add a
      `pending_save: Option<Graph>` field to `App` plus `pub(crate) fn
      take_pending_save(&mut self) -> Option<Graph>`; add the minimum
      match arms needed for the code to compile: `on_key` treats
      `Screen::Edit` as "Esc returns to `Screen::Present`, everything else
      is a no-op" (real editing lands in Phase 3), and
      `crates/fireside-tui/src/render/mod.rs`'s `draw` dispatch gets a
      `Screen::Edit { .. } => {}` stub arm
- [X] T006 [P] In `crates/fireside-cli/src/main.rs`, add `impl Watcher { fn
      write_back(&mut self, graph: &fireside_core::Graph) ->
      Result<(), fireside_tui::WriteBackError> }` per research.md §4:
      re-check `fingerprint(&self.path)` against `self.fingerprint` and
      return `Conflict` if it differs; otherwise serialize with
      `graph.to_json_pretty()`, write the file, and update
      `self.fingerprint` to the freshly written file's fingerprint,
      mapping any I/O error to `WriteBackError::Io`
- [X] T007 Add unit tests for `Watcher::write_back` in
      `crates/fireside-cli/src/main.rs`'s `#[cfg(test)]` module using a
      `tempfile` fixture: (a) a save with an unchanged on-disk file
      succeeds and the file's new contents parse back to an equal `Graph`;
      (b) a save after the file changed on disk (simulate by writing to
      the path directly, bypassing the `Watcher`) returns `Conflict`
      without touching the file; (c) a save to a path whose parent
      directory has been removed returns `Io`

**Checkpoint**: `cargo test --workspace` green, `cargo clippy --workspace
--all-targets` silent. `Screen::Edit` exists but is unreachable from
`Screen::Present` (no key opens it yet) — foundation compiles clean with
no dead behavior.

---

## Phase 3: User Story 1 - Fix a typo without leaving the presenter (Priority: P1) 🎯 MVP

**Goal**: a presenter can open the quick-edit modal on the current node,
edit a heading/text block's content, save, and see the change live on
screen and on disk — or cancel with no effect.

**Independent Test**: present a node with a heading and a text block, open
the modal, edit the heading, save, confirm the on-screen slide and the
deck file both show the new text; repeat and cancel instead, confirm
neither changed.

### Implementation for User Story 1

- [X] T008 [US1] In `crates/fireside-tui/src/app.rs`'s `on_present_key`, bind
      a key (`e`) that computes `editable_fields(self.session.current())`;
      if non-empty, sets `self.screen = Screen::Edit { fields, focused: 0
      }`; if empty, calls `self.set_flash("This slide has no editable
      text", FlashKind::Info)` and stays on `Screen::Present` (this guard
      is exercised by its own test in Phase 5, User Story 3)
- [X] T009 [US1] In `crates/fireside-tui/src/app.rs`, implement `fn
      on_edit_key(&mut self, code: KeyCode)` replacing the T005 stub:
      character insert, `Backspace`/`Delete`, `Enter` (newline) mutate the
      focused field's `buffer`/`cursor`; `Up`/`Down` move the cursor within
      a multi-line buffer or, at the first/last line, move `focused`
      between fields (wrapping is not required); `Esc` cancels (see T010);
      `Ctrl+S` saves (see T011); wire it from `on_key`'s `Screen::Edit { ..
      } => self.on_edit_key(key.code)` arm
- [X] T010 [US1] In `on_edit_key` (`crates/fireside-tui/src/app.rs`),
      implement cancel: on `Esc`, set `self.screen = Screen::Present`
      without touching `self.session` or `self.pending_save`
- [X] T011 [US1] In `on_edit_key` (`crates/fireside-tui/src/app.rs`),
      implement save: on `Ctrl+S`, clone `self.session.graph()`, find the
      current node by id, and for each field whose buffer changed from its
      initial value, walk its `BlockPath` to reach the corresponding
      `ContentBlock` and overwrite `Heading::text` or `Text::body` with
      `field.buffer.join("\n")`; store the resulting `Graph` in
      `self.pending_save`; set `self.screen = Screen::Present`
- [X] T012 [US1] In `crates/fireside-tui/src/app.rs`, implement handling for
      `Msg::SaveResult` in `App::update`: `Ok(())` → `set_flash("Saved",
      FlashKind::Info)`; `Err(message)` → `set_flash(&message,
      FlashKind::Error)` — reusing the existing flash mechanism so every
      outcome (success, conflict, unavailable, io failure) is visible
      per FR-005/FR-013/FR-014
- [X] T013 [US1] In `crates/fireside-tui/src/lib.rs`'s `event_loop`, replace
      the always-`None` pending-save poll with the real wiring: after
      `app.update(...)`, `if let Some(graph) = app.take_pending_save() {
      app.update(Msg::SaveResult(sink(&graph).map_err(|e| e.to_string())))
      }`, per contracts/tui-authoring-api.md
- [X] T014 [US1] In `crates/fireside-cli/src/main.rs`'s `present(path:
      &Path)`, switch from `fireside_tui::present_watching` to
      `fireside_tui::present_authoring`, passing `&mut |graph|
      watcher.write_back(graph)` as the sink
- [X] T015 [US1] In `crates/fireside-tui/src/render/mod.rs`, implement
      `fn draw_edit(frame: &mut Frame, area: Rect, fields: &[EditableField],
      focused: usize, tokens: &Tokens)` replacing the T005 stub: a centered
      bordered popup (reuse `overlay_rect` and the `draw_help`/`draw_notes`
      bordered-block style) listing each field's label (`"Heading
      (level N)"` / `"Text"`) with its buffer text and a visible cursor on
      the focused field; wire it into `draw`'s `Screen::Edit { fields,
      focused }` arm
- [X] T016 [US1] Update the footer hints in `draw_footer` and the key
      reference in `draw_help` (both `crates/fireside-tui/src/render/mod.rs`)
      to include the new `e` "quick-edit" key on the present screen, and
      add a small hint line inside `draw_edit` itself for `Ctrl+S save` /
      `Esc cancel`, per the constitution's "footer always shows exactly the
      valid keys" principle
- [X] T017 [US1] Add `TestBackend` scenario tests in
      `crates/fireside-tui/src/render/mod.rs`'s test module: (a) press `e`
      on a node with a heading and text block, edit the heading buffer via
      `on_edit_key`, press `Ctrl+S`, assert `take_pending_save()` returns a
      `Graph` whose node has the updated heading text and an unchanged text
      block; (b) repeat and press `Esc` instead, assert
      `take_pending_save()` returns `None` and the on-screen slide/session
      graph is unchanged

**Checkpoint**: User Story 1 is fully functional and independently testable
via `quickstart.md` scenarios 1–2.

---

## Phase 4: User Story 2 - Trust that only the intended text changed (Priority: P2)

**Goal**: confidence that a save never touches any node, block, or
traversal/branch structure other than the specific edited block(s).

**Independent Test**: on a multi-node branching deck, quick-edit and save
one text block on one node; confirm every other node's content and every
traversal/branch-point structure is semantically unchanged.

### Implementation for User Story 2

- [X] T018 [US2] Add a `TestBackend`/unit test in
      `crates/fireside-tui/src/app.rs`'s or `render/mod.rs`'s test module
      using a fixture with at least three nodes including one with a
      branch-point: quick-edit and save a text block on one non-branch
      node; assert the resulting `Graph`'s other two nodes (content,
      `traversal`, `branch-point`/`branch-options`) are `==` to the
      originals — proving T011's clone-and-patch-one-block approach never
      touches sibling nodes
- [X] T019 [US2] Add a unit test for `Watcher::write_back` in
      `crates/fireside-cli/src/main.rs` round-tripping a multi-node,
      multi-block deck fixture through a save (`to_json_pretty` → write →
      `Graph::from_json` the result): assert the re-parsed `Graph` is
      semantically equal to the original with one block's text changed,
      even though key order/whitespace may differ — locks in ADR-005's
      "reformat is fine, meaning isn't" guarantee (FR-007)

**Checkpoint**: User Story 2 verified; `quickstart.md` scenario 3 passes.

---

## Phase 5: User Story 3 - Know when there's nothing to quick-edit (Priority: P3)

**Goal**: opening the modal on a node with no heading/text content (even
nested in a container) gives clear feedback instead of a blank modal.

**Independent Test**: present a node whose content is only `code`/`image`
blocks, try to open the quick-edit modal, confirm a clear message appears
and no modal opens.

### Implementation for User Story 3

- [X] T020 [US3] Add a `TestBackend` scenario test in
      `crates/fireside-tui/src/render/mod.rs`'s test module: present a node
      whose content is a single `code` block (and, separately, a `container`
      whose only children are `image`/`divider` blocks), press `e`, assert
      `app.screen()` stays `Screen::Present` and `app.flash()` shows the
      "no editable text" message from T008 — this is a dedicated
      regression test for behavior T008 already implements

**Checkpoint**: all three user stories independently pass;
`quickstart.md` scenario 5 confirmed.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T021 [P] Add a unit test in `crates/fireside-tui/src/lib.rs` asserting
      that the stub sink used internally by `present_watching` (T004)
      returns `WriteBackError::Unavailable` when invoked directly — covers
      the `fireside demo` "no file to save to" path without needing a live
      terminal (`quickstart.md` scenario 4)
- [X] T022 Add a `TestBackend` test in `crates/fireside-tui/src/app.rs`'s or
      `render/mod.rs`'s test module: for each of
      `WriteBackError::{Unavailable, Conflict, Io("disk full")}` mapped to
      its `to_string()`/display message, feed
      `Msg::SaveResult(Err(message))` into `App::update` and assert
      `app.flash()` shows that exact message with `FlashKind::Error` —
      confirms FR-013/FR-014's feedback requirement at the `App` layer
- [X] T023 Run `cargo test --workspace` and
      `cargo clippy --workspace --all-targets` and fix any findings
- [X] T024 Manually walk through `quickstart.md` scenarios 1–6 in a real
      terminal (tmux smoke per the constitution's UI test-discipline
      requirement), including scenario 6's concurrent-edit conflict, which
      needs a second terminal/process writing to the same file mid-edit
- [X] T025 [P] Run `graphify update .` to refresh the knowledge graph after
      the code change, per the constitution's Operational Constraints
- [X] T026 [P] Update the Progress Log in
      `.claude/plans/2026-07-12-strategic-improvement-plan.md`, checking
      off "P0 Stage C — quick-edit modal in TUI" with the commit(s) and
      date

---

## Dependencies & Execution Order

- **Setup (Phase 1)**: no dependencies.
- **Foundational (Phase 2)**: depends on Setup; blocks every user story.
- **User Story 1 (Phase 3)**: depends on Foundational only. This is the
  MVP — it delivers the entire presenter-visible feature.
- **User Story 2 (Phase 4)**: depends on Foundational and on User Story 1's
  save implementation (T011) existing — it adds tests proving an invariant
  US1's implementation already establishes, so it follows US1 in practice.
- **User Story 3 (Phase 5)**: depends on Foundational and on User Story 1's
  T008 guard clause existing — it adds a dedicated regression test for
  behavior implemented as part of "open" in US1.
- **Polish (Phase 6)**: depends on all desired user stories being complete.

### Parallel Opportunities

- T002/T003 (fireside-tui `app.rs` BlockPath/EditableField + its test) and
  T006/T007 (fireside-cli `Watcher::write_back` + its tests) touch
  different crates and have no dependency on each other — safe to run in
  parallel during Phase 2.
- T021, T025, and T026 touch files untouched by the rest of Phase 6 — safe
  to run in parallel with each other once Phase 5 is done.
- All other tasks touch shared state within `app.rs`/`render/mod.rs`
  (`Screen`, `App`) or are sequential refinements of the same function, and
  are effectively sequential within their phase.

---

## Implementation Strategy

### MVP First (User Story 1 only)

1. Phase 1 (Setup) → Phase 2 (Foundational) → Phase 3 (User Story 1).
2. **Stop and validate**: run `quickstart.md` scenarios 1–2. This alone
   ships the core "fix a typo without leaving the presenter" loop that
   ADR-005 and the strategic plan's P0 Stage C exist to deliver.

### Incremental Delivery

1. Setup + Foundational → types and write-back plumbing compile clean,
   nothing user-visible yet.
2. + User Story 1 → the full quick-edit loop, open/edit/save/cancel, live
   on screen and on disk (MVP).
3. + User Story 2 → proven isolation from the rest of the deck.
4. + User Story 3 → clear feedback on nodes with nothing to edit.
5. + Polish → edge-case feedback tests (conflict/unavailable/io-failure),
   lint/test pass, manual quickstart walk including the concurrent-edit
   scenario, knowledge graph refresh, strategic-plan progress log update.
