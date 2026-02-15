---
title: 'Your First Fireside Session'
description: 'Build a small branching Fireside document with core blocks and a container.'
---

This guide creates a minimal but complete branching session.

## What You Will Build

A three-node document that includes:

1. Intro node
2. Branch question node
3. Two branch outcomes

It also demonstrates a `container` block and an `extension` fallback pattern.

## Step 1: Start a Document

Create `my-session.fireside.json`:

```json
{
  "$schema": "https://fireside.dev/schemas/0.1.0/Graph.json",
  "title": "My First Fireside Session",
  "nodes": [
    {
      "id": "intro",
      "content": [
        { "kind": "heading", "level": 1, "text": "Welcome" },
        { "kind": "text", "body": "Fireside sessions are branching graphs." }
      ],
      "traversal": { "next": "decision" }
    },
    {
      "id": "decision",
      "content": [
        {
          "kind": "container",
          "layout": "stack",
          "children": [
            { "kind": "heading", "level": 2, "text": "Pick a path" },
            { "kind": "text", "body": "Choose technical depth or business summary." }
          ]
        }
      ],
      "traversal": {
        "branch-point": {
          "prompt": "Where do we go next?",
          "options": [
            { "label": "Technical", "target": "technical", "key": "t" },
            { "label": "Business", "target": "business", "key": "b" }
          ]
        }
      }
    },
    {
      "id": "technical",
      "content": [{ "kind": "code", "language": "rust", "source": "println!(\"hello\");" }]
    },
    {
      "id": "business",
      "content": [
        {
          "kind": "extension",
          "type": "acme.metric-card",
          "value": "42%",
          "label": "Adoption uplift",
          "fallback": {
            "kind": "text",
            "body": "Adoption uplift: 42%"
          }
        }
      ]
    }
  ]
}
```

## Step 2: Run It

Use the reference engine:

```bash
cargo run -- present my-session.fireside.json
```

## Step 3: Navigate

Try these operations:

- `Next`
- `Choose` (branch options)
- `Back`
- `Goto` (by node ID)

## Next Steps

- Add `layout` hints like `focus-code` or `compare`.
- Add images and lists to enrich branch outcomes.
- Add more extension types with robust fallback blocks.
