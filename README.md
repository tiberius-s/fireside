# Fireside

A portable format for branching presentations and lessons.

## What is Fireside?

Fireside is a protocol for defining **branching, non-linear content** as a directed
graph of nodes. It's designed for presentations, interactive lessons, choose-your-own
adventures, museum kiosks, and anywhere you need structured content with choices.

The protocol is defined as a JSON Schema (2020-12) specification, with TypeSpec as the
source of truth for the domain model.

## Key Features

- **Graph-based navigation** — Nodes connected by traversal edges, not just linear slides
- **4 traversal operations** — Next, Choose, Goto, Back
- **8 core content blocks** — heading, text, code, list, image, divider, container, extension
- **Extension model** — Typed extension blocks with fallback contract
- **Schema-validated** — JSON Schema 2020-12 with TypeSpec generation
- **Implementation-agnostic** — Any runtime that parses JSON can be a Fireside engine

## Quick Example

```json
{
  "$schema": "typespec/tsp-output/schemas/Graph.json",
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
            { "label": "Deep Dive", "target": "deep-dive" },
            { "label": "Overview", "target": "overview" }
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
├── typespec/          # Source of truth — TypeSpec domain model
├── docs/              # Astro + Starlight documentation site
├── crates/            # Cargo workspace members
│   ├── fireside-core/   # Protocol types (Graph, Node, ContentBlock, etc.)
│   ├── fireside-engine/ # Loader, validation, traversal, session
│   ├── fireside-tui/    # Ratatui terminal UI, themes, rendering
│   └── fireside-cli/    # Binary entrypoint (`fireside` command)
├── examples/          # Example .json graph files
└── memory-bank/       # Project context for AI agents
```

## Protocol Version

**0.1.0** — Initial protocol baseline.

## Documentation

Build and view the docs locally:

```bash
cd docs && npm install && npm run dev
```

Visit `http://localhost:4321/fireside/`

Hosted docs: [tiberius-s.github.io/fireside](https://tiberius-s.github.io/fireside/)

## Reference Implementation (Rust)

The Rust codebase is a Cargo workspace with four crates, implementing a terminal
presentation engine using the TEA (The Elm Architecture) pattern with ratatui + crossterm.

```bash
# Build all crates
cargo build

# Present a graph
cargo run -- present examples/hello.json

# Validate a graph file
cargo run -- validate examples/hello.json

# Scaffold a new presentation
cargo run -- new my-talk
```

## Build & Test

```bash
# Rust
cargo build
cargo test --workspace
cargo clippy --workspace -- -D warnings

# TypeSpec → JSON Schema
cd typespec && npm run build

# Documentation
cd docs && npm run build
```

## License

MIT — see [LICENSE](LICENSE) for details.
