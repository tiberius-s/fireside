# Task 11 — Validation: required checks + lint codes (D8)

**Depends on:** 06, 09
**Crates:** fireside-engine
**Phase:** 2

## Goal

Implement all four Required Checks from validation.md §Layer 2 and tag every diagnostic with a stable lint code matching `protocol/validate.mjs`, so the JS and Rust validators speak the same language.

## Background

`crates/fireside-engine/src/validation.rs` covers checks 1–2 (unique ids via loader, dangling targets) but not:

- **#3** `branch-point.options` non-empty → Error
- **#4** `next` and `branch-point` both present on one `Traversal` → Error ("Validators MUST reject", `main.tsp:253`)

Recommended checks (warnings) to add: unreachable-from-entry, duplicate branch keys within one branch point. `validate.mjs` already emits codes like `[unique-node-ids]`, `[unreachable-node]`, `[dead-end-branch]` — read `protocol/validate.mjs` first and mirror its code strings exactly; invent new codes only for checks it lacks (e.g. `next-branch-point-conflict`, `duplicate-branch-key`), and add those to `validate.mjs` too so both stay in lockstep.

## Steps

1. Add `code: &'static str` to `Diagnostic`. Populate for all existing diagnostics (match validate.mjs naming).
2. Add the two Required Checks as `Severity::Error`.
3. Add unreachable-node (BFS from entry node index 0 following next/branch edges) and duplicate-branch-key as `Severity::Warning`.
4. Mirror any *new* codes into `protocol/validate.mjs` with identical strings and severities.
5. Fixture tests for each new check (extend `crates/fireside-engine/tests/validation_fixtures.rs` and its fixture JSON directory).
6. `validate_or_error` currently wraps everything as `DanglingReference` — add a proper `EngineError::Validation { code, message }` variant instead.

## Do NOT

- Change CLI output formatting (Task 14).
- Make warnings fail validation.

## Acceptance

```bash
cargo test -p fireside-engine
node protocol/validate.mjs docs/examples/hello.json   # unchanged: 0 errors
```
