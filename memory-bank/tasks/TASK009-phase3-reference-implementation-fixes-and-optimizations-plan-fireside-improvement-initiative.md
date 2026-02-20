# TASK009 - Phase 3 reference implementation fixes and optimizations plan-fireside-improvement-initiative

**Status:** Completed
**Added:** 2026-02-19
**Updated:** 2026-02-19

## Original Request

Proceed with Phase 3 from `.github/prompts/plan-fireside-improvement-initiative.prompt.md`:
fix index rebuild root-cause issues, cache syntect assets, add redraw gating,
and cap traversal history.

## Thought Process

Phase 3 combines correctness and performance work, so implementation order
matters:

1. Fix graph index maintenance at the source (`Graph`) and make the engine use
   that API, rather than ad-hoc index rebuilding in one module.
2. Replace repeated syntect setup in hot render paths with static initialization.
3. Add draw gating in the event loop while preserving animation behavior.
4. Bound traversal history growth to avoid unbounded memory over long sessions.

## Implementation Plan

- Add `Graph::rebuild_index()` in `fireside-core`.
- Switch structural command mutations in `fireside-engine` to call
  `graph.rebuild_index()`.
- Convert traversal history storage to `VecDeque`, cap at 256 entries.
- Add `LazyLock` statics in `fireside-tui/render/code.rs` for syntax/theme sets.
- Add `needs_redraw` lifecycle to `App` and use it from CLI session event loop.
- Run workspace build/test/clippy gates.

## Progress Tracking

**Overall Status:** Completed - 100%

### Subtasks

- **9.1** Add canonical graph index rebuild API — **Complete** (2026-02-19)
  Added `Graph::rebuild_index()` and duplicate-id validation in rebuild path.
- **9.2** Replace engine-local index rebuild helper — **Complete** (2026-02-19)
  Command mutations now call `graph.rebuild_index()` with `EngineError::CommandError` mapping.
- **9.3** Cache syntect sets — **Complete** (2026-02-19)
  Added `SYNTAX_SET` and `THEME_SET` as `LazyLock` statics.
- **9.4** Add redraw gating — **Complete** (2026-02-19)
  Added `needs_redraw` field and `take_needs_redraw()`; CLI event loop now draws conditionally.
- **9.5** Cap traversal history growth — **Complete** (2026-02-19)
  Switched to `VecDeque` and bounded history at 256 entries.
- **9.6** Verify quality gates — **Complete** (2026-02-19)
  `cargo build`, `cargo test --workspace`, and `cargo clippy --workspace -- -D warnings` passed.

## Progress Log

### 2026-02-19

- Implemented `Graph::rebuild_index()` in `fireside-core`.
- Removed engine-local index rebuild helper and centralized index rebuilding
  through `Graph` API.
- Updated traversal history implementation to bounded `VecDeque` with helper
  push method enforcing max length.
- Added static syntect caches using `LazyLock` to remove repeated theme/syntax
  initialization during rendering.
- Added redraw gate in app/session loop to avoid unnecessary `terminal.draw()`
  calls while preserving animation ticks and interaction updates.
- Verified full Rust workspace build/test/clippy passes after changes.
