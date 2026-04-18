---
title: '§2 Data Model'
description: 'Normative type model for Graph, Node, ContentBlock, traversal types, and enums.'
---

This chapter defines the protocol data model.

## Graph

A document root with metadata, defaults, and ordered nodes.

| Property           | Type           | Required | Notes                           |
| ------------------ | -------------- | -------- | ------------------------------- |
| `fireside-version` | `Versions?`    | No       | Protocol version label.         |
| `title`            | `string?`      | No       | Human-readable graph title.     |
| `author`           | `string?`      | No       | Author metadata.                |
| `date`             | `string?`      | No       | ISO 8601 recommended.           |
| `description`      | `string?`      | No       | Summary metadata.               |
| `version`          | `string?`      | No       | Semantic version of the graph.  |
| `defaults`         | `NodeDefaults?` | No       | Global view mode and transition |
| `nodes`            | `Node[]`       | Yes      | `minItems: 1`. Ordered graph.   |

## Node

A graph vertex containing renderable content and traversal hints.

| Property        | Type          | Required | Notes                          |
| --------------- | ------------- | -------- | ------------------------------ |
| `id`            | `NodeId`      | Yes      | Unique graph identifier.       |
| `title`         | `string?`     | No       | Human-readable node title.     |
| `view-mode`     | `ViewMode?`   | No       | Presentation frame hint.       |
| `transition`    | `Transition?` | No       | Pacing hint when entering.     |
| `speaker-notes` | `string?`     | No       | Presenter-only notes.          |
| `traversal`     | `NodeId?` / `Traversal?` | No | Explicit exit path or branch. |
| `content`       | `ContentBlock[]` | Yes   | Renderable blocks.             |

## ContentBlock Union

Conforming engines MUST support seven core block kinds:

- `heading`
- `text`
- `code`
- `list`
- `image`
- `divider`
- `container`

### ContainerBlock

`container` composes nested blocks.

| Property   | Type             | Required            |
| ---------- | ---------------- | ------------------- |
| `kind`     | `"container"`    | Yes                 |
| `children` | `ContentBlock[]` | Yes (`minItems: 1`) |
| `layout`   | `string?`        | No                  |

## Traversal Types

`Traversal` defines optional overrides:

- `next`: explicit next target
- `branch-point`: branch prompt with `options`

`BranchOption.target` values MUST resolve to existing node IDs.

## ViewMode Enum

`default`, `fullscreen`.

## Transition Enum

`none`, `fade`.
