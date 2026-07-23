# Fireside

[![CI](https://github.com/tiberius-s/fireside/actions/workflows/rust.yml/badge.svg)](https://github.com/tiberius-s/fireside/actions/workflows/rust.yml)
[![Docs](https://github.com/tiberius-s/fireside/actions/workflows/docs.yml/badge.svg)](https://tiberius-s.github.io/fireside/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A portable format for branching presentations — and a terminal presenter
anyone can drive.

![fireside demo — a branching deck presented in the terminal](docs/src/assets/demo.gif)

## Install

```bash
git clone https://github.com/tiberius-s/fireside.git
cd fireside
cargo install --path crates/fireside-cli
```

Requires Rust 1.88+ (MSRV). A truecolor terminal is recommended — the
presenter renders with 24-bit RGB, no 256-color fallback — at a comfortable
size of ~80 columns by 24 rows or larger.

## Try it

```bash
fireside demo
```

Press `Space` to move forward. Press `?` any time — the presenter teaches its
own keys. Then make your own, either by hand or from a Markdown outline:

```bash
fireside new my-first-deck
fireside my-first-deck.fireside.json

# or write talk.md as plain Markdown and compile it:
fireside import talk.md
fireside talk.fireside.json
```

Decks live-reload while you present: edit the JSON (or press `e` to
quick-edit a slide's heading, text, or list items in place), save, and the
slide on screen updates immediately.

For the full walkthrough — including live-edit and ASCII art — see the
[Quickstart guide](https://tiberius-s.github.io/fireside/guides/quickstart/).

## See it in action

| | |
| --- | --- |
| **Markdown → deck** — `fireside import` | **Quick-edit** — fix a typo without leaving the terminal |
| ![Compiling a Markdown talk with fireside import, then presenting it](docs/src/assets/import.gif) | ![Quick-editing a slide's heading and a list item, saving in place](docs/src/assets/quick-edit.gif) |
| **Photo → ASCII** — `fireside art image` | **Incremental reveal** — stage content one piece at a time |
| ![Converting a local image into ASCII shading with fireside art image](docs/src/assets/art-image.gif) | ![Revealing a slide's content one piece at a time](docs/src/assets/reveal.gif) |

## What is Fireside?

Fireside is a protocol for defining **branching, non-linear content** as a
directed graph of nodes: presentations, interactive lessons, choose-your-own
demos. The protocol is defined in [TypeSpec](https://typespec.io/), which
generates JSON Schema (2020-12). Any runtime that can parse JSON and maintain
a cursor plus a history stack can build a conforming engine.

- **4 operations** — next, choose, goto, back
- **8 content blocks** — heading, text, code, list, image, divider, container,
  ascii-art
- **Explicit edges** — no implicit sequential fallback; every traversal edge
  is visible in the document
- **Two-layer validation** — JSON Schema for structure, semantic rules for
  graph integrity
- **Incremental reveal** — stage content within a slide, one piece at a time
- **ASCII art** — turn a phrase or a local image into text art with
  `fireside art text`/`fireside art image`, no external tool needed; drop it
  straight into a deck with `fireside new --banner` or an `` ```ascii-art ``
  fence in `fireside import`
- **A presenter built for the room** — quick-edit a slide without leaving the
  terminal, an elapsed-time timer, and resume exactly where you left off
- **A full-screen, mouse-first editor** — `fireside edit` builds and
  restructures a deck as clickable blocks and slides, no JSON required,
  rendering through the same code path the presenter does

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
fireside art text <phrase> generate a text banner to paste in
fireside art image <path>  convert a local image to ASCII shading
fireside edit <file>       open a deck in the full-screen editor
```

`present` takes `--restart` (skip the saved resume position); `validate`
takes `--watch` (re-check on every save); `new` takes `--template` (`linear`,
`branching`, `workshop`), `--author`, and `--banner` (an ASCII title banner
generated from the deck title); `art image` takes `--width`, `--charset`,
`--invert`, and `--no-normalize`. Full flag and exit-code reference:
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
