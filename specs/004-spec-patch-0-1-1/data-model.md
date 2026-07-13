# Data Model: Protocol spec patch 0.1.1

No changes to the `Graph`/`Node`/`Traversal` wire model. This feature adds
two new *tooling* entities (not part of the wire protocol) plus one new
diagnostic value.

## Fixture document

A standalone Fireside JSON document (schema-valid, i.e. it would pass
Layer 1) stored under `protocol/fixtures/valid/*.json` or
`protocol/fixtures/invalid/*.json`. Each fixture is deliberately minimal —
just enough graph structure to make exactly one Layer-2 rule fire (or, for
`clean.json`, none at all).

| Field (informal) | Description |
| --- | --- |
| path | Relative path under `protocol/fixtures/`, e.g. `valid/self-loop.json` |
| contents | A complete `Graph` JSON document |
| directory | `valid/` (zero Errors) or `invalid/` (one or more Errors) |

## Fixture expectations map

`protocol/fixtures.expected.json` — a single JSON object read by both
validator test suites.

```json
{
  "valid/clean.json": [],
  "valid/self-loop.json": ["self-loop"],
  "valid/empty-traversal.json": ["empty-traversal"],
  "invalid/dangling-target.json": ["valid-traversal-target"]
}
```

| Field | Type | Description |
| --- | --- | --- |
| key | string | Fixture path relative to `protocol/fixtures/` |
| value | string[] | Sorted, deduplicated list of rule identifiers expected to fire for that fixture, across ALL nodes in the document |

Both the Rust test and the Node corpus runner load this file, run their
respective `validate()` over each fixture, sort+dedupe the rule-ids they
observed, and assert the two sets are equal. The `valid/`-vs-`invalid/`
directory placement is cross-checked separately against each validator's
Error-severity aggregate (`has_errors` in Rust; `errors.length > 0` in JS).

## Empty-traversal diagnostic

A new `Diagnostic` value (Rust) / diagnostic object (JS), following the
existing shape used by every other Layer-2 rule in both validators —
no new fields, just a new `rule` value and message.

| Field | Value |
| --- | --- |
| `severity` | `Warning` |
| `rule` | `"empty-traversal"` |
| `message` | Presenter-facing, names the node, explains `{}` behaves like an absent field (terminal) but is likely accidental |
| `node` / `nodeId` | The affected node's id |

No new Rust types are introduced — this reuses the existing `Diagnostic`
struct in `fireside-engine::validation` and the existing `diagnostic()`
helper in `validate.mjs`.
