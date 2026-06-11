# Task 06 — Remove `traversal.after` (D7)

**Depends on:** 02
**Crates:** fireside-core, fireside-engine
**Phase:** 1

## Goal

Delete the `after` field from `Traversal` and all logic that uses it. The spec (`protocol/tsp-output/schemas/Traversal.json`) defines exactly two properties: `next` and `branch-point`. Branch rejoin is expressed by each branch endpoint setting its own explicit `next` (see traversal.md "Branch return wiring").

## Background

`after` exists only in the Rust code: `crates/fireside-core/src/model/traversal.rs:23`, `Node::after_target()` (`node.rs:77-79`), engine `next()` follows it (`crates/fireside-engine/src/traversal.rs:90-97`), validation checks it, and two engine tests cover it (`next_uses_after_target_when_present`, `next_prefers_next_override_over_after_target`).

Prerequisite: Task 19's ADR-0001 should be accepted (or explicitly approved by the maintainer) before this lands, since it deletes a working feature.

## Steps

1. Remove `after` from `Traversal`, `Node::after_target()`, the engine `next()` branch that follows it, and the `traversal.after` dangling-reference check in `crates/fireside-engine/src/validation.rs`.
2. Delete the two `after` engine tests. Rewrite any fixture using `after` to use an explicit `next` on the branch endpoint instead.
3. Decide parse behavior for legacy documents containing `"after"`: ignore unknown fields (serde default) — do NOT add `deny_unknown_fields`; the schema layer owns strictness.

## Do NOT

- Add a deprecation shim or warning system — just remove it.
- Touch the sequential-fallback logic in `next()` (Task 09 owns that).

## Acceptance

```bash
grep -rn "after" crates/fireside-core/src/model/traversal.rs   # no field hit
cargo test --workspace
cargo run -q -p fireside-cli -- validate docs/examples/hello.json   # exit 0
```
