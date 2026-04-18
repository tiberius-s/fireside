---
title: '§1 Introduction'
description: 'Scope, conformance language, design principles, and terminology for Fireside.'
---

## Scope

Fireside defines a portable format for branching presentations and lessons.
Documents are directed graphs whose nodes contain typed content blocks and
explicit traversal rules.

The protocol is renderer-agnostic and runtime-agnostic. It describes what
content exists, how nodes connect, and how presenters move through the graph.

If you want the conceptual model first, read [Mental models](./mental-models/).

## What Fireside is for

Fireside is for content that benefits from:

- branching decisions
- revisits and backtracking
- reusable subflows
- explicit rejoin points
- graph-shaped presentations instead of linear slide stacks

## What Fireside is not for

- visual theme design
- animation inventory
- editor behavior
- file management
- platform-specific chrome

## Conformance

A conforming engine for `0.1.0`:

1. Parses and validates Fireside JSON documents.
2. Implements traversal semantics (`Next`, `Choose`, `Goto`, `Back`).
3. Renders all seven core block kinds.
4. Preserves the protocol's traversal and history rules.

## Design Principles

- Portability first
- Predictable traversal
- Minimal mandatory core
- Explicit edges over implicit sequence
- Protocol semantics before implementation detail

## Terminology

Normative chapters use the technical terms in the glossary:

- `Graph`
- `Node`
- `ContentBlock`
- `Traversal`
- `BranchPoint`
- `BranchOption`

Guides may use conversational aliases when they help onboarding.

## Wire Format Baseline

- UTF-8 JSON
- Kebab-case core property names
- `kind` as ContentBlock discriminator
- JSON Schema 2020-12 machine-readable contract

## Entry points for readers

- [Data model](./data-model/)
- [Traversal](./traversal/)
- [Serialization](./serialization/)
- [Validation](./validation/)
