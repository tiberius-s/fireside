---
title: '§1 Introduction'
description: 'Scope, conformance language, design principles, and terminology for Fireside.'
---

## Scope

Fireside defines a portable format for branching presentations and lessons.
Documents are directed graphs whose nodes contain typed content blocks and
traversal hints.

The protocol is renderer-agnostic and runtime-agnostic.

## Conformance

A conforming engine for `0.1.0`:

1. Parses and validates Fireside JSON documents.
2. Implements traversal semantics (`Next`, `Choose`, `Goto`, `Back`).
3. Renders all seven core block kinds.
4. Handles unsupported extension blocks via fallback behavior.

## Design Principles

- Portability first
- Predictable traversal
- Minimal mandatory core
- Safe extensibility
- Accessibility-aware rendering intent

## Terminology

Normative chapters use technical terms (`Graph`, `Node`, `ContentBlock`).
Guides may use conversational aliases (`Session`, `Moment`, `Block`).

## Wire Format Baseline

- UTF-8 JSON
- Kebab-case core property names
- `kind` as ContentBlock discriminator
- JSON Schema 2020-12 machine-readable contract
