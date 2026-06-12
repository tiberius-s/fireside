# Fireside

A portable format for branching presentations — and a terminal presenter
anyone can drive.

## Try it

```bash
cargo run -q -- new my-first-deck
cargo run -q -- my-first-deck.fireside.json
```

Press `Space` to move forward. Press `?` any time — the presenter teaches its
own keys.

## What is Fireside?

Fireside is a protocol for defining **branching, non-linear content** as a
directed graph of nodes: presentations, interactive lessons, choose-your-own
demos. The protocol is defined in [TypeSpec](https://typespec.io/), which
generates JSON Schema (2020-12). Any runtime that can parse JSON and maintain
a cursor plus a history stack can build a conforming engine.

- **4 operations** — next, choose, goto, back
- **7 content blocks** — heading, text, code, list, image, divider, container
- **Explicit edges** — no implicit sequential fallback; every traversal edge
  is visible in the document
- **Two-layer validation** — JSON Schema for structure, semantic rules for
  graph integrity

The normative spec lives in `protocol/main.tsp` and
`docs/src/content/docs/spec/`. `docs/examples/hello.json` is the canonical
example document.

## The CLI

```text
fireside <file>            present a deck
fireside validate <file>   check a deck for problems, in plain language
fireside new <name>        create a starter deck
```

## Repository structure

```text
fireside/
├── protocol/        # Source of truth — TypeSpec model, generated schemas,
│                    # and the Node.js semantic validator (validate.mjs)
├── crates/
│   ├── fireside-core/     # protocol data model (parse/serialize)
│   ├── fireside-engine/   # traversal state machine + validation
│   ├── fireside-tui/      # the ratatui presenter
│   └── fireside-cli/      # the `fireside` binary
└── docs/            # Astro/Starlight site: spec + decision records (ADRs)
```

## Development

```bash
cargo test --workspace          # full test suite
cargo clippy --workspace        # lints
cd protocol && npm run build    # regenerate schemas from TypeSpec
node protocol/validate.mjs docs/examples/hello.json
```

Engineering constraints for contributors (human or AI) are in
[`AGENTS.md`](AGENTS.md).

## License

MIT
