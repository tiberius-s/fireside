# Task 09 — Engine: explicit-edge traversal semantics (D2/D3/D4)

**Depends on:** 03, 06
**Crates:** fireside-engine, fireside-tui
**Phase:** 2

## Goal

Make `TraversalEngine` implement the normative algorithms in `docs/src/content/docs/spec/traversal.md`:

- `next()`: blocked at a branch point; follows the explicit edge; **no sequential fallback**; terminal node = stay put.
- `back()`: pops history only; empty history = no-op (**no sequential backward fallback**).

## Background

`crates/fireside-engine/src/traversal.rs`: `next()` never checks `branch_point()` and falls through to `current + 1` (lines 99–107); `back()` falls back to `current - 1` (lines 115–119). Both violate the spec's "no implicit sequential fallback" and "next() is BLOCKED" rules.

## Steps

1. Extend `TraversalResult` with `Blocked` (branch point present — caller must `choose`). Reuse `AtBoundary` for terminal/no-history no-ops (or rename to `NoOp` if clearer — pick one and update all matches).
2. `next()`: (a) if current node has a branch point → return `Blocked`, push nothing; (b) else if `next_override()` resolves → move (push history); (c) else → `AtBoundary`, push nothing. Failed/blocked operations MUST NOT mutate history (spec invariant 4).
3. `back()`: pop-and-move only; empty history → `AtBoundary`.
4. Rewrite engine tests: linear fixtures gain explicit `"traversal": "<next-id>"` edges; add tests for blocked-next, terminal no-op, empty-history back.
5. TUI: in `app/action_routing.rs`, handle `Blocked` by flashing a message via the existing `FlashKind` mechanism (e.g. "Choose an option (a/b/…) to continue") and `AtBoundary` on a terminal node by flashing "End of path — Backspace to go back". Use existing flash styling — no new UI elements (Task 18 polishes further).
6. The TUI `hello_smoke`/golden tests will change behavior (no auto-advance past unlinked nodes) — update expectations.

## Do NOT

- Change `goto`'s index-based signature or history representation (Task 10).
- "Helpfully" keep sequential fallback behind a flag. It is gone.

## Acceptance

```bash
cargo test -p fireside-engine -p fireside-tui
cargo run -q -p fireside-cli -- present docs/examples/hello.json --plain   # still renders all nodes
```
