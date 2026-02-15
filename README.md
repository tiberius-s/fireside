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
- **7 core content blocks** — heading, text, code, list, image, divider, group
- **Extension model** — `x-` prefix for custom blocks (x-table, x-video, x-poll)
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
├── specs/             # Quick-reference data model + vocabulary
├── examples/          # Example .fireside.json files
├── themes/            # TOML theme files
├── src/               # Rust reference implementation
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

## Reference Implementation (Rust)

The Rust codebase is a terminal presentation engine using the TEA (The Elm Architecture)
pattern with ratatui + crossterm.

```bash
cargo build
cargo run -- present examples/hello.json
```

> **Note:** The Rust code is being aligned with the 0.1.0 protocol. Some types
> and field names may still use older vocabulary.

## Build & Test

```bash
# Rust
cargo build
cargo test
cargo clippy -- -D warnings

# TypeSpec → JSON Schema
cd typespec && npm run build

# Documentation
cd docs && npm run build
```

## License

MIT — see [LICENSE](LICENSE) for details.
