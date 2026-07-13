# Implementation Plan: Markdown Authoring Frontend (`fireside import`)

**Branch**: `003-markdown-import` | **Date**: 2026-07-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-markdown-import/spec.md`

## Summary

Add a `fireside import <input.md> [output.fireside.json]` verb that
compiles a Markdown document into protocol-0.1.0 JSON, per ADR-006
(`.claude/adrs/adr-006-markdown-import.md`). A new `crates/fireside-cli/src/import.rs`
module parses Markdown with `pulldown-cmark` (newly permitted for
`fireside-cli` in the constitution, v1.1.0), using byte-range source
slicing to preserve inline Markdown verbatim rather than reconstructing it
from the parsed event tree. `##` headings become nodes; a two-pass approach
(collect node ids, then build content and resolve branch targets) supports
forward-referencing branch links. The generated graph is validated with the
existing `fireside_engine::validate` before any file is written; every
rejection case names a specific line/link rather than failing generically.
No protocol/wire-format change.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.88, 2024 edition.

**Primary Dependencies**: `pulldown-cmark` 0.13 (new — constitution v1.1.0
amendment via ADR-006, verified to build clean under MSRV 1.88); otherwise
only existing `fireside-cli` dependencies (`clap`, `anyhow`, `serde_json`,
`fireside_core`, `fireside_engine`). No YAML crate (frontmatter is a
hand-rolled flat key-value parser, research.md §4).

**Storage**: reads one Markdown file, writes one JSON file — both plain
`std::fs` calls in `main.rs`'s command handler; the `import()` parsing
function itself performs no I/O (contracts/cli-import.md).

**Testing**: `cargo test --workspace`; unit tests directly against
`import::import(&str) -> Result<Graph, ImportError>` for every FR (no
filesystem); one `cli_e2e.rs` integration test for CLI wiring
(default-output-path derivation, overwrite refusal).

**Target Platform**: same as the existing `fireside` binary.

**Project Type**: CLI — new module (`fireside-cli/src/import.rs`) plus
`main.rs` wiring for the new `Command::Import` variant. `fireside-core`,
`fireside-engine`, and `fireside-tui` are untouched.

**Performance Goals**: N/A beyond existing CLI responsiveness — import is a
one-shot batch parse of a single file, not a hot path.

**Constraints**: `import()` MUST NOT perform file I/O (contracts/cli-import.md,
keeps parsing logic unit-testable and matches constitution §V's
stratified error handling — `anyhow` stays at the `main.rs` boundary);
MUST NOT write any output file on any rejection case; MUST reuse
`fireside_engine::validate` rather than inventing import-specific
validation rules.

**Scale/Scope**: one new module, one new CLI verb, one new permitted
dependency (already amended into the constitution).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
|---|---|---|
| I. Spec Is the Source of Truth | No protocol/`main.tsp` change; importer only emits documents already valid under protocol 0.1.0 | PASS — ADR-006 confirms no wire-format change; ADR-006 itself is the required ADR for this feature since it does add a new permitted dependency (see Principle III) |
| II. Presenter-First Experience | Every rejection names a specific, locatable problem (FR-012, FR-018, FR-019, FR-022); every success reports where the output landed and what wasn't carried over (FR-023) | PASS |
| III. Crate Boundary Discipline | `pulldown-cmark` added to `fireside-cli`'s permitted deps — a **deliberate, ADR-backed amendment** (constitution v1.0.0 → v1.1.0, ADR-006). No other crate's boundary changes; `fireside-tui`/`fireside-engine`/`fireside-core` untouched | PASS — amendment already applied to `.specify/memory/constitution.md` before this plan was written |
| IV. Mandatory Code Idioms | No `unwrap()`/`expect()` outside `main()`/tests; `#[must_use]` on public value-returning functions; doc comments on public items | PASS — `import()` returns `Result`, no panics on malformed input by design (that's the whole point of `ImportError`) |
| V. Stratified Error Handling | `fireside-cli` boundary uses `anyhow::Result`; library-shaped logic uses typed errors | PASS — `import()` returns `Result<Graph, ImportError>` (typed), `main.rs`'s command handler wraps it with `anyhow::Context` at the I/O boundary, matching `validate_file`'s existing pattern |
| VI. MSRV 1.88 | New dependency verified MSRV-compatible before adoption | PASS — `pulldown-cmark` 0.13 built clean under `cargo +1.88 build` (ADR-006) |
| VII. Test Discipline | Feature has unit and/or integration test coverage at the correct layer | PASS — see Testing above; one unit test per FR plus one CLI wiring e2e test |

No unjustified violations. The one boundary-table change is justified,
ADR-backed, and already applied. Complexity Tracking is not needed beyond
noting that amendment.

## Project Structure

### Documentation (this feature)

```text
specs/003-markdown-import/
├── plan.md                    # This file
├── research.md                # Phase 0 output
├── data-model.md              # Phase 1 output
├── quickstart.md              # Phase 1 output
├── contracts/
│   └── cli-import.md
└── tasks.md                   # Phase 2 output (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
crates/fireside-cli/
├── Cargo.toml          # + pulldown-cmark dependency
├── src/
│   ├── main.rs          # + Command::Import variant, command handler
│   │                    #   (file I/O + anyhow wrapping only); extracts
│   │                    #   the existing new_deck slug logic into a
│   │                    #   shared fn slugify(&str) -> String used by
│   │                    #   both new_deck and import
│   └── import.rs         # NEW: pub fn import(&str) -> Result<Graph, ImportError>;
│                         #   frontmatter parsing, section walking,
│                         #   content-block conversion, branch-fence
│                         #   parsing and two-pass target resolution,
│                         #   ImportError + its Display impl
└── tests/
    └── cli_e2e.rs         # + one import-verb wiring test
```

**Structure Decision**: one new module (`import.rs`) inside the existing
`fireside-cli` crate — large enough (frontmatter, section walking, content
conversion, branch parsing, two-pass resolution) to warrant its own file
rather than growing `main.rs` further, but still entirely within the
existing crate boundary; no new crate. `slugify` is extracted from
`new_deck` into a small shared helper both `main.rs::new_deck` and
`import.rs` call, per research.md §6.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| New dependency (`pulldown-cmark`) on `fireside-cli`'s boundary table | A hand-rolled Markdown parser risks silently mis-handling edge cases (lazy continuation, fence info-string parsing, nested emphasis) that a non-technical presenter would have no way to diagnose | Hand-rolling was the alternative considered and rejected in ADR-006 — ADR-006 is the required accompanying ADR for this boundary-table change per the constitution's Governance section |
