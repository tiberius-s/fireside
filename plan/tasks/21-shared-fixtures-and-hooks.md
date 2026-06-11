# Task 21 — Shared conformance fixtures + restore git hooks

**Depends on:** 16, 20
**Crates:** all (test plumbing) + repo tooling
**Phase:** 5

## Goal

One fixture corpus consumed by both validators, and local git hooks back in service now that the workspace is green.

## Steps

1. Create `protocol/fixtures/` with paired fixtures, named by expectation:
   - `valid/` — linear, branching, fullscreen+containers, terminal-only, kitchen-sink (every block kind);
   - `invalid/` — duplicate-id, dangling-next, dangling-branch-target, empty-options, next-branch-point-conflict, missing-id.
   Migrate the existing Rust fixtures (`crates/fireside-engine/tests/fixtures/...`) here instead of duplicating; point the Rust tests at the new path.
2. Rust side: a parameterized test (`crates/fireside-engine/tests/conformance.rs`) walks `protocol/fixtures/valid` (must load + validate clean) and `invalid` (must produce ≥1 Error).
3. JS side: extend `protocol/validate.mjs` usage in the conformance workflow (Task 16) to walk the same directories with the same expectations.
4. Restore git hooks: recreate `githooks/` with `pre-commit` (cargo fmt --check) and `pre-push` (cargo clippy --workspace -- -D warnings + cargo test --workspace), plus the `install.sh` wiring `core.hooksPath`. This reverts the intent of 365ef85/c1c5df1 — verify the workspace is fully green first.
5. Document the fixture contract in `protocol/fixtures/README.md` (one paragraph: naming = expectation; both validators must agree).

## Do NOT

- Let fixtures drift from the schema — the conformance job is the referee.
- Make pre-commit run tests (too slow; pre-push owns tests).

## Acceptance

```bash
cargo test --workspace
for f in protocol/fixtures/valid/*.json; do node protocol/validate.mjs "$f" || exit 1; done
./githooks/install.sh && git config core.hooksPath   # prints githooks
```
