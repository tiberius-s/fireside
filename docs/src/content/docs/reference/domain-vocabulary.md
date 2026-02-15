---
title: 'Domain Vocabulary'
description: 'Technical and conversational terminology for Fireside concepts.'
---

## Technical Layer

| Term         | Meaning                          |
| ------------ | -------------------------------- |
| Graph        | Complete directed graph document |
| Node         | Traversable unit of content      |
| NodeId       | Node identifier string           |
| ContentBlock | Discriminated content unit       |
| BranchPoint  | Decision point with options      |
| BranchOption | Single branch choice             |
| Traversal    | Exit behavior from a node        |
| Engine       | Runtime that presents a document |

## Conversational Layer

| Conversational | Technical    |
| -------------- | ------------ |
| Session        | Graph        |
| Moment         | Node         |
| Block          | ContentBlock |
| Question       | BranchPoint  |
| Answer         | BranchOption |
| Flow           | Traversal    |

## Traversal Verbs

- Next
- Choose
- Goto
- Back
