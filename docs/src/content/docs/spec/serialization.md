---
title: '§6 Serialization'
description: 'JSON wire format, naming conventions, encoding, and schema relationship for Fireside documents.'
---

Fireside documents are serialized as UTF-8 JSON.
This chapter defines the canonical wire format and transport expectations.

## Canonical Format

- Conforming engines MUST accept JSON documents that validate against
  `Graph.json`.
- The root value MUST be a JSON object.
- Property order is not significant.
- Documents MAY be minified or pretty-printed.

## Property and Enum Naming

Core property names use kebab-case.

```json
{
  "speaker-notes": "Pause before the demo.",
  "highlight-lines": [2, 5],
  "show-line-numbers": true,
  "branch-point": {
    "prompt": "Choose a branch",
    "options": [{ "label": "Path A", "target": "path-a" }]
  }
}
```

Enum values also use kebab-case.

```json
{
  "layout": "split-horizontal",
  "transition": "slide-left"
}
```

## `$schema` Field

A document MAY include `$schema` for self-description.
Tools SHOULD use the URI as the schema identifier for validation behavior.

```json
{
  "$schema": "https://fireside.dev/schemas/0.1.0/Graph.json",
  "nodes": [{ "content": [{ "kind": "heading", "level": 1, "text": "Hello" }] }]
}
```

## Media Type

Fireside uses the standard JSON media type:

```text
application/json
```

When served over HTTP, payloads SHOULD include `charset=utf-8`.

## File Extensions

Recommended file extensions:

| Extension        | Description                                 |
| ---------------- | ------------------------------------------- |
| `.fireside.json` | Preferred extension for Fireside documents. |
| `.json`          | Acceptable general JSON extension.          |

Engines MUST NOT require a specific extension if content is valid JSON.

## Character Encoding

- Documents MUST be UTF-8 encoded.
- Engines SHOULD tolerate a UTF-8 BOM.
- Engines SHOULD normalize Node ID comparisons to NFC.

## Minimal Valid Document

```json
{
  "nodes": [{ "content": [] }]
}
```

## Schema Relationship

TypeSpec is the source-of-truth model.
JSON Schema 2020-12 files generated from TypeSpec are the normative machine-
readable contract for validation tooling.
