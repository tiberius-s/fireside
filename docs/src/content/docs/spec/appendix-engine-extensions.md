---
title: 'Appendix D — Engine Extensions (Non-Normative)'
description: 'Features of the Rust reference engine beyond the Fireside protocol: currently none.'
---

**This appendix is non-normative.** It is the registry for anything the
Rust reference engine (`fireside-core`, `fireside-engine`, `fireside-tui`)
implements beyond the protocol. The dividing line: anything defined in
`protocol/main.tsp` and its generated schemas is protocol; anything else
the engine does must be listed here. See ADR-003 and ADR-004 for the
history.

## Current extensions

**None.** As of the 2026-06-11 presenter-first rewrite (ADR-004), the
reference engine implements protocol 0.1.0 exactly. Earlier engine extras —
extension content blocks, graph `theme`/`font`/`tags` metadata, six
additional transitions, nested list items, and `BranchPoint.id` — were
removed rather than retained.

## Behavior near the protocol's edges

These are documented engine choices within latitude the spec already
grants, not extensions:

- **Inline Markdown in `text.body`** — the spec allows inline Markdown
  without pinning a subset. The engine renders `**bold**`, `*italic*`,
  `` `code` ``, and `[label](url)` links; unmatched markers render
  literally. A link's label renders as a distinctly-styled, clickable OSC 8
  hyperlink on terminals that support it, and as plain readable text
  otherwise; a malformed destination gets a `malformed-link-url` validation
  warning (spec 007 — Modern TUI leverage).
- **Unknown document fields** are ignored on read; the schema layer owns
  strictness (spec §4 Layer 1).
- **`fade` transition** — currently rendered as an instant switch, the
  spec's documented fallback for unsupported transitions.
- **Images** render as a placeholder with the `alt` text (or `src`) and
  caption; terminal graphics protocols are a possible future extension that
  would be registered here.
