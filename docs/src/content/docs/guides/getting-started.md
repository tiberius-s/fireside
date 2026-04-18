---
title: 'Your First Fireside Session'
description: 'Build a small branching Fireside document with core blocks and a container.'
---

This guide builds a small session you can read and present immediately.

## What you will make

A four-node graph with:

1. an opening node
2. a branch point
3. two branch outcomes
4. a shared ending

## Start with the graph

Create `my-session.fireside.json`:

```json
{
  "fireside-version": "0.1.0",
  "title": "My First Fireside Session",
  "nodes": [
    {
      "id": "intro",
      "content": [
        { "kind": "heading", "level": 1, "text": "Welcome" },
        { "kind": "text", "body": "Fireside sessions are branching graphs." }
      ],
      "traversal": "decision"
    },
    {
      "id": "decision",
      "content": [
        {
          "kind": "container",
          "layout": "center",
          "children": [
            { "kind": "heading", "level": 2, "text": "Pick a path" },
            {
              "kind": "text",
              "body": "Choose technical detail or a broader summary."
            }
          ]
        }
      ],
      "traversal": {
        "branch-point": {
          "prompt": "Where do you want to go next?",
          "options": [
            { "label": "Technical", "key": "t", "target": "technical" },
            { "label": "Summary", "key": "s", "target": "summary" }
          ]
        }
      }
    },
    {
      "id": "technical",
      "view-mode": "fullscreen",
      "transition": "fade",
      "traversal": "summary",
      "content": [
        {
          "kind": "code",
          "language": "rust",
          "source": "fn main() {\n    println!(\"Hello, Fireside!\");\n}"
        }
      ]
    },
    {
      "id": "summary",
      "content": [
        {
          "kind": "container",
          "layout": "center",
          "children": [
            { "kind": "heading", "level": 1, "text": "Thanks" },
            {
              "kind": "text",
              "body": "That was a tiny graph with an explicit rejoin."
            }
          ]
        }
      ]
    }
  ]
}
```

## Read the shape

- `intro` goes to `decision`
- `decision` blocks `next` and waits for `choose`
- both branch outcomes wire back to `summary`
- `summary` ends the flow

## Why this structure works

The graph is easy to explain:

- one entry node
- one decision node
- one branch node
- one shared ending

That is the smallest useful branching Fireside session.

## Run it

Use the reference engine or validator once you are ready to test the file.

## What to try next

- add another branch point inside `technical`
- swap the `container` from `center` to `stack`
- change `summary` to a terminal node with no traversal
