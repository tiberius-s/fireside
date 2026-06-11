# Task 13 — Round-trip fidelity in save_graph (D14)

**Depends on:** 07, 08
**Crates:** fireside-core, fireside-engine
**Phase:** 3

## Goal

`load → save` of an unmodified document must be semantically lossless. Today it both **loses** the document's `defaults` and **inflates** nodes with baked-in default values.

## Background

- `Graph::from_file` (`crates/fireside-core/src/model/graph.rs`) copies `defaults.layout`/`defaults.transition` into every node that lacks them — so saving writes the defaults onto each node.
- `graph_to_file` (`crates/fireside-engine/src/loader.rs`) writes `defaults: Some(NodeDefaults::default())`, discarding the original.

## Steps

1. Stop baking defaults into nodes in `Graph::from_file`. Store the raw `NodeDefaults` on `Graph` (e.g. `pub defaults: Option<NodeDefaults>` or inside `GraphMeta`).
2. All consumers resolve at read time via the Task 07 helpers (`resolved_view_mode`, `resolved_transition`). Grep for direct `node.layout` / `node.transition` reads in `fireside-tui` and route them through the helpers with the graph's defaults.
3. `graph_to_file` writes the stored defaults verbatim.
4. Round-trip test: load `docs/examples/hello.json`, save to a temp file, parse both as `serde_json::Value`, assert equality (key order may differ; compare Values, not strings).

## Do NOT

- Preserve byte-level formatting (pretty-print is fine); semantic equality is the bar.
- Resolve defaults during deserialization "for convenience" — that re-creates the bug.

## Acceptance

```bash
cargo test --workspace
```
