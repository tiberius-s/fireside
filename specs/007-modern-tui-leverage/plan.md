# Implementation Plan: Modern TUI Leverage

**Branch**: `007-modern-tui-leverage` | **Date**: 2026-07-17 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/007-modern-tui-leverage/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

The last item of the phase-1 strategic plan (P2 — Modern TUI leverage):
mouse support on the map/branch screens, resume-from-fingerprint, terminal
synchronized-output, and OSC 8 hyperlinks for link-bearing text. All four
use APIs already present in the workspace's existing `crossterm`/`ratatui`
dependencies — no new crates, no protocol/schema change. The one genuinely
open technical question (can ratatui render an OSC 8 span at all) was
resolved in Phase 0 research via `ratatui-core`'s documented
`CellDiffOption::ForcedWidth` mechanism, the same technique ratatui uses
internally for wide-character cells.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.88 (`resolver = "3"`, 2024 edition)

**Primary Dependencies**: `crossterm 0.29` and `ratatui 0.30` (both already
workspace dependencies) — mouse capture, `BeginSynchronizedUpdate`/
`EndSynchronizedUpdate`, and `CellDiffOption::ForcedWidth` are all already
available in the pinned versions; no version bumps required.

**Storage**: one new host-local file, `resume.json` (contracts/resume-state-format.md)
— not part of the portable deck format, not protocol-versioned.

**Testing**: `cargo test --workspace` (engine unit tests, `fireside-tui`
scenario suite against `TestBackend`, `fireside-cli` e2e), plus a
detached-tmux real-terminal smoke pass per Constitution Principle VII (see
quickstart.md's tmux section — mouse clicks and OSC 8 bytes both need a real
terminal, not just `TestBackend`, to verify).

**Target Platform**: any terminal `fireside` already supports; all four
capabilities are designed to degrade to current behavior on terminals that
lack them (no capability query needed — see research.md §3–4).

**Project Type**: single Rust workspace, 4 crates (existing layout below).

**Performance Goals**: no new performance targets; synchronized output must
not perceptibly slow transitions (still driven by the existing ~250ms/30ms
poll loop in `fireside-tui::event_loop`).

**Constraints**: no new dependencies (self-imposed, confirmed feasible in
research.md); no protocol/schema version bump; every new interaction must
degrade gracefully with no error/hang on incapable terminals (spec FR-016).

**Scale/Scope**: bounded to the four capabilities in spec.md's four user
stories; explicitly excludes kitty keyboard protocol, background images,
and heavy animation per the source plan's "not recommended now" list.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Result |
| --- | --- | --- |
| I. Spec Is the Source of Truth | No `protocol/main.tsp` field/schema change for any of the four capabilities (confirmed in research.md §1–4); only the non-normative `appendix-engine-extensions.md` gains one bullet for link syntax. `docs/examples/hello.json` unaffected. | PASS |
| II. Presenter-First Experience | Mouse is additive, footer stays the primary taught contract (FR-004); resume removes a real presenter pain point; scope stays within `present` (no new top-level product surface). | PASS |
| III. Crate Boundary Discipline | No new dependency in any crate. `fireside-tui` gains no file I/O (resume plumbed through a callback/initial-position parameter, mirroring `ReloadSource`/`WriteBackSink`); `fireside-cli` owns the new resume-state file I/O, already permitted. **Flagged**: resume-state path uses manual `std::env`/`std::path` construction rather than a `dirs`-style crate — see research.md §2 — an explicit, reviewable default, not a silent one. | PASS (with one flagged, non-blocking choice) |
| IV. Mandatory Code Idioms | New `Msg::Mouse` handled only inside `App::update`; hit-testing is a pure function called from both `render::draw` and the mouse handler (research.md §1) — rendering stays pure, `App::update` stays the sole mutator. No new `unwrap`/`expect` planned outside `main()`/tests. `#[must_use]`/doc comments apply to all new public items. | PASS |
| V. Stratified Error Handling | Resume-state file I/O lives in `fireside-cli` under `anyhow::Result` with context, matching existing file-op patterns in `main.rs`; no `anyhow` inside `fireside-tui`/`fireside-engine`/`fireside-core`. | PASS |
| VI. MSRV 1.88 | No new dependencies; `ForcedWidth`, mouse events, and synchronized-output commands are all already present in the currently-pinned `crossterm 0.29`/`ratatui 0.30` (via `ratatui-core 0.1.2`), which are already MSRV-1.88-compatible today. | PASS |
| VII. Test Discipline | Engine: resume fallback reuses already-unit-tested `Session::goto` guarded-no-op behavior (no new engine semantics). TUI: new scenario tests drive synthetic `MouseEvent`s through `App::update` against `TestBackend`, plus a scenario asserting a rendered link cell carries the expected `ForcedWidth`/OSC 8 content. CLI: e2e test for the resume-state round trip. All four capabilities additionally get a tmux real-terminal smoke pass (quickstart.md) — mouse and OSC 8 in particular cannot be verified by `TestBackend` alone. | PASS |

No violations requiring Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/007-modern-tui-leverage/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   ├── cli-flags.md
│   ├── resume-state-format.md
│   └── link-syntax.md
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

Existing 4-crate Rust workspace (no new crates); this feature touches all
four:

```text
crates/
├── fireside-core/          # pure model — untouched by this feature
├── fireside-engine/
│   └── src/validation.rs   # + malformed-link-url WARNING rule
├── fireside-tui/
│   └── src/
│       ├── app.rs          # + Msg::Mouse, size tracking, mouse click handlers
│       ├── lib.rs          # + resume plumbing params, sync-output bracketing,
│       │                   #   mouse capture enable/disable around the event loop
│       └── render/
│           ├── mod.rs      # + pure hit-test fns for map rows / branch options
│           └── markdown.rs # + `[label](url)` inline marker, ForcedWidth cells
└── fireside-cli/
    └── src/
        ├── main.rs         # + --restart flag, resume.json read/write, fingerprint reuse
        └── (resume state helpers, colocated with existing fingerprint() fn)

protocol/
├── validate.mjs            # + malformed-link-url WARNING rule (Node mirror)
└── fixtures/{valid,invalid}/*.json  # + link fixtures, fixtures.expected.json updated

docs/src/content/docs/spec/
└── appendix-engine-extensions.md    # + link-syntax bullet (non-normative)
```

**Structure Decision**: no structural change to the existing workspace —
this feature adds behavior inside all four existing crates plus the
existing Node validator mirror, following the crate boundary table exactly
as it stands today.

## Complexity Tracking

*No entries — Constitution Check reported no violations.*

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
