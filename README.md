# Fireside

A portable format for graph-structured presentations.

## What is Fireside?

Fireside is a protocol for defining **branching, non-linear content** as a
directed graph of nodes. It's designed for presentations, interactive lessons,
and anywhere you need structured content with choices.

The protocol is defined in [TypeSpec](https://typespec.io/), which generates
JSON Schema (2020-12). Any runtime that can parse JSON and maintain a cursor
plus a history stack can build a conforming Fireside engine.

## Quick Example

```json
{
  "fireside-version": "0.1.0",
  "title": "My First Presentation",
  "nodes": [
    {
      "id": "welcome",
      "traversal": {
        "branch-point": {
          "prompt": "What interests you?",
          "options": [
            { "label": "Deep Dive", "key": "a", "target": "deep-dive" },
            { "label": "Overview", "key": "b", "target": "overview" }
          ]
        }
      },
      "content": [
        { "kind": "heading", "level": 1, "text": "Welcome!" },
        { "kind": "text", "body": "Choose where to go next." }
      ]
    }
  ]
}
```

## Protocol

- **4 operations** — next, choose, goto, back
- **7 content blocks** — heading, text, code, list, image, divider, container
- **Explicit edges** — no implicit sequential fallback; every traversal edge
  is visible in the document
- **Two-tier validation** — JSON Schema for structure, semantic rules for
  graph integrity
- **Implementation-agnostic** — the protocol defines behavior, not rendering

See [`docs/PROTOCOL-REDESIGN.md`](docs/PROTOCOL-REDESIGN.md) for the full
design document.

## Repository Structure

```text
fireside/
├── protocol/                # Source of truth — TypeSpec domain model
│   ├── main.tsp             # Protocol definition
│   └── tsp-output/schemas/  # Generated JSON Schema files
├── docs/                    # Documentation
│   ├── PROTOCOL-REDESIGN.md # Protocol design document
│   └── examples/            # Example .json graph files
└── crates/                  # Rust reference implementation (rebuilding)
```

## Protocol Version

**0.1.0**

## Build

```bash
# Generate JSON Schemas from TypeSpec
cd protocol && npm run build

# Validate an example against the schema
ajv validate -s protocol/tsp-output/schemas/Graph.json \
  -d docs/examples/hello.json \
  -r 'protocol/tsp-output/schemas/!(Graph).json' \
  --spec=draft2020
```

## License

MIT — see [LICENSE](LICENSE) for details.
