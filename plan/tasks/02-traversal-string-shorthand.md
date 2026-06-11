# Task 02 — Traversal string shorthand (D1) + hello.json smoke test

**Depends on:** none
**Crates:** fireside-core
**Phase:** 0 — highest priority; this alone makes CI green

## Goal

`Node.traversal` must accept both a `NodeId` string (shorthand for `{ "next": "<id>" }`) and a full `Traversal` object, per `protocol/main.tsp:317` and `protocol/tsp-output/schemas/Node.json` (`anyOf: [NodeId, Traversal]`).

## Background

`crates/fireside-core/src/model/node.rs:54` declares `traversal: Option<Traversal>` (struct only). This makes the canonical example `docs/examples/hello.json` (line 14: `"traversal": "features"`) unparseable and fails 3 tests: `validate_hello_exits_zero` (cli_e2e), `hello_branch_choose_golden`, `hello_full_path_golden_ids` (tui harness_golden).

## Steps

1. In `crates/fireside-core/src/model/traversal.rs`, implement a custom `Deserialize` for `Traversal` that accepts:
   - a JSON string `"id"` → `Traversal { next: Some("id".into()), ..Default::default() }`
   - a JSON object → existing field-wise deserialization.
   Use the same string-or-object visitor pattern already proven at `crates/fireside-core/src/model/content.rs:113-159` (`ListItem`). Derive or implement `Default` for `Traversal` if needed.
2. `Serialize`: when only `next` is set (no `branch_point`, no other fields), serialize as the bare string to preserve authoring style; otherwise serialize the object form.
3. Add unit tests in `traversal.rs`: string form, object form, round-trip of both.
4. Add a conformance smoke test `crates/fireside-core/tests/hello_conformance.rs`: deserialize `../../docs/examples/hello.json` into `GraphFile` and assert 6 nodes, and that node `intro` has `next == Some("features")`.

## Do NOT

- Change any other field of `Traversal` (the `after` field is removed in Task 06, not here).
- Touch the engine or TUI.

## Acceptance

```bash
cargo test -p fireside-core
cargo test --workspace                                  # all green, including the 3 previously failing tests
cargo run -q -p fireside-cli -- validate docs/examples/hello.json   # exit 0
```
