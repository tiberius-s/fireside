# Task 14 — `fireside validate` output parity

**Depends on:** 11
**Crates:** fireside-cli
**Phase:** 3

## Goal

`fireside validate` output matches `protocol/validate.mjs` in structure: one line per diagnostic with severity symbol, `[lint-code]`, message, and a summary line `file: N error(s), M warning(s)`. Exit code 1 only when errors exist.

## Background

`crates/fireside-cli/src/commands/validate.rs` prints a bare ✓/error today. validate.mjs prints:

```text
  ✗ [unique-node-ids] Duplicate node ID "x" at index 2 (first seen at index 1)
  ⚠ [unreachable-node] Node "y" is not reachable from entry point "intro"

file.json: 1 error(s), 1 warning(s)
```

## Steps

1. Render the Task 11 `Diagnostic` list in exactly that shape (✗ errors, ⚠ warnings, include `node_id` context in messages where present).
2. Exit codes: parse failure or any Error → 1; warnings only → 0.
3. Keep serde parse errors as-is (they already carry line/column) but prefix with the file path.
4. e2e tests in `cli_e2e.rs`: a fixture with one error and one warning produces the expected lines and exit code; hello.json prints `0 error(s)`.

## Do NOT

- Add JSON/`--format` output flags (defer until someone needs them).
- Colorize beyond the existing symbols (keep output grep-able; this command is used by CI in Task 16).

## Acceptance

```bash
cargo test -p fireside-cli
cargo run -q -p fireside-cli -- validate docs/examples/hello.json   # summary line, exit 0
```
