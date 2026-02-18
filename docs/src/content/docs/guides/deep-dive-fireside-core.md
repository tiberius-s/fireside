---
title: 'Deep Dive: fireside-core'
description: 'Protocol model, serde wire format, and type-level invariants in Fireside core.'
---

## Why This Crate Exists

`fireside-core` is your protocol boundary. It defines the data model and JSON
wire format for Fireside, with no runtime, no terminal UI, and no app logic.

This separation is exactly what you want when learning Rust architecture:

- stable types in one place
- behavior in higher layers
- easy testing of serialization contracts

## Code Map

- `src/lib.rs`: crate boundary and public re-exports
- `src/model/content.rs`: `ContentBlock` tagged enum + custom list item de/serialization
- `src/model/graph.rs`: top-level `GraphFile` and runtime `Graph` index construction
- `src/model/node.rs`: node shape and traversal helper accessors
- `src/model/traversal.rs`: per-node traversal overrides
- `src/error.rs`: protocol-level errors (`CoreError`)

## Rust Patterns Used

### Tagged enums for protocol unions

`ContentBlock` uses `#[serde(tag = "kind", rename_all = "kebab-case")]`.
That gives you a type-safe Rust enum while preserving human-readable wire JSON.

### Newtype-by-alias for domain IDs

`type NodeId = String` gives a clear domain name in signatures while keeping
serialization simple.

### Runtime index cache

`Graph::from_file` builds `HashMap<NodeId, usize>` for O(1) node lookup.
This is a standard optimization pattern: pay once during load, simplify later.

### Focused error enum

`CoreError` keeps this crate’s concerns narrow: file read, JSON parse,
empty graph, duplicate IDs.

## Rust Book References

- Ownership and borrowing (Chapter 4):
  <https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html>
- Enums and pattern matching (Chapter 6):
  <https://doc.rust-lang.org/book/ch06-00-enums.html>
- Error handling with `Result` (Chapter 9):
  <https://doc.rust-lang.org/book/ch09-00-error-handling.html>
- Collections (`HashMap`) (Chapter 8):
  <https://doc.rust-lang.org/book/ch08-03-hash-maps.html>
- Tests (Chapter 11):
  <https://doc.rust-lang.org/book/ch11-00-testing.html>

## Concepts To Know Before Editing

- How serde attributes shape wire compatibility
- When to keep `Option<T>` versus required fields
- Why protocol crates should avoid UI dependencies
- How to avoid hidden panics in model-layer code

## Gotchas To Watch

- `NodeId` is a type alias, not a true newtype, so accidental string mixing is possible
- `Graph::from_file` currently returns `Result<_, String>` internally before mapping
- `ListItem` custom deserializer is flexible, but this flexibility can hide malformed inputs

## Improvement Playbook

### 1) Strengthen ID typing

Goal: replace `type NodeId = String` with a real newtype.

Steps:

1. Add `pub struct NodeId(String);` with `Serialize`, `Deserialize`, `Display`.
2. Implement smart constructor validation (non-empty, maybe kebab-case).
3. Update all signatures to accept `&NodeId` where possible.
4. Add compile-time and serde round-trip tests.

### 2) Tighten Graph construction errors

Goal: make graph build errors typed instead of plain strings.

Steps:

1. Add a `GraphBuildError` enum in `fireside-core`.
2. Change `Graph::from_file` to `Result<Graph, GraphBuildError>`.
3. Map errors in loader without string parsing.
4. Add tests for each error variant.

### 3) Validate content-level invariants in one place

Goal: avoid impossible states entering engine logic.

Steps:

1. Add optional `Graph::validate_basic()` in core.
2. Check obvious invariants (heading level range, empty list items, etc.).
3. Keep cross-node validation in `fireside-engine`.
4. Document which crate owns which validation responsibility.
