# Copilot Instructions for Fireside

## Project Identity

- **Fireside** — portable JSON format for branching presentations; spec-first monorepo.
- Protocol `0.1.0`. Wire format: kebab-case JSON / JSON Schema 2020-12 (generated via TypeSpec).
- Rust reference implementation: Ratatui + crossterm TUI in TEA (The Elm Architecture) pattern.
- Read `memory-bank/activeContext.md` before starting any multi-step work.

## Repo Map

```
models/           # TypeSpec source (main.tsp) + generated schemas (tsp-output/schemas/)
docs/             # Astro 5 + Starlight 0.32 site (cd docs && npm run dev)
crates/
  fireside-core/  # Protocol types, serde — no I/O, no UI
  fireside-engine/# Loader, validation, traversal, commands, session — no ratatui
  fireside-tui/   # Ratatui UI, themes, config — no direct file I/O
  fireside-cli/   # main, terminal lifecycle, clap — no state or rendering
memory-bank/      # AI agent context (read activeContext.md first)
```

## Key Patterns

**TEA invariant** — `App::update` is the sole mutation point; all render functions are pure `&App`.

**AppMode** — `Presenting ↔ Editing (e/Esc)` · `Presenting → GotoNode (g+digits+Enter)` · `Any ↔ GraphView (v/Esc)`

**Wire format** — All JSON keys kebab-case. `ContentBlock` uses `#[serde(tag = "kind")]`.
`GraphFile` is the serde target; `Graph` adds `node_index` HashMap. Call `rebuild_index()`
after any structural mutation.

**Errors** — `*Error` via `thiserror` in library crates; `anyhow::Result` at CLI boundaries.

**Keybinding dispatch** — keys → `Action` variants in `keybindings.rs` → `App::update`.
Never match keys in `update` directly.

## Build & Quality Gate

```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings
cargo nextest run --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cd models && npm run build    # TypeSpec → JSON Schemas
cd docs && npm run build      # Docs site
cargo run -- present docs/examples/hello.json   # smoke run
```

## Conventions

- MSRV **1.88** · Rust 2024 edition · `resolver = "3"`
- No `unwrap()`/`expect()` in library code — return `Result`/`Option`
- `#[must_use]` on all value-returning functions; `///` on all public items; `//!` on all modules
- `LazyLock` statics for `SYNTAX_SET`/`THEME_SET` in `render/code.rs` — never re-init per render
- Protocol `0.1.x`: additive only — new fields must be `Option` or `#[serde(default)]`
- Vocabulary: never use Journey, Waypoint, Marker, Crossroads, Hyphae, Slideways, Slide, SlideDeck
- Directory index files in Starlight must be named `index.md` (NOT `_index.md`)

## MCP Servers

| Server   | Prefix           | When to use                                                               |
| -------- | ---------------- | ------------------------------------------------------------------------- |
| Context7 | `mcp_context7_*` | Any external crate API — always `resolve-library-id` before `query-docs`  |
| GitHub   | `mcp_github_*`   | CI results, PR/issue search, remote file reads, branch history            |
| Penpot   | `mcp_penpot_*`   | Design artifacts — read `penpot-uiux-design` skill and use context7 first |

## Skills

| Skill                | Use when                                                 |
| -------------------- | -------------------------------------------------------- |
| `prd`                | Planning a feature or protocol version from a vague idea |
| `refactor`           | Structural code improvements without behaviour change    |
| `protocol-change`    | Additive TypeSpec → Rust → docs → tests cascade          |
| `penpot-uiux-design` | UI/UX design work in Penpot                              |

## Agents and Skills

Use these deliberately so research, audits, design work, and implementation do
not blur together.

### Agents

| Agent             | Use when                                                                 |
| ----------------- | ------------------------------------------------------------------------ |
| `SE: UX Designer` | Broad UX, UI, and TUI direction; flow critique; design intent; usability framing |
| `SE: TUI Auditor` | Focused audit of the implemented TUI; reproduce jank; prioritize concrete fixes |
| `Rust-Expert`     | Rust crate APIs, idioms, MSRV checks, crate-boundary validation          |
| `Context7-Expert` | Up-to-date external library or framework documentation                   |

### Routing Rules

- Use `SE: UX Designer` when the problem is "what should this experience do and why is it hard to use?"
- Use `SE: TUI Auditor` when the problem is "what is wrong with the current TUI behavior and how do we fix it?"
- Use `penpot-uiux-design` when the work is primarily inside Penpot: boards, components, tokens, exports, and visual validation.
- Use `Rust-Expert` when the hard part is Rust implementation details rather than UX direction.
- Use `Context7-Expert` when advice depends on current third-party APIs or best practices.

### Preferred Flow For TUI Work

1. `SE: UX Designer` for framing or redesign direction.
2. `SE: TUI Auditor` for concrete behavior review and fix prioritization.
3. `penpot-uiux-design` for Penpot execution and visual validation.
4. `Rust-Expert` for implementation details that need crate or idiom verification.
