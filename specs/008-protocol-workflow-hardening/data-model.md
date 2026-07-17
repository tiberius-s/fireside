# Data Model: Protocol & Workflow Hardening

This feature adds no new wire-format entities. It adds one new validator
concept and reuses the two existing entities (`Graph`, fixture corpus
entries) already defined by `fireside-core` and the Week 1 fixture-corpus
work (spec 004).

## New: nesting depth (validator concept, not a model field)

Not a serialized field — a derived property computed by walking a node's
`content: ContentBlock[]` tree.

| Property         | Type  | Rule                                                                 |
| ---------------- | ----- | --------------------------------------------------------------------|
| depth of a block | usize | `0` for a non-container block; for a `Container`, `1 + max(depth of children)`, or `1` if it has no `Container` children |
| node violates limit | bool | `true` if any top-level block in `node.content` has depth `> 8` |

New diagnostic: `container-nesting-depth-exceeded` (rule id, kebab-case per
existing convention), severity **Error** (structural rejection, matching
the severity class of `unique-node-ids`/`valid-traversal-target`, not the
Warning class used for reachability-style soft issues — a runaway nesting
depth is a structural defect an author should fix, not a stylistic note).
Implemented identically in `fireside-engine::validation` and
`protocol/validate.mjs`, per the existing dual-validator parity discipline.

## Reused: `Graph` (`fireside-core::model`)

No field changes. Property tests generate arbitrary values of this existing
type (and its constituents — `Node`, `ContentBlock` variants, `Traversal`)
via hand-written `proptest::Strategy` implementations (see research.md §2),
not new production types.

## Reused: conformance fixture entry (`protocol/fixtures/`, spec 004)

No format changes. Each new fixture is:

- a `.json` document under `protocol/fixtures/valid/` or
  `protocol/fixtures/invalid/`, plus
- one new key in `protocol/fixtures.expected.json` mapping the fixture's
  relative path to the sorted list of rule IDs both validators must report.

New fixtures added by this feature (see quickstart.md for the exact
commands that verify them):

| Fixture                                   | Directory | Expected rule IDs                         |
| ------------------------------------------ | --------- | ------------------------------------------ |
| `nesting-depth-at-limit.json`              | `valid/`  | `[]`                                        |
| `nesting-depth-exceeds-limit.json`         | `invalid/`| `["container-nesting-depth-exceeded"]`      |
| `large-deck-1000-nodes.json`               | `valid/`  | `[]`                                        |

## New (test-only): `SessionOp`

Not a production type — a small enum used only inside the
`fireside-engine` property-test module to describe one step of a generated
navigation sequence, replayed against a `Session` in the property test
described in research.md §3:

```text
enum SessionOp {
    Next,
    Choose(String),  // an option key, may or may not be legal at the current node
    Goto(String),    // a node id, may or may not exist
    Back,
}
```

This lives entirely in `#[cfg(test)]` code in `fireside-engine`; it is not
part of the crate's public API and has no bearing on the crate-boundary
dependency table.
