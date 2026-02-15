---
title: 'Appendix C — Content Block Reference'
description: 'Non-normative reference for core blocks and extension block patterns.'
---

:::note
This appendix is non-normative. It summarizes practical rendering guidance for
content blocks defined in §2.
:::

## Core Blocks

### `heading`

For section titles and hierarchy.

### `text`

For prose and inline markdown-compatible narrative text.

### `code`

For source examples with optional `language`, `highlight-lines`, and
`show-line-numbers`.

### `list`

For ordered/unordered bullet structures.

### `image`

For visual assets with optional `alt` and `caption`.

### `divider`

For visual separation between block groups.

### `container`

For nested composition.

| Property   | Type             | Required |
| ---------- | ---------------- | -------- |
| `kind`     | `"container"`    | Yes      |
| `children` | `ContentBlock[]` | Yes      |
| `layout`   | `string?`        | No       |

## Extension Block Pattern

Extensions are explicit typed blocks.

| Property   | Type            | Required |
| ---------- | --------------- | -------- |
| `kind`     | `"extension"`   | Yes      |
| `type`     | `string`        | Yes      |
| `fallback` | `ContentBlock?` | No       |
| `...`      | `unknown`       | No       |

### Example: Table Extension

```json
{
  "kind": "extension",
  "type": "acme.table",
  "headers": ["Name", "Role"],
  "rows": [
    ["Alice", "Engineer"],
    ["Bob", "Designer"]
  ],
  "fallback": {
    "kind": "list",
    "ordered": false,
    "items": ["Name: Alice, Role: Engineer", "Name: Bob, Role: Designer"]
  }
}
```

## Rendering Notes

- Render core blocks directly.
- For unsupported extensions, render `fallback`.
- If fallback is absent, render a visible placeholder.
- Preserve block order in node content arrays.
