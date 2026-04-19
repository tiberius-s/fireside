---
title: 'Data Model Quick Reference'
description: 'Compact reference for Fireside Graph, Node, ContentBlock, and traversal types.'
---

## Root Shape

`Graph` contains metadata plus `nodes`.

For quick lookup, the most important facts are that `nodes` is required and
must contain at least one node, the first node is the graph entry point, and
`defaults` can provide graph-wide `view-mode` and `transition` values.

## Core Types

| Type           | Notes                    |
| -------------- | ------------------------ |
| `Graph`        | Top-level document       |
| `Node`         | Traversable content unit |
| `Traversal`    | Explicit exit behavior   |
| `BranchPoint`  | Decision point           |
| `BranchOption` | One branch choice        |
| `ContentBlock` | Renderable content block |
| `NodeDefaults` | Graph-wide node defaults |
| `NodeId`       | Node identifier scalar   |

## ContentBlock Kinds

| Kind        | Purpose                    |
| ----------- | -------------------------- |
| `heading`   | Titles and hierarchy       |
| `text`      | Prose content              |
| `code`      | Source examples            |
| `list`      | Ordered or unordered items |
| `image`     | Visual assets              |
| `divider`   | Visual separation          |
| `container` | Nested composition         |

## Traversal Operations

| Operation | Effect                                          |
| --------- | ----------------------------------------------- |
| `Next`    | Follows the explicit next edge when one exists. |
| `Choose`  | Resolves a branch option.                       |
| `Goto`    | Jumps directly to a node ID.                    |
| `Back`    | Returns through history.                        |

## Traversal Shapes

- `traversal: "target-id"` for a simple next edge
- `traversal: { "next": "target-id" }` for explicit object form
- `traversal: { "branch-point": { ... } }` for choice-driven traversal
- omitted `traversal` for a terminal node

`Traversal` object form must not contain both `next` and `branch-point`.

## Enums

View mode:

- `default`
- `fullscreen`

Transition:

- `none`
- `fade`

Versions:

- `0.1.0`
