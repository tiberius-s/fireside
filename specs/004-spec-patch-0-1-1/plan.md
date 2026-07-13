# Implementation Plan: Protocol spec patch 0.1.1

**Branch**: `004-spec-patch-0-1-1` | **Date**: 2026-07-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/004-spec-patch-0-1-1/spec.md`

## Summary

Resolve the seven protocol ambiguities catalogued in the strategic plan's
audit by (a) documenting six already-settled reference behaviors in
`protocol/main.tsp` and `docs/src/content/docs/spec/`, (b) adding one new
symmetric validator rule (`empty-traversal`, WARNING) to both
`fireside-engine/src/validation.rs` and `protocol/validate.mjs`, (c) fixing
a doc/impl severity mismatch for the existing `unique-branch-keys` rule, and
(d) building a shared fixture corpus at `protocol/fixtures/{valid,invalid}/`
that both validator test suites run, proving Rust/Node rule parity instead
of just asserting it. Protocol version gains `"0.1.1"` as an additive enum
value; `tsp-output/` is regenerated. Per ADR-007.

## Technical Context

**Language/Version**: Rust 1.88 (2024 edition) for `fireside-engine`; Node.js (ESM, `.mjs`) for `protocol/validate.mjs`; TypeSpec for `protocol/main.tsp`.

**Primary Dependencies**: No new dependencies. Reuses `fireside-core` (Graph/Node model), existing `fireside-engine::validation` module, existing `protocol/validate.mjs` Tier-2 checks, TypeSpec/`@typespec/json-schema` toolchain already wired via `npm run build` in `protocol/`.

**Storage**: N/A — this feature adds static JSON fixture files under `protocol/fixtures/`, not a data store.

**Testing**: `cargo test --workspace` (new unit tests in `fireside-engine/src/validation.rs`'s existing `#[cfg(test)] mod tests`, plus a new fixture-corpus test); a new Node test/script under `protocol/` that runs the same fixture corpus through `validate.mjs`'s `validate()` function and compares rule-id sets.

**Target Platform**: Same as the rest of the workspace — cross-platform CLI/TUI (Rust) plus a Node.js dev-tooling script (protocol validation, not shipped to end users).

**Project Type**: Existing 4-crate Rust workspace + a `protocol/` TypeSpec/Node subproject. This feature touches `fireside-engine` (validator), `fireside-core` is read-only (no model changes needed — `Node::is_terminal()` and `TraversalSpec` already support what's needed), `protocol/` (spec source, validator, fixtures), and `docs/` (spec prose).

**Performance Goals**: N/A — validation already runs in well under a second on realistic decks; this feature doesn't change complexity class (still O(nodes + edges) per rule pass).

**Constraints**: Protocol version bump MUST be additive only (Principle I). New validator rule MUST NOT alter `Node::is_terminal()`/engine traversal behavior — diagnostics only. Fixture corpus MUST be consumed identically by both validators (no separate/divergent fixture sets).

**Scale/Scope**: ~9 fixture files (one per existing Layer-2 rule, plus one clean valid document), one new Rust validation function + test, one new Node validation function + a small corpus-runner script, prose edits across 4 existing docs pages, `main.tsp` doc-comment edits + one enum value.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Spec Is the Source of Truth)**: PASS. This feature exists to close spec/impl drift. Any extension it documents (none — no new fields) would need spec-first registration, but this feature adds zero new wire fields; it only adds one enum value (`v0_1_1`) and prose. `docs/examples/hello.json` continues to validate with zero errors (SC-004) — verified by test.
- **Principle II (Presenter-First Experience)**: PASS. The one behavior change (`empty-traversal` warning) is argued directly from presenter feedback (US2) — surfaces a silent, likely-accidental state.
- **Principle III (Crate Boundary Discipline)**: PASS. No new dependencies, no crate boundary changes. `empty-traversal` is added inside `fireside-engine`'s existing `validation.rs`, using only `fireside-core` types already imported there.
- **Principle IV (Mandatory Code Idioms)**: PASS. New Rust function follows existing `check_*` naming/signature pattern in `validation.rs`, returns via the existing `Diagnostic::new` constructor, no `unwrap()`/`expect()` outside tests.
- **Principle V (Stratified Error Handling)**: N/A. This feature adds a diagnostic, not an error type — no new `Result`/error variants.
- **Principle VI (MSRV 1.88)**: PASS. No new dependency; nothing raises MSRV.
- **Principle VII (Test Discipline)**: PASS. New engine rule gets a unit test in `validation.rs`'s existing suite (per constitution: "Engine semantics ... are unit tests in ... validation.rs") plus fixture-corpus coverage. No TUI-visible state changes, so no scenario test or tmux smoke test is required — this feature has zero UI surface.
- **Wire format / ADR gate**: This feature touches the protocol version enum (`main.tsp`), which the constitution flags as requiring an ADR before code. ADR-007 (`.claude/adrs/adr-007-spec-patch-0-1-1.md`) is already written and accepted, satisfying this gate.

**Result**: PASS, no violations. Complexity Tracking table not needed.

## Project Structure

### Documentation (this feature)

```text
specs/004-spec-patch-0-1-1/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md         # Phase 1 output
├── quickstart.md         # Phase 1 output
├── contracts/            # Phase 1 output
└── tasks.md              # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
protocol/
├── main.tsp                          # Versions enum gains v0_1_1; ListBlock doc comment;
│                                      # TraversalOps.choose() doc comment
├── validate.mjs                      # + checkEmptyTraversal(), wired into validate()
├── fixtures/
│   ├── valid/
│   │   ├── clean.json                # zero diagnostics
│   │   ├── unreachable-node.json
│   │   ├── self-loop.json
│   │   ├── trivial-cycle.json
│   │   ├── dead-end-branch.json
│   │   └── empty-traversal.json      # new rule's fixture
│   └── invalid/
│       ├── duplicate-node-ids.json
│       ├── dangling-target.json
│       ├── next-branch-point-conflict.json
│       └── duplicate-branch-keys.json
├── fixtures.expected.json            # rule-id expectations per fixture, read by both suites
└── tsp-output/                       # regenerated (npm run build)

crates/fireside-engine/src/
└── validation.rs                     # + check_empty_traversal(), wired into validate();
                                       # + unit tests; + fixture-corpus test

docs/src/content/docs/spec/
├── validation.md                     # promote unique-branch-keys to Required; add empty-traversal
├── traversal.md                      # Choose operation: option-scoping sentence
├── appendix-engine-guidelines.md     # + ViewMode persistence, image clamp, history cap
└── appendix-content-blocks.md        # + ListBlock inline-Markdown note
```

**Structure Decision**: No new crates or top-level directories. `protocol/fixtures/` is new but lives inside the existing `protocol/` subproject alongside `validate.mjs` and `main.tsp`. The fixture-expectation file (`protocol/fixtures.expected.json`) is the single source both `validate.mjs`'s corpus runner and the new Rust test read, keeping the "same expectation, two checkers" design honest — no fixture data is duplicated per-language.

## Complexity Tracking

*No Constitution Check violations — table not needed.*
