# Implementation Plan: Presenter Polish

**Branch**: `010-presenter-polish` (worked directly on `main`, matching every
prior spec 001-009 in this repo — no dedicated feature branch) | **Date**:
2026-07-18 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/010-presenter-polish/spec.md`

## Summary

Five small, independently shippable presenter/authoring feedback gaps from
`.claude/plans/2026-07-18-ux-polish-plan.md` Wave 2: a validator warning for
branch option keys that collide with the presenter's reserved global keys
(P1 — closes the exact bug class that shipped in the demo deck and had to be
hotfixed in Wave 1); a one-line rehearsal summary after a graceful `q` quit;
a flash message announcing a resumed session; an interactive `fireside new`
prompt to present the just-created deck immediately; and a stderr width
note on `fireside art text` output that would exceed the 76-column
authoring threshold. All five reuse existing mechanisms (flash messages,
`Session::visited()`/`Outcome`, the `add_title_banner` width-guard pattern,
the `Diagnostic` shape) — no new subsystems, no protocol/wire changes.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.88 (2024 edition, `resolver =
"3"`)

**Primary Dependencies**: No new dependencies. Touches existing
`fireside-core`, `fireside-engine`, `fireside-tui`, `fireside-cli` crates
only (`ratatui`, `crossterm`, `clap`, `anyhow` already in place per crate).

**Storage**: N/A (no new persisted state; the existing resume-state file
`fireside-cli/src/resume.rs` is read, not changed)

**Testing**: `cargo test --workspace` (unit tests in
`fireside-engine/src/validation.rs` for the new rule; scenario tests in
`fireside-tui/src/render/tests.rs`/`app.rs` for the flash and the
reserved-key regression guard; `fireside-cli/tests/cli_e2e.rs` for the exit
summary, wizard prompt, and `art text` note); real-terminal tmux smoke tests
for the two TUI-visible changes (exit summary, resume flash) per
Constitution Principle VII.

**Target Platform**: Same as the rest of the project — any terminal Fireside
already runs on (macOS/Linux, plus whatever the CI matrix covers); no new
platform surface.

**Project Type**: CLI + TUI (existing single Cargo workspace, four crates)

**Performance Goals**: N/A — no hot-path or throughput-sensitive code; all
five changes are O(1) or O(nodes)/O(branch options) checks already within
existing linear-scan validation passes and per-frame flash rendering.

**Constraints**: Must respect Constitution Principle III's crate dependency
allowlist (notably: `fireside-engine` cannot depend on `fireside-tui`, which
shapes where the reserved-key constant lives — see `research.md` §1) and
Principle IV's TEA invariant (`App::update` remains the sole `App` mutator;
the resume flash is set once at construction in `present_authoring`, before
the event loop's first `update` call, not from inside it).

**Scale/Scope**: 5 independent user stories, each a handful of files; no
architectural change. See Project Structure below for the concrete file
list.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

- **I. Spec Is the Source of Truth** — PASS. No protocol/wire-format change;
  `reserved-branch-key` is a reference-engine validation rule (like
  `malformed-link-url`), not a new protocol field, so it needs no
  `main.tsp`/schema change, only the Appendix D + validation-doc entries
  already planned (research.md §7). `docs/examples/hello.json` is
  unaffected.
- **II. Presenter-First Experience** — PASS, directly serves this
  principle: every one of the five stories is presenter- or author-facing
  feedback for an action that previously gave none or gave it too late.
  Product scope stays within `present`/`validate`/`new` (ADR-004) —
  `art text` is existing scope, not an addition.
- **III. Crate Boundary Discipline** — PASS, with one specific design
  choice recorded to satisfy it: `RESERVED_PRESENTER_KEYS` lives in
  `fireside-engine` (not `fireside-tui`) precisely because `fireside-engine`
  cannot depend on `fireside-tui` (research.md §1). `fireside-tui`'s
  `present`/`present_watching`/`present_authoring` return a plain summary
  struct rather than printing themselves, so the CLI (not the TUI crate)
  owns the one new line of stdout output (research.md §2) — no rendering or
  state management moves into `fireside-cli`. `new_deck` returning
  `Option<PathBuf>` and letting `main.rs` call the existing `present()`
  avoids `new.rs` needing to depend on `resume`/`watch` wiring it doesn't
  otherwise need (research.md §6).
- **IV. Mandatory Code Idioms** — PASS. No `unwrap()`/`expect()` introduced
  outside tests. `set_flash` widening from private to `pub(crate)` (not
  `pub`) preserves encapsulation. The resume flash is set once, synchronously,
  during `App::new`'s call site in `present_authoring` — before the event
  loop starts, not a new mutation path bypassing `App::update`. New public
  items (`PresentSummary`, `RESERVED_PRESENTER_KEYS`) get `///` docs;
  `#[must_use]` where applicable.
- **V. Stratified Error Handling** — PASS. No new error variant needed:
  `PresentSummary` travels on the existing `Ok` path of `Result<_,
  TuiError>`; the width guard and wizard prompt use existing `anyhow`
  context at the CLI boundary; the validation rule uses the existing
  `Diagnostic`/`Severity` types, no new error type.
- **VI. MSRV 1.88** — PASS. No new dependency, no new `std` API beyond what
  the workspace already uses (`Duration` formatting, `char` comparisons).
- **VII. Test Discipline** — PASS, planned explicitly: engine-layer unit
  tests for the new rule, TUI scenario tests for the flash, a
  cross-crate regression test tying `RESERVED_PRESENTER_KEYS` to
  `on_present_key`'s actual dispatch, CLI e2e tests for the summary/prompt/
  width note, and tmux smoke tests for the two TUI-visible stories (US2,
  US3) per the project's own memory note that `TestBackend` alone misses
  timing/ordering bugs.

No violations — Complexity Tracking is not needed.

## Project Structure

### Documentation (this feature)

```text
specs/010-presenter-polish/
├── plan.md              # This file
├── research.md           # Phase 0 output — 7 design decisions
├── data-model.md         # Phase 1 output — entity/field mapping, no new persisted data
├── quickstart.md         # Phase 1 output — 5 manual validation scenarios
├── contracts/            # Phase 1 output
│   ├── validation-reserved-branch-key.md
│   ├── present-summary-and-resume-flash.md
│   ├── art-text-width-guard.md
│   └── new-wizard-present-now.md
└── tasks.md              # Phase 2 output (/speckit-tasks — not yet created)
```

### Source Code (repository root)

Existing single-workspace CLI+TUI layout (no new crates, no new top-level
directories). Concrete files this feature touches:

```text
protocol/
├── validate.mjs            # + checkReservedBranchKeys mirroring validation.rs, HELP text entry (Rust/Node parity, research.md §8)
├── fixtures/valid/reserved-branch-key.json  # new fixture proving Rust/Node rule-id parity
└── fixtures.expected.json  # + entry for the new fixture

crates/fireside-engine/src/
├── validation.rs          # + RESERVED_PRESENTER_KEYS, check_reserved_branch_keys, its Diagnostic + unit tests

crates/fireside-tui/src/
├── lib.rs                 # present/present_watching/present_authoring return PresentSummary; resume-flash call site
├── app.rs                 # set_flash: private -> pub(crate); PresentSummary reads session.visited()/elapsed() via existing accessors
└── render/tests.rs         # + scenario test(s) for resume flash on first frame; + reserved-key regression test

crates/fireside-cli/
├── src/main.rs             # present()/demo(): print exit summary on Ok(summary); New arm: call present() when new_deck returns Some(path)
├── src/new.rs               # new_deck: Result<()> -> Result<Option<PathBuf>>; interactive_new: + present-now prompt
├── src/art.rs                # art_text: stderr width note (reuses DEFAULT_ART_WIDTH + widest-line measurement)
└── tests/cli_e2e.rs          # + e2e coverage for exit summary, wizard present-now prompt, art text width note

docs/src/content/docs/spec/
├── validation.md            # + reserved-branch-key bullet in Recommended Checks (~L68)
└── appendix-engine-extensions.md  # + one bullet in "Behavior near the protocol's edges" (malformed-link-url precedent)
```

**Structure Decision**: Single Cargo workspace, four existing crates
(`fireside-core`, `fireside-engine`, `fireside-tui`, `fireside-cli`) plus the
Astro docs site — unchanged from every prior spec in this repo. No new
crate, module, or top-level directory; every change is additive within
files that already own the relevant concern (validation rules in
`validation.rs`, presenting in `lib.rs`/`app.rs`, CLI-shell concerns in
`fireside-cli`).

## Complexity Tracking

*No Constitution Check violations — this section is not needed.*
