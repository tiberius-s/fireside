# Implementation Plan: Live Validation While Authoring (`validate --watch`)

**Branch**: `001-validate-watch` | **Date**: 2026-07-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-validate-watch/spec.md`

## Summary

Add a `--watch` flag to the existing `fireside validate` verb. On `--watch`,
the CLI checks the file immediately, then polls its mtime/size on a 250ms
cadence (matching `present`'s existing idle-poll rate) and re-checks on
every change, printing either a success confirmation or the full diagnostic
report — reusing the same caret-block parse-error rendering and diagnostic
formatting the codebase already has, extracted into a shared, non-exiting
helper so watch and one-shot `validate` render identically. No new crate
dependencies, no `fireside-tui` involvement, no protocol change.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.88, 2024 edition.

**Primary Dependencies**: none added — `clap`, `anyhow`, `serde_json`,
`fireside_core`, `fireside_engine` (all already `fireside-cli` dependencies).

**Storage**: N/A (reads one file from disk per poll).

**Testing**: `cargo test --workspace`; unit tests for the new pure
report-building function in `fireside-cli/src/main.rs`; one `cli_e2e.rs`
integration test for CLI wiring.

**Target Platform**: same as the existing `fireside` binary — no new
platform-specific behavior.

**Project Type**: CLI — single-crate change (`fireside-cli`).

**Performance Goals**: sub-second-to-a-few-seconds latency after a save
(250ms poll cadence, matching `present`'s existing idle poll).

**Constraints**: default (non-`--watch`) `validate` output MUST be
byte-for-byte unchanged (FR-002, SC-004); no new dependency; must not touch
`fireside-tui`.

**Scale/Scope**: one flag on one existing verb, one crate.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
|---|---|---|
| I. Spec Is the Source of Truth | No protocol/`main.tsp` change; nothing added that isn't already in the spec | PASS — this is authoring tooling, not a wire-format change |
| II. Presenter-First Experience | Every watch cycle gives clear feedback (success or diagnostics); first result shown immediately, no silent states | PASS — FR-003, FR-005, FR-008, FR-009 all require explicit feedback |
| III. Crate Boundary Discipline | Change stays inside `fireside-cli`; no new dependency; no `fireside-tui`/`ratatui`/`crossterm` coupling | PASS — see Research: own `std::thread::sleep` loop instead of borrowing the TUI's event loop |
| IV. Mandatory Code Idioms | No `unwrap()`/`expect()` outside `main()`/tests; matches existing `main.rs` style | PASS — new code follows the same `anyhow::Result` + `?` style already used in `main.rs` |
| V. Stratified Error Handling | CLI boundary uses `anyhow::Result` with context; no raw `Box<dyn Error>` | PASS — unchanged pattern from existing `validate_file`/`load` |
| VI. MSRV 1.88 | No new crate; no post-1.88 std API | PASS — `std::thread::sleep`, `std::time::Duration` are long-stable |
| VII. Test Discipline | Feature has unit and/or integration test coverage at the correct layer | PASS — see Research: pure function unit-tested, thin loop covered by one e2e wiring test |

No violations. Complexity Tracking is not needed.

## Project Structure

### Documentation (this feature)

```text
specs/001-validate-watch/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md         # Phase 1 output
├── quickstart.md         # Phase 1 output
├── contracts/
│   └── cli-validate-watch.md
└── tasks.md              # Phase 2 output (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
crates/fireside-cli/
├── src/
│   └── main.rs           # Command enum gains --watch on Validate; new
│                          # helper functions; existing validate_file's
│                          # diagnostic-rendering extracted for reuse
└── tests/
    └── cli_e2e.rs         # One new test for the --watch flag
```

**Structure Decision**: single-project CLI change, entirely within the
existing `fireside-cli` crate. No new files, modules, or crates — `main.rs`
gains a flag, a small polling loop, and a couple of extracted helper
functions alongside the existing `validate_file`, `load`, `parse_report`,
and `fingerprint`.

## Complexity Tracking

*No Constitution Check violations — this section is intentionally empty.*
