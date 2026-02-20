---
title: "Chapter 5: When Derive Isn't Enough"
description: 'Manual Deserialize and visitor patterns for mixed JSON input shapes.'
---

## Learning Objectives

- Identify cases where derive-only serde is insufficient.
- Implement manual `Deserialize` with a visitor.
- Accept multiple wire shapes while preserving clear errors.
- Understand trade-offs of `#[serde(untagged)]` vs custom visitors.

## Concept Introduction

Serde derive handles most serialization tasks, but protocol authors eventually
hit mixed-shape input problems. A classic case is “string or object” input:
users can write a compact form in simple cases, while advanced use requires a
richer object shape. Fireside’s `ListItem` supports both bare strings and full
objects with nested children, which is exactly where custom deserialization is
worth the complexity.

Why not just use `#[serde(untagged)]`? Sometimes you can, but ambiguity and poor
error messages become painful as shapes overlap. With untagged enums, serde
tries variants in order. If multiple variants partially fit, errors may mention
unexpected internals instead of user intent. A manual visitor gives you full
control over accepted forms and failure text.

The visitor pattern may look verbose, but it maps directly to serde’s parsing
model. You implement `expecting`, then specific methods such as `visit_str` and
`visit_map`. This makes mixed-form support explicit and readable in one place.
It also avoids sprinkling fallback logic across call sites.

Another benefit is compatibility control. Protocol layers often need to keep old
documents working while adding richer authoring forms. A custom visitor lets you
add support incrementally without changing domain model shape. You can continue
returning one internal struct while accepting multiple syntaxes externally.

The cost is maintenance burden. Manual visitors require tests for every accepted
and rejected shape, including nested edge cases. In return, you get precise
parsing behavior and better diagnostics. In protocol code, that trade is usually
worth it because deserialization is a public boundary.

## Fireside Walkthrough

Source anchor: `crates/fireside-core/src/model/content.rs` (`ListItem`).

```rust
impl<'de> serde::Deserialize<'de> for ListItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ListItemVisitor;
        // visit_str => bare string form
        // visit_map => object form with text + children
        deserializer.deserialize_any(ListItemVisitor)
    }
}
```

Why this design:

- Supports concise authoring (`"item"`) and rich structure.
- Keeps one runtime representation.
- Produces domain-specific expectation text.

## Exercise

Add an `InlineStyle` type that accepts both:

- bare string: `"bold"`
- object: `{ "style": "bold", "color": "red" }`

Then add an integration test named `inline_style_roundtrip`.

## Verification

Run:

```bash
cargo test -p fireside-core inline_style_roundtrip
```

## What would break if…

If you switch to a naive `#[serde(untagged)]` setup where variants overlap,
serde may select an unintended branch or emit low-quality errors when object
fields are partial. You might accept malformed input silently or reject valid
input with confusing messages.

## Key Takeaways

Manual serde is for boundary cases where shape flexibility and error quality
matter more than brevity. Visitors make accepted forms explicit and keep runtime
models clean. Fireside’s `ListItem` is a practical template for mixed syntax
support in protocol code. Use derive first, then move to custom deserialize only
when behavior or diagnostics demand it.
