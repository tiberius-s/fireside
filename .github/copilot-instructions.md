# Copilot Instructions for Fireside

## Project Identity

- **Fireside** is a portable format for branching presentations and lessons.
- This is a **spec-first monorepo**: the TypeSpec protocol model is the primary artifact.
- The Rust codebase is a mature reference implementation — a full terminal presentation
  and editing engine using Ratatui + crossterm in a TEA (The Elm Architecture) pattern.
- Protocol version: **0.1.0**. Wire format: JSON Schema 2020-12.

## How to Research Before Implementing

Before writing any new code or making architectural decisions, use available tools in
this order:

1. **Read the codebase first.** The implementation is the ground truth. Read the
   relevant source files before guessing at types, function signatures, or patterns.
   Use `semantic_search` for broad discovery and `grep_search` for exact symbol lookup.

2. **Look up library and API docs via Context7 MCP.** For any external crate (ratatui,
   serde, syntect, clap, crossterm, anyhow, thiserror, plist, etc.), use Context7 to
   retrieve up-to-date documentation and code examples _before_ writing usage code.
   Always call `resolve-library-id` first, then `query-docs` with a specific query.
   Examples of when this is mandatory:
   - Adding a new ratatui widget or layout constraint
   - Using a serde attribute you haven't seen in this codebase before
   - Adding a crossterm event type or terminal mode
   - Working with syntect's `HighlightLines` or `ThemeSet`
   - Any use of `plist`, `font-kit`, or `image` crate APIs

3. **Use GitHub MCP for repository-level research.** When you need to inspect CI
   results, search pull requests, read issues, or look up file contents on the remote,
   use the GitHub MCP tools (`mcp_github_*`) rather than running `gh` CLI commands.
   Prefer `search_*` tools for targeted queries and `list_*` tools for broad pagination.

4. **Verify patterns against existing implementations in this repo.** Before adding a
   new `ContentBlock` variant, read `crates/fireside-core/src/model/content.rs` to
   confirm the serde tag pattern. Before adding a new `Action`, read
   `crates/fireside-tui/src/event.rs`. Before adding a new CLI subcommand, read
   `crates/fireside-cli/src/main.rs`. Consistency with existing patterns is more
   important than novelty.

5. **Check the memory bank.** Read `memory-bank/activeContext.md` and relevant task
   files in `memory-bank/tasks/` before starting any multi-step work. These contain
   architectural decisions and in-progress context that may not be visible in code.

## Repository Structure

```
fireside/
├── models/                    # Source of truth — TypeSpec domain model
│   ├── main.tsp               # Namespace Fireside, v0.1.0
│   └── tsp-output/schemas/    # 18 generated JSON Schema 2020-12 files
├── docs/                      # Astro 5 + Starlight 0.32 documentation site
│   ├── astro.config.mjs       # Sidebar config, disable404Route: true
│   ├── examples/              # Example .json graph files (hello.json)
│   └── src/content/docs/
│       ├── spec/              # 6 normative chapters + 3 appendices + migration
│       ├── schemas/           # Schema reference (graph, node, content-blocks)
│       ├── guides/            # User and developer guides
│       ├── reference/         # Quick-reference vocabulary + keybindings
│       ├── crates/            # Expert deep-dives per Rust crate
│       └── explanation/       # Understanding-oriented context (Diátaxis)
├── crates/
│   ├── fireside-core/         # Protocol types only — no I/O, no UI
│   ├── fireside-engine/       # Loader, validation, traversal, commands, session
│   ├── fireside-tui/          # Ratatui TUI — App, render, ui, theme, config
│   └── fireside-cli/          # Binary — `fireside` command, terminal lifecycle
├── memory-bank/               # Project context for AI agents (read this!)
│   ├── activeContext.md       # Current focus and recent changes
│   ├── progress.md            # What works and what's left
│   └── tasks/                 # Per-task files (TASK001–TASK012)
├── .cargo/config.toml         # Linker optimisations (ld_prime on macOS)
└── .githooks/                 # Git hooks (install once: bash .githooks/install.sh)
```

## Crate Responsibilities

Each crate has a strict responsibility boundary. Do not add dependencies that violate
these boundaries:

| Crate             | Owns                                              | Must NOT contain                  |
| ----------------- | ------------------------------------------------- | --------------------------------- |
| `fireside-core`   | Protocol types, serde, wire format                | I/O, UI, validation logic, engine |
| `fireside-engine` | Loading, validation, traversal, undo/redo session | Ratatui, crossterm, rendering     |
| `fireside-tui`    | Ratatui UI, rendering, themes, settings           | Business logic, direct file I/O   |
| `fireside-cli`    | `main`, terminal lifecycle, clap dispatch         | State, rendering, business logic  |

## Domain Model (Fireside Protocol)

### Wire Format: kebab-case JSON

All JSON property names use **kebab-case**. Serde derives handle this with
`#[serde(rename_all = "kebab-case")]` on structs and enums, plus explicit
`#[serde(rename = "...")]` on variants that don't map cleanly by convention.

| Rust field name     | JSON wire name        |
| ------------------- | --------------------- |
| `speaker_notes`     | `"speaker-notes"`     |
| `branch_point`      | `"branch-point"`      |
| `highlight_lines`   | `"highlight-lines"`   |
| `show_line_numbers` | `"show-line-numbers"` |
| `extension_type`    | `"type"`              |

Enum values also use kebab-case in JSON: `"split-horizontal"`, `"slide-left"`,
`"align-right"`, `"code-focus"`.

### Type Vocabulary

| Rust Type        | Purpose                                                         | JSON `kind`   |
| ---------------- | --------------------------------------------------------------- | ------------- |
| `Graph`          | Runtime top-level document (has `node_index` HashMap)           | —             |
| `GraphFile`      | Wire-format serde target (maps 1:1 to JSON)                     | —             |
| `GraphMeta`      | Extracted document metadata                                     | —             |
| `NodeDefaults`   | Default layout/transition for all nodes                         | —             |
| `Node`           | Graph vertex with content and traversal                         | —             |
| `NodeId`         | `type alias for String` (minLength: 1 enforced in validation)   | —             |
| `ContentBlock`   | Discriminated union of 8 content types                          | via `"kind"`  |
| `HeadingBlock`   | Heading with level 1–6                                          | `"heading"`   |
| `TextBlock`      | Prose text with inline Markdown                                 | `"text"`      |
| `CodeBlock`      | Source code with language, highlight-lines, line numbers        | `"code"`      |
| `ListBlock`      | Ordered or unordered list of `ListItem`s                        | `"list"`      |
| `ImageBlock`     | Image with alt text and optional caption                        | `"image"`     |
| `DividerBlock`   | Horizontal rule                                                 | `"divider"`   |
| `ContainerBlock` | Generic container with layout hint and children                 | `"container"` |
| `ExtensionBlock` | Typed extension with fallback and payload                       | `"extension"` |
| `Traversal`      | Per-node traversal overrides (`next`, `after`, `branch-point`)  | —             |
| `BranchPoint`    | Decision point with prompt and options                          | —             |
| `BranchOption`   | Single choice: `label`, `key`, `target`, optional `description` | —             |
| `Layout`         | Enum: 12 layout modes                                           | —             |
| `Transition`     | Enum: 8 transition effects                                      | —             |

### `ContentBlock` Discriminator Pattern

```rust
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ContentBlock { ... }
```

The `"kind"` field is embedded in the JSON object. Extension blocks always use
`"kind": "extension"` plus a required `"type"` identifier (reverse-domain convention,
e.g. `"acme.table"`).

### `GraphFile` → `Graph` Boundary

`GraphFile` is the serde target (matches JSON wire format). `Graph` is what the engine
uses at runtime — it adds `node_index: HashMap<NodeId, usize>` for O(1) ID lookup and
applies `NodeDefaults` to nodes. The only construction path is `Graph::from_file`.

After any structural mutation (node add, remove, reorder), the index must be
reconstructed via `Graph::rebuild_index()`.

### Traversal Operations (4)

| Operation | Engine method             | Description                                                                                  |
| --------- | ------------------------- | -------------------------------------------------------------------------------------------- |
| Next      | `TraversalEngine::next`   | Advance — respects `traversal.next` override, then `traversal.after` rejoin, then sequential |
| Choose    | `TraversalEngine::choose` | Select a `BranchOption` by `key` character                                                   |
| Goto      | `TraversalEngine::goto`   | Jump to any node by index                                                                    |
| Back      | `TraversalEngine::back`   | Pop history stack (capped at 256 entries, uses `VecDeque`)                                   |

## Rust Reference Implementation

### Architecture: TEA (The Elm Architecture)

```text
crossterm::Event
    └── map_key_to_action(key, &app.mode) → Option<Action>
              └── App::update(&mut self, action)   ← SOLE mutation point
                        └── terminal.draw(|f| app.view(f))   ← pure render
```

**Key rule:** `App::update` is the only function that mutates `App` state. All render
and UI functions receive `&App` and produce ratatui output. This is a hard invariant.

### Crate Module Maps

**`fireside-core/src/`**

```
lib.rs, error.rs, model/{mod,branch,content,graph,layout,node,transition,traversal}.rs
```

**`fireside-engine/src/`**

```
lib.rs, error.rs, commands.rs, loader.rs, session.rs, traversal.rs, validation.rs
```

**`fireside-tui/src/`**

```
lib.rs, app.rs, error.rs, event.rs, theme.rs
config/{keybindings,settings}.rs
design/{tokens,templates,fonts,iterm2}.rs
render/{markdown,code,layout}.rs
ui/{presenter,editor,graph,branch,help,progress}.rs
```

**`fireside-cli/src/`**

```
main.rs, commands/{mod,fonts,project,scaffold,session,theme,validate}.rs
```

### Key Implementation Patterns

**Error handling stratification:**

- `fireside-core`: `CoreError` via `thiserror` — typed, matchable
- `fireside-engine`: `EngineError` via `thiserror` — typed, matchable
- `fireside-tui`: `TuiError` via `thiserror` — typed, matchable
- Application boundaries (loader, CLI): `anyhow::Result` — rich context chains

**Keybinding dispatch:**
Keys are mapped to `Action` enum variants in `config/keybindings.rs` before reaching
`App::update`. No key handling in `update` directly — only `Action` matching. Mode
(`AppMode::Presenting`, `AppMode::Editing`, `AppMode::GotoNode`) determines which
keybinding map is active.

**`AppMode` transitions:**

```text
Presenting  ←→  Editing       (e / Esc)
Presenting  →   GotoNode      (g, then digits, then Enter)
Any         ←→  GraphView     (v / Esc)
Any         →   Quitting      (q / Esc / Ctrl-C)
```

**Theme resolution order (highest priority first):**
CLI `--theme` flag → document `theme` metadata field → user config
`~/.config/fireside/config.json` → `Theme::default()`

**`syntect` assets are cached as statics:**
`SYNTAX_SET` and `THEME_SET` are `LazyLock` statics in `render/code.rs`.
Do not call `two_face::syntax::extra_newlines()` or `two_face::theme::extra()` outside
of their initializers.

**Image path security:**
`local_image_path` in `render/markdown.rs` sanitizes paths: relative paths are
canonicalized and verified to remain within the document's base directory. Paths
containing `..` are rejected before canonicalization.

### Build, Lint, Test

```bash
# Full quality gate (run before every commit)
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace          # preferred; fallback: cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Install git hooks once per checkout
bash .githooks/install.sh

# TypeSpec → JSON Schema
cd models && npm run build

# Docs site
cd docs && npm run dev       # localhost:4321/fireside
cd docs && npm run build     # static build check

# Smoke run
cargo run -- present docs/examples/hello.json
```

### Conventions

- Rust 2024 edition. `resolver = "3"` workspace. MSRV **1.88** (required by `darling@0.23`).
- `#[must_use]` on all functions that return a value the caller should act on.
- Doc comments (`///`) on every public item. Module-level `//!` docs on every file.
- `pretty_assertions` in dev-dependencies for readable test failure messages.
- `tracing::warn!` instead of silent failures in rendering code.
- No `unwrap()` or `expect()` in library code — return `Result` or `Option`.
- New nodes must be accessible via `node_by_id` after any editor command — always
  call `rebuild_index` after structural graph mutations.

## Documentation Site

- **Stack:** Astro 5 + Starlight 0.32 + astro-mermaid
- **Build:** `cd docs && npm run build`
- **Dev:** `cd docs && npm run dev` (localhost:4321/fireside)
- `disable404Route: true` in `astro.config.mjs`; custom `404.md` used instead
- Sidebar uses **manual ordering** throughout; `Crates` section comes first.
- Directory index files must be named `index.md` (NOT `_index.md`) for Starlight routing.

## TypeSpec Workflow

When the protocol model changes:

1. Edit `models/main.tsp`
2. Run `cd models && npm run build` — regenerates all schemas in `tsp-output/schemas/`
3. Update `crates/fireside-core/src/model/` structs to match
4. Update relevant `docs/src/content/docs/` pages
5. Update `docs/examples/hello.json` if needed

Never edit the generated JSON Schema files directly.

## When Making Changes

- **Protocol changes:** TypeSpec first → compile → update Rust structs → update docs.
  All 0.1.x changes must be additive (new fields must be `Option` or have `#[serde(default)]`).
- **Rust changes:** Preserve TEA flow — mutation only in `App::update`.
- **New `ContentBlock` variant:** Add to both TypeSpec and Rust enum simultaneously.
  Add a JSON example. Add a round-trip test in `crates/fireside-core/tests/`.
- **New keybinding:** Add `Action` variant → add arm in `keybindings.rs` → handle in
  `App::update` → update help overlay text in `ui/help.rs` → update keybinding reference doc.
- **New CLI subcommand:** Add `Command` variant in `main.rs` → add handler in
  `commands/` → add e2e test in `crates/fireside-cli/tests/cli_e2e.rs`.
- **Doc changes:** Run `npm run build` in `docs/` for clean output before committing.
- **Vocabulary:** Use Fireside terminology exclusively. Never use: Journey, Waypoint,
  Marker, Crossroads, Hyphae, Slideways, Slide, SlideDeck.
- **JSON examples:** Always use kebab-case property names and `kind` discriminator.
  Include `key` on every `BranchOption`. Include `fallback` on `Extension` blocks where practical.

### Spec Structure (6 + 3)

| Chapter | Title                   | Content                                     |
| ------- | ----------------------- | ------------------------------------------- |
| §1      | Introduction            | Conformance, terminology, design principles |
| §2      | Data Model              | Graph, Node, ContentBlock, Traversal types  |
| §3      | Traversal               | Next/Choose/Goto/Back algorithms            |
| §4      | Validation              | JSON Schema rules, graph integrity          |
| §5      | Extensibility           | Typed extension model, fallback contract    |
| §6      | Serialization           | JSON format, media types, file extensions   |
| App A   | Design System           | Tokens, themes, WCAG (non-normative)        |
| App B   | Engine Guidelines       | TEA guarantees, modes (non-normative)       |
| App C   | Content Block Reference | Full catalog + extensions (non-normative)   |

## Agentic Tools & Skills Registry

### Available MCP Servers

| Server       | Tool prefix      | Use when                                                                                                                      |
| ------------ | ---------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| **Context7** | `mcp_context7_*` | Looking up any external crate or library API before writing code. Always `resolve-library-id` first.                          |
| **GitHub**   | `mcp_github_*`   | CI status, PR searches, issue management, remote file reads, branch/commit history. Prefer over `gh` CLI for structured data. |

#### Context7 — usage rules

- Call `mcp_context7_resolve-library-id` with the crate name before any `query-docs` call.
- Provide a specific, detailed query — not just a crate name (e.g., `"ratatui Paragraph wrapping"` not `"ratatui"`).
- Do not call `query-docs` more than 3 times per question; use the best result you have.
- Mandatory for: any crate API you haven't verified in the current session.

#### GitHub MCP — usage rules

- Use `search_*` tools for targeted queries (keywords, filters, author, labels).
- Use `list_*` tools for broad enumeration with pagination (batches of 5–10 items).
- Use `minimal_output: true` where available to reduce context consumption.
- Prefer GitHub MCP over spawning `gh` CLI commands when the operation maps to a structured API call.
- Use for: checking CI run results, reading remote file contents, searching PRs/issues, listing branches.

### When to invoke a Skill

| Skill             | Invoke when                                                                                               |
| ----------------- | --------------------------------------------------------------------------------------------------------- |
| `prd`             | Planning a new feature, protocol version, or user-facing initiative from a vague idea                     |
| `refactor`        | Improving code structure without behavior change (extract function, rename, reduce god object)            |
| `adr`             | Documenting a significant architectural decision (new crate, protocol breaking change, dependency choice) |
| `protocol-change` | Making additive changes to Fireside protocol types end-to-end (TypeSpec → Rust → docs → tests)            |

### When to use subagents vs. direct execution

Use `runSubagent` when:

- Broad multi-file research is needed before implementation (e.g., "find all callers of X" across a large codebase).
- A task is clearly separable: one agent researches Context7 docs, another implements.
- You need to parallelize work on independent subsystems without context collision.

Use the **Context7 agent** (`.github/agents/context7.agent.md`) when:

- Working with any external crate API you haven't seen in this codebase yet (ratatui, crossterm, serde, plist, syntect, clap, etc.).
- Always call `resolve-library-id` first, then `query-docs` with a specific query.

Use the **Rust expert agent** (`.github/agents/rust-expert.agent.md`) when:

- Evaluating a performance tradeoff, unsafe usage, lifetime complexity, or crate selection.

### Model routing guidance

- **Complex multi-step planning** (protocol design, initiative scoping): use `prd` skill first, then spawn implementation subagents per phase.
- **Mechanical cascades** (TypeSpec field → core struct → engine test → doc): protocol-change skill exists but many simple edits can be done directly without invoking any skill.
- **Research-heavy unknowns** (new crate API, unfamiliar Rust idiom): resolve via Context7 before writing any code.
- **Memory bank updates**: read `memory-bank/activeContext.md` + `progress.md` first; apply PRD structure to `projectbrief.md` when scope changes.
