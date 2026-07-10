---
title: 'ADR-003: Non-normative engine extras'
status: 'superseded'
date: '2026-06-11'
deciders: ['@tiberius']
---

# ADR-003: Non-normative engine extras

## Status

Superseded by ADR-004 — the same-day presenter-first rewrite removed these
extras from the implementation instead of retaining them.

## Context

The Rust reference implementation grew several features that protocol 0.1.0
does not define:

- `kind: "extension"` content blocks and the graph-level
  `ExtensionDeclaration` list (`crates/fireside-core/src/model/graph.rs`)
- Graph metadata fields `theme`, `font`, and `tags`
- Six transitions beyond the spec's `none`/`fade`: `slide-left`,
  `slide-right`, `wipe`, `dissolve`, `matrix`, `typewriter`
  (`crates/fireside-core/src/model/transition.rs`)
- Nested list items (`ListItem.children` — the spec's `ListBlock.items` is
  `string[]`)
- `BranchPoint.id` (`crates/fireside-core/src/model/branch.rs`)

Each is working, tested code with real value in the TUI. But none appears in
`protocol/main.tsp` or the generated schemas, so a document using them is not
portable to other conforming engines, and leaving their status undefined
invites drift: future work cannot tell protocol surface from engine surface.

Two alternatives were considered. Delete them (rejected — they are working
features users rely on, and deleting them buys no conformance because the spec
already permits engines to extend). Spec them (rejected for 0.1.0 — each would
need design review, schema work, and conformance fixtures; that is scope the
0.1.0 milestone does not have).

## Decision

These features are **engine features, not protocol**. They remain implemented
in the Rust crates, are excluded from all conformance claims and conformance
fixtures, and are documented in a dedicated "Engine Extensions
(Non-Normative)" appendix in the spec docs, clearly marked non-normative.

The dividing line going forward: anything in `protocol/main.tsp` and its
generated schemas is protocol; anything else the engine does is an extension
and must be listed in the appendix.

## Consequences

### Positive

- Working features stay; no user-visible regression.
- The 0.1.0 protocol surface stays small and fully specified.
- Conformance claims become honest: fixtures exercise only specified behavior.

### Negative or Trade-offs

- Documents authored with these extras are not portable to other conforming
  engines (other engines may ignore or reject them).
- The appendix is a second source that must be kept current as the engine
  evolves — stale entries would reintroduce exactly the ambiguity this ADR
  removes.

### Neutral / Follow-up

- The "Engine Extensions (Non-Normative)" appendix page is added alongside
  this ADR (Appendix D in the spec docs).
- Conformance fixture work (Tasks 12 and 21) must not use any feature listed
  in the appendix.
- Individual extras may be promoted to protocol in a future minor/major via
  their own ADRs.
