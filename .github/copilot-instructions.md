# Copilot Instructions for Fireside

## Project Identity

- **Fireside** is a portable format for branching presentations and lessons.
- This is a spec-first monorepo: the protocol specification is the primary artifact.
- The Rust codebase is a reference implementation (terminal presentation engine).
- Protocol version: **0.1.0**. JSON Schema 2020-12.

## Repository Structure

```
fireside/
├── models/            # Source of truth — TypeSpec domain model
│   ├── main.tsp       # ~400 lines, namespace Fireside, v0.1.0
│   └── tsp-output/schemas/  # 18 generated JSON Schema files
├── docs/              # Astro + Starlight documentation site
│   └── src/content/docs/
│       ├── spec/      # 6 normative chapters + 3 appendices
│       ├── schemas/   # Schema reference pages (graph, node, content-blocks)
│       ├── guides/    # 3 user guides
│       ├── reference/ # Quick-reference domain vocabulary + data model
│       └── decisions/ # Architecture Decision Records (ADRs)
├── schemas/           # (empty — schemas are in models/tsp-output/)

├── docs/examples/     # Example .json files using Fireside format
├── src/               # Rust reference implementation (TEA architecture)
└── memory-bank/       # Project context for AI agents
```

## Domain Model (Fireside Protocol)

### Wire Format: kebab-case JSON

All property names use kebab-case in the JSON wire format:

- `speaker-notes` (not `speakerNotes`)
- `branch-point` (not `branchPoint`)
- `highlight-lines` (not `highlightLines`)
- `show-line-numbers` (not `showLineNumbers`)

Enum values also use kebab-case: `split-horizontal`, `slide-left`, `align-right`.

### Type Vocabulary

| TypeSpec Type    | Purpose                            | JSON `kind` Value |
| ---------------- | ---------------------------------- | ----------------- |
| `Graph`          | Top-level document (root)          | —                 |
| `Node`           | Graph vertex with content          | —                 |
| `NodeId`         | String identifier (minLength: 1)   | —                 |
| `ContentBlock`   | Union of 7 core + extension blocks | —                 |
| `HeadingBlock`   | Heading with level (1-6)           | `"heading"`       |
| `TextBlock`      | Prose text with inline Markdown    | `"text"`          |
| `CodeBlock`      | Source code with language hint     | `"code"`          |
| `ListBlock`      | Ordered or unordered list          | `"list"`          |
| `ImageBlock`     | Image with alt text and caption    | `"image"`         |
| `DividerBlock`   | Horizontal rule                    | `"divider"`       |
| `ContainerBlock` | Generic container with layout hint | `"container"`     |
| `ExtensionBlock` | Extension block with typed payload | `"extension"`     |
| `Traversal`      | Per-node traversal overrides       | —                 |
| `BranchPoint`    | Decision point with options        | —                 |
| `BranchOption`   | Single choice at a branch point    | —                 |
| `Layout`         | Enum: 12 layout modes              | —                 |
| `Transition`     | Enum: 8 transition effects         | —                 |

### Content Discriminator

ContentBlock uses `"kind"` as the discriminator field. Extension blocks use
`"kind": "extension"` with a required `type` identifier (e.g., `"acme.table"`).

### Traversal Operations (4)

| Operation | Description                                         |
| --------- | --------------------------------------------------- |
| Next      | Advance to next node (or `traversal.next` override) |
| Choose    | Select an option at a BranchPoint                   |
| Goto      | Jump to any node by ID                              |
| Back      | Pop history stack, return to previous node          |

## Rust Reference Implementation

### Architecture: TEA (The Elm Architecture)

```text
Event (crossterm) → Action (enum) → App::update(&mut self, action) → View (ratatui)
```

- `src/main.rs` — terminal lifecycle, event poll/draw loop
- `src/app.rs` — `App` state machine (sole owner of navigation state)
- `src/loader.rs` — JSON → deserialization
- `src/render/*` — stateless rendering (consumes model refs → ratatui lines)
- `src/ui/*` — UI composition layer

**Key rule:** state mutation ONLY in `App::update`; render/UI modules are pure.

**Note:** The Rust codebase still uses old vocabulary (Slide, SlideDeck, etc.) and
snake_case/camelCase field names. A code alignment task will update these to match
the Fireside 0.1.0 protocol.

### Conventions

- Error strategy: typed errors (`thiserror`) in library modules; `anyhow::Result` at app boundaries.
- Key handling: maps to `Action` first in `src/config/keybindings.rs`; no direct state mutation in handlers.
- Theme resolution: CLI `--theme` > document `theme` field > default theme.
- Rust 2024 edition patterns.

### Build, Lint, Test

- Build: `cargo build`
- Lint (quality gate): `cargo clippy -- -D warnings`
- Test: `cargo test`
- Format check: `cargo fmt --check`
- Smoke run: `cargo run -- present docs/examples/hello.json`

## Documentation Site

- **Stack:** Astro 5.17 + Starlight 0.32 + astro-mermaid
- **Build:** `cd docs && npm run build`
- **Dev:** `cd docs && npm run dev` (localhost:4321/fireside)

### Spec Structure (6 + 3)

| Chapter | Title                   | Content                                     |
| ------- | ----------------------- | ------------------------------------------- |
| §1      | Introduction            | Conformance, terminology, design principles |
| §2      | Data Model              | Graph, Node, ContentBlock, Traversal types  |
| §3      | Traversal               | Next/Choose/Goto/Back algorithms            |
| §4      | Validation              | JSON Schema rules, graph integrity          |
| §5      | Extensibility           | Typed extension model, fallback contract    |
| §5      | Extensibility           | Typed extension model, fallback contract    |
| §6      | Serialization           | JSON format, media types, file extensions   |
| App A   | Design System           | Tokens, themes, WCAG (non-normative)        |
| App B   | Engine Guidelines       | TEA guarantees, modes (non-normative)       |
| App C   | Content Block Reference | Full catalog + extensions (non-normative)   |

## TypeSpec Workflow

- Edit: `models/main.tsp`
- Compile: `cd models && npm run build`
- Output: 18 JSON Schema 2020-12 files in `models/tsp-output/schemas/`
- After schema changes: update relevant doc pages in `docs/src/content/docs/schemas/`

## When Making Changes

- **Spec changes**: Edit TypeSpec first → compile → update docs
- **Rust changes**: Preserve TEA flow; mutation only in `App::update`
- **Doc changes**: Check `npm run build` in `docs/` for clean output
- **Vocabulary**: Use Fireside terminology; never use Journey/Waypoint/Marker/Crossroads/Hyphae/Slideways
- **JSON examples**: Always use kebab-case property names and `kind` discriminator
- **Extension blocks**: Use `kind: "extension"` and include `type`; include `fallback` in examples when practical
