---
title: 'Chapter 2: Errors That Help'
description: 'Build recoverable error flows with Result, ?, and thiserror in layered crates.'
---

## Learning Objectives

- Distinguish protocol errors from engine/runtime errors.
- Use `Result<T, E>` and `?` for clean propagation.
- Design typed errors with `thiserror`.
- Preserve error context without panicking.

## Concept Introduction

Rust error handling is explicit by default. Instead of invisible control flow,
functions return `Result<T, E>`, making failure part of the API contract.
Fireside uses this aggressively because loading and traversing presentation
graphs touches JSON parsing, schema shape, graph integrity, and user input. If
all failures collapsed into strings, callers would lose precision and tests
would become brittle.

The core pattern is small: use `Result` for recoverable failures, reserve panic
for impossible states in narrow internal contexts, and use `?` to bubble errors
up with minimal ceremony. But the real leverage comes from layered error types.
`fireside-core` reports protocol-level issues such as invalid JSON or duplicate
IDs. `fireside-engine` wraps those and adds traversal or command errors. This
keeps responsibilities clear and lets each crate communicate in domain terms.

`thiserror` makes typed errors ergonomic. You define enum variants with rich
messages and optional sources. Deriving `Error` and `Debug` gives display and
chain support without manual trait boilerplate. In layered systems, `#[from]`
variants are especially valuable because they encode conversion rules and make
`?` compose naturally across crates.

A subtle but important benefit is user communication quality. With typed enums,
you can map specific variants to actionable hints in UI/CLI output: dangling
references suggest fixing node IDs, while invalid traversal errors suggest input
issues. If everything is a string blob, that distinction is expensive to
recover later.

The final piece is tests. Error-handling code is logic code. You should test
variant selection, message quality where relevant, and chain behavior when
wrapping lower-level failures. Fireside’s fixture tests demonstrate that it is
reasonable to assert on substrings for top-level user meaning while still
keeping variants typed internally.

## Fireside Walkthrough

Source anchors: `crates/fireside-core/src/error.rs` and
`crates/fireside-engine/src/error.rs`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("invalid JSON: {0}")]
    InvalidJson(String),
    #[error("duplicate node id: {0}")]
    DuplicateNodeId(String),
}
```

```rust
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Core(#[from] fireside_core::error::CoreError),
    #[error("invalid traversal: {0}")]
    InvalidTraversal(String),
}
```

Why this design:

- Core stays protocol-focused.
- Engine composes core errors via `#[from]`.
- Callers can still pattern-match specific variants.

## Exercise

Add a new typed variant where appropriate for a repeated string-based failure
path in the engine, then update one caller to return that variant instead of a
generic `CommandError` string.

## Verification

Run:

```bash
cargo test -p fireside-engine
```

## What would break if…

If you replace typed enums with `anyhow::Error` inside library boundaries, you
lose stable matchability for unit tests and higher-level routing. You can still
print errors, but you cannot reliably branch behavior by variant without fragile
string matching.

## Key Takeaways

Error types are part of your API, not incidental plumbing. `Result` plus `?`
keeps control flow linear, while `thiserror` gives expressive, typed failures.
Layered enums let each crate speak its domain language and still compose cleanly.
This yields better UX, better tests, and safer future refactors.
