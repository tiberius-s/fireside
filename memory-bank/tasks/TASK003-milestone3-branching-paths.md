# TASK003 - Milestone 3 branching paths

**Status:** In Progress
**Added:** 2026-02-14
**Updated:** 2026-02-19

## Original Request

Implement branching-path navigation as the primary differentiator.

## Thought Process

Branching requires model-level graph semantics and app-level navigation history.
Directive parsing support exists at a foundational level, so Milestone 3 should
focus on full navigation behavior and branch-selection UI.

## Implementation Plan

- Extend slide graph and metadata handling
- Finalize branch directive parsing semantics
- Implement branch selection mode in app state machine
- Add branch-selection UI and overview navigation
- Validate branch backtracking and rejoin behavior

## Progress Tracking

**Overall Status:** In Progress - 99%

### Subtasks

| ID  | Description                         | Status      | Updated    | Notes                                                        |
| --- | ----------------------------------- | ----------- | ---------- | ------------------------------------------------------------ |
| 3.1 | Extend graph model and indices      | Complete    | 2026-02-19 | Graph indices and traversal are active                       |
| 3.2 | Complete branch directive semantics | Complete    | 2026-02-19 | Branch directives are implemented                            |
| 3.3 | Implement branch navigation engine  | Complete    | 2026-02-19 | Choose/goto/back flows are active                            |
| 3.4 | Build branch selection UI           | Complete    | 2026-02-19 | Branch overlay and key selection ship                        |
| 3.5 | Add overview/jump workflow          | In Progress | 2026-02-19 | Topology connector alignment shipped; final visual tune left |

## Progress Log

### 2026-02-14

- Task created from roadmap and indexed as pending

### 2026-02-19

- Updated milestone to in-progress based on completed branch traversal and UI overlay work
- Confirmed branch choose/backtrack flows are covered by engine and smoke tests
- Remaining work is concentrated in deeper graph overview/navigation tooling

### 2026-02-19 (graph overlay slice)

- Added editor graph-view overlay toggle (`v`) with an ASCII node list and edge summaries (`next`, `after`, branch option keys).
- Wired graph-view keyboard navigation (`j/k` and arrows), Enter-to-jump, and close controls (`Esc`/`v`).
- Added graph-view mouse support for scroll navigation and click-to-jump behavior.
- Added focused tests for overlay toggle and keyboard jump flow, and validated with:
  - `cargo test -p fireside-tui graph_overlay --no-fail-fast`
  - `cargo clippy -p fireside-tui -- -D warnings`

### 2026-02-19 (edge-map polish slice)

- Upgraded graph overlay rendering to richer ASCII topology lines with boxed node labels and clearer edge separators.
- Added mini-map side panel showing total nodes, visible window, selected/current markers, and a compact vertical index strip.
- Added graph overlay viewport controls: `PgUp/PgDn` for page movement and `Home/End` for bounds jumps.
- Aligned graph-overlay mouse hit-testing with rendered list panel geometry to keep click-to-jump accurate.
- Added and passed focused tests for page and bounds navigation, validated with:
  - `cargo test -p fireside-tui graph_overlay --no-fail-fast`
  - `cargo clippy -p fireside-tui -- -D warnings`

### 2026-02-19 (fan-out rendering slice)

- Implemented multi-line branch fan-out edge rendering per node in graph overlay (one row for node, additional rows for `next`, `after`, and branch option edges).
- Added shared graph-overlay windowing helpers so rendering, paging, and mouse row-to-node hit-testing use the same variable-height item model.
- Updated graph overlay paging behavior (`PgUp/PgDn`) to use visible node span from the shared window model.
- Kept jump semantics intact: Enter and click still jump selected graph node into editor selection and traversal state.
- Re-validated this slice with:
  - `cargo test -p fireside-tui graph_overlay --no-fail-fast`
  - `cargo clippy -p fireside-tui -- -D warnings`

### 2026-02-19 (graph-to-presenter handoff slice)

- Added graph overlay shortcut `p`/`P` to jump to the selected node and switch directly into `Presenting` mode.
- Preserved selection/traversal consistency by applying selected graph node to both editor selection and traversal before mode switch.
- Updated graph overlay legend and in-app help to surface the new handoff shortcut.
- Added regression test for presenter handoff behavior and re-validated with:
  - `cargo test -p fireside-tui graph_overlay --no-fail-fast`
  - `cargo clippy -p fireside-tui -- -D warnings`

### 2026-02-19 (presenter-to-editor breadcrumb slice)

- Added reverse handoff breadcrumb when entering editor mode from presenter (`e`), preserving current node index and showing contextual status in editor footer.
- Confirmed this does not alter existing selection synchronization behavior (`editor_selected_node` still tracks current traversal index on entry).
- Added focused regression coverage:
  - `cargo test -p fireside-tui presenter_enter_edit_mode_sets_breadcrumb_status --no-fail-fast`
  - `cargo clippy -p fireside-tui -- -D warnings`

### 2026-02-19 (connector alignment polish slice)

- Refined graph fan-out rows to render explicit aligned connector glyphs (`├╼`/`└╼`) with padded edge-kind labels and aligned target labels for dense branch nodes.
- Kept variable-height graph row behavior and hit-testing intact while improving topology readability.
- Re-validated graph overlay interaction and rendering paths with:
  - `cargo test -p fireside-tui graph_overlay --no-fail-fast`
  - `cargo clippy -p fireside-tui -- -D warnings`
