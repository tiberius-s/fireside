---
title: '§6 Serialization'
description: 'JSON wire format, naming conventions, encoding, and schema relationship for Fireside documents.'
---

Fireside documents are serialized as UTF-8 JSON.
This chapter defines the canonical wire format and transport expectations.

The goal of this chapter is to separate the document model from its JSON
representation. The data model says what the protocol means; serialization says
how that meaning appears on the wire.

## Canonical Format

Conforming engines MUST accept JSON documents that validate against
`Graph.json`. The root value must be a JSON object, property order is not
significant, documents may be minified or pretty-printed, and array order is
significant only where the data model says it is significant.

## Property and Enum Naming

Core property names use kebab-case.

The JSON snippets in this chapter are illustrative fragments. A complete
document still has to satisfy the full `Graph.json` schema.

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
  "view-mode": "fullscreen",
  "transition": "fade"
}
```

## Media Type

When transported over HTTP, the standard JSON media type is appropriate:

```text
application/json
```

When served over HTTP, payloads SHOULD include `charset=utf-8`.

## File Extensions

The protocol does not require a specific extension, but these are the
recommended conventions:

| Extension        | Description                                 |
| ---------------- | ------------------------------------------- |
| `.fireside.json` | Preferred extension for Fireside documents. |
| `.json`          | Acceptable general JSON extension.          |

Engines MUST NOT require a specific extension if content is valid JSON.

## Character Encoding

Documents MUST be UTF-8 encoded. Engines SHOULD tolerate a UTF-8 BOM.

## Schema Relationship

TypeSpec is the source-of-truth model. JSON Schema 2020-12 files generated from
TypeSpec are the machine-readable contract for validation tooling.

Protocol authors should treat the generated schemas as the validation surface
and the TypeSpec source as the design surface.
