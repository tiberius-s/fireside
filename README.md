# Fireside

A portable format for branching presentations and lessons.

## What is Fireside?

Fireside is a protocol for defining **branching, non-linear content** as a directed
graph of nodes. It's designed for presentations, interactive lessons,
choose-your-own adventures, museum kiosks, and anywhere you need structured
content with choices.

The protocol is defined as a JSON Schema (2020-12) specification, with TypeSpec as
the source of truth for the domain model. The Rust codebase is the reference
implementation: a terminal presentation engine with a built-in node editor.

## Key Features

- **Graph-based navigation** — nodes connected by traversal edges, not just linear slides
- **4 traversal operations** — Next, Choose, Goto, Back
- **8 core content blocks** — heading, text, code, list, image, divider, container,
  extension
- **Extension model** — typed extension blocks with fallback contract
- **Interactive editor** — browse and edit your graph without leaving the terminal
- **Graph overview** — visualise all nodes and jump directly to any of them
- **Branching paths** — present choices to your audience with a visual overlay
- **Schema-validated** — JSON Schema 2020-12 with TypeSpec generation
- **Themeable** — JSON themes, iTerm2 `.itermcolors` import, syntect syntax
  highlighting
- **Implementation-agnostic** — any runtime that parses JSON can be a Fireside engine

## Quick Example

```json
{
  "$schema": "models/tsp-output/schemas/Graph.json",
  "title": "My First Session",
  "nodes": [
    {
      "id": "welcome",
      "content": [
        { "kind": "heading", "level": 1, "text": "Welcome!" },
        { "kind": "text", "body": "Choose where to go next." }
      ],
      "traversal": {
        "branch-point": {
          "prompt": "What interests you?",
          "options": [
            { "label": "Deep Dive", "key": "a", "target": "deep-dive" },
            { "label": "Overview", "key": "b", "target": "overview" }
          ]
        }
      }
    }
  ]
}
```

## Repository Structure

```text
fireside/
├── models/            # Source of truth — TypeSpec domain model
│   └── tsp-output/schemas/  # 18 generated JSON Schema files
├── docs/              # Astro + Starlight documentation site
├── crates/            # Cargo workspace members
│   ├── fireside-core/   # Protocol types (Graph, Node, ContentBlock, etc.)
│   ├── fireside-engine/ # Loader, validation, traversal, session, commands
│   ├── fireside-tui/    # Ratatui terminal UI, themes, rendering
│   └── fireside-cli/    # Binary entrypoint (`fireside` command)
├── docs/examples/     # Example .json graph files
└── memory-bank/       # Project context for AI agents
```

## Protocol Version

**0.1.0** — Initial protocol baseline.

## Reference Implementation (Rust)

### Installation

```bash
cargo install --path crates/fireside-cli
```

Or build and run directly from the workspace:

```bash
cargo run -- present docs/examples/hello.json
```

### Commands

```bash
# Present a graph in the terminal
fireside present talk.json
fireside present talk.json --theme nord
fireside present talk.json --start 3   # start at node 3 (1-indexed)

# Open a project directory
fireside open ./my-course/

# Edit a graph or project
fireside edit talk.json
fireside edit                           # opens project in current directory

# Scaffold a new presentation
fireside new my-talk
fireside new my-course --project        # create a full project directory

# Validate structural integrity
fireside validate talk.json

# List available system fonts
fireside fonts

# Import an iTerm2 color scheme as a theme
fireside import-theme ~/Downloads/Nord.itermcolors --name nord
```

### Presentation Mode Keys

| Key                           | Action                                   |
| ----------------------------- | ---------------------------------------- |
| `→` / `Space` / `Enter` / `l` | Next node                                |
| `←` / `h`                     | Previous node                            |
| `g`                           | Go to node by number                     |
| `a`–`f`                       | Choose branch option (at a branch point) |
| `s`                           | Toggle speaker notes                     |
| `?`                           | Toggle help overlay                      |
| `e`                           | Enter editor mode                        |
| `q` / `Esc` / `Ctrl-C`        | Quit                                     |

### Editor Mode Keys

| Key             | Action                                         |
| --------------- | ---------------------------------------------- |
| `j` / `↓`       | Select next node                               |
| `k` / `↑`       | Select previous node                           |
| `PgDn` / `PgUp` | Scroll through node list                       |
| `Home` / `End`  | Jump to first/last node                        |
| `/`             | Search nodes by ID                             |
| `[` / `]`       | Previous/next search hit                       |
| `g`             | Jump to node by number                         |
| `Tab`           | Toggle focus between node list and detail pane |
| `i`             | Edit node text content inline                  |
| `o`             | Edit node speaker notes inline                 |
| `a`             | Append a text block                            |
| `n`             | Add a new node                                 |
| `d`             | Remove the selected node                       |
| `l`             | Open layout picker                             |
| `t`             | Open transition picker                         |
| `v`             | Toggle graph overview overlay                  |
| `u`             | Undo                                           |
| `r`             | Redo                                           |
| `w` / `Ctrl-S`  | Save                                           |
| `?`             | Help overlay                                   |
| `Esc`           | Return to presentation mode                    |

## Build & Test

```bash
# Rust
cargo build
cargo test --workspace
cargo clippy --workspace -- -D warnings

# TypeSpec → JSON Schema
cd models && npm run build

# Documentation site
cd docs && npm install && npm run build
```

Smoke run to verify everything works end to end:

```bash
cargo run -- present docs/examples/hello.json
```

## Documentation

Build and view the docs locally:

```bash
cd docs && npm install && npm run dev
```

Visit `http://localhost:4321/fireside/`

Hosted docs: [tiberius-s.github.io/fireside](https://tiberius-s.github.io/fireside/)

## Crate README Files

Each crate has its own README with architecture detail and Rust-specific
guidance:

- [`crates/fireside-core/`](crates/fireside-core/README.md) — protocol types,
  serde patterns, wire format
- [`crates/fireside-engine/`](crates/fireside-engine/README.md) — traversal
  state machine, command/undo pattern, validation
- [`crates/fireside-tui/`](crates/fireside-tui/README.md) — TEA architecture,
  rendering pipeline, keybindings, themes
- [`crates/fireside-cli/`](crates/fireside-cli/README.md) — command dispatch,
  terminal lifecycle, configuration

## License

MIT — see [LICENSE](LICENSE) for details.
