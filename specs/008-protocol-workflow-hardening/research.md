# Research: Protocol & Workflow Hardening

## 1. Property testing library

**Decision**: `proptest` (latest 1.x, e.g. `"1"`), added as a `[dev-dependencies]`-only
entry to `fireside-core` and `fireside-engine`.

**Rationale**: `proptest` is the de facto standard Rust property-testing
crate, MSRV-compatible well below 1.88, integrates with plain `#[test]`
via its `proptest!` macro (no custom test harness), and — critically for
this project's crate-boundary discipline — it never appears in a
production dependency list, so no constitution amendment (Principle III
table) is required. Failure minimization ("shrinking") gives reproducible
minimal counterexamples, which directly satisfies spec Acceptance
Scenario 3 (a reintroduced bug must fail with a clear counterexample, not
just "sometimes red").

**Alternatives considered**:
- `quickcheck` — smaller ecosystem, weaker shrinking for structured/recursive
  types like `Graph`/`ContentBlock`; proptest's `prop_oneof!`/`prop_recursive!`
  combinators are a better fit for the recursive `Container` type.
- Hand-rolled fuzzing loop — rejected; reinvents shrinking and reporting for
  no benefit over a well-established dev-dependency.

## 2. Arbitrary `Graph`/`ContentBlock` generation strategy

**Decision**: Hand-written `proptest::strategy::Strategy` implementations
(not `#[derive(Arbitrary)]`) for `Graph`, `Node`, and `ContentBlock`, built
with `prop_recursive` to bound container nesting depth during generation
and keep generated graphs valid-shaped (unique node ids, syntactically
sane traversal targets pointing at generated ids).

**Rationale**: `fireside-core`'s types live behind the crate boundary table
with only `serde`, `serde_json`, `thiserror` as production deps — adding
the `proptest-derive` crate would put a proc-macro dependency on the
production dependency graph unless carefully scoped to dev-only, which is
more fragile than just writing the ~5 strategy functions by hand in the
test module. `prop_recursive` directly bounds the depth of generated
`ContentBlock::Container` trees, which doubles as the generator this
feature also needs for the nesting-depth robustness fixture (a maximal
generated depth can be turned into a fixture).

**Alternatives considered**:
- `proptest-derive`'s `#[derive(Arbitrary)]` — would need to be a dev
  dependency of `fireside-core`; still viable, but hand-written strategies
  give tighter control over invariants (unique ids, valid targets) that a
  derived impl can't express without a lot of `prop_filter`/post-processing
  that ends up no simpler than writing the strategy directly.

## 3. Session-invariant property test shape

**Decision**: Generate an arbitrary valid `Graph` plus an arbitrary sequence
of `SessionOp` values (`Next`, `Choose(key)`, `Goto(id)`, `Back`), replay
them against a fresh `Session`, and after each op assert (a) every id in
`session.history()` is a node id in the graph, (b) the last history entry
equals `session.current().id`, (c) `session.visited()` (or equivalent) is a
subset of the graph's node-id set. Illegal ops (e.g. `Choose` on a node with
no branch point) are allowed in the generated sequence — they must return an
`Outcome`/error rather than panic or corrupt history, which is itself part
of what the property checks.

**Rationale**: This directly encodes spec Acceptance Scenario 2 and FR-002/
FR-003. Allowing illegal ops in the sequence (rather than only generating
legal ones) is deliberately closer to "any sequence a confused presenter or
a hostile input could produce," which is a stronger property than "only
sequences we know are legal don't break things."

**Alternatives considered**:
- Constraining the generator to only produce legal op sequences (checking
  legality before including an op) — rejected as a weaker property; the
  existing hand-written unit tests in `session.rs` already cover specific
  legal-sequence examples, so the property test earns its keep more by
  covering illegal/mixed sequences.

## 4. Container nesting depth limit

**Decision**: Maximum nesting depth of **8** levels (a `Container` whose
deepest descendant `Container` is more than 7 levels below it is rejected).
Recorded via a new ADR before implementation, per constitution Principle I
(spec-first extensions) — this is engine-defined latitude the spec
explicitly grants ("Engines MAY impose practical limits"), not a wire-format
change.

**Rationale**: `docs/examples/hello.json`, the canonical reference document,
nests containers at most 1 level deep. No fixture, template, or existing
test anywhere in the repository exceeds 2 levels. 8 is comfortably above
any realistic authored deck (an order of magnitude beyond observed usage)
while still bounding recursive validator/render functions (`walk_reveal_masking`,
`walk_link_urls`, the TUI's `container()`/`render_blocks()`) against
pathological or malicious input (e.g. a machine-generated or adversarial
deck with thousands of nested containers) that could otherwise cause deep
recursion.

**Alternatives considered**:
- No limit — rejected; leaves recursive validator and render functions
  unbounded against pathological input, the exact gap this story exists to
  close.
- A much smaller limit (e.g. 3) — rejected as presenter-hostile; a
  columns-inside-a-centered-container-inside-a-stack deck is a plausible
  3-level design already, and the limit should have headroom, not hug
  observed usage.

## 5. Watcher robustness for half-saved JSON

**Decision**: No production code change is required. `Watcher::poll()`
(`crates/fireside-cli/src/main.rs`) already returns `Err(String)` — never
panics — on both an unreadable file and a JSON parse failure, and both
consumers (`fireside-tui`'s `App::on_reload`, and `validate --watch`'s
`watch_loop`) already handle that `Err` by keeping the last good state and
surfacing a message, per the existing `write_back_reports_io_failure_without_panicking`
and `watch_report_shows_a_caret_report_for_malformed_json` tests. This
story adds a new regression test exercising a *rapid, multi-step* invalid
sequence (valid → truncated/malformed → still-malformed → valid again)
against the watcher directly, which the existing single-malformed-write
tests don't cover, to lock in that recovery doesn't depend on the invalid
streak "settling."

**Rationale**: Verified by reading `Watcher::poll`/`write_back` and
`App::on_reload` directly — both are already `Result`-based with no
`unwrap`/`expect` on the parse path, consistent with constitution Principle
IV. Confirms the plan's "hardening" framing: the goal here is proving the
existing guarantee under a harsher, more realistic input pattern, not
building new resilience machinery.

**Alternatives considered**:
- Adding a debounce/settle delay to the watcher — rejected; unnecessary,
  since the fingerprint-based poll already treats each distinct on-disk
  state as an independent event and the existing consumers already recover
  on the next valid read with no special-casing needed.

## 6. Multi-codepoint / emoji / CJK-width rendering coverage

**Decision**: No new rendering mechanism. `fireside-tui/src/render/blocks.rs`
already measures and clips using `unicode_width::UnicodeWidthStr`/`UnicodeWidthChar`
(not byte or `char` count) throughout `clip`/`clip_spans`/`heading`/`center`.
This story adds new `TestBackend` scenario tests with emoji- and
CJK-bearing headings and multi-column content to the existing suite in the
same file, asserting the already-correct width-aware behavior holds for
these inputs specifically (closing a coverage gap, not a behavior gap).

**Rationale**: Verified by reading the `clip`/`clip_spans` implementations
directly — they already operate on display width via `unicode-width`, the
same crate already in `fireside-tui`'s permitted dependency list. No
constitution amendment needed.

**Alternatives considered**: none — this is confirmed to be a pure test
addition once the existing width-measurement code was read.

## 7. ~1,000-node performance fixture

**Decision**: A generated fixture of ~1,000 nodes (simple linear/lightly
branching structure, generated by a small script or `build.rs`-free helper
committed as a fixture file, not hand-written) added to
`protocol/fixtures/valid/`, with a Rust test asserting `Graph::from_json`
plus `validate()` together complete in under 1 second on the fixture, run
as part of the normal `cargo nextest run --workspace` job (no separate CI
job needed). The Node side is not required to repeat the timing assertion
(timing budgets are reference-implementation-specific, not a cross-language
parity concern like rule IDs are) but the fixture file itself is still
valid input for `run-fixtures.mjs`'s existing pass/fail check.

**Rationale**: 1 second is generous headroom above what a ~1,000-node graph
(simple struct traversal, no I/O beyond one file read) should take even on
a loaded CI runner — matching the "generous margin, not flaky" requirement
in the spec's Edge Cases section. Reusing `cargo nextest run --workspace`
avoids a bespoke perf-CI job the plan didn't ask for.

**Alternatives considered**:
- A dedicated benchmark harness (`criterion`) — rejected as disproportionate;
  the requirement is a regression tripwire ("still fast"), not statistical
  benchmarking with trend tracking.

## 8. CI: cargo-deny and MSRV verification

**Decision (cargo-deny)**: Already runs in CI (`.github/workflows/audit.yml`),
but currently only on `push` to `main` and a weekly schedule — **not** on
pull requests. This is a real gap relative to the plan's intent (catch a
disallowed license or advisory before merge, not after). Add a
`pull_request` trigger to `audit.yml` scoped to the same `paths` filter
already used (`Cargo.lock`, `deny.toml`, the workflow file itself).

**Decision (MSRV)**: The existing `msrv` job in `.github/workflows/rust.yml`
(`cargo check --workspace` on an actual Rust 1.88 toolchain via
`dtolnay/rust-toolchain@1.88`, on every relevant PR and push to `main`)
already satisfies the plan's "cargo msrv verify" ask. It compiles the
*exact* locked dependency graph (`Cargo.lock`) under the *actual* pinned
MSRV toolchain — a strictly stronger check than the dedicated `cargo-msrv`
tool's bisection-based "find the minimum version that compiles," which
answers a different question (what's the true minimum) rather than
"does our declared minimum actually hold" (which is what CI needs
enforced every PR). No new tooling is needed; this decision and its
reasoning gets documented (in the ADR or plan progress log) rather than
adding `cargo-msrv` as a new CI dependency for no marginal safety gain.

**Rationale**: Read both workflow files directly. `audit.yml`'s `on:` block
has no `pull_request:` key, confirmed by direct inspection — cargo-deny
currently cannot block a PR, only flag `main` after the fact or on the
weekly cron. This is the one concrete, actionable CI gap found during
research; everything else the plan asked to "confirm" is already true.

**Alternatives considered**:
- Adding `cargo-msrv` as a second, redundant MSRV job — rejected; it
  verifies a different, less-relevant property (the true floor) and would
  duplicate the existing job's runtime cost for no PR-blocking benefit the
  existing job doesn't already provide.
