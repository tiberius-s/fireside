# Contract: `reserved-branch-key` validation rule

## Trigger

A `branch-point.options[].key` (a single presenter-facing character) equals
one of the twelve reserved global presenter keys:
`e f g h j k m n p q s t`.

## Severity

`Warning` — never blocks presenting or `fireside validate`'s exit code the
way an `Error` does.

## Diagnostic shape

Same `Diagnostic` type every other Layer-2 rule uses
(`fireside-engine::validation::Diagnostic`):

```json
{
  "severity": "warning",
  "rule": "reserved-branch-key",
  "message": "\"<node-id>\" assigns key \"e\" to \"<option label>\", but \"e\" is a reserved presenter key (quick-edit) — this option can never be selected",
  "node_id": "<node-id>"
}
```

- `message` MUST name: the colliding key, the node id, the option's label,
  and (for author clarity) what the key already does globally. Exact
  wording is an implementation detail; the four facts above are the
  contract.
- One diagnostic per colliding option — a branch point with two colliding
  options produces two diagnostics, not one.

## Non-triggers

- A branch option with no `key` at all (label/mouse-only selection).
- A branch option whose key is not in the reserved set (including digits,
  punctuation, and any letter not in `e f g h j k m n p q s t`).
- `unique-branch-keys` (existing rule) already covers two branch options
  colliding with *each other*; this rule is additive and independent —
  a key can trigger both rules at once (reserved AND duplicated within the
  same branch point) with no suppression between them.

## Relationship to presenting

This rule is descriptive of existing runtime behavior, not new behavior:
pressing a reserved key while a branch menu with a colliding option is on
screen already resolves to the global action (e.g. `e` opens quick-edit),
never the branch option — today, silently. This rule only makes that fact
visible at authoring time; `App::on_present_key`'s dispatch order is
unchanged.

## Cross-crate contract

The reserved key set is the single constant
`fireside_engine::validation::RESERVED_PRESENTER_KEYS` (`[char; 12]`).
`fireside-tui` MUST NOT define its own separate copy of this list; a
`fireside-tui` test imports and checks against it directly (see
`research.md` §1). If the presenter's global key bindings ever change, this
constant is the one place that must be updated, and the `fireside-tui`
regression test will fail loudly if the two drift.

## Rust/Node parity contract

This rule MUST also exist in `protocol/validate.mjs` (`checkReservedBranchKeys`,
same rule id, same reserved-key set as a local JS array literal — no
cross-language import mechanism exists, so the two lists are kept in sync
by convention plus the fixture check below, same as every other rule in
this validator) and be listed in its `HELP` text's warnings section. A
fixture, `protocol/fixtures/valid/reserved-branch-key.json`, with a
matching entry in `protocol/fixtures.expected.json`, is what actually
proves the two implementations agree — `crates/fireside-engine/tests/
fixtures.rs` and `protocol/run-fixtures.mjs` both run this fixture and
must report exactly `["reserved-branch-key"]`. See `research.md` §8.
