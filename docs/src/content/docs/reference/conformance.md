---
title: 'Conformance'
description: 'How the reference validators are tested for agreement, and how a third-party engine can check its own semantic validation against the same corpus.'
---

Fireside has two reference validators — `fireside-engine` (Rust) and
`protocol/validate.mjs` (Node) — that are meant to implement the exact same
Layer-2 semantic rules from [§4 Validation](/spec/validation/). Rather than
asking readers to trust that the two implementations agree, both are run
against a shared, versioned fixture corpus and asserted to fire the same
rule IDs on the same documents. A third-party engine can use the same
corpus to check its own conformance.

## The corpus

`protocol/fixtures/` holds one Fireside document per fixture, split into two
directories:

- `protocol/fixtures/valid/*.json` — documents with **no error-severity**
  diagnostic (they may still have warnings or info).
- `protocol/fixtures/invalid/*.json` — documents with **at least one**
  error-severity diagnostic.

`protocol/fixtures.expected.json` maps each fixture's relative path
(`valid/clean.json`, `invalid/dangling-target.json`, …) to the exact,
sorted list of rule IDs a validator must fire for that document — no more,
no fewer.

## Running the corpus

Two runners consume the same files:

```sh
# Node
node protocol/run-fixtures.mjs

# Rust
cargo test -p fireside-engine --test fixtures
```

Both fail loudly on any mismatch: a rule ID fired that wasn't expected, an
expected rule ID that didn't fire, a fixture in the wrong directory for its
actual error/no-error status, or a fixture on disk with no corresponding
entry in `fixtures.expected.json` (and vice versa). This asymmetry check
means the corpus can't silently drift out of sync with either runner.

## Claiming conformance

A third-party engine's semantic validator can check itself against the same
corpus:

1. For every file in `protocol/fixtures/valid/` and `protocol/fixtures/invalid/`,
   run your validator and collect the distinct set of rule IDs it fires.
2. Compare that set against the same key's entry in
   `protocol/fixtures.expected.json`.
3. Confirm error-severity presence matches the fixture's directory (`valid/`
   → no errors, `invalid/` → at least one error).

An engine that matches on every fixture can claim Layer-2 semantic
conformance with the reference implementations. Layer 1 (schema)
conformance is separate — validate against the generated `Graph.json`
schema (JSON Schema 2020-12) described in [§4 Validation](/spec/validation/).

Rule IDs are part of the corpus's stability contract: a conformant engine
should fire the same rule ID for the same defect, not just agree on
severity, since the corpus asserts exact ID sets.

## Rule catalog

| Rule ID                              | Severity | Trigger                                                              |
| -------------------------------------- | -------- | ----------------------------------------------------------------------- |
| `unique-node-ids`                      | Error    | Two nodes share the same `id`.                                          |
| `valid-traversal-target`               | Error    | A `traversal` (`next`, or a branch option's `target`) references a node ID that doesn't exist. |
| `next-branch-point-conflict`           | Error    | A `Traversal` object sets both `next` and `branch-point`.                |
| `empty-branch-options`                 | Error    | A `branch-point.options` array has zero entries.                        |
| `unique-branch-keys`                   | Error    | Two options at the same branch point share a `key`.                      |
| `container-nesting-depth-exceeded`     | Error    | A `container` block nests deeper than the reference limit (8; see ADR-010, `.claude/adrs/adr-010-container-nesting-depth-limit.md`). |
| `empty-traversal`                      | Warning  | `"traversal": {}` — present but sets neither `next` nor `branch-point`.  |
| `reveal-masked-by-container`           | Warning  | A block's `reveal` value is lower than its enclosing container's, so it can never appear first. |
| `malformed-link-url`                   | Warning  | An inline `[label](url)` link's URL doesn't look like a usable destination. |
| `unreachable-node`                     | Warning  | A node has no traversal path from the entry node.                        |
| `self-loop`                            | Warning  | A node's `next` (or a branch option) targets itself.                     |
| `trivial-cycle`                        | Warning  | Two nodes' traversals point directly at each other.                      |
| `dead-end-branch`                      | Info     | A branch option's target is itself terminal with no way back.            |

This table is generated from `fireside-engine/src/validation.rs`; treat the
fixture corpus, not this table, as the source of truth if they ever
disagree — the corpus is what both runners actually assert against.

## Regenerating expectations

If a rule's behavior changes deliberately, update
`protocol/fixtures.expected.json` by hand and re-run both runners — there is
no auto-generation step, since the expectations file is meant to be
reviewed as a diff, not regenerated blindly.
