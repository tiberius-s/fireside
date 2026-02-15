---
title: 'Data Model Quick Reference'
description: 'Compact reference for Fireside Graph, Node, ContentBlock, and traversal types.'
---

## Root Shape

`Graph` contains metadata plus `nodes`.

- `nodes` is required and must contain at least one node.
- `defaults` can provide graph-wide `layout` and `transition` defaults.

## ContentBlock Kinds

Core kinds:

- `heading`
- `text`
- `code`
- `list`
- `image`
- `divider`
- `container`

Extension kind:

- `extension` with required `type`

## Traversal Operations

- `Next`
- `Choose`
- `Goto`
- `Back`

## Enums

Layout:
`default`, `center`, `split-horizontal`, `split-vertical`, `fullscreen`,
`align-left`, `align-right`, `focus-code`, `agenda`, `compare`, `image-left`,
`image-right`.

Transition:
`none`, `fade`, `slide-left`, `slide-right`, `slide-up`, `slide-down`,
`dissolve`, `matrix`.
