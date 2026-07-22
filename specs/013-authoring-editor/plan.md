# Implementation Plan: Authoring Editor (`fireside edit`)

**Branch**: `013-authoring-editor` | **Date**: 2026-07-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/013-authoring-editor/spec.md`

## Summary

Add a fourth verb, `fireside edit <deck>`, opening a full-screen, mouse-first,
block-based authoring studio that reuses the presenter's own renderer for the
editing canvas (WYSIWYG by construction) and represents every slide as a
stack of discrete, clickable blocks — never raw JSON or graph vocabulary.
Technical approach: a pure `engine::authoring` transform layer
(`(Graph, Op) -> Result<Graph, AuthoringError>`) in `fireside-engine`; a new
`EditorApp` TEA state machine and generalized `hit()` region-testing function
in `fireside-tui`, rendering through an extracted `SlideView` so the canvas
and the presenter share one rendering path; and a thin `fireside-cli`
`edit.rs` owning file I/O, draft-sidecar persistence, and the
create-if-missing flow. No new dependencies, no protocol change. Full detail
in the pre-spec design brief this plan formalizes:
`.claude/plans/2026-07-19-wysiwyg-editor-plan.md` (rev 3).

## Technical Context

**Language/Version**: Rust 1.88 (2024 edition, `resolver = "3"`) — workspace MSRV, unchanged.

**Primary Dependencies**: `ratatui`, `crossterm` (mouse capture already enabled process-wide), `unicode-width`, `syntect`/`two-face`, `thiserror` — all already on `fireside-tui`'s/`fireside-engine`'s permitted-dependency list (Constitution III). No new crate is added by this feature.

**Storage**: Deck files are the existing native JSON protocol format (no schema change). A new per-deck draft sidecar under `$XDG_STATE_HOME/fireside/drafts/` (same directory family and hashing scheme as `fireside-cli/src/session.rs`'s session-state files).

**Testing**: `cargo test --workspace` (unit + proptest in `fireside-engine`; `TestBackend` scenario + `insta` snapshot tests in `fireside-tui`, driving both key and synthetic `MouseEvent`s); `fireside-cli/tests/cli_e2e.rs`; real-terminal tmux smoke (`scripts/smoke.sh`), including injected SGR mouse sequences.

**Target Platform**: Same terminal targets as the existing presenter (macOS/Linux terminal emulators + tmux); no new platform surface.

**Project Type**: Single Cargo workspace, CLI + TUI (existing 4-crate layered structure: `fireside-core` → `fireside-engine` → `fireside-tui` → `fireside-cli`).

**Performance Goals**: Every editor interaction (select, navigate, undo, drag) completes within 100ms, for decks up to 500 slides with deep branching (spec SC-009, clarified 2026-07-21).

**Constraints**: No new dependencies (Constitution III/VI); TEA invariant — exactly one `update` function per TUI application struct (Constitution IV, generalization required — see Constitution Check); MSRV 1.88 unaffected; offline, single-user, single-window (spec Assumptions).

**Scale/Scope**: Decks up to 500 slides with deep branching remain responsive per SC-009; five delivery waves (E0–E4) per the design brief, each independently releasable; total size L–XL (several focused weeks), decomposed by `/speckit-tasks`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Spec Is the Source of Truth)**: PASS. No field, enum, or
  traversal-behavior change to `protocol/main.tsp` — `engine::authoring`
  operates entirely on the existing `Graph`/`Node`/`ContentBlock` model via
  operations that produce the same shapes validation already accepts. No
  `tsp-output/` regeneration needed.
- **Principle II (Presenter-First Experience)**: PASS, with a recorded scope
  extension — identical mechanism to spec 012's `notes` verb. `edit` is a
  fourth verb beyond the ADR-004 baseline (`present`/`validate`/`new`),
  explicitly requested by the user on 2026-07-19 (audit addendum A-3,
  promoted to scoped work the same day). **Action**: ADR-017 records this
  extension (mirrors `adr-014-dual-screen-presenter-view-scope.md`'s
  pattern) plus the editor-only inversion of interaction posture
  (mouse-first, keyboard-complete — presenter itself stays keyboard-first,
  mouse-additive; this inversion is scoped to `fireside edit` only). The
  presenter's own footer/key-teaching discipline is unaffected.
- **Principle III (Crate Boundary Discipline)**: PASS, no allowlist change.
  `engine::authoring` (new module in `fireside-engine`) uses only
  `fireside-core` + `thiserror`, already permitted. The editor's state
  machine, hit-testing, and rendering (new modules in `fireside-tui`) use
  only `ratatui`/`crossterm`/`thiserror`, already permitted — mouse capture
  is already enabled process-wide, no new crossterm feature needed. `edit.rs`
  (new module in `fireside-cli`) uses only already-permitted deps (`clap`,
  `anyhow`, `serde_json`, plus the existing `figlet-rs`/`rascii_art` for the
  text-art-generation callback). No crate gains a dependency it doesn't
  already have.
- **Principle IV (Mandatory Code Idioms)**: PASS, with a required PATCH
  constitution amendment (pre-approved by the design brief, see below). The
  TEA-invariant wording currently names `App::update` in `fireside-tui`
  specifically; this feature adds a second, independent TEA struct
  (`EditorApp`) with its own sole mutator, `EditorApp::update`. The wording
  must generalize to "each TUI application struct has exactly one update
  function" so the invariant unambiguously covers both `App` and
  `EditorApp` rather than reading as `App`-specific. New theme tokens
  (`affordance`, `selection`, `drop-target`, `ghost`) extend
  `theme.rs::Tokens` following the existing pattern (no raw `Style`
  construction in render code) — additive, not a new rule.
- **Principle V (Stratified Error Handling)**: PASS. `fireside-engine` gains
  a second `thiserror` type, `authoring::AuthoringError`, alongside the
  existing `EngineError` — precedented by `fireside-tui` already carrying
  two typed errors (`TuiError`, `WriteBackError`). `fireside-cli` continues
  to wrap everything in `anyhow::Result` at the boundary. No `anyhow` inside
  library crates, no raw `Box<dyn Error>`.
- **Principle VI (MSRV 1.88)**: PASS. No new dependency, no `std` API newer
  than 1.88 required (draft-sidecar I/O and hashing reuse the exact patterns
  already in `fireside-cli/src/session.rs` and `resume.rs`).
- **Principle VII (Test Discipline)**: PASS, discipline mapped per layer —
  see "Test discipline map" carried over from the design brief and restated
  in `quickstart.md`. `engine::authoring` gets unit tests plus two proptests
  (no-dangling-reference invariant across arbitrary op sequences; retitle
  never dangles a reference) before any TUI code consumes it (TDD, per
  Constitution VII and repo convention on this project).

**Two ADRs and one constitution PATCH amendment required before
implementation** (same mechanism spec 012 used for ADR-014/015):

1. **ADR-017 (proposed): ADR-004 scope extension — `fireside edit`.** Records
   the 2026-07-19 user request satisfying Principle II's scope-addition
   gate, and the editor-only mouse-first/keyboard-complete interaction
   posture (an explicit, scoped inversion of the presenter's own posture).
2. **ADR-018 (proposed): `engine::authoring` module charter.** Records the
   `Op`/`AuthoringError` design (pure `(Graph, Op) -> Result<Graph,
   AuthoringError>`, full-clone undo snapshots over op-inversion, the
   id-slug/rename algorithm, the outline depth-first ordering algorithm) and
   bundles the TEA-wording PATCH amendment (Constitution 1.3.0 → 1.3.1) plus
   the new `theme.rs::Tokens` entries.

Both ADRs (and the bundled constitution amendment) are the first tasks
`/speckit-tasks` generates — governance artifacts, not code, landing before
any file-writing or state-machine code is merged, exactly as spec 012 did.

*No unjustified Constitution Check violations — this section is empty by
design. The one deliberate scope addition (Principle II) and the one
deliberate wording generalization (Principle IV) are not violations; they
are recorded, user-requested extensions per the mechanisms Principles II and
the Governance amendment process specify, both already approved via the
2026-07-19 design brief (rev 3) that commissioned this exact plan.*

## Project Structure

### Documentation (this feature)

```text
specs/013-authoring-editor/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md         # Phase 1 output (/speckit-plan command)
├── quickstart.md         # Phase 1 output (/speckit-plan command)
├── contracts/             # Phase 1 output (/speckit-plan command)
│   ├── cli-edit-command.md
│   ├── authoring-ops.md
│   └── hit-testing.md
└── tasks.md              # Phase 2 output (/speckit-tasks command — NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/
├── fireside-core/                    # unchanged — no model/field additions
│
├── fireside-engine/
│   └── src/
│       ├── authoring.rs              # NEW: Op enum, transforms, AuthoringError,
│       │                             #      outline-ordering fn, slug/rename algorithm,
│       │                             #      unit tests + 2 proptests
│       ├── session.rs                # unchanged
│       └── validation.rs             # unchanged (editor construction avoids most
│                                      #   invalid states by design; remaining
│                                      #   diagnostics reuse existing rules())
│
├── fireside-tui/
│   └── src/
│       ├── app.rs                    # unchanged (presenter's own App/update)
│       ├── theme.rs                  # + affordance/selection/drop-target/ghost tokens
│       ├── editor/                   # NEW module
│       │   ├── mod.rs                #   EditorApp struct + the sole `update()`
│       │   ├── hit.rs                #   hit(app, area, x, y) -> Option<Target>
│       │   ├── history.rs            #   undo/redo snapshot stack (full Graph clones)
│       │   └── forms.rs              #   per-block-kind form state (reuses EditableField)
│       ├── render/
│       │   ├── content.rs            # refactored (E0, behavior-neutral) to expose a
│       │   │                         #   `SlideView` input consumed by both the
│       │   │                         #   presenter and the editor canvas
│       │   └── editor/               # NEW: canvas overlay, outline, toolbar, forms,
│       │       ├── mod.rs            #   status/hint lines — calls content.rs's
│       │       ├── canvas.rs         #   SlideView path, then overlays affordances
│       │       ├── outline.rs
│       │       └── forms.rs
│       └── lib.rs                    # + visibility change only: event_loop() becomes
│                                      #   callable from editor::present_embedded
│
└── fireside-cli/
    └── src/
        ├── main.rs                   # + `Edit { file }` Command variant
        ├── edit.rs                   # NEW: subcommand entry, create-if-missing
        │                             #   (reuses new.rs/templates.rs), non-tty guard,
        │                             #   draft sidecar read/write/delete (mirrors
        │                             #   session.rs's fnv1a64 + resume.rs's resume_key)
        ├── new.rs                    # unchanged, templates reused by edit.rs
        └── session.rs                # unchanged (pattern reused, not shared code —
                                       #   session state and drafts are different files)
```

**Structure Decision**: Extends the existing 4-crate layered workspace with
no new crates and no crate-boundary changes. All new code lands as new
modules inside the existing crates listed above. `/speckit-tasks` decomposes
delivery into the design brief's five waves — **E0** foundations (ADRs,
constitution amendment, `engine::authoring`, `SlideView` refactor, `hit()`
skeleton), **E1** read-only studio, **E2** block editing, **E3** structure
editing, **E4** foolproofing polish — each wave independently releasable,
per Constitution's incremental-delivery norm and the design brief's own
wave gates (`scripts/verify.sh` passing, tmux smoke run, `graphify update .`
run, Progress Log ticked, per wave).

## Post-Design Constitution Check

*Re-evaluated after Phase 0/1 (research.md, data-model.md, contracts/).*
No new finding changes the gate above. Research confirmed, against actual
code rather than the design brief's assumptions, that every architectural
claim holds: the hit-testing pattern to generalize already exists
(`render/hits.rs:26,51`), the text-editing primitive to reuse already
exists (`app.rs:97`), the hashing/keying scheme to reuse already exists
twice (`session.rs:56`, `resume.rs:128`), the non-tty guard and parse-error
report to reuse already exist (`lib.rs:236`, `main.rs:284`), and a second
`thiserror` type per crate is already precedented
(`TuiError`/`WriteBackError`). One refinement: `render/map.rs`'s slide
ordering turned out to be fused with its rail-diagram layout rather than a
cleanly extractable function (`research.md` §8) — the outline-ordering
algorithm is implemented fresh in `engine::authoring` instead of shared,
which is a smaller, lower-risk footprint than the design brief's
"extract and share if present" framing, not a larger one. Still PASS, no
new Complexity Tracking entry.

## Complexity Tracking

*No entries — Constitution Check above passed with only the two recorded,
user-requested extensions described there; neither required a simpler
alternative to be rejected.*
