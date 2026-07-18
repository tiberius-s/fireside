# Fireside

[![CI](https://github.com/tiberius-s/fireside/actions/workflows/rust.yml/badge.svg)](https://github.com/tiberius-s/fireside/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A portable format for branching presentations — and a terminal presenter
anyone can drive.

![fireside demo — a branching deck presented in the terminal](.github/demo.gif)

## Install

```bash
cargo install --path crates/fireside-cli
```

Requires Rust 1.88+ (MSRV). A truecolor terminal is recommended — the
presenter renders with 24-bit RGB, no 256-color fallback — at a comfortable
size of ~80 columns by 24 rows or larger.

## Try it

```bash
cargo run -q -- demo
```

Press `Space` to move forward. Press `?` any time — the presenter teaches its
own keys. Then make your own:

```bash
cargo run -q -- new my-first-deck
cargo run -q -- my-first-deck.fireside.json
```

Decks live-reload while you present: edit the JSON, save, and the slide on
screen updates in place.

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
- **Incremental reveal** — stage content within a slide, one piece at a time
- **A presenter built for the room** — quick-edit a slide without leaving the
  terminal, an elapsed-time timer, and resume exactly where you left off

The normative spec lives in `protocol/main.tsp` and
`docs/src/content/docs/spec/`. `docs/examples/hello.json` is the canonical
example document.

## The CLI

```text
fireside demo              see what a deck can do
fireside <file>            present a deck (live-reloads on save)
fireside validate <file>   check a deck for problems, in plain language
fireside new <name>        create a starter deck
fireside import <file.md>  compile a Markdown talk into a deck
```

`present` takes `--restart` (skip the saved resume position); `validate`
takes `--watch` (re-check on every save); `new` takes `--template` (`linear`,
`branching`, `workshop`) and `--author`. Full flag and exit-code reference:
[CLI Reference](https://tiberius-s.github.io/fireside/reference/cli/).

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
└── docs/            # Astro/Starlight site: spec, guides, and reference
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
