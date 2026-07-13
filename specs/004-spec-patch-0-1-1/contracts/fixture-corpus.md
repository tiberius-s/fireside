# Contract: shared conformance fixture corpus

## Layout

```text
protocol/
├── fixtures/
│   ├── valid/*.json      # zero Error-severity diagnostics
│   └── invalid/*.json    # one or more Error-severity diagnostics
└── fixtures.expected.json
```

## `fixtures.expected.json` shape

A flat JSON object. Keys are fixture paths relative to `protocol/fixtures/`
(e.g. `"valid/self-loop.json"`). Values are a sorted array of every
distinct rule identifier expected to fire anywhere in that fixture's
document, across all nodes and all severities.

## Consumer contract — both validators MUST

1. Enumerate every file under `protocol/fixtures/valid/` and
   `protocol/fixtures/invalid/`.
2. For each fixture, run their `validate(graph)` function.
3. Collect the sorted, deduplicated set of `rule` values from the result.
4. Assert that set equals `fixtures.expected.json[<relative path>]`.
5. Separately assert: fixtures under `valid/` produce zero Error-severity
   diagnostics; fixtures under `invalid/` produce at least one.
6. Fail loudly (non-zero exit / failing test) if a fixture exists with no
   entry in `fixtures.expected.json`, or vice versa — the corpus and its
   expectations file must stay in lockstep.

## Rust consumer

A new `#[test]` in `fireside-engine/src/validation.rs` (or a small
integration test in `fireside-engine/tests/` if that reads more naturally
alongside existing test placement) that:
- reads `../../protocol/fixtures.expected.json` relative to the crate,
- walks `../../protocol/fixtures/{valid,invalid}/*.json`,
- parses each via `Graph::from_json`,
- runs `validate()`,
- performs the assertions above.

## Node consumer

A script under `protocol/` (e.g. `protocol/run-fixtures.mjs`, wired to an
npm script such as `npm run test:fixtures --prefix protocol` or folded into
an existing test command if one exists) that performs the same steps using
`validate.mjs`'s exported `validate` function directly (no subprocess/CLI
invocation needed — import the module).

## Fixture inventory (minimum required by this feature)

| Fixture | Directory | Rule(s) exercised |
| --- | --- | --- |
| `clean.json` | valid | (none — zero diagnostics) |
| `unreachable-node.json` | valid | `unreachable-node` |
| `self-loop.json` | valid | `self-loop` |
| `trivial-cycle.json` | valid | `trivial-cycle` |
| `dead-end-branch.json` | valid | `dead-end-branch` |
| `empty-traversal.json` | valid | `empty-traversal` |
| `duplicate-node-ids.json` | invalid | `unique-node-ids` |
| `dangling-target.json` | invalid | `valid-traversal-target` |
| `next-branch-point-conflict.json` | invalid | `next-branch-point-conflict` |
| `duplicate-branch-keys.json` | invalid | `unique-branch-keys` |

("valid" here means presentable / no Errors — several of these fixtures
still carry Warning or Info diagnostics, which is expected and asserted via
the exact rule-id match, not just the directory placement.)
