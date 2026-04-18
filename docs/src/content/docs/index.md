---
id: docs
title: 'The Fireside Protocol'
description: 'A portable format for branching presentations and lessons.'
---

Fireside is a portable, implementation-independent protocol for graph-structured
presentations.

The canonical source of truth is the spec set under `spec/`:

- [Mental models](spec/mental-models/)
- [Introduction](spec/introduction/)
- [Data model](spec/data-model/)
- [Traversal](spec/traversal/)
- [Serialization](spec/serialization/)
- [Validation](spec/validation/)
- [Content blocks](spec/appendix-content-blocks/)
- [Engine guidance](spec/appendix-engine-guidelines/)
- [Domain vocabulary](reference/domain-vocabulary/)
- [Quick reference](reference/data-model-quick-reference/)

## Start Here

If you are new to Fireside, read these in order:

1. [Mental models](spec/mental-models/)
2. [Introduction](spec/introduction/)
3. [Data model](spec/data-model/)
4. [Traversal](spec/traversal/)
5. [Getting started](guides/getting-started/)

## What Fireside Gives You

- Explicit graph edges instead of hidden linear flow
- Branching, revisiting, and rejoining without special cases
- A small core vocabulary that works across runtimes
- JSON Schema for machine validation
- Clear terminology for authors, presenters, and engine authors

## What Fireside Does Not Define

- Themes and visual chrome
- Editor UI or keybindings
- Animation libraries
- File loading and saving strategies
- Platform-specific rendering choices
