# Implementation Plan: Container Block Editing

**Branch**: `014-container-block-editing` | **Date**: 2026-07-23 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/014-container-block-editing/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

`fireside edit`'s block editor can select, edit, reorder, and delete
top-level blocks on a slide, but a `Container` block's own children
(columns/box/stack layout) are dead ends: `Tab` cycling and canvas
click hit-testing both stop at depth 1, and the container's edit form
shows a read-only summary of its children with no way to act on them.
Codebase inspection during planning confirms the engine layer
(`fireside-engine::authoring`) already fully supports nested
`BlockPath`s for every operation (`AddBlock`, `DeleteBlock`,
`EditBlock`, `MoveBlock`, `SetRevealStep` all recurse through
`Container::children` today), and `editor::forms::open` already builds
the correct per-kind form for *any* `BlockPath` regardless of depth.
The gap is entirely in `fireside-tui::editor`'s selection/hit-testing/
rendering layer, which was deliberately built depth-1-only in spec
013 (`hit.rs`'s own comment: "container children are reached through
the container form's breadcrumb, T033" — T033 never landed that
part). This feature closes that gap: extend `Tab`/`Shift+Tab`
cycling, canvas hit-testing, canvas selection rendering, and the
container form's child list to address and act on nested block paths,
with no protocol or engine change required.

## Technical Context

**Language/Version**: Rust 1.88 (2024 edition, `resolver = "3"`) — matches workspace MSRV, no change.

**Primary Dependencies**: `fireside-tui` only (`ratatui`, `crossterm`) for the UI-facing work; `fireside-engine::authoring` is consumed as-is (no changes expected — see Summary).

**Storage**: N/A — no on-disk format change; decks remain `*.fireside.json` via the existing `fireside-core::Graph`/`ContentBlock` model.

**Testing**: `cargo test --workspace` (unit tests in `fireside-tui/src/editor/{mod,hit,forms}.rs`), `fireside-tui/src/render/mod.rs` `TestBackend` scenario tests, `fireside-cli/tests/cli_e2e.rs` (unaffected unless a CLI-visible behavior changes), `scripts/smoke.sh` real-terminal tmux smoke test (mouse click/drag on a container child) — per Constitution Principle VII, all four layers apply since this changes user-visible TUI state and mouse-driven interaction.

**Target Platform**: Same as `fireside-tui` today — any terminal `crossterm` supports (macOS/Linux primary, per existing CI).

**Project Type**: Single Rust workspace, existing crates (`fireside-core`, `fireside-engine`, `fireside-tui`, `fireside-cli`) — no new crate.

**Performance Goals**: No new performance target; must stay within the editor's existing per-frame render budget (60fps-class TUI redraw, same as today — recursive block-extent computation is bounded by node content size, not a hot loop).

**Constraints**: Must not change the on-disk deck format or the `fireside-engine::authoring` public API's observable behavior (Constitution Principle I — the protocol/spec is unaffected by this feature; it is TUI-only). Must preserve the TEA invariant: `EditorApp::update` remains the sole mutator (Constitution Principle IV).

**Scale/Scope**: One level of container nesting (a container's direct children), per spec Assumptions. Bounded by the same node/slide sizes the editor already handles; no change to expected deck size.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Result |
| --- | --- | --- |
| I. Spec Is the Source of Truth | No protocol/wire-format change — `ContentBlock::Container` and its `children` already exist in `protocol/main.tsp`; this feature only changes how the TUI addresses/renders them. | PASS — no protocol change needed |
| II. Presenter-First Experience | Out of scope for the presenter itself (this is the *editor*, `fireside edit`); the editor's own "no JSON required" promise is what this feature restores for container slides, consistent with ADR-018's ownership of the editor. | PASS |
| III. Crate Boundary Discipline | All changes confined to `fireside-tui` (`editor/{mod,hit,forms}.rs`, `render/editor/*.rs`); `fireside-engine::authoring` is consumed, not modified, per the Summary's finding. No new dependency needed anywhere. | PASS |
| IV. Mandatory Code Idioms | `EditorApp::update` remains the sole mutator; nested selection/hit-testing follows the existing pull-based clamp-at-read pattern (`canvas_layout`, `outline_scroll_offset`) rather than introducing new mutable render-side state. New selection-glow styling reuses existing `Tokens` entries (`selection`); no raw `Style` construction. | PASS — verify during implementation that no new `unwrap()`/`expect()` is introduced outside tests/`main` |
| V. Stratified Error Handling | No new error surface expected — `authoring::apply` already returns `Result<Graph, AuthoringError>` for nested paths; the TUI layer's own hit-testing returns `Option<Target>`, unchanged shape. | PASS |
| VI. MSRV 1.88 | No new dependency or `std` API expected beyond what's already used in `editor/hit.rs` (`Vec`, slice indexing, recursion). | PASS |
| VII. Test Discipline | Plan commits to unit tests (`hit.rs`, `mod.rs`, `forms.rs`), a `TestBackend` scenario test for nested selection rendering, and a `scripts/smoke.sh` real-terminal case for click/drag on a container child, per the Testing field above. | PASS — enforced at task-generation time |

No violations; Complexity Tracking is not needed.

**Post-Phase-1 re-check**: research.md's four decisions and
data-model.md/contracts/nested-block-selection.md confirm the design
stays entirely inside `fireside-tui`, reuses existing `Tokens`/`Target`/
`BlockPath`/`FormChipKind` shapes, and introduces no new crate
dependency, error type, or protocol change. All gates above still PASS
unchanged after design.

## Project Structure

### Documentation (this feature)

```text
specs/014-container-block-editing/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/fireside-engine/
└── src/authoring.rs         # Consumed as-is; nested BlockPath already
                              # supported by every Op — no change expected,
                              # confirmed during Phase 0 research

crates/fireside-tui/
├── src/editor/
│   ├── mod.rs                # EditorApp::update, select_adjacent_block
│   │                          # (Tab/Shift+Tab cycling) extended to
│   │                          # descend into a selected container
│   ├── hit.rs                 # block_extents made recursive; canvas_hit /
│   │                          # resolve_drop_slot extended to resolve a
│   │                          # container child's own rendered range
│   └── forms.rs               # ChildSummary rows wired to a Target so
│                               # selecting one opens that child's form
├── src/render/editor/
│   ├── canvas.rs (or similar) # selection-glow drawing for a nested
│   │                          # block, reusing the same geometry hit.rs
│   │                          # now computes
│   └── forms.rs (or similar)  # container form's child list becomes
│                               # interactive rows, not plain text
└── src/render/mod.rs           # TestBackend scenario tests

crates/fireside-tui/src/editor/mod.rs (tests), hit.rs (tests), forms.rs (tests)
                                # unit tests per Constitution Principle VII

scripts/smoke.sh                # real-terminal tmux case: click/drag a
                                 # container child (Constitution Principle VII)
```

**Structure Decision**: Single Rust workspace, existing crate layout
(Option 1-equivalent, already in place). All work is contained inside
`fireside-tui`; exact file/function names for the render-side changes are
confirmed during Phase 1 design against the current file layout, since
`render/editor/` module names are implementation detail this plan does
not need to freeze early.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
