# Quickstart: validating Protocol & Workflow Hardening

These are the commands that prove each user story works, run from the repo
root. No new setup beyond what the workspace already requires (Rust 1.88+
toolchain, Node for `protocol/`).

## US1 — Property tests (serde round-trip + session invariants)

```sh
cargo test -p fireside-core --lib proptest
cargo test -p fireside-engine --lib proptest
```

Expected: all cases pass (proptest's default 256 cases per property,
deterministic across runs via its default RNG seeding unless a failure
seed is printed). To verify the suite actually catches regressions (spec
Acceptance Scenario 3): temporarily reintroduce a known historical bug
(e.g. skip pushing to history on one navigation op), rerun, confirm a
minimal failing case is printed, then revert.

## US2 — Expanded conformance corpus

```sh
cargo test -p fireside-engine --test fixtures
node --test protocol/run-fixtures.mjs   # or: npm run test:fixtures --prefix protocol
```

Expected: both report the same rule IDs for every fixture, including the
three new ones (`nesting-depth-at-limit.json`,
`nesting-depth-exceeds-limit.json`, `large-deck-1000-nodes.json`). The
1,000-node fixture's load+validate time is asserted under 1 second by a
dedicated Rust test (part of the same `cargo test -p fireside-engine`
run).

## US3 — Watcher survives half-saved JSON

```sh
cargo test -p fireside-cli watcher
```

Expected: a new test drives `Watcher::poll()` through valid → truncated
(simulated non-atomic save) → still-invalid → valid again, asserting no
panic at any step and that the last-valid `Graph` remains available to the
caller (via the existing `Err(String)`-returning contract) until the final
valid poll.

Manual smoke check (optional, matches Principle VII for live-reload
timing-sensitive work — this story is example-based recovery, already
covered by unit tests, so a full tmux pass is not required unless the unit
test reveals a real gap):

```sh
cargo run --bin fireside -- validate --watch docs/examples/hello.json
# in another terminal: truncate the file mid-`cp`, then let the copy finish
```

## US4 — Emoji/CJK render-width correctness

```sh
cargo test -p fireside-tui --lib
```

Expected: new `TestBackend` scenario tests (emoji-bearing heading, CJK
multi-column content) pass alongside the existing suite, with no changes
to any pre-existing scenario test's expected output.

## CI review (FR-012)

```sh
gh workflow view audit.yml   # or read .github/workflows/audit.yml directly
gh workflow view rust.yml
```

Expected after implementation: `audit.yml` has a `pull_request` trigger on
the same `paths` filter as its `push` trigger; `rust.yml`'s `msrv` job is
unchanged (already sufficient, per research.md §8).

## Whole-suite gate (run before calling any story done)

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
node protocol/validate.mjs docs/examples/hello.json
npm run check --prefix docs
graphify update .
```
