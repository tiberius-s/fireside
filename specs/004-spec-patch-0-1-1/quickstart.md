# Quickstart: Protocol spec patch 0.1.1

## Prerequisites

- Rust workspace builds (`cargo build --workspace`)
- `protocol/` npm deps installed (`npm install --prefix protocol`)

## Scenario 1 — protocol version bump is additive

```sh
cd protocol && npm run build
```

Expect: build succeeds, `tsp-output/schemas/Graph.json`'s `fireside-version`
enum now includes both `"0.1.0"` and `"0.1.1"`. Then:

```sh
node protocol/validate.mjs docs/examples/hello.json
```

Expect: `hello.json` (still declaring `"0.1.0"` or no `fireside-version` at
all) validates with the same result as before this feature (per SC-004).

## Scenario 2 — empty traversal produces a warning, not a behavior change

```sh
cat > /tmp/empty-traversal.json <<'JSON'
{"nodes":[{"id":"a","traversal":{},"content":[]}]}
JSON
node protocol/validate.mjs /tmp/empty-traversal.json
cargo run -p fireside-cli -- validate /tmp/empty-traversal.json
```

Expect: both report a warning naming node `"a"` under rule
`empty-traversal`, exit code `0` (warnings don't block presentation, per
Layer-2 severity guidance).

## Scenario 3 — an absent traversal field does not warn

```sh
cat > /tmp/absent-traversal.json <<'JSON'
{"nodes":[{"id":"a","content":[]}]}
JSON
node protocol/validate.mjs /tmp/absent-traversal.json
cargo run -p fireside-cli -- validate /tmp/absent-traversal.json
```

Expect: neither reports `empty-traversal` — this is the normal, silent
terminal-node case.

## Scenario 4 — fixture corpus proves Rust/Node parity

```sh
cargo test --workspace -- fixture
node protocol/run-fixtures.mjs   # or the wired npm script
```

Expect: both pass, and both report checking the same fixture count. Break
one intentionally (e.g. rename a rule string in only one validator) to
confirm the corpus actually fails when the two validators disagree —
then revert.

## Scenario 5 — spec docs read correctly end-to-end

Read, in order: `docs/src/content/docs/spec/validation.md`,
`traversal.md`, `appendix-engine-guidelines.md`,
`appendix-content-blocks.md`. Confirm each of the seven ambiguities from
the strategic plan's audit (§1) now has a plain-language answer, per US1's
acceptance scenarios in `spec.md`.

## Full verification

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
cd protocol && npm run build && node validate.mjs ../docs/examples/hello.json && node run-fixtures.mjs
npm run check --prefix docs
```
