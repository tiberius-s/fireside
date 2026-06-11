# Task 08 — Require `Node.id` (D6)

**Depends on:** 07
**Crates:** fireside-core, fireside-engine, fireside-cli, fireside-tui
**Phase:** 1

## Goal

`Node.id` becomes required (`id: NodeId`, not `Option`), per `protocol/tsp-output/schemas/Node.json` (`"required": ["id", "content"]`).

## Background

`crates/fireside-core/src/model/node.rs:22` has `id: Option<NodeId>`. Optionality ripples everywhere: `Graph::from_file`/`rebuild_index` skip anonymous nodes, `validation.rs` prints `<anonymous>`, the engine has `PresentationSession::ensure_node_id` to invent ids, and the scaffold emits id-less nodes (which the protocol validator rejects as duplicate `"undefined"`).

## Steps

1. `Node.id: NodeId` (required). Fix all `Option` handling: `graph.rs` (`from_file`, `rebuild_index`), `validation.rs` (drop `<anonymous>`), engine `session.rs` (`ensure_node_id` simplifies to a lookup or disappears — if the editor needs to create nodes, it must generate an id at creation time, e.g. `node-{n}` avoiding collisions).
2. Update the scaffold templates in `crates/fireside-cli/src/commands/scaffold.rs` and the TUI editor node-creation paths (`app/editor_navigation.rs::add_node_after_selected`, `design/templates.rs`) to always set a unique id.
3. Keep the duplicate-id error path and its test.
4. Update every test fixture that omits `id` (engine traversal/validation tests, core graph tests, TUI tests).

## Do NOT

- Rewrite the scaffold content beyond adding ids (Task 12 does the full conformance rewrite).
- Add id auto-generation on *load* — a document without ids is invalid and must fail to load with a clear serde error naming the node index.

## Acceptance

```bash
cargo test --workspace
cargo run -q -p fireside-cli -- new t08 --dir /tmp && node protocol/validate.mjs /tmp/t08.json 2>&1 | grep -v "unique-node-ids"   # no duplicate-undefined errors
cargo run -q -p fireside-cli -- validate docs/examples/hello.json   # exit 0
```
