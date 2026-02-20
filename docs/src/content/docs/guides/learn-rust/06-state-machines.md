---
title: 'Chapter 6: State Machines'
description: 'Model traversal behavior with enums, explicit transitions, and bounded history.'
---

## Learning Objectives

- Model workflow transitions with explicit state structs/enums.
- Keep transition logic deterministic and testable.
- Use bounded history for predictable memory behavior.
- Apply `#[must_use]` mindset to state-returning APIs.

## Concept Introduction

State machines are one of Rust’s strongest design fits because enums make state
space explicit and `match` forces transition handling to stay exhaustive. In
Fireside, traversal is not just incrementing an index: it must respect explicit
`next` overrides, branch choices, rejoin semantics (`after`), and backtracking.
A state machine expresses these rules without hidden mutable side effects.

The central idea is to represent state and transitions as data, not scattered
conditionals. `TraversalEngine` stores `current` and `history`, then exposes
operations (`next`, `back`, `goto`, `choose`) that each return a typed
`TraversalResult`. This keeps call sites simple: they react to either
`Moved { from, to }` or `AtBoundary`.

Bounded history is a practical systems concern. Unlimited stacks can grow over
long sessions. By capping history with `VecDeque` and pruning oldest entries,
Fireside keeps memory predictable while preserving recent navigation context.
This is a good example of policy encoded at the state-machine layer rather than
left to ad hoc cleanup.

Error handling also belongs in transitions. Invalid go-to indices and branch key
mismatches return typed `EngineError` values instead of silent no-ops inside the
engine. The app may choose to ignore or display them, but correctness starts at
the state machine boundary.

A final pattern: keep transition internals private. `push_history` in traversal
centralizes retention policy, so all movement methods share the same invariant.
That prevents drift where one transition path forgets pruning or logging.

## Fireside Walkthrough

Source anchor: `crates/fireside-engine/src/traversal.rs`.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraversalResult {
    Moved { from: usize, to: usize },
    AtBoundary,
}

const MAX_HISTORY: usize = 256;
```

Why this design:

- Results are explicit and pattern-matchable.
- Bounded history avoids unbounded growth.
- Branch and override behavior is centralized in `next`/`choose`.

## Exercise

Add a test that repeatedly moves through a long generated graph and asserts
history length never exceeds `MAX_HISTORY`.

## Verification

Run:

```bash
cargo test -p fireside-engine traversal
```

## What would break if…

If transitions directly mutated `current` from multiple modules without a single
engine API, behavior would diverge quickly: some paths might ignore branch
rejoin rules, others might skip history updates, and backtracking correctness
would become accidental.

## Key Takeaways

State machines make behavior legible and robust. In Rust, enums and explicit
results are ideal for this pattern because the compiler enforces complete
handling. Centralized transition APIs, bounded history, and typed errors produce
navigation logic that is easy to test and hard to corrupt.
