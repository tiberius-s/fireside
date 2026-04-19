---
title: '§1 Introduction'
description: 'Scope, conformance language, design principles, and terminology for Fireside.'
---

## Scope

Fireside defines a portable format for graph-structured presentations.
Documents are directed graphs whose nodes contain typed content blocks and
explicit traversal rules.

The protocol is renderer-agnostic and runtime-agnostic. It describes what
content exists, how nodes connect, and how presenters move through the graph.

If you want the conceptual model first, read [Mental models](./mental-models/).

For documentation maintenance, treat `protocol/main.tsp` as the normative
source of truth for terminology and model shape. Treat the generated schemas in
`protocol/tsp-output/schemas/` as the validation surface. If this
documentation and the protocol files disagree, the docs should be updated to
match the protocol.

## What Fireside is for

Fireside fits presentations that need explicit structure rather than an implied
linear order. In practice, that means it works well when the presenter needs to
branch, revisit earlier material, reuse subflows, or make return paths obvious
instead of relying on hidden sequence.

## Boundaries

The protocol is intentionally narrow. It describes content shape and traversal,
but it does not define product-level concerns like styling or editor behavior.

In other words, Fireside covers the structure of the document and the rules
for moving through it. It leaves the surrounding product experience to the
implementation:

- Graph structure and typed content blocks are part of the protocol.
- Traversal semantics are part of the protocol.
- Wire format and validation are part of the protocol.
- Visual theme design, animation style, editor behavior, file loading and
  saving, and platform-specific UI details are left to implementations.

## Conformance

A conforming engine for `0.1.0` has a small but strict contract. It must be
able to load valid documents, preserve the traversal rules, and render the
core block set.

1. Parses and validates Fireside JSON documents.
2. Implements traversal semantics (`Next`, `Choose`, `Goto`, `Back`).
3. Renders all seven core block kinds.
4. Preserves the protocol's traversal and history rules.

## Design Principles

The specification follows a few design choices consistently.

| Principle                                       | What it means in practice                                             |
| ----------------------------------------------- | --------------------------------------------------------------------- |
| Portability first                               | A document should move between runtimes without changing its meaning. |
| Predictable traversal                           | Navigation behavior should be explicit and stable.                    |
| Minimal mandatory core                          | Engines must agree on a small shared baseline.                        |
| Explicit edges over implicit sequence           | Traversal targets are declared, not inferred from array order.        |
| Protocol semantics before implementation detail | The spec defines behavior, not UI preferences.                        |

## Terminology

Normative chapters use the technical terms in the glossary:

- `Graph`
- `Node`
- `ContentBlock`
- `Traversal`
- `BranchPoint`
- `BranchOption`

Guides may use conversational aliases when they help onboarding, but the
canonical terms remain the source of truth for protocol text, schemas, and
engine APIs.

## Wire Format Baseline

At the wire level, Fireside uses UTF-8 JSON, kebab-case property names for core
fields, `kind` as the `ContentBlock` discriminator, and JSON Schema 2020-12 as
the machine-readable validation contract.

## Entry points for readers

From here, most readers should continue into the [Data model](./data-model/)
chapter and then the [Traversal](./traversal/) chapter. Readers who are working
on tooling or validation should also read [Serialization](./serialization/)
and [Validation](./validation/).
