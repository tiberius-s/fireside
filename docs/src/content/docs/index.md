---
title: 'The Fireside Protocol'
description: 'A portable format for branching presentations and lessons.'
---

> **Version:** 0.1.0

Fireside is a portable, implementation-independent format for branching
presentations and lessons. Documents are directed graphs of nodes containing
content blocks and traversal metadata.

## Choose by Goal

### Tutorial

Learn by building a complete session:

- [Your First Fireside Session](guides/getting-started/)

### How-to Guides

Solve specific authoring problems:

- [Branching Adventures](guides/branching-adventures/)
- [For Designers](guides/for-designers/)

### Reference

Look up protocol definitions and schema details:

- [Specification](spec/introduction/)
- [Graph schema](schemas/graph/)
- [Node schema](schemas/node/)
- [Content blocks schema](schemas/content-blocks/)
- [Domain vocabulary](reference/domain-vocabulary/)
- [Data model quick reference](reference/data-model-quick-reference/)

### Explanation

Understand design rationale and tradeoffs:

- [Design Decisions](explanation/design-decisions/)

## Protocol Snapshot

- Wire format: kebab-case JSON properties
- Block discriminator: `kind`
- Core block kinds: `heading`, `text`, `code`, `list`, `image`, `divider`,
  `container`
- Extension block: `kind: "extension"` with explicit `type`
- Traversal operations: Next, Choose, Goto, Back
- Media type: `application/json`
- Schema dialect: JSON Schema 2020-12

Now that we have a specification, we can define and design engines that implement it. The next chapter defines normative traversal algorithms and state rules for Next, Choose, Goto, and Back operations.
