<!--
Sync Impact Report
- Version change: 1.0.0 → 1.1.0
- Modified principles: III. Crate Boundary Discipline — `fireside-cli`'s
  permitted dependency list gains `pulldown-cmark`, per ADR-006 (Markdown
  authoring frontend, `fireside import`). No principle removed or
  redefined; this materially expands existing guidance, hence MINOR.
- Added sections: none
- Removed sections: none
- Templates requiring updates: none (boundary table is referenced, not
  duplicated, elsewhere)
- Follow-up TODOs: none

Sync Impact Report (previous)
- Version change: (template) → 1.0.0
- Modified principles: n/a (initial ratification)
- Added sections: Core Principles (I–VII), Operational Constraints,
  Development Workflow & Quality Gates, Governance
- Removed sections: none (all template placeholders filled)
- Templates requiring updates:
  ✅ .specify/templates/plan-template.md — Constitution Check gate is
     populated per-feature from this file; no static edit required
  ✅ .specify/templates/spec-template.md — no constitution-mandated
     sections beyond defaults
  ✅ .specify/templates/tasks-template.md — test-first ordering already
     compatible with Principle VII
  ✅ AGENTS.md — slimmed to operational pointer (same change set)
- Follow-up TODOs: none
-->

# Fireside Constitution

## Core Principles

### I. Spec Is the Source of Truth (NON-NEGOTIABLE)

The protocol specification is canonical: `protocol/main.tsp`, the generated
schemas in `protocol/tsp-output/schemas/`, and `docs/src/content/docs/spec/`.
When code and spec disagree, the code changes. No field, enum variant, or
traversal behavior may exist in code that is not in the spec. Any extension
MUST be specified first and registered in the spec's "Engine Extensions"
appendix before implementation begins. `docs/examples/hello.json` is the
canonical document — it MUST parse, validate, and present correctly after
every change.

*Rationale: the protocol is a portable format; third-party engines can only
trust it if the reference implementation never drifts ahead of the spec.*

### II. Presenter-First Experience

The presenter MUST be usable by non-technical people. Every design decision
is argued from the presenter's experience: the footer shows exactly the
valid keys, every blocked action gives feedback, and simplicity beats
surface area. Product scope is presenter-first — `present`, `validate`,
`new` (per ADR-004); scope additions are rejected unless the user
explicitly asks for them.

*Rationale: a deck tool that requires technical knowledge to drive fails
its audience at the worst possible moment — live, on stage.*

### III. Crate Boundary Discipline

Each crate has a dependency allowlist. Anything not listed is forbidden.

| Crate             | Permitted dependencies                                        | Explicitly forbidden                              |
| ----------------- | ------------------------------------------------------------- | ------------------------------------------------- |
| `fireside-core`   | `serde`, `serde_json`, `thiserror`                             | Any I/O, UI, validation, or rendering code        |
| `fireside-engine` | `fireside-core`, `thiserror`                                   | File I/O, ratatui, crossterm, clap, anyhow        |
| `fireside-tui`    | `fireside-core`, `fireside-engine`, `ratatui`, `crossterm`, `unicode-width`, `syntect`, `two-face`, `thiserror` | Direct file I/O, business logic duplication |
| `fireside-cli`    | All workspace crates, `clap`, `anyhow`, `serde_json`, `pulldown-cmark` | State management, rendering outside `fireside-tui` |

Any proposal that would violate this table MUST be flagged with an explicit
warning and an alternative that respects the boundaries.

*Rationale: the layering (pure model → state machine → renderer → shell) is
what keeps the engine portable and the TUI testable.*

### IV. Mandatory Code Idioms

- No `unwrap()` or `expect()` in library code; return `Result`/`Option`.
  Acceptable only in `main()`, tests, and `LazyLock` initializers.
- `#[must_use]` on every public function returning a value the caller
  should act on.
- `///` doc comments on every public item; `//!` module docs on every file.
- TEA invariant: `App::update` in `fireside-tui` is the ONLY function that
  mutates `App` state; rendering is pure.
- All visual styling flows through `theme.rs::Tokens` — never construct a
  `Style` from raw colors in render code.
- Engine operations return `Outcome` — no traversal operation may become a
  silent no-op; the UI MUST be able to give feedback for every keypress.
- Serde attributes use `rename_all = "kebab-case"`; content blocks use the
  `kind` discriminator.
- Sessions own an immutable graph; the node index is built once at
  `Session::new`.

### V. Stratified Error Handling

| Layer                      | Required approach                        |
| -------------------------- | ---------------------------------------- |
| `fireside-core`            | `thiserror` typed errors — `CoreError`   |
| `fireside-engine`          | `thiserror` typed errors — `EngineError` |
| `fireside-tui`             | `thiserror` typed errors — `TuiError`    |
| CLI / application boundary | `anyhow::Result` with context chains     |

`anyhow` MUST NOT appear inside library crates. Raw `Box<dyn Error>` is
forbidden everywhere.

### VI. MSRV 1.88

The workspace MSRV is 1.88 (`resolver = "3"`, 2024 edition). Before
recommending a crate, verify its MSRV is ≤ 1.88. Before recommending a
`std` API, verify it stabilized before 1.88. Any proposal that raises the
MSRV MUST be flagged and requires an explicit user decision.

### VII. Test Discipline

- Engine semantics (history invariants, branch gating) are unit tests in
  `fireside-engine/src/session.rs` and `validation.rs`.
- Every user-visible TUI state gets a scenario test in the
  `fireside-tui/src/render/mod.rs` suite: drive real key events through
  `App::update`, render to ratatui's `TestBackend`, assert the screen.
- CLI behavior is covered end-to-end in `fireside-cli/tests/cli_e2e.rs`.
- UI changes additionally get a real-terminal smoke test: drive the built
  binary in a detached tmux session (`tmux send-keys` / `capture-pane`).

A feature is not done until its tests exist at the correct layer.

## Operational Constraints

- `cargo test --workspace` — full test suite; MUST pass before any task is
  marked complete.
- `cargo clippy --workspace --all-targets` — MUST stay silent.
- `node protocol/validate.mjs <file>` — semantic validation of a document.
- `cd protocol && npm run build` — regenerate schemas from TypeSpec after
  any `main.tsp` change; `tsp-output/` is committed (CI enforces this).
- `npm run check --prefix docs` — docs site type/build check.
- `graphify update .` — refresh the knowledge graph after modifying code.

## Development Workflow & Quality Gates

- Features follow the Spec Kit pipeline: `/speckit-specify` →
  (`/speckit-clarify` when ambiguous) → `/speckit-plan` → `/speckit-tasks`
  → `/speckit-implement`, with artifacts in `specs/NNN-feature-name/`.
  Bug fixes and mechanical chores may skip the pipeline.
- The plan's Constitution Check gate MUST pass before implementation; any
  violation is either redesigned away or justified in Complexity Tracking
  with an explicit user decision.
- Architectural decisions are recorded as ADRs in `.claude/adrs/`. A change
  that touches the wire format requires a spec change (Principle I) and an
  ADR before code.
- Protocol changes MUST regenerate and commit `tsp-output/`.

## Governance

This constitution supersedes all other practice documents. AGENTS.md is an
operational pointer to this file plus day-to-day commands; if they
disagree, the constitution wins.

- **Amendments**: proposed as a diff to this file, accompanied by an ADR
  when the change is architectural, and approved by the project owner.
- **Versioning**: semantic. MAJOR for principle removals or redefinitions,
  MINOR for new principles or materially expanded guidance, PATCH for
  clarifications.
- **Compliance review**: every `/speckit-plan` run re-checks this file via
  its Constitution Check gate; reviewers verify compliance on every PR.

**Version**: 1.1.0 | **Ratified**: 2026-07-12 | **Last Amended**: 2026-07-12
