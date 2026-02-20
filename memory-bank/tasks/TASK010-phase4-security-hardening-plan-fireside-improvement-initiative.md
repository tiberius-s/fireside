# TASK010 - Phase 4 security hardening plan-fireside-improvement-initiative

**Status:** Completed
**Added:** 2026-02-20
**Updated:** 2026-02-20

## Original Request

Proceed with Phase 4 from `.github/prompts/plan-fireside-improvement-initiative.prompt.md`:
implement security hardening for image path handling, iTerm2 import size checks,
extension payload safety documentation, and typed traversal-path error coverage.

## Thought Process

Phase 4 is a defense-in-depth pass and should preserve existing rendering UX
while tightening unsafe edges.

Implementation order was chosen to minimize regressions:

1. Introduce typed error surface (`EngineError::PathTraversal`) first.
2. Harden image path resolution in one place (`local_image_path`) and add tests
   for traversal/containment behavior.
3. Add iTerm2 plist pre-parse size guard before touching parser logic.
4. Add normative security wording in extensibility docs so behavior and spec are
   aligned.
5. Run full workspace verification and fix any platform-specific regressions.

## Implementation Plan

- Add `EngineError::PathTraversal` in `fireside-engine/src/error.rs`.
- Harden `local_image_path` in `fireside-tui/src/render/markdown.rs`:
  - reject parent traversal components,
  - constrain resolved paths to canonicalized `base_dir`,
  - log `tracing::warn!` on rejection.
- Add iTerm2 plist import size guard (>1 MB) in
  `fireside-tui/src/design/iterm2.rs`.
- Add/extend tests for path sanitization and oversized plist rejection.
- Add a `Security Considerations` section to
  `docs/src/content/docs/spec/extensibility.md`.
- Run full Rust quality gates.

## Progress Tracking

**Overall Status:** Completed - 100%

### Subtasks

- **10.1** Add typed path traversal error — **Complete** (2026-02-20)
  Added `EngineError::PathTraversal(String)`.
- **10.2** Harden image path sanitization — **Complete** (2026-02-20)
  Added parent-traversal rejection, base-dir confinement, and warning logs.
- **10.3** Add iTerm2 file size guard — **Complete** (2026-02-20)
  Rejects `.itermcolors` files larger than 1 MB before parse.
- **10.4** Add/adjust tests — **Complete** (2026-02-20)
  Added path-sanitization tests and oversized plist rejection test; adjusted
  canonical-path assertion to be macOS-safe.
- **10.5** Update spec security guidance — **Complete** (2026-02-20)
  Added extension payload security considerations in extensibility chapter.
- **10.6** Run verification gates — **Complete** (2026-02-20)
  `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` passed.

## Progress Log

### 2026-02-20

- Added `EngineError::PathTraversal` variant for typed traversal-path errors.
- Hardened image path handling in markdown renderer:
  - parent directory traversal is rejected,
  - absolute/relative paths are constrained to canonicalized base dir when set,
  - rejected paths emit `tracing::warn!` diagnostics.
- Added iTerm2 import file-size limit check with clear rejection behavior.
- Added regression tests for:
  - parent traversal rejection,
  - absolute out-of-base path rejection,
  - valid relative in-base path acceptance,
  - oversized iTerm2 file rejection.
- Resolved a macOS path alias (`/var` vs `/private/var`) test assertion issue
  by comparing canonicalized expected and actual paths.
- Resolved a hello-smoke regression by ensuring relative missing files still
  resolve within canonical base-dir scope and produce graceful fallback text.
- Verified full workspace quality gates are green:
  - `cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`
