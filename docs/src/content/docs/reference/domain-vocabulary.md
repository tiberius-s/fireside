---
title: 'Domain Vocabulary'
description: 'Technical and conversational terminology for Fireside concepts.'
---

## Canonical Terms

| Term | Meaning |
| --- | --- |
| Graph | The complete Fireside document |
| Node | A traversable content unit |
| NodeId | The node identifier string |
| ContentBlock | A discriminated content unit |
| Traversal | The explicit exit behavior from a node |
| BranchPoint | A decision point with options |
| BranchOption | One branch choice at a decision point |
| ViewMode | A presentation frame hint |
| Transition | A pacing hint when entering a node |
| Engine | A runtime that presents a document |

## Conversational Layer

| Conversational | Technical |
| --- | --- |
| Session | Graph |
| Moment | Node |
| Block | ContentBlock |
| Question | BranchPoint |
| Answer | BranchOption |
| Flow | Traversal |

## Traversal Verbs

- Next
- Choose
- Goto
- Back

## Ubiquitous Language Notes

Use the canonical terms in protocol text, schemas, and engine APIs.
Use conversational aliases in tutorials and onboarding guides only.
