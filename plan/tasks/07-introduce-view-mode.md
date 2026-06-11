# Task 07 — Introduce `ViewMode` (D9, core only)

**Depends on:** 02
**Crates:** fireside-core
**Phase:** 1

## Goal

Add the spec's `ViewMode` enum and wire it through `Node` and `NodeDefaults` with the spec's cascading resolution. The TUI starts *consuming* it in Task 17 — this task only adds the model.

## Background

`protocol/main.tsp:63-69` defines `ViewMode { default, fullscreen }`, resolved node → graph defaults → built-in `default`. The Rust model has no `view-mode` anywhere (0 grep hits); it has a node-level `layout: Option<Layout>` with 12 variants (`crates/fireside-core/src/model/layout.rs`) — a pre-rewrite concept the spec replaced with view-mode + container layouts.

## Steps

1. New `crates/fireside-core/src/model/view_mode.rs`: `enum ViewMode { #[default] Default, Fullscreen }`, kebab-case serde, `#[serde(other)]`-style tolerance is NOT needed (schema enum is closed — unknown values should fail).
2. `Node`: add `view_mode: Option<ViewMode>` (`rename = "view-mode"`). Keep the existing `layout` field for now (Task 17 migrates the TUI; Task 19's ADR decides its final status).
3. `NodeDefaults` (`graph.rs`): add `view_mode: Option<ViewMode>` alongside the existing fields.
4. Add resolution helpers (this is where the cascade lives — render code must never re-implement it):
   - `Node::resolved_view_mode(&self, defaults: Option<&NodeDefaults>) -> ViewMode`
   - `Node::resolved_transition(&self, defaults: Option<&NodeDefaults>) -> Transition`
5. Unit tests: node-level wins; defaults apply when node is `None`; built-in `Default`/`None` when both absent.

## Do NOT

- Remove or modify the `Layout` enum (Task 17).
- Bake defaults into nodes at load time — that is the bug Task 13 removes; the helpers exist so resolution happens at read time.

## Acceptance

```bash
cargo test -p fireside-core
cargo run -q -p fireside-cli -- validate docs/examples/hello.json   # hello.json's "view-mode": "fullscreen" on code-demo now parses into the model
```
