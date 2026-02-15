---
title: '§2 Data Model'
description: 'Normative type model for Graph, Node, ContentBlock, traversal types, and enums.'
---

This chapter defines the protocol data model for version `0.1.0`.

## Graph

A document root with metadata, defaults, and ordered nodes.

| Property      | Type            | Required | Notes                                    |
| ------------- | --------------- | -------- | ---------------------------------------- |
| `$schema`     | `string?`       | No       | Schema identifier URI.                   |
| `title`       | `string?`       | No       | Human-readable graph title.              |
| `author`      | `string?`       | No       | Author metadata.                         |
| `date`        | `string?`       | No       | Date metadata.                           |
| `description` | `string?`       | No       | Summary metadata.                        |
| `version`     | `string?`       | No       | Document version metadata.               |
| `tags`        | `string[]?`     | No       | Classification tags.                     |
| `theme`       | `string?`       | No       | Theme hint for engines.                  |
| `font`        | `string?`       | No       | Font hint for engines.                   |
| `defaults`    | `NodeDefaults?` | No       | Global defaults for nodes.               |
| `nodes`       | `Node[]`        | Yes      | `minItems: 1`. Entry point is index `0`. |

## Node

A graph vertex containing renderable content and traversal hints.

| Property        | Type             | Required | Notes                            |
| --------------- | ---------------- | -------- | -------------------------------- |
| `id`            | `NodeId?`        | No       | Must be unique when present.     |
| `layout`        | `Layout?`        | No       | Node layout hint.                |
| `transition`    | `Transition?`    | No       | Node transition hint.            |
| `speaker-notes` | `string?`        | No       | Presenter-only notes.            |
| `traversal`     | `Traversal?`     | No       | Traversal overrides.             |
| `content`       | `ContentBlock[]` | Yes      | Renderable blocks for this node. |

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

### ExtensionBlock

Extensions use explicit typed blocks.

| Property         | Type              | Required |
| ---------------- | ----------------- | -------- |
| `kind`           | `"extension"`     | Yes      |
| `type`           | `string`          | Yes      |
| `fallback`       | `ContentBlock?`   | No       |
| `publisher`      | `string?`         | No       |
| `schema-version` | `string?`         | No       |
| `...`            | `Record<unknown>` | No       |

## Traversal Types

`Traversal` defines optional overrides:

- `next`: explicit next target
- `after`: post-return target override
- `branch-point`: branch prompt with `options`

`BranchOption.target` values MUST resolve to existing node IDs.

## Layout Enum

`default`, `center`, `split-horizontal`, `split-vertical`, `fullscreen`,
`align-left`, `align-right`, `focus-code`, `agenda`, `compare`, `image-left`,
`image-right`.

## Transition Enum

`none`, `fade`, `slide-left`, `slide-right`, `slide-up`, `slide-down`,
`dissolve`, `matrix`.
