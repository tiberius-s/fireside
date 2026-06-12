# Fireside Engineering Constraints

Canonical rules for all AI surfaces and human contributors. Other instruction
files reference this document — do not duplicate these rules elsewhere.

## Source of Truth

The spec is the source of truth: `protocol/main.tsp`, the generated schemas in
`protocol/tsp-output/schemas/`, and `docs/src/content/docs/spec/`. When code
and spec disagree, the code changes. `docs/examples/hello.json` is the
canonical document — it must parse, validate, and present correctly after
every change.

Never add a field, enum variant, or traversal behavior that is not in the
spec. The reference engine implements protocol 0.1.0 exactly (see ADR-004);
any extension must be specified first and registered in the spec's
"Engine Extensions" appendix.

## Product North Star

The presenter must be usable by non-technical people. Argue every design
decision from the presenter's experience: the footer shows exactly the valid
keys, every blocked action gives feedback, and simplicity beats surface area.
Scope is presenter-first — `present`, `validate`, `new` — per ADR-004; reject
scope additions unless the user asks.

## MSRV

The workspace MSRV is **1.88** (`resolver = "3"`, 2024 edition).

- Before recommending a crate, verify its MSRV is ≤ 1.88.
- Before recommending a `std` API, verify it was stabilized before 1.88.
- Flag any proposed dependency that raises the MSRV — this requires an
  explicit user decision.

## Crate Boundary Rules

| Crate             | Permitted dependencies                                                       | Explicitly forbidden                                |
| ----------------- | ---------------------------------------------------------------------------- | --------------------------------------------------- |
| `fireside-core`   | `serde`, `serde_json`, `thiserror`                                           | Any I/O, UI, validation, or rendering code          |
| `fireside-engine` | `fireside-core`, `thiserror`                                                 | File I/O, ratatui, crossterm, clap, anyhow          |
| `fireside-tui`    | `fireside-core`, `fireside-engine`, `ratatui`, `crossterm`, `unicode-width`, `thiserror` | Direct file I/O, business logic duplication |
| `fireside-cli`    | All workspace crates, `clap`, `anyhow`, `serde_json`                         | State management, rendering outside `fireside-tui`  |

Any recommendation that would add a dependency violating these boundaries must
be flagged with an explicit warning and an alternative that respects them.

## Mandatory Idioms

- **No `unwrap()` or `expect()` in library code.** Return `Result` or `Option`
  instead. Only acceptable in `main()`, test code, or `LazyLock` initializers.
- **`#[must_use]`** on every public function that returns a value the caller
  should act on.
- **`///` doc comments** on every public item. **`//!`** module docs on every
  file.
- **TEA invariant**: `App::update` in `fireside-tui` is the **only** function
  that mutates `App` state. Rendering is pure.
- **All visual styling flows through `theme.rs::Tokens`** — never construct a
  `Style` from raw colors in render code.
- **Engine ops return `Outcome`** — no traversal operation may become a silent
  no-op; the UI must be able to give feedback for every keypress.
- Serde attributes use `rename_all = "kebab-case"`; content blocks use the
  `kind` discriminator.
- Sessions own an **immutable** graph; the node index is built once at
  `Session::new`.

## Error Handling Stratification

| Layer                      | Correct approach                         |
| -------------------------- | ---------------------------------------- |
| `fireside-core`            | `thiserror` typed errors — `CoreError`   |
| `fireside-engine`          | `thiserror` typed errors — `EngineError` |
| `fireside-tui`             | `thiserror` typed errors — `TuiError`    |
| CLI / application boundary | `anyhow::Result` with context chains     |

Do not use `anyhow` inside library crates. Do not use raw `Box<dyn Error>`.

## Testing

- Engine semantics (history invariants, branch gating) are unit tests in
  `fireside-engine/src/session.rs` and `validation.rs`.
- Every user-visible TUI state gets a scenario test in the
  `fireside-tui/src/render/mod.rs` suite: drive real key events through
  `App::update`, render to ratatui's `TestBackend`, assert the screen.
- CLI behavior is covered end-to-end in `fireside-cli/tests/cli_e2e.rs`.
- UI changes additionally get a real-terminal smoke: drive the built binary
  in a detached tmux session (`tmux send-keys` / `capture-pane`).

## Build and Test Commands

- `cargo test --workspace` — full test suite.
- `cargo clippy --workspace --all-targets` — lints (keep silent).
- `node protocol/validate.mjs <file>` — validate a document against the
  semantic rules.
- `cd protocol && npm run build` — regenerate schemas from TypeSpec; commit
  `tsp-output/` (CI enforces this).
- `npm run check --prefix docs` — docs site type/build check.
- After modifying code, run `graphify update .` to keep the knowledge graph
  current.
