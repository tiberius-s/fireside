# Task 05 — Transition unknown-value fallback (D10)

**Depends on:** 02
**Crates:** fireside-core
**Phase:** 1

## Goal

Per `protocol/main.tsp:78-84`, the protocol defines exactly two transitions (`none`, `fade`) and says unsupported transitions "SHOULD fall back to none". Parsing a document with an unknown transition string must not fail.

## Background

`crates/fireside-core/src/model/transition.rs` defines 8 variants (none, fade, slide-left, slide-right, wipe, dissolve, matrix, typewriter). The 6 extras are engine features beyond the protocol — they stay (pending Task 19's ADR documents them as non-normative), but an unknown string like `"zoom"` currently fails deserialization.

## Steps

1. Add a fallback variant: `#[serde(other)] Unknown` on the `Transition` enum. In the TUI transition renderer (`crates/fireside-tui/src/ui/transitions.rs`), treat `Unknown` exactly like `None` (instant switch).
2. Ensure `Unknown` is never serialized: when saving, map `Unknown` → `None` (handle in the serializer or in `save_graph` normalization — pick the smallest change and test it).
3. Unit tests: `"transition": "zoom"` parses to `Unknown`; a graph containing it loads, presents, and saves with `"none"`.

## Do NOT

- Delete the 6 extra variants (Task 19 decides their documentation status).
- Touch `ViewMode`/`Layout` (Task 07).

## Acceptance

```bash
cargo test -p fireside-core -p fireside-tui
cargo run -q -p fireside-cli -- validate docs/examples/hello.json   # exit 0
```
