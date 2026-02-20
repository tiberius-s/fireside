# Project Brief — Fireside

## Executive Summary

**Problem Statement**: Developers need a portable, spec-first format for interactive branching presentations that integrates with terminal workflows and version control, without locking content to a proprietary tool.

**Proposed Solution**: Fireside defines a JSON-based directed-graph wire format (`0.1.0`) with a TypeSpec-generated JSON Schema, a mature Rust reference implementation (TUI presenter + editor), and a comprehensive documentation site. Authors write a single `.json` graph; any conforming engine can render it.

**Success Criteria**:

1. A `.json` graph written by one author runs identically in any conforming Fireside engine.
2. The TypeSpec model, generated schemas, Rust implementation, and documentation stay in sync — enforced by CI.
3. `fireside present graph.json` launches a full presenter session with no additional configuration.
4. Users can import any iTerm2 color scheme and use it as a theme.
5. The spec is fully documented in six normative chapters plus three appendices before `1.0.0`.

---

## Scope & Non-Goals

### In Scope

- **Protocol**: JSON wire format (kebab-case), 8 content block kinds, 4 traversal operations, typed extension model, JSON Schema 2020-12 output via TypeSpec.
- **Reference implementation**: `fireside-cli` binary with presenter, editor, validate, scaffold, and theme import subcommands. Full Ratatui TUI.
- **Documentation**: Astro + Starlight site hosted at `/fireside` with spec, schema reference, guides, crates deep-dives, and a "Learn Rust with Fireside" tutorial series.
- **DX tooling**: CI (lint + test + MSRV + audit + TypeSpec drift), cargo deny policy, git hooks, iTerm2 theme import.

### Non-Goals (explicitly deferred)

- HTML export / PDF output — deferred until `1.0.0` milestone.
- YAML/TOML format variants for the wire format.
- GUI tooling; Fireside is terminal-native by design.
- Multi-author / collaboration features.

---

## Domain Model

| Concept          | Description                                                                                                    |
| ---------------- | -------------------------------------------------------------------------------------------------------------- |
| **Graph**        | Root document. Has `nodes[]`, optional `defaults`, `meta`.                                                     |
| **Node**         | A vertex with `content[]`, optional `traversal`, `layout`, `transition`.                                       |
| **ContentBlock** | Discriminated union (`kind`): `heading`, `text`, `code`, `list`, `image`, `divider`, `container`, `extension`. |
| **Traversal**    | Per-node overrides: `next`, `after` (rejoin), `branch-point`.                                                  |
| **BranchPoint**  | Decision point: `prompt` + `options[]` each with a `key` character.                                            |
| **Layout**       | One of 12 layout modes (e.g., `title`, `code-focus`, `split-horizontal`).                                      |
| **Transition**   | One of 8 transition effects (e.g., `fade`, `slide-left`).                                                      |

Wire format invariant: **all JSON property names use kebab-case**.

---

## Architecture

```text
models/main.tsp  →  TypeSpec compile  →  18 JSON Schema files
                                           ↕  kept in sync by CI
crates/fireside-core    — Protocol types (no I/O, no UI)
crates/fireside-engine  — Loader, validation, traversal, session, commands
crates/fireside-tui     — Ratatui TUI, render, theme, design tokens
crates/fireside-cli     — Binary entry point, terminal lifecycle, event loop
```

TEA loop in TUI (invariant): `Event → Action → App::update (only mutation point) → App::view (pure render)`

---

## Key Technical Constraints

- **Rust 2024 edition**, MSRV **1.88** (required by `darling@0.23`).
- All Rust `pub` items have `///` doc comments; `#[must_use]` on all value-returning functions.
- No `unwrap()` / `expect()` in library code; all errors go through `Result`/`Option`.
- Protocol changes within `0.1.x` must be **additive only** (all new fields `Option` or have `#[serde(default)]`).
- Crate boundary rule: `fireside-core` has zero I/O; `fireside-engine` has zero ratatui; `fireside-tui` has zero direct file I/O.
- Wire format is frozen at kebab-case for all property names.

---

## Quality Gates (run before every commit)

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace          # or: cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cd models && npm run build
cd docs && npm run build
```

---

## Current Status (as of 2026-02-20)

All 6 phases of the Fireside Improvement Initiative are complete. The project is in a **maintenance + roadmap** state:

- Protocol `0.1.0` stable, fully tested, fully documented.
- Rust workspace builds cleanly; all tests pass; clippy clean.
- Docs site builds cleanly; 45 pages including full crate deep-dives and Learn Rust series.
- CI: `rust.yml` (lint/test/MSRV), `docs.yml`, `models.yml`, `audit.yml` all green.
- Git hooks installed (pre-commit: fmt; pre-push: clippy + tests).

**Next horizon**: protocol `0.2.0` planning (export formats, richer extension ecosystem), or UX polish pass.
