# Implementation Plan: Protocol & Workflow Hardening

**Branch**: `008-protocol-workflow-hardening` | **Date**: 2026-07-17 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/008-protocol-workflow-hardening/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

The last item of the phase-1 strategic plan (P2 — Protocol & workflow
hardening): property tests for serde round-trip and session invariants,
three new robustness fixtures in the shared conformance corpus (a new
container-nesting-depth limit + rule, a ~1,000-node performance fixture),
an emoji/CJK render-width scenario-test gap closed in `fireside-tui`, a
regression test locking in the watcher's already-correct half-saved-JSON
recovery, and one concrete CI gap closed (`cargo deny` gains a
`pull_request` trigger). No protocol/wire-format change; the one new
validator rule (`container-nesting-depth-exceeded`) uses latitude the spec
already grants engines. Research (research.md) found that two of the
plan's four sub-asks (watcher resilience, CJK/emoji width) are already
correctly implemented — this feature's job there is adding the regression
coverage that proves it, not new production behavior.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.88 (`resolver = "3"`, 2024 edition); Node (existing `protocol/` tooling, unchanged version)

**Primary Dependencies**: `proptest` (new, `[dev-dependencies]`-only in `fireside-core` and `fireside-engine` — never a production dependency, so no constitution Principle III table amendment); all other work reuses dependencies already present (`unicode-width` in `fireside-tui`, `serde_json` in `fireside-engine`'s existing dev-deps)

**Storage**: N/A (no new persisted state; fixture files are the only new on-disk artifacts, under existing `protocol/fixtures/`)

**Testing**: `cargo test --workspace` / `cargo nextest run --workspace` (new proptest cases, new fixture-corpus entries, new `fireside-cli` watcher regression test, new `fireside-tui` `TestBackend` scenario tests), `protocol/run-fixtures.mjs` (Node side of fixture parity), no new tmux smoke-test requirement (US3/US4 close coverage gaps on already-correct behavior per research.md §5–6, not new event-loop-timing-sensitive code — Constitution Principle VII's tmux requirement is for UI changes, and this feature makes none)

**Target Platform**: Same as existing workspace — any terminal `fireside` already supports; CI runs on `ubuntu-latest` and `macos-latest` per the existing `rust.yml` matrix

**Project Type**: Existing 4-crate Rust workspace (`fireside-core`/`fireside-engine`/`fireside-tui`/`fireside-cli`) plus the `protocol/` Node-based validator/fixture tooling — no new crate, no new top-level project

**Performance Goals**: A ~1,000-node deck's `Graph::from_json` + `validate()` completes in under 1 second on CI hardware (SC-003); proptest's default case count (256/property) keeps the new tests within the existing CI time budget (no unbounded fuzzing job)

**Constraints**: No wire-format/schema change (Constitution Principle I); no production-dependency additions (Principle III); MSRV 1.88 must hold for `proptest`'s pinned version (verify during implementation per Principle VI); nesting-depth limit and perf budget must have enough margin to avoid CI flakiness (spec Edge Cases)

**Scale/Scope**: Four independently-testable stories (P1 property tests, P2 corpus + watcher regression test, P3 render-width coverage) plus one CI-configuration edit; no shared foundational phase required (each story touches a different crate/file, same pattern as spec 007)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Spec is source of truth)**: PASS. The one new validator
  rule (`container-nesting-depth-exceeded`) uses latitude the spec already
  grants ("Engines MAY impose practical limits" — `main.tsp` ContainerBlock
  doc comment) — no `main.tsp`/schema/`tsp-output/` change, so no protocol
  version bump. `docs/examples/hello.json` is unaffected (nests 1 level
  deep, well under the chosen limit of 8). An ADR will still be written
  before implementation (engine-defined-limit decisions have gotten ADRs
  before, e.g. the reveal-ordinal-steps decision in ADR-009) even though no
  spec text changes, to record the chosen number and its rationale.
- **Principle II (Presenter-first)**: PASS. No product-surface change; this
  is entirely test/CI infrastructure and one defensive validator rule. The
  new rule's error message will follow the existing actionable-diagnostic
  convention (name the violating node, per data-model.md).
- **Principle III (Crate boundary discipline)**: PASS. `proptest` is
  dev-dependency-only in `fireside-core` and `fireside-engine` — neither
  crate's production dependency list changes, so no table amendment. No
  other new dependency anywhere.
- **Principle IV (Mandatory code idioms)**: PASS. New validator rule
  function follows the existing `check_*(graph, diags)` pattern in
  `validation.rs`; no `unwrap`/`expect` introduced outside tests; `Outcome`
  contract is untouched (no engine-operation semantics change — property
  tests observe existing `Outcome`s, they don't add new ones).
- **Principle V (Stratified error handling)**: PASS. No new error types;
  the watcher regression test exercises the existing `Result`/`Err(String)`
  contract, doesn't add one.
- **Principle VI (MSRV 1.88)**: Gate — `proptest`'s pinned version's MSRV
  MUST be verified ≤ 1.88 before it's added (tasks.md must include this as
  an explicit check, not an assumption); proptest 1.x has historically
  supported Rust versions well below 1.88, but the plan's own precedent
  (ADR-008's `ratatui-image` MSRV surprise) means this gets verified with
  an actual `cargo +1.88 check`, not trusted from `Cargo.toml` metadata
  alone.
- **Principle VII (Test discipline)**: PASS — this feature *is* test
  discipline being extended: property tests land at the
  `fireside-engine::session`/`fireside-core` unit-test layer, the new
  emoji/CJK cases land in `fireside-tui`'s existing `TestBackend` scenario
  suite, the watcher regression test lands in `fireside-cli`'s test module
  alongside its existing `write_back_*`/`watch_report_*` tests, and the
  large-deck fixture is consumed by the existing dual-validator fixture
  test. No tmux smoke test is added (see Technical Context's Testing row) —
  this is a deliberate, documented exception: Principle VII requires tmux
  smoke for *UI changes*, and US3/US4 confirm already-correct non-UI-visible
  behavior rather than changing it.

No violations requiring Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/
├── fireside-core/
│   └── src/model/mod.rs        # arbitrary Graph/Node/ContentBlock proptest strategies (new, test-only)
├── fireside-engine/
│   ├── src/session.rs          # session-invariant property test (new)
│   ├── src/validation.rs       # check_container_nesting_depth (new rule)
│   └── tests/fixtures.rs       # existing dual-validator fixture test (extended, unchanged mechanism)
├── fireside-tui/
│   └── src/render/blocks.rs    # new emoji/CJK TestBackend scenario tests (existing suite, extended)
└── fireside-cli/
    └── src/main.rs             # new Watcher rapid-invalid-sequence regression test

protocol/
├── fixtures/valid/             # + nesting-depth-at-limit.json, large-deck-1000-nodes.json
├── fixtures/invalid/           # + nesting-depth-exceeds-limit.json
├── fixtures.expected.json      # + entries for the three new fixtures
├── validate.mjs                # + container-nesting-depth-exceeded rule (Node side)
└── main.tsp                    # doc-comment update noting the reference limit (no schema change)

.github/workflows/
└── audit.yml                   # + pull_request trigger for cargo-deny

docs/src/content/docs/spec/
└── appendix-engine-guidelines.md  # (or nearest equivalent) + note on the chosen nesting-depth example
```

**Structure Decision**: No new crate or top-level directory. Every change
lands inside the existing 4-crate workspace at the layer the constitution's
boundary table already assigns it (validation logic in
`fireside-engine`, render-width tests in `fireside-tui`, CLI/watcher tests
in `fireside-cli`, wire-adjacent generation strategies in `fireside-core`),
plus the existing `protocol/` fixture-corpus and CI-workflow files — the
same shape as every prior spec in this repo (001–007).

## Complexity Tracking

No Constitution Check violations. This section is intentionally empty.
