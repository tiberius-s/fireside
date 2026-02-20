---
title: 'Chapter 4: Traits and Polymorphism'
description: 'Use trait-based design with Display, From, Error, and derive-driven capabilities.'
---

## Learning Objectives

- Explain how traits enable reusable behavior without inheritance.
- Use standard traits (`Debug`, `Display`, `Error`, `From`) effectively.
- Understand derive-based polymorphism in everyday Rust.
- Apply trait thinking to API design and ergonomics.

## Concept Introduction

Rust polymorphism is trait-first. Instead of subclass trees, behavior is
expressed as capabilities that types implement. This is more composable and
usually easier to reason about in systems code because interfaces stay explicit.
In Fireside, traits appear everywhere: error formatting, cloning command state,
comparisons in tests, and serde-driven serialization/deserialization.

Start with the practical traits. `Debug` is for diagnostics; you should derive
it almost everywhere in domain models and errors. `Display` is for user-facing
messages, including CLI output. `std::error::Error` marks a type as an error
source and unlocks chaining. `From` enables ergonomic conversions and powers `?`
by letting the compiler transform lower-level errors into your function’s error
type. Together, these traits reduce boilerplate while preserving intent.

`thiserror` is trait composition in action. A simple enum with attributes can
implement `Display`, `Error`, and conversion wiring through `#[from]`. This is
not magic; it is generated trait impl code following Rust conventions. You get
human-readable messages and machine-friendly variants at the same time.

Traits also help API stability. A function returning `impl Iterator` can hide
internal iterator types while keeping call sites efficient. A function accepting
`impl AsRef<Path>` is more flexible than hardcoding `&Path` when ownership or
string literals are common inputs. These small trait-oriented choices make
libraries easier to integrate.

In application code, derive traits are often enough. But the mindset matters:
ask “what behavior does this type need to expose?” rather than “what class does
this belong to?” That shift leads to clearer boundaries and fewer leaky
abstractions, especially in multi-crate workspaces like Fireside.

## Fireside Walkthrough

Source anchors: `crates/fireside-core/src/error.rs` and
`crates/fireside-engine/src/error.rs`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Core(#[from] fireside_core::error::CoreError),
    #[error("dangling node reference: {0}")]
    DanglingReference(String),
}
```

Why this design:

- `Error` + `Display` are generated consistently.
- `From<CoreError>` conversion is explicit and compiler-checked.
- Callers can still match concrete variants.

## Exercise

Add a small utility type in one crate that implements `Display` manually and
`FromStr` for parsing. Use it in a focused unit test to compare manual impls
versus derive-heavy types.

## Verification

Run:

```bash
cargo test -p fireside-core
```

## What would break if…

If you remove `#[from]` from wrapped error variants but still use `?`, compile
errors will surface because Rust can no longer convert source errors into your
function’s declared error type. This is a good failure mode: conversion rules
must be explicit.

## Key Takeaways

Traits are the primary polymorphism tool in Rust. Standard traits deliver most
of the value: formatting, conversions, and error integration. Derive macros help
you move quickly, but the important part is designing clear capability
boundaries. In Fireside, trait-driven error composition is a central ergonomics
win.
