# Task 12 — Scaffold conformance

**Depends on:** 08, 11
**Crates:** fireside-cli
**Phase:** 3

## Goal

`fireside new` must emit documents that pass **both** validators with zero errors. The first-run experience currently produces files the protocol rejects.

## Background

The scaffold (`crates/fireside-cli/src/commands/scaffold.rs`) currently emits: node-level `"layout"` values, nodes without ids (fixed in Task 08), **no traversal edges** (relies on the removed sequential fallback — after Task 09 the scaffold deck cannot be navigated), object-form list items (fixed in Task 04's serializer), and a `$schema` URL (`https://fireside.dev/schemas/graph.schema.json`) that does not resolve.

## Steps

1. Rewrite the single-file and project templates to be canonical 0.1.0 documents:
   - every node has a unique kebab-case `id` and an explicit `"traversal": "<next-id>"` (last node terminal);
   - use `"view-mode"` (e.g. `"fullscreen"` on a code node) instead of node `layout`; use a `container` with `layout: "center"` for the title node (copy the shape from `docs/examples/hello.json`);
   - include one branch-point node so new users see branching immediately — options must target existing nodes;
   - `"fireside-version": "0.1.0"`; drop the `$schema` key entirely (or point at the real raw GitHub URL of `protocol/tsp-output/schemas/Graph.json` — only if that URL is verified to exist).
2. Keep `defaults` minimal: `{ "transition": "none" }`.
3. Update `cli_e2e.rs` scaffold tests to validate the output through `fireside validate` AND assert key structure (ids present, traversal chain complete).

## Do NOT

- Generate content referencing the old 12-layout system.
- Add theme/font keys to the scaffold (non-normative extras; opt-in only).

## Acceptance

```bash
cargo test -p fireside-cli
cargo run -q -p fireside-cli -- new t12 --dir /tmp
node protocol/validate.mjs /tmp/t12.json        # 0 errors
cargo run -q -p fireside-cli -- validate /tmp/t12.json   # exit 0
```
