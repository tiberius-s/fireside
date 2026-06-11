# Task 15 — Optional: JSON Schema Layer-1 validation spike

**Depends on:** 14
**Crates:** fireside-engine (or fireside-cli — see boundary note)
**Phase:** 3 — OPTIONAL. Skip without guilt; semantic parity (Task 14) already closes the trust gap.

## Goal

True Layer-1 schema validation per validation.md: validate raw JSON against the generated `Graph.json` (2020-12) before deserialization, reporting failing instance paths.

## Verified API (Context7, jsonschema 0.40)

- `jsonschema::options().with_base_uri(...).build(&schema)` → reusable `Validator`; supports draft 2020-12.
- `validator.iter_errors(&instance)` yields errors with `error.instance_path()` — the "failing path + rule" output the spec asks for.
- **Dependency hygiene:** default features pull in `reqwest`. Use `jsonschema = { version = "0.40", default-features = false }` and embed all schema files from `protocol/tsp-output/schemas/` via `include_str!`, registering them in a `Registry` so no file/network resolution is needed.

## Gate (do this first, 5 minutes)

```bash
cargo add jsonschema --no-default-features -p fireside-engine --dry-run
cargo +1.88 check -p fireside-engine   # after adding for real
```

If MSRV 1.88 fails or the no-default-features tree still pulls heavy deps, **abort the task** and record the result in `plan/tasks/15-RESULT.md`.

## Steps

1. New `crates/fireside-engine/src/schema.rs`: build the validator once (`LazyLock`), embed schemas, expose `validate_schema(&serde_json::Value) -> Vec<Diagnostic>` with code `schema` and the instance path in the message.
2. Wire into `fireside validate` ahead of semantic checks: schema errors print first; semantic checks still run on whatever deserializes.
3. Boundary note: the rust-expert boundary table allows "validation libs" in `fireside-engine` — keep it there, not in core.
4. Tests: a doc with a wrong enum value (e.g. `"view-mode": "huge"`) reports the failing path `/nodes/0/view-mode`.

## Acceptance

```bash
cargo +1.88 check --workspace
cargo test -p fireside-engine -p fireside-cli
```
