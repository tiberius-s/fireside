# Task 10 — Engine: NodeId-based history (D5)

**Depends on:** 09
**Crates:** fireside-engine, fireside-tui
**Phase:** 2

## Goal

History entries become node IDs, not array indices (spec History Invariant 5, traversal.md). Editing/reordering nodes must not corrupt the back-stack.

## Background

`TraversalEngine.history` is `VecDeque<usize>`; `clamp_to_graph` retains stale indices after edits — after a node reorder in the editor, `back()` can land on the wrong node.

## Steps

1. `history: VecDeque<NodeId>` (requires Task 08's required ids). Push the current node's id; `back()` resolves id → index via `graph.index_of(...)`; if the id no longer exists (node deleted), pop again until a valid id or empty (document this behavior in a doc comment — it is an engine decision the spec leaves open).
2. Replace `clamp_to_graph`'s history filtering with id-existence filtering; keep the `current` index clamp.
3. Add `goto_id(&mut self, id: &str, graph: &Graph)` implementing the spec's Goto (validate id exists → push current id → move; unknown id = no-op per `main.tsp:417`). Keep the existing index-based `goto` as a thin wrapper used by the TUI's numeric goto overlay.
4. Tests: back-after-reorder returns to the right node by id; goto unknown id is a no-op; history caps at `MAX_HISTORY` unchanged.
5. TUI breadcrumb/timeline (`ui/breadcrumb.rs`, `ui/timeline.rs`) read history — update to ids (display node titles when available, falling back to ids; styling via `DesignTokens`).

## Do NOT

- Persist history across sessions.
- Change `choose`/`next` semantics (done in Task 09).

## Acceptance

```bash
cargo test -p fireside-engine -p fireside-tui
```
