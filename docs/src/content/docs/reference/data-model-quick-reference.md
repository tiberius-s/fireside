---
title: 'Data Model Quick Reference'
description: 'Compact reference for Fireside Graph, Node, ContentBlock, and traversal types.'
---

## Root Shape

`Graph` contains metadata plus `nodes`.

- `nodes` is required and must contain at least one node.
- `defaults` can provide graph-wide `view-mode` and `transition` defaults.

## Core Types

| Type | Notes |
| --- | --- |
| `Graph` | Top-level document |
| `Node` | Traversable content unit |
| `Traversal` | Explicit exit behavior |
| `BranchPoint` | Decision point |
| `BranchOption` | One branch choice |
| `ContentBlock` | Renderable content block |

## ContentBlock Kinds

- `heading`
- `text`
- `code`
- `list`
- `image`
- `divider`
- `container`

## Traversal Operations

- `Next`
- `Choose`
- `Goto`
- `Back`

## Enums

View mode:

- `default`
- `fullscreen`

Transition:

- `none`
- `fade`
