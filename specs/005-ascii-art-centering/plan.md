# Implementation Plan: ASCII art centering and clipping

**Branch**: `005-ascii-art-centering` | **Date**: 2026-07-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/005-ascii-art-centering/spec.md`

## Summary

Change `crates/fireside-tui/src/render/blocks.rs`'s `code()` function so
that code blocks classified as ASCII art (language absent, `"text"`, or
`"ascii"`) size their box to their content's natural width and center that
box within the available width, while explicit-language code blocks keep
today's full-width, left-aligned rendering unchanged. Reuses the existing
`clip`/`clip_spans` ellipsis helpers for the oversized case — no new
clipping logic. Engine-only, no protocol change (strategic plan's P1
"ASCII art, bounded to the window," layer 1 of 2).

## Technical Context

**Language/Version**: Rust 1.88 (2024 edition), `fireside-tui` crate.

**Primary Dependencies**: No new dependencies. Uses only what
`blocks.rs` already imports (`ratatui::text::{Line, Span}`,
`unicode_width::UnicodeWidthStr`).

**Storage**: N/A.

**Testing**: `cargo test -p fireside-tui` — new unit tests in
`blocks.rs`'s existing `#[cfg(test)] mod tests`, plus a new scenario test
in `fireside-tui/src/render/mod.rs`'s `TestBackend`-driven suite at
80×24 per constitution Test Discipline (every user-visible TUI state gets
scenario coverage) and per this feature's SC-001.

**Target Platform**: Same as the rest of `fireside-tui` — any terminal via
`ratatui`/`crossterm`.

**Project Type**: Existing single-crate change within the 4-crate
workspace; touches only `fireside-tui`.

**Performance Goals**: N/A — this is a pure layout computation over
already-bounded content (a single slide's blocks); no new complexity
class introduced.

**Constraints**: MUST NOT change rendering for any non-ASCII-art-classified
code block (FR-004, SC-002) — zero diff to existing code-block tests.
MUST reuse existing `clip`/`clip_spans` for overflow (FR-005) — no new
clipping logic. MUST compose correctly with the existing `container {
layout: "center" }` whole-unit-centering behavior (FR-007) without
altering the `centered_code_keeps_its_internal_alignment` test's outcome.

**Scale/Scope**: One function (`code()`) modified in one file; no new
public API surface (block classification stays an internal implementation
detail of `blocks.rs`); a handful of new unit tests plus one scenario test.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Spec Is the Source of Truth)**: PASS. No wire-format
  change — `CodeBlock.language` already exists; this feature only changes
  how the existing field's value is interpreted by the renderer. No ADR
  required (constitution: an ADR is required for changes that touch the
  wire format; this touches rendering only).
- **Principle II (Presenter-First Experience)**: PASS. Directly serves a
  presenter-visible quality gap (US1) with a graceful degradation path
  (US2) rather than a crash or silent truncation.
- **Principle III (Crate Boundary Discipline)**: PASS. No new
  dependencies; change is entirely inside `fireside-tui`, using only
  already-permitted deps.
- **Principle IV (Mandatory Code Idioms)**: PASS. No `unwrap()`/`expect()`
  introduced; styling continues to flow through `tokens` (`Tokens`), no
  raw `Style` construction; `render_block`'s pure-render contract is
  preserved (still `&[ContentBlock], u16, &Tokens -> Vec<Line>`, no state
  mutation — TEA invariant untouched since this is outside `App::update`).
- **Principle V (Stratified Error Handling)**: N/A — no new error paths;
  this is a pure layout computation with no fallible operations.
- **Principle VI (MSRV 1.88)**: PASS. No new dependency or API.
- **Principle VII (Test Discipline)**: PASS. New unit tests in
  `blocks.rs`; new scenario test in `render/mod.rs`'s `TestBackend` suite
  at 80×24 per SC-001. No live-reload/event-loop surface touched, so no
  tmux smoke test is required by the constitution's UI-change clause —
  this is pure/deterministic rendering, exactly the class of change
  `feedback_tmux_smoke_catches_timing_bugs` says TestBackend already
  covers well (the memory's caveat is specifically about event-loop
  ordering and file-watch timing, neither of which this feature touches).

**Result**: PASS, no violations. Complexity Tracking table not needed.

## Project Structure

### Documentation (this feature)

```text
specs/005-ascii-art-centering/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md         # Phase 1 output
├── quickstart.md         # Phase 1 output
├── contracts/            # Phase 1 output
└── tasks.md              # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/fireside-tui/src/render/
├── blocks.rs   # code() reworked to classify + size/center ASCII art;
│               # new unit tests in the existing #[cfg(test)] mod tests
└── mod.rs      # + one new scenario test at 80x24 in the TestBackend suite
```

**Structure Decision**: No new files, modules, or crates. Everything lives
inside the existing `blocks.rs` render module and the existing scenario
test suite in `render/mod.rs`.

## Complexity Tracking

*No Constitution Check violations — table not needed.*
