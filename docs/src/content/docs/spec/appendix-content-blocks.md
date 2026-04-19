---
title: 'Appendix C — Content Block Reference'
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

`container` is the only core block that nests other blocks, so it carries most
of the layout-oriented guidance in this appendix.

| Property   | Type             | Required            |
| ---------- | ---------------- | ------------------- |
| `kind`     | `"container"`    | Yes                 |
| `children` | `ContentBlock[]` | Yes (`minItems: 1`) |
| `layout`   | `string?`        | No                  |

## Rendering Notes

Render core blocks directly and preserve block order in node content arrays.
For containers, treat `layout` as a local arrangement hint rather than a global
theme instruction.
