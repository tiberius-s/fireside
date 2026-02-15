---
title: 'Branching Adventures'
description: 'How to model non-linear Sessions with three practical branching patterns in Fireside 0.1.0.'
---

Use this guide when you need to design branching flow intentionally, not by
trial and error.

## Before You Start

You should already know:

- how a `Graph` is structured
- how a `branch-point` works
- the four traversal operations: `Next`, `Choose`, `Goto`, `Back`

If you are new to the protocol, start with
[Your First Fireside Session](./getting-started/).

## Pattern 1: Branch and Rejoin

Use this pattern when you want optional detours that converge to one shared
continuation node.

### Shape

```text
intro -> question -> topic-a -> summary
                \-> topic-b --^
```

### Why it works

- `question` contains a `branch-point`
- each branch node sets `traversal.next` to `summary`
- convergence is explicit and deterministic

### Example

```json
{
  "$schema": "https://fireside.dev/schemas/0.1.0/Graph.json",
  "title": "Branch and Rejoin",
  "nodes": [
    {
      "id": "intro",
      "content": [{ "kind": "heading", "level": 1, "text": "Welcome" }],
      "traversal": { "next": "question" }
    },
    {
      "id": "question",
      "traversal": {
        "branch-point": {
          "prompt": "Pick a topic",
          "options": [
            { "label": "Topic A", "key": "a", "target": "topic-a" },
            { "label": "Topic B", "key": "b", "target": "topic-b" }
          ]
        }
      },
      "content": [{ "kind": "heading", "level": 2, "text": "Choose" }]
    },
    {
      "id": "topic-a",
      "traversal": { "next": "summary" },
      "content": [{ "kind": "text", "body": "Details for Topic A." }]
    },
    {
      "id": "topic-b",
      "traversal": { "next": "summary" },
      "content": [{ "kind": "text", "body": "Details for Topic B." }]
    },
    {
      "id": "summary",
      "content": [{ "kind": "heading", "level": 2, "text": "Summary" }]
    }
  ]
}
```

## Pattern 2: Hub and Spoke

Use this pattern when users should choose topics in flexible order and return to
one central menu.

### Shape

```text
hub -> spoke-a -> hub
   -> spoke-b -> hub
   -> spoke-c -> hub
   -> done
```

### Why it works

- `hub` is a menu node with a `branch-point`
- each spoke ends with `traversal.next: "hub"`
- one option exits to `done`

### Example

```json
{
  "$schema": "https://fireside.dev/schemas/0.1.0/Graph.json",
  "title": "Hub and Spoke",
  "nodes": [
    {
      "id": "hub",
      "traversal": {
        "branch-point": {
          "prompt": "What do you want to explore?",
          "options": [
            { "label": "Pyramids", "key": "p", "target": "pyramids" },
            { "label": "Nile", "key": "n", "target": "nile" },
            {
              "label": "Hieroglyphics",
              "key": "h",
              "target": "hieroglyphics"
            },
            { "label": "Done", "key": "d", "target": "done" }
          ]
        }
      },
      "content": [{ "kind": "heading", "level": 1, "text": "Exhibit Menu" }]
    },
    {
      "id": "pyramids",
      "traversal": { "next": "hub" },
      "content": [{ "kind": "text", "body": "Pyramids overview." }]
    },
    {
      "id": "nile",
      "traversal": { "next": "hub" },
      "content": [{ "kind": "text", "body": "Nile overview." }]
    },
    {
      "id": "hieroglyphics",
      "traversal": { "next": "hub" },
      "content": [{ "kind": "text", "body": "Hieroglyphics overview." }]
    },
    {
      "id": "done",
      "content": [{ "kind": "heading", "level": 2, "text": "Thanks" }]
    }
  ]
}
```

## Pattern 3: Open World

Use this pattern when choices create long-running divergence and multiple
possible endings.

### Shape

```text
crossroads -> cave -> dragon -> ending-a
          \-> forest -> hermit -> ending-b
          \-> river -> bridge -> ending-c
```

### Why it works

- no forced rejoin edges
- each node can branch again
- `Back` gives recoverability without flattening the structure

### Example

```json
{
  "$schema": "https://fireside.dev/schemas/0.1.0/Graph.json",
  "title": "Open World",
  "nodes": [
    {
      "id": "crossroads",
      "traversal": {
        "branch-point": {
          "prompt": "Choose your direction",
          "options": [
            { "label": "Cave", "key": "c", "target": "cave" },
            { "label": "Forest", "key": "f", "target": "forest" },
            { "label": "River", "key": "r", "target": "river" }
          ]
        }
      },
      "content": [{ "kind": "heading", "level": 1, "text": "Crossroads" }]
    },
    {
      "id": "cave",
      "traversal": { "next": "dragon" },
      "content": [{ "kind": "text", "body": "You enter the cave." }]
    },
    {
      "id": "dragon",
      "traversal": { "next": "ending-a" },
      "content": [{ "kind": "text", "body": "A dragon appears." }]
    },
    {
      "id": "forest",
      "traversal": { "next": "hermit" },
      "content": [{ "kind": "text", "body": "The forest is quiet." }]
    },
    {
      "id": "hermit",
      "traversal": { "next": "ending-b" },
      "content": [{ "kind": "text", "body": "A hermit offers advice." }]
    },
    {
      "id": "river",
      "traversal": { "next": "bridge" },
      "content": [{ "kind": "text", "body": "You follow the river." }]
    },
    {
      "id": "bridge",
      "traversal": { "next": "ending-c" },
      "content": [{ "kind": "text", "body": "A narrow bridge sways." }]
    },
    {
      "id": "ending-a",
      "content": [{ "kind": "heading", "level": 2, "text": "Dragon Ending" }]
    },
    {
      "id": "ending-b",
      "content": [{ "kind": "heading", "level": 2, "text": "Hermit Ending" }]
    },
    {
      "id": "ending-c",
      "content": [{ "kind": "heading", "level": 2, "text": "Bridge Ending" }]
    }
  ]
}
```

## Pattern Selection Cheat Sheet

- choose **Branch and Rejoin** for optional deep dives
- choose **Hub and Spoke** for menu-like exploration
- choose **Open World** for persistent divergence

## Common Mistakes

- Branch options targeting missing node IDs
- Hub flows with no explicit exit option
- Rejoin patterns relying on implicit array order instead of explicit `next`

## Validate Before Publishing

Run protocol validation and then run the session:

```bash
cargo run -- present my-session.fireside.json
```
