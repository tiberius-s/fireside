---
title: 'ADR-001: Remove traversal.after'
status: 'accepted'
date: '2026-06-11'
deciders: ['@tiberius']
---

# ADR-001: Remove `traversal.after`

## Status

Accepted

## Context

The Rust reference implementation carries a field the protocol does not have:
`Traversal.after` (`crates/fireside-core/src/model/traversal.rs`), exposed as
`Node::after_target()` and followed by the engine's `next()` operation. It is a
pre-rewrite mechanism for branch rejoin: the branching node records where
branch endpoints should return to, and the engine follows that hidden edge when
an endpoint has no `next` of its own.

The spec is the source of truth, and its `Traversal` schema
(`protocol/tsp-output/schemas/Traversal.json`) defines exactly two properties:
`next` and `branch-point`. The specified rejoin mechanism is explicit wiring —
each branch endpoint sets its own `next` back to the rejoin node (see the
"Branch return wiring" section of §3 Traversal). The spec's design principle is
that every edge in the graph is explicit; `after` violates that by creating
edges that live on a *different* node than the one being left.

Two options were considered besides removal: spec `after` (rejected — it is
redundant with explicit edges and would give the protocol two competing rejoin
mechanisms), or keep it as a documented engine extra (rejected — it changes
traversal *semantics*, so a document relying on it would silently behave
differently in other conforming engines). Doing nothing leaves the reference
implementation non-conformant with its own protocol.

## Decision

We will delete `after` from the Rust model entirely: the `Traversal.after`
field, `Node::after_target()`, the engine `next()` branch that follows it, the
dangling-reference validation check, and the tests that cover it. Branch rejoin
is expressed exclusively via explicit `next` on each branch endpoint.

Legacy documents containing `"after"` parse fine — serde ignores unknown fields
by default, and the schema layer (Layer 1 validation) owns strictness. No
deprecation shim or warning system is added.

## Consequences

### Positive

- The engine's traversal surface matches the spec exactly: `next`,
  `branch-point`, nothing else.
- One rejoin mechanism instead of two — no precedence rules between `next` and
  `after` to specify, implement, or test.
- Every edge is visible on the node it leaves from, which keeps graph tooling
  (validation, visualization, dead-end lints) simple.

### Negative or Trade-offs

- Legacy documents that relied on `after` lose rejoin behavior silently: the
  field is ignored on parse and their branch endpoints become terminal nodes.
- Authors must wire each branch endpoint explicitly, which is more verbose for
  branches with many endpoints rejoining the same node.

### Neutral / Follow-up

- Implemented by the presenter-first rewrite (ADR-004): the new
  `fireside-core` model never had the field.
- Branch endpoints left without a `next` surface through the `dead-end-branch`
  validation rule, which replaces the safety `after` used to provide.
