---
title: 'Node'
description: 'Schema reference for Node: content, traversal, layout, and transition fields.'
---

A `Node` is a single traversable unit in a Fireside graph.

## Properties

| Property        | Type             | Required | Description                           |
| --------------- | ---------------- | -------- | ------------------------------------- |
| `id`            | `string`         | No       | Unique identifier when present.       |
| `title`         | `string`         | No       | Human-readable node title.            |
| `tags`          | `string[]`       | No       | Node-level categorization tags.       |
| `duration`      | `string`         | No       | Duration hint (ISO 8601 recommended). |
| `layout`        | `Layout`         | No       | Node-level layout override.           |
| `transition`    | `Transition`     | No       | Node-level transition override.       |
| `speaker-notes` | `string`         | No       | Presenter-only notes.                 |
| `traversal`     | `Traversal`      | No       | Navigation overrides for this node.   |
| `content`       | `ContentBlock[]` | Yes      | Renderable blocks for this node.      |

## Layout Values

`default`, `center`, `split-horizontal`, `split-vertical`, `fullscreen`,
`align-left`, `align-right`, `focus-code`, `agenda`, `compare`, `image-left`,
`image-right`.

## Traversal Notes

`traversal.next` and branch option `target` values should resolve to valid node
IDs in the same graph.
