---
title: '§4 Validation'
description: 'Schema and graph-integrity validation rules for Fireside documents.'
---

Validation has two layers: schema validation and graph integrity checks.

## Layer 1: Schema Validation

A conforming document MUST validate against `Graph.json` (JSON Schema 2020-12).

Schema validation enforces:

- required properties
- primitive/object/array types
- enum value constraints
- `minItems` and scalar constraints
- discriminated content block structure

## Layer 2: Graph Integrity

After schema validation, engines SHOULD validate semantic integrity.

### Required Checks

1. Node IDs are unique when present.
2. All traversal targets reference existing Node IDs.
3. `branch-point.options` contains at least one option.
4. Branch option `key` values are unique per branch point when present.

### Recommended Checks

- Unreachable node detection from entry node.
- Self-loop warnings for authoring diagnostics.
- Duplicate labels or confusing branch prompts.

## ContentBlock Validation Rules

### Core Blocks

Core kinds (`heading`, `text`, `code`, `list`, `image`, `divider`,
`container`) MUST validate against their specific block schemas.

### Extension Blocks

Extension blocks MUST follow this shape:

- `kind` is exactly `"extension"`
- `type` is present and non-empty
- `fallback` is optional but strongly recommended

Unknown extension payload fields are allowed.

## Error Severity Guidance

| Severity | Meaning                                        | Engine Behavior          |
| -------- | ---------------------------------------------- | ------------------------ |
| Error    | Document is invalid and unsafe to present.     | Reject load.             |
| Warning  | Document is valid but potentially problematic. | Load with diagnostics.   |
| Info     | Optional best-practice feedback.               | Surface in tooling logs. |

## Failure Handling

- Parse failures: return explicit location and parser message.
- Schema failures: return failing path and rule.
- Integrity failures: identify source node and unresolved target.

Engines SHOULD favor clear, actionable diagnostics over generic failure output.
