# TASK007 - Phase 2 build speed and dependency cleanup plan-fireside-improvement-initiative

**Status:** Completed
**Added:** 2026-02-19
**Updated:** 2026-02-19

## Original Request

Proceed with Phase 2 from `.github/prompts/plan-fireside-improvement-initiative.prompt.md`:
remove unused dependencies, add linker and dev profile optimizations, evaluate
syntect feature tradeoffs, add nextest to CI/dev workflow, and verify.

## Thought Process

Phase 2 should deliver immediate iteration speed improvements with minimal risk.
The approach was:

1. Apply deterministic safe wins first (remove unused deps, add linker/profile).
2. Evaluate syntect feature swap through an actual compile-and-test pass before
   deciding to keep or revert.
3. Add CI support for `cargo-nextest` so faster test execution is part of
   project workflow, not just local docs.
4. Verify with full Rust gates and then sync memory-bank state.

## Implementation Plan

- Update root `Cargo.toml` dependencies and profiles.
- Add `.cargo/config.toml` for linker optimization.
- Evaluate candidate syntect feature set with TUI build/tests.
- Update README build/test commands.
- Add Rust CI workflow using `cargo-nextest`.
- Run full verification (`build`, `test`, `clippy`).

## Progress Tracking

**Overall Status:** Completed - 100%

### Subtasks

- **7.1** Remove unused workspace deps — **Complete** (2026-02-19)
  Removed `serde_yaml` and `toml` from root `Cargo.toml`.
- **7.2** Add linker optimization config — **Complete** (2026-02-19)
  Added `.cargo/config.toml` with `-ld_prime` for `aarch64-apple-darwin`.
- **7.3** Add dev dependency optimization profile — **Complete** (2026-02-19)
  Added `[profile.dev.package."*"] opt-level = 2`.
- **7.4** Evaluate syntect feature swap — **Complete** (2026-02-19)
  Switched to `default-syntaxes`, `default-themes`, `regex-fancy`; TUI build and tests passed.
- **7.5** Add nextest to workflow/docs — **Complete** (2026-02-19)
  Added `.github/workflows/rust.yml` and README command for `cargo nextest run --workspace`.
- **7.6** Verification gate run — **Complete** (2026-02-19)
  `cargo build`, `cargo test --workspace`, and `cargo clippy --workspace -- -D warnings` passed.

## Progress Log

### 2026-02-19

- Completed Phase 2 implementation.
- Added Rust CI workflow with checks: fmt, clippy, and nextest.
- Adopted pure-Rust syntect regex path after successful evaluation build/test.
- Addressed one clippy regression (`large_enum_variant` in `Command`) with a
  targeted allow annotation to preserve existing command semantics.
- Verified all required Phase 2 Rust quality gates are green.
