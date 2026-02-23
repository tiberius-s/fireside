# TASK017 — TUI Second-Pass UX and Bug Fixes

**Status:** In Progress
**Added:** 2026-02-22
**Updated:** 2026-02-23

## Original Request

Ten UX issues were reported directly by the author after the TASK016 pass was considered
complete:

1. Editor text editing is rough; only text blocks have any editing, with no cursor
   movement support.
2. Cannot edit an existing content block — only append new ones.
3. Graph overview is basic — just node IDs and titles, no visual topology.
4. Mouse click during presentation advances two nodes instead of one.
5. Clicking a node in the editor does nothing.
6. When two nodes away from a branch point, presentation gets stuck in a loop.
7. Need an agent-driven testing strategy using expect-test / insta snapshots.
8. Editor layout confusing — metadata and content blocks in same pane without
   visual distinction.
9. Content blocks overall need more work — only text block has any editing.
10. Branch point overlay is a plain list; should be more visual and interactive.

## Thought Process

Four user jobs were identified from these ten issues:

- **Job A** (content editing) — maps to issues 1, 2, 8, 9
- **Job B** (navigation correctness) — maps to issues 4, 5, 6
- **Job C** (verification & testing) — maps to issues 7, 8 (validation side)
- **Job D** (graph topology visibility) — maps to issues 3, 10

### Four User Jobs (JTBD Analysis)

**Job A — "Get my content onto a node quickly and correctly"**
Maps to issues 1, 2, 8, 9. Pain: text editing rough, can't edit existing blocks, no validation.

**Job B — "Navigate a presentation with confidence"**
Maps to issues 4, 5, 6. Pain: mouse double-fire (confirmed bug), clicking doesn't work, loop before branch.

**Job C — "Verify the presentation works before sharing"**
Maps to issue 7 + validation. Pain: manual testing required, no automated traversal checks.

**Job D — "Understand graph structure at a glance"**
Maps to issues 3, 10. Pain: graph overlay shows linear list (not tree), branch overlay plain text (not interactive).

### Root Causes Confirmed

**B1 Double-fire (CONFIRMED)**: Both `MouseEventKind::Down` and `MouseEventKind::Up` dispatch `Action::MouseClick`
in `app.rs` lines 675–683. Every physical click fires twice. Fix: delete `Up` arm (4 lines).

**B3 Branch loop (NEEDS INVESTIGATION)**: Suspected traversal loop when one step before a branch node.
May be related to `next()` auto-selecting successor when `branch_target == current_index + 1`.

### Five User Journeys (Emotional Arcs)

| Journey                     | Start emotion | Peak pain                                | End emotion |
| --------------------------- | ------------- | ---------------------------------------- | ----------- |
| Author edits existing block | Confident     | No commit feedback, cursor at position 0 | Frustrated  |
| Presenter clicks slides     | Confident     | Double-jump, then stuck in loop          | Alarmed     |
| Verify traversal            | Reassured     | Manual multi-path re-test                | Frustrated  |
| Add code block              | Uncertain     | Formless edit widget                     | Anxious     |
| Explore 30-node graph       | Overwhelmed   | Monochrome list, no topology             | Frustrated  |

### Nine Implementation-Ready Flows (F1–F9)

All flows include exact code locations, acceptance criteria, and accessibility requirements.

**F1 — Fix mouse double-fire (P0, Trivial)**
Delete `MouseEventKind::Up` arm in `app.rs:681–684`.
Acceptance: One click = one node advance.

**F2 — Investigate branch loop (P0, Small)**
Add regression test in `fireside-engine/tests/traversal_tests.rs` for pre-branch `next()`.
Acceptance: Graph A → B → C(branch) → D plays A→B→C correctly without auto-looping.

**F3 — Inline edit existing blocks (P1, Large)**
Implement `i` on any block row opens edit widget. Per-block widgets for all 8 `ContentBlock` types.
Cursor at end of content. `Esc` commits via `Command::UpdateBlock`. `Ctrl+C` discards.
Acceptance: Edit heading/text/code blocks and commit changes.

**F4 — Reorder blocks with keyboard (P1, Small)**
`J/K` swaps blocks; selection follows. Fire `Command::MoveBlock`.
Acceptance: After `J` on block 1, it becomes block 2 with cursor following.

**F5 — Content block validation (P1, Small)**
Add `validate_content_block()` in engine. Run on `Esc` commit. Show amber flash + `⚠` chip for warnings.
Validation rules: heading/text not empty, code language not empty, etc.
Acceptance: Commit empty-language code block shows amber warning flash.

**F6 — AppHarness testing framework (P1, Medium)**
Add `crates/fireside-tui/tests/harness.rs` with `AppHarness` struct. Uses `ratatui::backend::TestBackend`.
Write golden test for full `hello.json` path and branch-choose interaction.
Acceptance: `AppHarness::for_graph` constructs without panic; `press(Action::Next)` advances and returns frame text.

**F7 — Block picker with preview (P2, Small)**
Show 1-line synopsis per block type in add-block picker (e.g., "Heading: A large title line").
Acceptance: Picker shows all 8 types with descriptions.

**F8 — Graph overlay ASCII tree (P2, Large)**
Replace linear list with indented tree layout. Branch arms fork with `┬`. Edge colours: blue=next, gold=branch, green=after, red=goto.
Current node highlighted with `border_active` box. Detect cycles to avoid infinite loops.
Acceptance: 5-node graph with one branch renders tree showing fork and coloured edges.

**F9 — Branch overlay affordance (P2, Small)**
Verify TASK016 Phase 4 is complete: arrow keys move focus, first option highlighted, row separators, `Enter` selects.
Acceptance: Opening branch overlay shows first option with `▌` left accent bar; arrow keys move highlight.

### Penpot Design Board Handoff

- **F3 (inline edit widgets)**: Create "08 — Block Edit Widgets" board with per-block form wireframes.
- **F7 (picker preview)**: Amend "05 — Editor Mode" board to show picker with synopsis rows.
- **F8 (graph tree)**: Create "09 — Graph Tree View" board showing ASCII tree with edge colour coding.
- **F9 (branch overlay)**: Verify "03 — Presenter + Branch" board matches F9 spec.

## Implementation Plan

### Phase A — P0 Bugs (must land first)

- [ ] **A1**: Remove `MouseEventKind::Up` arm in `app.rs` (1 line) → fixes double-fire
- [ ] **A2**: Add regression test in `fireside-engine/tests/traversal_tests.rs` for
      pre-branch-point next() → confirms or denies the loop hypothesis

### Phase B — Test Infrastructure (enables safe refactoring)

- [ ] **B1**: Add `AppHarness` in `crates/fireside-tui/tests/harness.rs`
- [ ] **B2**: Write golden test for `hello.json` full path
- [ ] **B3**: Write golden test for branch-choose interaction

### Phase C — Content Block Editing

- [ ] **C1**: Add `Command::UpdateBlock { index: usize, block: ContentBlock }` to engine
- [ ] **C2**: Implement `i` on any block row opens block-specific edit widget
      (heading, text, code, list, image, divider, extension)
- [ ] **C3**: Fix cursor placement — textarea opens at end of content
- [ ] **C4**: Add `J/K` block reorder with `Command::MoveBlock`
- [ ] **C5**: Add `validate_content_block` in engine validation module

### Phase D — Graph & Branch Visual Improvements

- [ ] **D1**: Verify TASK016 Phase 4 branch overlay focus and affordance is complete
- [ ] **D2**: Implement block type picker with 1-line synopsis
- [ ] **D3**: Replace graph overlay linear list with ASCII tree rendering

### Phase E — Penpot Design Boards (parallel with D)

- [ ] **E1**: Create "08 — Block Edit Widgets" board for per-block form wireframes
- [ ] **E2**: Amend "05 — Editor Mode" board to show picker with synopsis
- [ ] **E3**: Create "09 — Graph Tree View" board

## Progress Tracking

**Overall Status:** In Progress — 30% (Phases E+F complete; A–D not started)

### Subtasks

| ID  | Description                                     | Status      | Updated    | Notes                                          |
| --- | ----------------------------------------------- | ----------- | ---------- | ---------------------------------------------- |
| A1  | Remove MouseEventKind::Up arm — fix double-fire | Not Started | 2026-02-22 | 1-line fix confirmed                           |
| A2  | Regression test for pre-branch next() loop      | Not Started | 2026-02-22 | Needs investigation first                      |
| B1  | AppHarness test infrastructure                  | Not Started | 2026-02-22 | ratatui TestBackend                            |
| B2  | Golden test hello.json full path                | Not Started | 2026-02-22 | Depends on B1                                  |
| B3  | Golden test branch-choose                       | Not Started | 2026-02-22 | Depends on B1                                  |
| C1  | Command::UpdateBlock in engine                  | Not Started | 2026-02-22 |                                                |
| C2  | Per-block edit widgets (all 8 variants)         | Not Started | 2026-02-22 | Largest change                                 |
| C3  | Cursor at end of content on widget open         | Not Started | 2026-02-22 |                                                |
| C4  | J/K block reorder + Command::MoveBlock          | Not Started | 2026-02-22 |                                                |
| C5  | validate_content_block in engine                | Not Started | 2026-02-22 |                                                |
| D1  | Verify TASK016 Phase 4 complete                 | Not Started | 2026-02-22 |                                                |
| D2  | Block type picker with synopsis                 | Not Started | 2026-02-22 |                                                |
| D3  | Graph overlay ASCII tree                        | Not Started | 2026-02-22 | Largest visual change                          |
| E1  | Penpot board 08 block edit widgets              | Complete    | 2026-02-23 | All 8 block types; verified visible            |
| E2  | Amend Penpot board 05 picker                    | Complete    | 2026-02-23 | Picker already had synopsis rows; verified     |
| E3  | Penpot board 09 graph tree                      | Complete    | 2026-02-23 | ASCII tree + colour-coded edges; verified      |
| E4  | Amend board 03 branch focus state               | Complete    | 2026-02-23 | Row highlight + accent bar + nav annotation    |
| E5  | Library 06 UI Components                        | Complete    | 2026-02-23 | Buttons/badges/chips/inputs/progress/keybinds  |
| E6  | Library 07 Typography & Color                   | Complete    | 2026-02-23 | 8-step type scale + full palette with edge map |
| E7  | Board 10 GotoNode Input Mode                    | Complete    | 2026-02-23 | New exploration; previously undesigned mode    |
| E8  | Board 11 Undo/Redo Chip States                  | Complete    | 2026-02-23 | Implements UX Proposal 08 visually             |
| E9  | Library typographies (8 styles)                 | Complete    | 2026-02-23 | Display/H1/H2/H3/Body/Small/Caption/Code       |
| E10 | Token set fireside/core (22 tokens)             | Complete    | 2026-02-23 | 16 color + 8 font-size + 6 spacing             |
| E11 | Token Catalog font correction                   | Complete    | 2026-02-23 | "Roboto Mono" → "JetBrains Mono"               |
| F1  | Fix 8 typographies (all were 14px)              | Complete    | 2026-02-24 | Correct sizes: 48/32/24/20/16/14/12/14         |
| F2  | Fix color naming + surface value                | Complete    | 2026-02-24 | 6 renames, surface #282C34→#21252B, +2 colors  |
| F3  | Add accent-cyan + foreground tokens             | Complete    | 2026-02-24 | 36 total tokens                                |
| F4  | Delete duplicate empty board                    | Complete    | 2026-02-24 | Was 0-children copy of board 10                |
| F5  | Create 31 reusable components                   | Complete    | 2026-02-24 | 10 categories, all with flex layouts           |
| F6  | Fix component paths (tripled → single)          | Complete    | 2026-02-24 | path=Category, name=Variant                    |
| F7  | Visual verification (all 31 components)         | Complete    | 2026-02-24 | Every component exported and inspected         |
| F8  | Component Showcase board                        | Complete    | 2026-02-24 | All 31 organized by category with labels       |

## Progress Log

### 2026-02-24

Completed Phase F — comprehensive Penpot design system overhaul:

**Typography fixes (F1)**: All 8 library typographies were broken (every one had fontSize
"14"). Fixed to correct sizes: Display 48px/700, H1 32px/700, H2 24px/700, H3 20px/700,
Body 16px/400, Small 14px/400, Caption 12px/400, Code 14px/400 JetBrains Mono. Source Sans
Pro 600 weight unavailable; used 400/700 instead.

**Color fixes (F2)**: Renamed 6 colors for semantic consistency (`h1-primary` → `accent`,
`h2-success` → `success-heading`, `h3-warning` → `warning`, `toolbar-bg` → `surface-dark`,
`h4-accent` → `accent-purple`, `footer-text` → `muted`). Fixed `surface` value from
`#282C34` (same as background) to `#21252B`. Added `accent-cyan` (`#56B6C2`) and `foreground`
(#DCDFE4).

**Token additions (F3)**: Added 2 new color tokens (accent-cyan, foreground) bringing
total to 36. Note: existing token names/values are immutable via API (read-only), so old
misnamed tokens remain but are harmless.

**Duplicate board deletion (F4)**: Removed empty 0-children duplicate of board 10.

**Component library creation (F5-F7)**: Created 31 reusable components with flex layouts
across 10 categories: Button (Primary, Secondary, Danger, Disabled), Mode Badge
(Presenting, Editing, Goto Node, Branch), Status Chip (Saved, Unsaved, No File),
Keybinding Chip, Input (Default, Active, Error), Progress Bar, Block Type (Heading, Text,
Code, List, Image, Divider, Container, Extension), Content Block (Heading, Text, Code),
Footer Bar, Branch Option (Focused, Default A, Default B).

**Component path fix (F6)**: Discovered Penpot uses `comp.path` for category and
`comp.name` for variant — setting `comp.name = 'Cat / Var'` concatenates to path on each
rename. Fixed all 28 affected components from tripled paths to clean single-category paths.

**Component Showcase board (F8)**: Created dedicated board at y=7000 organizing all 31
components by category with blue labels, divider lines, variant instances, and sub-labels.

**API learnings**: Token names/values are immutable once created. Component `name` with `/`
splits into `path` + `name`. `width`/`height` are read-only (use `resize()`).
`parentX`/`parentY` are read-only (use `penpotUtils.setParentXY()`). Source Sans Pro lacks
600 weight in Penpot.

### 2026-02-23

Completed Phase E (E1–E11): 4 new exploration boards (08 Block Edit Widgets, 09 Graph
Tree View, 10 GotoNode Input, 11 Undo/Redo Chip States), 2 library boards (06 UI
Components, 07 Typography & Color); fixed Board 03 branch overlay visibility; added 8
typography styles and 22 design tokens; corrected Token Catalog font to JetBrains Mono.
**Key positioning fix**: call `board.insertChild(n, s)` first, then set `s.x/s.y`.

### 2026-02-22

- Reviewed all ten reported issues against the TASK016 implementation.
- Confirmed mouse double-fire bug root cause in `app.rs` lines 675–683 (both Down and Up
  dispatch `Action::MouseClick`).
- Created three UX research artifacts in `docs/ux/`:
  - `tui-second-pass-jtbd.md` — four-job JTBD analysis with priority matrix
  - `tui-second-pass-journey.md` — five user journey maps with emotional arc summary
  - `tui-second-pass-flow.md` — nine implementation-ready flow specifications
- Identified nine flows (F1–F9) ready for a coding agent to implement.
- Prepared Penpot handoff notes (new boards 08, 09; amendments to 03 and 05).
