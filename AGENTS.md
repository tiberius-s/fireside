# Fireside Engineering Constraints

Canonical rules for all AI surfaces (Claude Code, Copilot agents, Copilot instructions)
and human contributors. Other instruction files reference this document — do not
duplicate these rules elsewhere.

## Source of Truth

The spec is the source of truth: `protocol/main.tsp`, the generated schemas in
`protocol/tsp-output/schemas/`, and `docs/src/content/docs/spec/`. When code and spec
disagree, the code changes.

## MSRV

The workspace MSRV is **1.88** (`resolver = "3"`, 2024 edition).

- Before recommending a crate, verify its MSRV is ≤ 1.88.
- Before recommending a `std` API, verify it was stabilized before 1.88.
- Flag any proposed dependency that raises the MSRV — this requires an explicit user decision.

## Crate Boundary Rules

| Crate             | Permitted dependencies                                                                         | Explicitly forbidden                               |
| ----------------- | ---------------------------------------------------------------------------------------------- | -------------------------------------------------- |
| `fireside-core`   | `serde`, `serde_json`, `thiserror`                                                             | Any I/O, UI, rendering crate                       |
| `fireside-engine` | `fireside-core`, `serde_json`, `thiserror`, `anyhow` (boundaries), validation libs             | Ratatui, crossterm, clap                           |
| `fireside-tui`    | `fireside-core`, `fireside-engine`, `ratatui`, `crossterm`, `syntect`, `two-face`, `thiserror` | Direct file I/O, business logic duplication        |
| `fireside-cli`    | All workspace crates, `clap`, `anyhow`, `tracing`                                              | State management, rendering outside `fireside-tui` |

Any recommendation that would add a dependency violating these boundaries must be flagged
with an explicit warning and an alternative that respects the boundaries.

## Mandatory Idioms

- **No `unwrap()` or `expect()` in library code.** Return `Result` or `Option` instead.
  Only acceptable in `main()`, test assertions, or `LazyLock` initializers.
- **`#[must_use]`** on every public function that returns a value the caller should act on.
- **`///` doc comments** on every public item. **`//!`** module-level docs on every file.
- **TEA invariant**: `App::update` in `fireside-tui` is the **only** function that mutates
  `App` state. Do not suggest patterns that move mutation elsewhere.
- **Index rebuild**: After any structural mutation to `Graph` (add/remove/reorder nodes),
  `Graph::rebuild_index()` must be called. Flag any code path that skips this.
- Serde attributes use `rename_all = "kebab-case"`.

## Error Handling Stratification

| Layer                      | Correct approach                         |
| -------------------------- | ---------------------------------------- |
| `fireside-core`            | `thiserror` typed errors — `CoreError`   |
| `fireside-engine`          | `thiserror` typed errors — `EngineError` |
| `fireside-tui`             | `thiserror` typed errors — `TuiError`    |
| CLI / application boundary | `anyhow::Result` with context chains     |

Do not suggest `anyhow` inside library crates. Do not suggest raw `Box<dyn Error>`.

## Build and Test Commands

- `cargo test --workspace` — run the full test suite.
- `node protocol/validate.mjs <file>` — validate a Fireside document against the schemas.
- `cd protocol && npm run build` — regenerate schemas from the TypeSpec source.
