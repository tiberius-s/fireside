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
- Array order is significant where the data model says it is significant.

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
  "view-mode": "fullscreen",
  "transition": "fade"
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

## Schema Relationship

TypeSpec is the source-of-truth model.
JSON Schema 2020-12 files generated from TypeSpec are the machine-readable
contract for validation tooling.

Protocol authors should treat the generated schemas as the validation surface
and the TypeSpec source as the design surface.
