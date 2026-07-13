# Research: Protocol spec patch 0.1.1

All open questions here were resolved by reading the reference implementation
directly (see ADR-007 Context) rather than by external research — this is an
internal-consistency patch, not new technology adoption. No
NEEDS CLARIFICATION markers remained in the spec.

## Decision: `empty-traversal` fires only for the object form

**Decision**: The new rule inspects `node.traversal` only when it is the
`Traversal` object form (`TraversalSpec::Rules` in Rust / a non-string
object in JS) and both `next` and `branch-point` are absent from it.

**Rationale**: The string shorthand (`"traversal": "some-id"`) always
carries a non-empty target by construction (`NodeId` has `@minLength(1)`),
so it can never be "empty." An absent `traversal` field is the normal way
to declare a terminal node and must NOT warn — only a present-but-vacuous
object (`{}`) is the accidental-looking case worth flagging.

**Alternatives considered**: Warning on any terminal node regardless of
form (rejected — would make every intentional dead end noisy, defeating
the `dead-end-branch` info-level pattern already established for
legitimate endings). Treating `{}` as a schema error instead of a warning
(rejected — it's already valid per the schema, both keys being optional;
promoting it to a hard error would be a behavior change beyond what
ADR-007 scoped, and `Node::is_terminal()` already handles it gracefully).

## Decision: fixture corpus format — one shared expectations file

**Decision**: `protocol/fixtures/{valid,invalid}/*.json` holds the Fireside
documents; a single `protocol/fixtures.expected.json` maps each fixture's
relative path to its expected sorted rule-id array. Both `validate.mjs`'s
corpus runner and the new Rust test read this same expectations file.

**Rationale**: Duplicating expected rule-ids into both a JS test file and a
Rust test file would recreate exactly the drift risk this feature exists to
close — a maintainer could update one language's expectations and forget
the other. A single JSON file both languages parse trivially (Rust already
depends on `serde_json`; Node's `JSON.parse` is native) removes that risk
structurally.

**Alternatives considered**: Encoding expectations as a comment/sibling
file per fixture (rejected — harder to diff, no clear single source).
Hardcoding expected rule-ids as inline test assertions in each language
(rejected — exactly the duplication problem above).

## Decision: fixture directory split (`valid/` vs `invalid/`)

**Decision**: `valid/` holds fixtures with zero Error-severity diagnostics
(warnings/info are fine — e.g. `dead-end-branch.json` is valid but has an
info diagnostic); `invalid/` holds fixtures with at least one Error.

**Rationale**: Matches the plan's own naming (`protocol/fixtures/{valid,invalid}/*.json`)
and gives the corpus a clear presentability signal independent of the
rule-id-matching assertion — a fixture in `valid/` should also be
presentable (`has_errors` false), a fixture in `invalid/` should not be.
This is a second, cheap cross-check beyond exact rule-id matching.

**Alternatives considered**: One flat directory with only the expectations
file distinguishing valid/invalid (rejected — the plan explicitly names
the two-directory structure, and the directory split is free documentation
for a human skimming the corpus).

## Decision: `unique-branch-keys` doc fix has no fixture behavior change

**Decision**: The `duplicate-branch-keys.json` fixture already added to the
corpus documents the existing Error behavior; no code changes accompany the
`validation.md` doc promotion (Recommended → Required Checks).

**Rationale**: The doc fix corrects text to match code, not the reverse.
Adding this fixture is what proves the doc fix is now accurate — the
fixture's expected rule-id set fires as an Error in both validators,
matching the corrected doc.

## Decision: protocol version bump lands via `Versions` enum addition only

**Decision**: Add `v0_1_1: "0.1.1"` to the `Versions` enum in `main.tsp`;
do not touch the `@versioned(Versions)` decorator's mechanics or introduce
per-property `@added`/`@removed` TypeSpec versioning annotations, since no
field is gated by version in this patch.

**Rationale**: TypeSpec's `@versioned` machinery exists for fields that
differ across versions (via `@added(Versions.vX)` etc.). This patch adds no
such field — every change is prose or validator-only — so the simplest
correct change is the bare enum value addition, letting `0.1.1` documents
validate identically to `0.1.0` ones at the schema layer.

**Alternatives considered**: Skipping the version bump entirely and
treating this as a docs-only patch (rejected — ADR-007 and the plan both
name this "spec patch 0.1.1," and the new validator rule is a real,
if backward-compatible, behavior addition worth a version signal for
anyone diffing engine capabilities by protocol version).
