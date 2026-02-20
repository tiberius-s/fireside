---
title: 'Chapter 3: Ownership, Borrowing, and Collections'
description: 'Understand Vec, HashMap, and borrow choices through Graph::from_file and indexing.'
---

## Learning Objectives

- Explain ownership transfer vs borrowing in function boundaries.
- Use `Vec` and `HashMap` together for ordered plus indexed access.
- Recognize when cloning is necessary and when it is avoidable.
- Reason about mutation safety with `&mut` and index rebuilds.

## Concept Introduction

Ownership and borrowing are Rust’s core safety model. They replace implicit
runtime aliasing rules with compile-time guarantees about who can read or write
what at a given point. Fireside’s graph loader is a practical example because it
transforms incoming wire data into a runtime structure optimized for traversal:
a `Vec<Node>` for stable order plus a `HashMap<NodeId, usize>` for fast lookup.

A useful framing is “move at boundaries, borrow internally.” `Graph::from_file`
consumes `GraphFile` by value, so it can move node vectors and metadata without
extra allocation. Inside methods like `index_of` and `node_by_id`, borrowing is
used to avoid cloning. Returning `Option<&Node>` gives callers read access while
keeping the graph owner authoritative.

Collections express intent. `Vec` preserves presentation order, which is
semantically important for sequential traversal. `HashMap` gives O(1)-style
index lookup for IDs. Holding both is a deliberate trade-off: slightly more
memory for significantly simpler traversal and command code. The important
maintenance rule is consistency. After structural mutation, the map must be
rebuilt or stale indices create subtle bugs.

Borrowing rules help here. Methods that only inspect graph state take `&self`.
Structural operations take `&mut self`, forcing exclusive access during mutation.
That exclusivity is exactly what prevents concurrent stale updates from leaking.
When you mutate `nodes`, you must re-establish map invariants before returning.

Cloning still has a place, but it should be intentional. In undo/redo systems,
clones may be cheaper than recomputing inverses, especially for small-to-medium
payloads. In read paths, cloning often signals missing borrowing opportunities.
As a heuristic, start by borrowing; clone only when ownership transfer or
lifetime boundaries require it.

## Fireside Walkthrough

Source anchor: `crates/fireside-core/src/model/graph.rs`.

```rust
pub struct Graph {
    pub nodes: Vec<Node>,
    pub node_index: HashMap<NodeId, usize>,
}

pub fn index_of(&self, id: &str) -> Option<usize> {
    self.node_index.get(id).copied()
}
```

Why this design:

- `Vec` keeps user-visible node order.
- `HashMap` prevents repeated linear scans.
- `copied()` returns a tiny value, avoiding borrowed map internals at call sites.

Also note `rebuild_index(&mut self)`: this is an ownership-safe invariant repair
step after add/remove/reorder operations.

## Exercise

Instrument one command path to intentionally skip `rebuild_index`, run tests,
then restore the call and observe why indexed access correctness depends on
post-mutation repair.

## Verification

Run:

```bash
cargo test -p fireside-engine command_history
```

## What would break if…

If `node_index` stored references into `nodes` instead of indices, reallocation
or reordering in the vector could invalidate references, making lifetime and
mutation management significantly harder. Indices are simpler and robust.

## Key Takeaways

Ownership design is architecture, not syntax trivia. Move data at clear
boundaries, borrow for read-heavy operations, and reserve cloning for explicit
trade-offs. Pairing `Vec` with `HashMap` is a powerful pattern when you need
order plus fast lookup. Invariant-repair methods like `rebuild_index` keep the
model coherent after mutation.
