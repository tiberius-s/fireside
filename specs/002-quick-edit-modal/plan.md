# Implementation Plan: Quick-Edit Modal for Text and Heading Blocks

**Branch**: `002-quick-edit-modal` | **Date**: 2026-07-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-quick-edit-modal/spec.md`

## Summary

Add a quick-edit modal to `fireside-tui`, scoped by ADR-005
(`.claude/adrs/adr-005-quick-edit-modal-scope.md`) to content-only edits of
the current node's heading/text blocks (including nested inside
containers). The modal is a new `Screen::Edit` variant, edited with a
hand-rolled multi-line text buffer (no new crate dependency). On save, `App`
produces an edited `Graph` and hands it to a new `WriteBackSink` callback —
symmetric to the existing `ReloadSource` used for live reload — so
`fireside-tui` never performs file I/O itself. `fireside-cli` implements the
sink on top of the existing `Watcher`, adding one `write_back` method that
detects concurrent on-disk changes (reusing the existing mtime+size
fingerprint) before writing, and reuses the existing reload path to show the
saved result. No protocol/wire-format change.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.88, 2024 edition.

**Primary Dependencies**: none added — `ratatui`, `crossterm`,
`fireside_core`, `fireside_engine` (already `fireside-tui` dependencies);
`fireside_tui`, `anyhow`, `serde_json` (already `fireside-cli` dependencies).
No `tui-textarea` or similar — see `research.md` §1.

**Storage**: N/A for `fireside-tui` (no file I/O by design). One file
read/write in `fireside-cli`'s `Watcher::write_back`, reusing the existing
`fingerprint`/read/write helpers already in `main.rs`.

**Testing**: `cargo test --workspace`; `fireside-tui`'s `TestBackend`
scenario suite (`render/mod.rs`) for modal open/edit/save/cancel and the
"nothing to quick-edit" path; a focused unit test for
`Watcher::write_back`'s success/conflict/io-failure paths in
`fireside-cli/src/main.rs`; one `cli_e2e.rs` test wiring the whole
`present` → save flow is out of reach for e2e (needs a live terminal
session) — covered instead by the TUI scenario suite plus the `Watcher`
unit test, per `quickstart.md`.

**Target Platform**: same as the existing `fireside` binary — no new
platform-specific behavior.

**Project Type**: CLI + TUI — two-crate change (`fireside-tui`,
`fireside-cli`); `fireside-core` and `fireside-engine` are untouched.

**Performance Goals**: N/A beyond existing responsiveness — modal
open/edit/save is a synchronous, in-process, sub-millisecond operation
except for the one disk write on save, which is a single small JSON file.

**Constraints**: `fireside-tui` MUST NOT perform direct file I/O
(constitution §III); no protocol/wire-format change (ADR-005, spec
Assumptions); structural edits/undo/non-text blocks stay out of scope
(ADR-005) — this plan does not implement them even as unreachable dead
code.

**Scale/Scope**: one new `Screen` variant, one new render function, a
handful of new `App` methods/fields, one new public `fireside-tui` function
+ two new public types, one new `Watcher` method in `fireside-cli`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
|---|---|---|
| I. Spec Is the Source of Truth | No protocol/`main.tsp` change; nothing added that isn't already representable by existing `ContentBlock` fields | PASS — content-only edits of existing `text`/`heading` string fields, no new JSON shape (spec Assumptions) |
| II. Presenter-First Experience | Every keypress gets feedback (open/edit/save/cancel/conflict/unavailable/nothing-to-edit all have explicit UI responses) | PASS — FR-005, FR-011, FR-013, FR-014 all require explicit feedback, no silent no-ops |
| III. Crate Boundary Discipline | `fireside-tui` gains no new dependency and no file I/O; `fireside-cli` owns the only new I/O (`Watcher::write_back`) | PASS — see research.md §§1, 3; `WriteBackSink` is the same shape as `ReloadSource` |
| IV. Mandatory Code Idioms | No `unwrap()`/`expect()` outside `main()`/tests; TEA invariant (`App::update` sole mutation point) preserved; styling through `theme.rs::Tokens` | PASS — `Screen::Edit` follows the existing `Screen::Map`/`Help` pattern exactly; new render code uses `Tokens`, no raw `Style` construction |
| V. Stratified Error Handling | `fireside-tui` uses `TuiError`; `fireside-cli` uses `anyhow::Result` at the boundary | PASS — `WriteBackError` is a plain enum returned across the sink boundary (not `anyhow`, since it crosses into a library crate's public API), matching how `ReloadSource` already returns `Result<Graph, String>` across the same boundary |
| VI. MSRV 1.88 | No new crate; no post-1.88 std API | PASS — only `Vec`, `String`, existing `SystemTime` fingerprinting |
| VII. Test Discipline | Feature has unit and/or integration test coverage at the correct layer | PASS — see Testing above and `tasks.md` |

No violations. Complexity Tracking is not needed.

## Project Structure

### Documentation (this feature)

```text
specs/002-quick-edit-modal/
├── plan.md                          # This file
├── research.md                      # Phase 0 output
├── data-model.md                    # Phase 1 output
├── quickstart.md                    # Phase 1 output
├── contracts/
│   └── tui-authoring-api.md
└── tasks.md                         # Phase 2 output (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
crates/fireside-tui/
├── src/
│   ├── lib.rs        # + present_authoring, WriteBackSink, WriteBackError;
│   │                 #   present/present_watching become thin wrappers
│   ├── app.rs         # + Screen::Edit, EditableField, BlockPath, Msg::SaveResult,
│   │                 #   on_edit_key, save/cancel handling, take_pending_save
│   └── render/
│       └── mod.rs     # + draw_edit (modal popup), following draw_help/draw_notes
└── (scenario tests live inline in render/mod.rs's existing test module)

crates/fireside-cli/
└── src/
    └── main.rs         # Watcher gains write_back(); present() switches to
                        # present_authoring with a Watcher-backed sink;
                        # demo() unchanged (present() resolves to an
                        # Unavailable sink internally)
```

**Structure Decision**: two-crate change, no new files/modules — everything
lands inside the existing `fireside-tui` (`lib.rs`, `app.rs`,
`render/mod.rs`) and `fireside-cli` (`main.rs`) source layout, following the
same pattern the M3 live-reload work already established. `fireside-core`
and `fireside-engine` are untouched, matching ADR-005's "no wire-format
change" boundary.

## Complexity Tracking

*No Constitution Check violations — this section is intentionally empty.*
