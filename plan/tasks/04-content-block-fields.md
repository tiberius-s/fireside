# Task 04 — Content blocks: image size, optional alt, list serialization (D13)

**Depends on:** 02
**Crates:** fireside-core, fireside-engine, fireside-tui
**Phase:** 1

## Goal

Align `ContentBlock` fields with `protocol/main.tsp:147-177`: `ImageBlock` gains optional `width`/`height` (`int32`) and `alt` becomes optional; `ListBlock` items serialize as plain strings when they have no children.

## Background

`crates/fireside-core/src/model/content.rs`:

- `Image` has `alt: String` (defaulted) and no `width`/`height`. Spec: `alt?: string`, `width?: int32`, `height?: int32`.
- `ListItem` always serializes as `{ "text": ... }` objects. Spec `ListBlock.items` is `string[]`; the object form is an engine superset (kept for deserialization), but **emitting** objects for flat items produces non-conforming documents (this is what the scaffold currently does).

## Steps

1. `Image` variant: `alt: Option<String>`, add `width: Option<u16>`, `height: Option<u16>` (all `skip_serializing_if = "Option::is_none"`). Update `crates/fireside-engine/src/validation.rs` (the empty-alt warning becomes "image has no alt text" when `None` or empty) and `crates/fireside-tui/src/render/blocks_image.rs` to use width/height as render hints if present (clamp to area; tokens-based styling).
2. `ListItem`: implement custom `Serialize` — if `children.is_empty()`, serialize as a bare string; otherwise as the object form. Deserialization already accepts both.
3. Extend `crates/fireside-core/tests/content_roundtrip.rs`: image with width/height round-trips; flat list serializes to `["a","b"]`; nested list still round-trips as objects.

## Do NOT

- Remove the nested-list deserialization support (documented engine superset; ADR in Task 19).
- Change `CodeBlock` or `ContainerBlock` (already conformant).

## Acceptance

```bash
cargo test --workspace
cargo run -q -p fireside-cli -- validate docs/examples/hello.json   # exit 0
```
