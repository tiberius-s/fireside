# System Patterns

## Protocol Source of Truth

- Type definitions live in `models/main.tsp` (TypeSpec).
- JSON Schema 2020-12 output is generated to `models/tsp-output/schemas/` (18 files).
- **Never edit generated schema files directly** — edit TypeSpec, recompile, cascade.
- Protocol changes within `0.1.x` must be additive: `Option` fields or `#[serde(default)]`.

## Content Block Discriminator Pattern

```rust
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ContentBlock { ... }
```

The `"kind"` field is embedded in the JSON object (internal serde tag). 8 variants:
`heading`, `text`, `code`, `list`, `image`, `divider`, `container`, `extension`.

Extension shape: `{ "kind": "extension", "type": "<reverse.domain.id>", optional "fallback" }`.

## Crate Boundary Rules

| Crate             | Owns                                      | Must NOT contain                 |
| ----------------- | ----------------------------------------- | -------------------------------- |
| `fireside-core`   | Protocol types, serde, wire format        | I/O, UI, validation logic        |
| `fireside-engine` | Loading, validation, traversal, undo/redo | Ratatui, crossterm, rendering    |
| `fireside-tui`    | Ratatui UI, rendering, themes, config     | Business logic, direct file I/O  |
| `fireside-cli`    | `main`, terminal lifecycle, clap dispatch | State, rendering, business logic |

## TEA Pattern (The Elm Architecture)

```text
crossterm::Event
    └─► map_key_to_action(key, &app.mode) → Option<Action>
              └─► App::update(&mut self, action)   ← SOLE mutation point
                        └─► terminal.draw(|f| app.view(f))  ← pure render
```

**Invariant**: `App::update` is the only function that ever mutates `App`. All render/UI functions take `&App`.

## AppMode State Machine

```text
Presenting  ←→  Editing       (e / Esc)
Presenting  →   GotoNode      (g, then digits, then Enter)
Any         ←→  GraphView     (v / Esc)
Any         →   Quitting      (q / Ctrl-C)
```

## Error Handling Stratification

- `fireside-core`: `CoreError` via `thiserror` — typed, matchable.
- `fireside-engine`: `EngineError` via `thiserror` — typed, matchable.
- `fireside-tui`: `TuiError` via `thiserror` — typed, matchable.
- Application boundary (CLI, loader entry): `anyhow::Result` — rich context chains.

## Design Token Pattern

`DesignTokens` is the semantic color layer (35+ roles) derived from `Theme` via `DesignTokens::from_theme`. Leaf renderers never receive `&Theme` directly; they receive `&DesignTokens`. `Breakpoint` (Compact/Standard/Wide) drives responsive padding.

## Index Rebuild Pattern

After any structural graph mutation (node add, remove, reorder), you **must** call `Graph::rebuild_index()`. The `node_index: HashMap<NodeId, usize>` is the O(1) lookup table and goes stale if skipped.

## Syntect Asset Pattern

`SyntaxSet` and `ThemeSet` are `LazyLock` statics in `render/code.rs`. Never call `two_face::syntax::extra_newlines()` or `two_face::theme::extra()` outside of their `LazyLock` initializers — doing so re-compiles the syntax and theme databases on every call.

## Docs Architecture Pattern

- `spec/`: normative chapters and appendices (manual sidebar ordering: §1–§6, then A–C).
- `schemas/`: generated-schema-oriented references.
- `reference/`: concise vocabulary, keybindings, quick-reference pages.
- `guides/`: user and developer guides (getting started, branching, themes, extensions).
- `crates/`: expert-level deep dives per Rust crate.
- `explanation/`: understanding-oriented context (Diátaxis quadrant).
- Sidebar: `Crates` section is first, then `Specification`, `Schemas`, `Reference`, `Guides`, `Explanation`.
