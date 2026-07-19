---
title: 'Appendix B — Content Block Reference'
description: 'Non-normative reference for core blocks.'
---

This appendix is non-normative. It summarizes practical rendering guidance for
the core content blocks defined in §2 and is meant to complement the formal
data model with a quick rendering-oriented view.

## Core Blocks

| Kind        | Typical use                     | Key fields                                                            |
| ----------- | ------------------------------- | --------------------------------------------------------------------- |
| `heading`   | Titles and hierarchy            | `level`, `text`                                                       |
| `text`      | Prose and narrative copy        | `body`                                                                |
| `code`      | Source examples                 | `source`, optional `language`, `highlight-lines`, `show-line-numbers` |
| `list`      | Ordered or unordered item lists | `items`, optional `ordered`                                           |
| `image`     | Visual assets                   | `src`, optional `alt`, `caption`, `width`, `height`                   |
| `divider`   | Visual separation               | `kind` only                                                           |
| `container` | Nested composition              | `children`, optional `layout`                                         |
| `ascii-art` | Pre-rendered ASCII/text art     | `art`, optional `alt`                                                 |

For `image`, `width` and `height` are measured in terminal cells: `width` in
columns, `height` in rows. Percentage sizing is out of scope for 0.1.0.
Engines MUST clamp requested dimensions to the available content area (see
Appendix A, Image Overflow Handling).

For `list`, `items` entries MAY contain inline Markdown formatting, the
same as `text`'s `body` — the reference renderer runs list items through
the same inline-Markdown path as text blocks.

Every block kind also accepts an optional `reveal` field for incremental
reveal — see [§2 Data Model](/spec/data-model/#the-reveal-field-all-kinds)
and [§3 Traversal](/spec/traversal/#incremental-reveal-precedence). A
container hidden by its own `reveal` value hides all of its children
regardless of their own `reveal` values; `fireside-engine::validation`'s
`reveal-masked-by-container` warning catches the common authoring mistake
of giving a child a lower value than its enclosing container.

`container` is the only core block that nests other blocks, so it carries most
of the layout-oriented guidance in this appendix.

| Property   | Type             | Required            |
| ---------- | ---------------- | ------------------- |
| `kind`     | `"container"`    | Yes                 |
| `children` | `ContentBlock[]` | Yes (`minItems: 1`) |
| `layout`   | `"stack" \| "columns" \| "center"` | No (default `"stack"`) |

For `ascii-art` (added in `0.1.3`), `art` is pre-rendered, plain-text
content — engines render it as-is, centered and sized to its own widest
line, the same treatment the reference renderer already gives a
language-less `code` block. No engine generates or transforms the art at
render time; text-to-banner and image-to-ASCII conversion are authoring-time
concerns (see the reference implementation's `fireside art text`/`fireside
art image` commands). Unlike every other core block, `ascii-art` is not
safely ignorable by an engine older than `0.1.3` — see
[§2 Data Model, AsciiArtBlock](/spec/data-model/#asciiartblock).

## Rendering Notes

Render core blocks directly and preserve block order in node content arrays.
For containers, treat `layout` as a local arrangement hint rather than a global
theme instruction.
