---
title: 'The Fireside Protocol'
description: 'A portable format for branching presentations and lessons.'
---

> **Version:** 0.1.0

Fireside is a portable, implementation-independent format for branching
presentations and lessons. Documents are directed graphs of nodes containing
content blocks and traversal metadata.

## Core Concepts

| Technical Term | Conversational Term | Meaning                                   |
| -------------- | ------------------- | ----------------------------------------- |
| Graph          | Session             | Complete presentation or lesson document. |
| Node           | Moment              | One visited unit of content in traversal. |
| ContentBlock   | Block               | Renderable content element inside a node. |
| BranchPoint    | Question            | A decision point with choices.            |
| BranchOption   | Answer              | One selectable branch target.             |
| Traversal      | Flow                | Movement rules between nodes.             |

## Specification Order

The specification is ordered as:

1. §1 Introduction
2. §2 Data Model
3. §3 Traversal
4. §4 Validation
5. §5 Extensibility
6. §6 Serialization
7. Appendix A–C

## Getting Started

- [Your First Fireside Session](guides/getting-started/)
- [Branching Adventures](guides/branching-adventures/)
- [For Designers](guides/for-designers/)

## Quick Reference

- Wire format: kebab-case JSON properties
- Block discriminator: `kind`
- Core block kinds: `heading`, `text`, `code`, `list`, `image`, `divider`,
  `container`
- Extension block: `kind: "extension"` with explicit `type`
- Traversal operations: Next, Choose, Goto, Back
- Media type: `application/json`
- Schema dialect: JSON Schema 2020-12
