---
title: 'Chapter 7: Undo/Redo with the Command Pattern'
description: 'Apply reversible graph mutations with command inverses and history stacks.'
---

## Learning Objectives

- Explain command pattern fundamentals in Rust.
- Use inverse commands for undo/redo correctness.
- Evaluate clone-vs-recompute trade-offs in history systems.
- Validate invariants after structural mutations.

## Concept Introduction

The command pattern models a user action as data that can be executed,
reversed, and replayed. In editors, this is the backbone of undo/redo. Fireside
implements commands at the engine layer so TUI/CLI stay thin: UI emits intent,
engine applies graph mutations, and history tracks reversibility.

A command system needs three pieces. First, a command enum representing atomic
mutations (`AddNode`, `RemoveNode`, `UpdateNodeContent`, traversal edits). Second,
an apply function that performs the mutation and returns an inverse command.
Third, history stacks for applied and undone entries. Undo pops from applied,
executes inverse, and pushes to undone. Redo pops undone, re-applies original,
and returns it to applied.

Returning an inverse from `apply_command` is a useful Rust-centric strategy. It
keeps inversion logic co-located with mutation logic, reducing drift. For
example, removing a node returns `RestoreNode` with full node payload and index.
That makes undo precise even after intermediate edits. The trade-off is cloning:
large payloads can be expensive, but correctness and simplicity often justify it
for interactive editing workloads.

Another key invariant is index integrity after structural operations. Commands
that alter node ordering must call `rebuild_index`; otherwise, subsequent ID
lookups can point to wrong positions. Fireside’s tests include a full sequence
(AddNode → UpdateNodeContent → RemoveNode → undo all) to guarantee snapshot
restoration and catch stale index regressions.

In command systems, deterministic behavior matters more than micro-optimizing
single operations. If apply/undo symmetry is strong and tests are broad, you can
refactor internals later with confidence.

## Fireside Walkthrough

Source anchor: `crates/fireside-engine/src/commands.rs`.

```rust
#[derive(Debug)]
pub struct CommandHistory {
    applied: Vec<HistoryEntry>,
    undone: Vec<HistoryEntry>,
}

pub fn apply_command(&mut self, graph: &mut Graph, command: Command)
    -> Result<(), EngineError>
```

Why this design:

- History entries carry both command and inverse.
- New command clears redo stack, matching editor expectations.
- Structural commands rebuild graph index immediately.

## Exercise

Add a command-level test that performs two edits, one remove, two undos, one
redo, and verifies both node content and node order are exactly expected.

## Verification

Run:

```bash
cargo test -p fireside-engine command_history
```

## What would break if…

If undo reconstructed inverses from current graph state instead of storing them
at apply time, later mutations could make reversal ambiguous or wrong. You might
restore nodes to incorrect indices or lose original content snapshots.

## Key Takeaways

The command pattern is a natural fit for Rust editor engines: commands are enums,
inverses are explicit, and history is straightforward vector bookkeeping.
Correctness depends on apply/undo symmetry and invariant repair after structural
changes. Fireside’s approach favors predictable behavior and testability over
premature optimization.
