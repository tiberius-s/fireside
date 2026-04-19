---
id: docs
title: 'The Fireside Protocol'
description: 'A portable format for graph-structured presentations.'
---

Fireside is a portable, implementation-independent protocol for
graph-structured presentations. It defines how a document is shaped, how
content is represented, and how a presenter moves through that content. It does
not prescribe a specific UI, renderer, or editor.

This documentation set is organized so that you can start with the conceptual
model, move into the normative specification, and then drop into reference or
guides as needed. For maintainers, `protocol/main.tsp` is the normative source
for terminology and model shape, and the generated schemas under
`protocol/tsp-output/schemas/` are the validation surface for wire-format
examples.

## Start Here

If you are new to Fireside, this reading order gives you the clearest path from
concept to implementation:

| Read first                                 | Why it matters                                                   |
| ------------------------------------------ | ---------------------------------------------------------------- |
| [Mental models](spec/mental-models/)       | Builds intuition for why Fireside is a graph rather than a deck. |
| [Introduction](spec/introduction/)         | Defines scope, conformance, and terminology.                     |
| [Data model](spec/data-model/)             | Shows the protocol types and how they fit together.              |
| [Traversal](spec/traversal/)               | Defines the runtime behavior of moving through a graph.          |
| [Getting started](guides/getting-started/) | Walks through a small complete example.                          |

## Specification Map

The spec and reference pages have distinct jobs. Reading them with that intent
in mind makes the set easier to navigate.

| Section                                                  | Role                                                    |
| -------------------------------------------------------- | ------------------------------------------------------- |
| [Mental models](spec/mental-models/)                     | Explanation of the conceptual model.                    |
| [Introduction](spec/introduction/)                       | Normative scope, design intent, and reader orientation. |
| [Data model](spec/data-model/)                           | Normative type definitions and relationships.           |
| [Traversal](spec/traversal/)                             | Normative runtime behavior.                             |
| [Serialization](spec/serialization/)                     | Wire-format rules and naming conventions.               |
| [Validation](spec/validation/)                           | Schema and semantic validation expectations.            |
| [Content blocks](spec/appendix-content-blocks/)          | Non-normative rendering guidance for blocks.            |
| [Engine guidance](spec/appendix-engine-guidelines/)      | Non-normative implementation guidance for runtimes.     |
| [Domain vocabulary](reference/domain-vocabulary/)        | Canonical and conversational terminology.               |
| [Quick reference](reference/data-model-quick-reference/) | Compact lookup page for the core types.                 |

## What Fireside Gives You

At the protocol level, Fireside gives you explicit graph edges instead of an
implicit slide order, a small vocabulary that works across runtimes, and a
machine-readable contract through generated JSON Schema. That combination makes
it practical to build different engines or tools without inventing a new format
for each one.

## What Fireside Does Not Define

Fireside intentionally leaves themes, editor UI, platform-specific rendering,
animation details, and file-management workflows to implementations. The
protocol is about document shape and traversal semantics, not the surrounding
application UI.
