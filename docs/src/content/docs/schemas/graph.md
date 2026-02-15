---
title: 'Graph'
description: 'Schema reference for the Graph type — the top-level Fireside document containing metadata and an ordered array of Nodes.'
---

The **Graph** is the top-level document type in the Fireside Protocol. It
represents a self-contained directed graph of content nodes with descriptive
metadata. Every conforming Fireside document is a single Graph object.

A Graph has two conceptual sections: optional **metadata** fields that describe
the presentation, and a required **nodes** array that contains the content. The
first node (`nodes[0]`) is always the entry point.

## Properties

| Property      | Type           | Required | Description                                            |
| ------------- | -------------- | -------- | ------------------------------------------------------ |
| `$schema`     | `string`       | No       | JSON Schema URI for self-describing documents.         |
| `title`       | `string`       | No       | The graph's display name.                              |
| `author`      | `string`       | No       | The graph creator's name.                              |
| `date`        | `string`       | No       | Creation or presentation date (ISO 8601 recommended).  |
| `description` | `string`       | No       | A brief summary of the graph's purpose.                |
| `version`     | `string`       | No       | Semantic version of this graph document.               |
| `tags`        | `string[]`     | No       | Categorization tags for organization and filtering.    |
| `theme`       | `string`       | No       | Default theme name for the engine to use.              |
| `font`        | `string`       | No       | Preferred monospace font family.                       |
| `defaults`    | `NodeDefaults` | No       | Default values applied to all nodes unless overridden. |
| `nodes`       | `Node[]`       | **Yes**  | The ordered array of nodes. `minItems: 1`.             |

All metadata properties are optional. The only required property is `nodes`,
which must contain at least one Node object.

## NodeDefaults

The `defaults` object provides default property values applied to every Node in
the Graph unless the node specifies its own value.

| Property     | Type         | Description                       |
| ------------ | ------------ | --------------------------------- |
| `layout`     | `Layout`     | Default layout for all nodes.     |
| `transition` | `Transition` | Default transition for all nodes. |

When a node omits `layout` or `transition`, the engine resolves in this order:

1. Node-level value
2. Graph-level `defaults`
3. Built-in default (`"default"` for layout, `"none"` for transition)

## Layout Enum

Spatial arrangement strategy for content blocks within a node.

| Value                | Description                                                |
| -------------------- | ---------------------------------------------------------- |
| `"default"`          | Standard top-to-bottom stacking with configurable padding. |
| `"center"`           | Content centered both vertically and horizontally.         |
| `"split-horizontal"` | Two-column split layout.                                   |
| `"split-vertical"`   | Two-row split layout.                                      |
| `"fullscreen"`       | Full terminal area, no chrome.                             |
| `"align-left"`       | Content anchored to the left with right margin.            |
| `"align-right"`      | Content anchored to the right with left margin.            |

Engines MUST support all seven values. Unrecognized values from future protocol
versions should fall back to `"default"`.

## Transition Enum

Animation effect applied when entering a node.

| Value           | Description                          |
| --------------- | ------------------------------------ |
| `"none"`        | No animation, instant switch.        |
| `"fade"`        | Crossfade between nodes.             |
| `"slide-left"`  | New node slides in from the right.   |
| `"slide-right"` | New node slides in from the left.    |
| `"slide-up"`    | New node slides in from the bottom.  |
| `"slide-down"`  | New node slides in from the top.     |
| `"dissolve"`    | Old node dissolves into the new one. |
| `"matrix"`      | Matrix-style character rain effect.  |

Engines that do not support a requested transition should fall back to `"none"`.

## Minimal Example

The smallest valid Fireside document — a single node with one content block:

```json
{
  "nodes": [
    {
      "content": [{ "kind": "heading", "level": 1, "text": "Hello, Fireside" }]
    }
  ]
}
```

## Full Example

A Graph using all metadata fields, defaults, and multiple nodes:

```json
{
  "$schema": "https://fireside.dev/schemas/0.1.0/Graph.json",
  "title": "An Introduction to Fireside",
  "author": "Dana",
  "date": "2026-02-15",
  "description": "A quick tour of the Fireside Protocol.",
  "version": "1.0.0",
  "tags": ["tutorial", "introduction"],
  "theme": "campfire",
  "font": "JetBrains Mono",
  "defaults": {
    "layout": "center",
    "transition": "fade"
  },
  "nodes": [
    {
      "id": "welcome",
      "content": [
        { "kind": "heading", "level": 1, "text": "Welcome" },
        { "kind": "text", "body": "Let's get started." }
      ]
    },
    {
      "id": "overview",
      "layout": "split-horizontal",
      "transition": "slide-left",
      "content": [
        { "kind": "heading", "level": 2, "text": "What You'll Learn" },
        {
          "kind": "list",
          "ordered": true,
          "items": ["The Graph data model", "Content blocks", "Traversal operations"]
        }
      ]
    },
    {
      "id": "end",
      "content": [
        { "kind": "heading", "level": 2, "text": "Thank You" },
        { "kind": "text", "body": "Go build something great." }
      ]
    }
  ]
}
```

## Schema File

The canonical JSON Schema 2020-12 definition for the Graph type is
**Graph.json**, generated by TypeSpec. Related schema files:

- **NodeDefaults.json** — The defaults sub-model.
- **Layout.json** — Layout enumeration.
- **Transition.json** — Transition enumeration.
