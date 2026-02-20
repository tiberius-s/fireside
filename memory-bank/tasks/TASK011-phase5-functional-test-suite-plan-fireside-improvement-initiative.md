# TASK011 - Phase 5 functional test suite plan-fireside-improvement-initiative

**Status:** Completed
**Added:** 2026-02-19
**Updated:** 2026-02-19

## Original Request

Proceed with Phase 5 from `.github/prompts/plan-fireside-improvement-initiative.prompt.md`:
expand functional coverage across `fireside-core`, `fireside-engine`,
`fireside-cli`, and `fireside-tui`, then verify workspace quality gates.

## Thought Process

Phase 5 is coverage-oriented and spans all crates, so test placement and
assertion style must mirror existing crate conventions.

Implementation sequence used:

1. Add protocol round-trip coverage in `fireside-core` first.
2. Add deterministic fixture and invariant tests in `fireside-engine`.
3. Add binary-level command assertions in `fireside-cli` e2e tests.
4. Extend existing TUI smoke tests with a low-friction render invariant.
5. Run focused test batches, fix failures, then run full workspace gates.

## Implementation Plan

- Add `crates/fireside-core/tests/content_roundtrip.rs` with one test per
  `ContentBlock` variant plus edge cases.
- Add engine fixtures under `crates/fireside-engine/tests/fixtures/`.
- Add `crates/fireside-engine/tests/validation_fixtures.rs` and
  `crates/fireside-engine/tests/command_history.rs`.
- Add CLI e2e tests in `crates/fireside-cli/tests/cli_e2e.rs` and test-only
  dependencies in `crates/fireside-cli/Cargo.toml`.
- Extend `crates/fireside-tui/tests/hello_smoke.rs` with per-node non-empty
  render assertions.
- Run workspace verification commands.

## Progress Tracking

**Overall Status:** Completed - 100%

### Subtasks

- **11.1** Add core content round-trip suite — **Complete** (2026-02-19)
  Added `content_roundtrip.rs` with tests for heading, text, code, list,
  image, divider, container, and extension blocks.
- **11.2** Add engine fixture set — **Complete** (2026-02-19)
  Added `valid_linear.json`, `valid_branching.json`, `invalid_dangling_ref.json`,
  `invalid_empty.json`, and `invalid_duplicate_id.json`.
- **11.3** Add engine validation fixture tests — **Complete** (2026-02-19)
  Added `validation_fixtures.rs` for valid/invalid fixture assertions.
- **11.4** Add engine command-history invariant test — **Complete** (2026-02-19)
  Added `command_history.rs` ensuring add/update/remove + undo restores
  original node sequence.
- **11.5** Add CLI e2e tests and deps — **Complete** (2026-02-19)
  Added `cli_e2e.rs` and `assert_cmd`/`predicates`/`tempfile` dev dependencies.
- **11.6** Extend TUI smoke render coverage — **Complete** (2026-02-19)
  Added hello-smoke assertion that each node renders non-empty content lines.
- **11.7** Run verification gates — **Complete** (2026-02-19)
  `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` passed.

## Progress Log

### 2026-02-19

- Added integration tests in `fireside-core/tests/content_roundtrip.rs` for all
  content block variants and edge-shape serialization scenarios.
- Added engine fixtures and test harnesses in `fireside-engine/tests/` for
  structural validation and command-history invariants.
- Added CLI e2e tests covering:
  - validate success (`docs/examples/hello.json`),
  - validate failure for missing files,
  - single-file scaffold creation,
  - project scaffold directory structure creation.
- Added test-only dependencies in `fireside-cli/Cargo.toml`.
- Extended TUI hello smoke tests with per-node non-empty render assertion.
- Resolved fixture assertion mismatch by checking full `anyhow` error chains
  (`{error:#}`) for load-failure diagnostics.
- Verified complete workspace gates after all test additions:
  - `cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`
