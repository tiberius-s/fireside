---
title: 'Appendix C — Content Block Reference'
description: 'Non-normative reference for core blocks.'
---

:::note
This appendix is non-normative. It summarizes practical rendering guidance for
content blocks defined in §2.
:::

## Core Blocks

### `heading`

For section titles and hierarchy.

### `text`

For prose and narrative text.

### `code`

For source examples with optional `language`, `highlight-lines`, and
`show-line-numbers`.

### `list`

For ordered or unordered item lists.

### `image`

For visual assets with optional `alt`, `caption`, `width`, and `height`.

### `divider`

For visual separation between block groups.

### `container`

For nested composition.

| Property   | Type             | Required            |
| ---------- | ---------------- | ------------------- |
| `kind`     | `"container"`    | Yes                 |
| `children` | `ContentBlock[]` | Yes (`minItems: 1`) |
| `layout`   | `string?`        | No                  |

## Rendering Notes

- Render core blocks directly.
- Preserve block order in node content arrays.
- Treat container `layout` as a local arrangement hint, not a global theme.
