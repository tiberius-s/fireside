---
title: 'Chapter 1: Your First Data Model'
description: 'Use structs, enums, derive macros, and Option fields through Fireside content blocks.'
---

## Learning Objectives

- Define a Rust `struct` and `enum` that serialize cleanly.
- Use `#[derive(...)]` to get behavior with minimal boilerplate.
- Decide when to use required fields vs `Option<T>`.
- Read serde attributes that shape JSON wire format.

## Concept Introduction

Rust data modeling starts with a simple question: what invariants should your
compiler protect before runtime? In many dynamic systems, your shape checks
happen after data is loaded. In Rust, you encode those checks directly in type
shape, then let serde map between external JSON and internal domain types.
Fireside is a strong example because it has a public protocol, a strict wire
format, and a runtime model that must stay backward compatible.

`struct` is used when your value has one stable shape. A node has fields such
as `id`, `layout`, `traversal`, and `content`, so `Node` is a struct. `enum` is
used when a value can be one of several distinct variants. A content block can
be heading, text, code, list, image, divider, container, or extension, so
`ContentBlock` is an enum. Rust then forces exhaustive matches: every renderer,
validator, and serializer must account for each variant.

Derive macros are practical power tools. `Debug` helps diagnostics, `Clone`
supports safe value duplication for undo/redo flows, and `PartialEq` powers
assertions in round-trip tests. `Serialize` and `Deserialize` let serde bridge
Rust values and JSON. When the schema requires a discriminator, attributes such
as `#[serde(tag = "kind", rename_all = "kebab-case")]` keep wire names aligned
with the protocol while preserving idiomatic Rust naming in code.

`Option<T>` communicates optionality at the type level. In protocol evolution,
that matters: additive 0.1.x fields should not break old documents. A required
field should model truly required semantics, not convenience. If a field can be
absent in valid input, represent it as `Option<T>` or provide a default. This
is cleaner than sentinel values and avoids ambiguity around empty strings.

Finally, model design is not just about syntax. It drives ergonomics. If you
choose precise types and clear serde annotations up front, every downstream
layer benefits: loader errors are clearer, render logic is easier to read, and
tests become smaller because invalid states are unrepresentable.

## Fireside Walkthrough

Source anchor: `crates/fireside-core/src/model/content.rs`.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ContentBlock {
    Heading { level: u8, text: String },
    Text { body: String },
    // ...
}
```

Why this design:

- The `kind` tag gives stable wire discrimination.
- `rename_all = "kebab-case"` enforces protocol naming.
- Variant fields remain strongly typed for compiler-checked logic.

The mapping is direct: `Heading { level: 1, text: "Hello" }` serializes to
`{"kind":"heading","level":1,"text":"Hello"}`.

## Exercise

Add an `Aside` variant to `ContentBlock` with `body: String`, then add a
round-trip integration test in `crates/fireside-core/tests/content_roundtrip.rs`.

## Verification

Run:

```bash
cargo test -p fireside-core content_roundtrip
```

## What would break if…

If you remove `Serialize` from the derive list, the compiler rejects any call to
`serde_json::to_string(&block)` with an error that `ContentBlock` does not
implement `serde::Serialize`. That failure is desirable: serialization support
is a hard requirement for this protocol layer.

## Key Takeaways

Rust data modeling is most effective when your enum/struct shapes match domain
truth, not temporary UI assumptions. Serde attributes let you preserve a stable
wire contract while keeping internal code idiomatic. Optional protocol fields
belong in `Option<T>` so additive evolution stays safe. Derive macros reduce
noise, which lets you focus on semantics and invariants.
