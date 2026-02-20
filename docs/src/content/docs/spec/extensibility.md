---
title: '§5 Extensibility'
description: 'Extension model using explicit extension blocks, typed identifiers, fallback rendering, and compatibility guidance.'
---

Fireside supports evolution without fragmenting baseline interoperability.
Extensions are represented with a dedicated `extension` block kind and an
explicit `type` identifier.

## Design Goals

1. Keep the core protocol small and predictable.
2. Allow domain-specific content to evolve independently.
3. Ensure unsupported extensions still render meaningful output.

## Extension Block Shape

An extension block uses this shape:

| Property         | Type            | Required | Description                                      |
| ---------------- | --------------- | -------- | ------------------------------------------------ |
| `kind`           | `"extension"`   | Yes      | Discriminator for non-core blocks.               |
| `type`           | `string`        | Yes      | Extension identifier (for example `acme.table`). |
| `fallback`       | `ContentBlock?` | No       | Content rendered when extension is unsupported.  |
| `publisher`      | `string?`       | No       | Publisher/namespace metadata.                    |
| `schema-version` | `string?`       | No       | Extension contract version metadata.             |
| `...`            | `unknown`       | No       | Extension-specific payload properties.           |

### Example

```json
{
  "kind": "extension",
  "type": "acme.table",
  "publisher": "acme",
  "schema-version": "1.0.0",
  "headers": ["Name", "Role"],
  "rows": [
    ["Alice", "Engineer"],
    ["Bob", "Designer"]
  ],
  "fallback": {
    "kind": "list",
    "ordered": false,
    "items": ["Name: Alice, Role: Engineer", "Name: Bob, Role: Designer"]
  }
}
```

## Compatibility Contract

- Engines that support `type` SHOULD render extension payload natively.
- Engines that do not support `type` MUST render `fallback` when present.
- If no `fallback` exists, engines SHOULD render a visible placeholder.
- Engines MUST NOT fail parsing solely because unknown extension payload fields
  are present.

## Identifier Guidance

Extension identifiers SHOULD use stable, collision-resistant names, such as:

- Reverse-domain style: `dev.fireside.table`
- Organization scope: `acme.video`
- Product scope: `org.learning.quiz`

Identifiers are compared case-sensitively.

## Evolution Guidance

- Backward-compatible payload additions SHOULD be additive.
- Breaking changes SHOULD use a new `type` value or increment
  `schema-version` with clear migration notes.
- Fallback content SHOULD preserve core semantics of the extension payload.

## Conformance Summary

For protocol version `0.1.0`:

- Core blocks remain fixed and mandatory.
- Extension support is optional per engine.
- Fallback rendering is mandatory for unsupported extensions.

## Security Considerations

- Extension payload fields are **data**, not executable instructions.
- Engines **MUST NOT** evaluate, compile, or execute extension payload values.
- Engines **SHOULD** validate payloads against extension-specific schemas before rendering.
- Engines **SHOULD** apply input-size limits to extension payloads to avoid memory pressure.
- Engines **SHOULD** preserve fallback rendering behavior when payload validation fails.
