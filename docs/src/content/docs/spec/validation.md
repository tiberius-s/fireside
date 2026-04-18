---
title: '§4 Validation'
description: 'Schema and graph-integrity validation rules for Fireside documents.'
---

Validation has two layers: schema validation and semantic checks.

## Layer 1: Schema Validation

A conforming document MUST validate against the generated `Graph.json`
schema (JSON Schema 2020-12).

Schema validation enforces:

- required properties
- primitive, object, and array types
- enum value constraints
- `minItems` and scalar constraints
- discriminated content block structure
- traversal union shape

## Layer 2: Semantic Checks

After schema validation, tools SHOULD validate semantic integrity.

### Required Checks

1. Node IDs are unique.
2. All traversal targets reference existing Node IDs.
3. `branch-point.options` contains at least one option.
4. Branch option `key` values are unique per branch point when present.
5. A node MUST NOT have both `next` and `branch-point`.
6. A node with a branch point MUST NOT use string traversal shorthand.

### Recommended Checks

- Unreachable node detection from entry node.
- Self-loop warnings for authoring diagnostics.
- Duplicate labels or confusing branch prompts.
- Cycles that are likely accidental.
- Empty nodes that may need a content block.

## ContentBlock Validation Rules

### Core Blocks

Core kinds (`heading`, `text`, `code`, `list`, `image`, `divider`,
`container`) MUST validate against their specific block schemas.

## Error Severity Guidance

| Severity | Meaning                                        | Engine Behavior        |
| -------- | ---------------------------------------------- | ---------------------- |
| Error    | Document is invalid and unsafe to present.     | Reject load.           |
| Warning  | Document is valid but potentially problematic. | Load with diagnostics. |
| Info     | Optional best-practice feedback.               | Surface in logs.       |

## Failure Handling

- Parse failures: return explicit location and parser message.
- Schema failures: return failing path and rule.
- Integrity failures: identify source node and unresolved target.

Engines SHOULD favor clear, actionable diagnostics over generic failure output.
