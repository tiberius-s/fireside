---
title: 'Design Decisions'
description: 'Why Fireside uses a graph model, JSON wire format, deterministic traversal rules, and explicit extension blocks.'
---

This page explains the major architectural choices in Fireside 0.1.0 and the
tradeoffs behind them.

## Why a graph model?

Fireside sessions need branching, rejoin flows, hub-and-spoke navigation, and
cycles. A strict linear deck cannot express those patterns, and a tree cannot
rejoin without content duplication.

A directed graph gives:

- explicit branch edges
- reusable shared nodes
- optional non-linear progression
- linear behavior as a simple subset when no overrides are used

## Why JSON and JSON Schema?

The protocol is designed for broad interoperability across languages and
engines.

JSON was selected because it is:

- universally parseable
- easy to inspect and diff
- directly supported by JSON Schema tooling

JSON Schema 2020-12 provides machine-checkable validation contracts, and
TypeSpec generates the canonical schema artifacts.

## Why deterministic traversal rules?

Conforming engines should behave the same way for the same document and input.
For that reason, traversal defines strict operation semantics for:

- `Next`
- `Choose`
- `Goto`
- `Back`

The model is inspired by TEA-style guarantees (predictable state transitions,
sequential updates, and pure rendering), but no specific framework is required.

## Why seven core content blocks?

Fireside keeps the mandatory core intentionally small so engines remain
implementable while still expressive enough for common sessions.

Core kinds are:

- `heading`
- `text`
- `code`
- `list`
- `image`
- `divider`
- `container`

A compact core reduces compatibility risk and keeps conformance practical.

## Why explicit extension blocks?

Fireside uses explicit typed extension blocks:

- `kind: "extension"`
- required `type`
- optional `fallback`

This approach makes extensions discoverable, validateable, and forward-friendly
for engines that do not implement a custom type.

## Why not adopt Twine or Ink directly?

Twine and Ink are excellent authoring ecosystems, but Fireside is a
format-first protocol with a different goal: portable, engine-agnostic,
schema-validated content.

Fireside borrows useful interaction concepts while preserving:

- runtime independence
- explicit wire-format contracts
- cross-engine conformance semantics

## Tradeoffs and non-goals

Every protocol decision trades flexibility for clarity in some area.

Current tradeoffs include:

- more verbosity than compact custom formats
- explicit schema discipline over free-form authoring
- deterministic traversal over implicit runtime behavior

Current non-goals include:

- prescribing one UI framework
- requiring one implementation language
- coupling the protocol to a single editor or runtime
