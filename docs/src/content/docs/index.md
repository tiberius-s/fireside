---
id: docs
title: 'The Fireside Protocol'
description: 'A portable format for graph-structured presentations.'
---

Fireside is a branching presentation you drive from the terminal — and the
portable protocol underneath it. Write a talk or an interactive lesson as a
graph of nodes instead of a linear slide deck, then present it with a
built-in TUI that live-reloads while you edit.

## Start Here

New to Fireside? This is the fastest path from nothing installed to a deck on
screen:

| Step                                                       | Why it matters                                                    |
| ------------------------------------------------------------ | --------------------------------------------------------------------- |
| [Quickstart](guides/quickstart/)                           | Install, run the demo, and present your first deck.               |
| [Presenting a deck](guides/presenting/)                    | Every key the TUI responds to — map, notes, fullscreen, resume.   |
| [Authoring a Deck in Markdown](guides/authoring-markdown/) | Write a talk as an outline and compile it with `fireside import`.  |

## What Fireside Gives You

A small vocabulary of content blocks, explicit graph edges instead of an
implicit slide order, and a presenter built for the room: quick-edit a slide
without leaving the terminal, an elapsed-time timer, and resume exactly where
you left off. Themes, editor UI, and platform-specific rendering are left to
implementations — Fireside is about document shape and traversal semantics,
not the surrounding application UI.

## For Implementers

Fireside is also a portable, implementation-independent protocol: it defines
how a document is shaped, how content is represented, and how a presenter
moves through that content, without prescribing a specific UI, renderer, or
editor. `protocol/main.tsp` is the normative source for terminology and model
shape; the generated schemas under `protocol/tsp-output/schemas/` are the
validation surface for wire-format examples.

If you're building an engine, or want the conceptual model before the
normative spec, this reading order gives the clearest path:

| Read first                                           | Why it matters                                                    |
| ------------------------------------------------------- | --------------------------------------------------------------------- |
| [Mental models](spec/mental-models/)                  | Builds intuition for why Fireside is a graph rather than a deck.  |
| [Introduction](spec/introduction/)                    | Defines scope, conformance, and terminology.                      |
| [Data model](spec/data-model/)                        | Shows the protocol types and how they fit together.               |
| [Traversal](spec/traversal/)                          | Defines the runtime behavior of moving through a graph.           |
| [Your First Fireside Graph](guides/getting-started/) | Hand-writes a small deck's JSON directly, to see the model up close. |

### Specification Map

The spec and reference pages have distinct jobs. Reading them with that intent
in mind makes the set easier to navigate.

| Section                                                  | Role                                                    |
| --------------------------------------------------------- | ---------------------------------------------------------- |
| [Mental models](spec/mental-models/)                     | Explanation of the conceptual model.                    |
| [Introduction](spec/introduction/)                       | Normative scope, design intent, and reader orientation. |
| [Data model](spec/data-model/)                           | Normative type definitions and relationships.           |
| [Traversal](spec/traversal/)                             | Normative runtime behavior.                             |
| [Serialization](spec/serialization/)                     | Wire-format rules and naming conventions.               |
| [Validation](spec/validation/)                           | Schema and semantic validation expectations.            |
| [Content blocks](spec/appendix-content-blocks/)          | Non-normative rendering guidance for blocks.            |
| [Engine guidance](spec/appendix-engine-guidelines/)      | Non-normative implementation guidance for runtimes.     |
| [Engine extensions](spec/appendix-engine-extensions/)    | Registry of engine-specific extensions and their edge-case behavior. |
| [Domain vocabulary](reference/domain-vocabulary/)        | Canonical and conversational terminology.               |
| [Quick reference](reference/data-model-quick-reference/) | Compact lookup page for the core types.                 |
| [CLI reference](reference/cli/)                          | Every `fireside` subcommand, flag, and exit code.        |
| [Conformance](reference/conformance/)                    | How to check a third-party engine's validator against the shared fixture corpus. |

## What Fireside Does Not Define

Fireside intentionally leaves themes, editor UI, platform-specific rendering,
animation details, and file-management workflows to implementations. The
protocol is about document shape and traversal semantics, not the surrounding
application UI.
