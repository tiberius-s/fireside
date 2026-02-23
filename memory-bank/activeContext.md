# Active Context

## Current State (as of 2026-02-26)

The **Fireside Improvement Initiative** (6 phases) is **100% complete**. The project is in a clean, stable, well-documented state. All CI jobs are green. The **Penpot design system** has been fully recoloured to Rosé Pine, deduplicated, and reorganized into a single cohesive 9-section document. A comprehensive **TUI Implementation Guidelines** document (`memory-bank/tui-implementation-guidelines.md`) is ready for coding agents. A **full codebase audit** has been completed and a phased implementation plan (Phase H + existing A–D) is tracked in TASK017.

### What is fully working right now

| Layer                                                                             | Status       |
| --------------------------------------------------------------------------------- | ------------ |
| Protocol `0.1.0` TypeSpec model + 18 JSON Schemas                                 | ✅ Complete  |
| `fireside-core` types (all ContentBlock variants, Graph, Node)                    | ✅ Complete  |
| `fireside-engine` (loader, validation, traversal, commands, session)              | ✅ Complete  |
| `fireside-tui` (presenter, editor, graph view, help overlay, themes)              | ✅ Complete  |
| `fireside-cli` binary (present, open, edit, new, validate, fonts, import-theme)   | ✅ Complete  |
| Test suite (core roundtrips, engine fixtures, CLI e2e, TUI smoke)                 | ✅ Complete  |
| CI (rust.yml lint/test/MSRV, docs.yml, models.yml, audit.yml)                     | ✅ All green |
| Git hooks (pre-commit fmt, pre-push clippy+test)                                  | ✅ Installed |
| Docs site (45 pages: spec, schemas, guides, crates deep-dives, Learn Rust series) | ✅ Complete  |
| Penpot design system (Rose Pine palette, 31 components, 17 UX boards, 9 sections) | ✅ Complete  |

### Key architectural decisions now locked

- **TEA invariant**: `App::update` is the sole mutation point; all render functions are pure.
- **Wire format**: kebab-case JSON everywhere; frozen for `0.1.x`.
- **Crate boundary rule**: `fireside-core` has zero I/O; `fireside-engine` has zero ratatui; `fireside-tui` has zero direct file I/O.
- **Protocol changes within `0.1.x`**: additive only — all new fields must be `Option` or `#[serde(default)]`.
- **Config surface**: JSON-only (`fireside.json` project config, `~/.config/fireside/config.json` user config). No YAML/TOML.
- **MSRV**: 1.88 (required by `darling@0.23` in the dependency tree).

## Active Decisions & Constraints

### Deferred (do not implement without explicit approval)

- HTML export / PDF output — deferred to `1.0.0` planning.
- YAML/TOML format variants — explicitly rejected for now.
- GUI tooling — outside project scope.
- Multi-author / real-time collaboration — outside project scope.

### Working conventions (active)

- No `unwrap()`/`expect()` in library code — use `Result`/`Option`.
- All public items get `///` doc comments; modules get `//!` file-level docs.
- `#[must_use]` on every value-returning function.
- `tracing::warn!` for recoverable render failures (e.g., bad image path) — never panic.
- After any structural graph mutation, call `Graph::rebuild_index()`.
- Syntax and theme assets are `LazyLock` statics — never re-initialize per render.
- Redraw is gated by `App::take_needs_redraw()` — only draw on state change or animation tick.

## Next Steps / Open Questions

**Immediate coding pass:**

1. **TASK017 is complete** — no remaining mandatory code items in phases A–H.
2. **Optional polish backlog** — richer per-block field editing beyond primary-field
   inline widgets can be scheduled as a follow-up task if requested.

**Longer-horizon:**

- Protocol `0.2.0` planning — additive fields, new block kinds.
- Performance profiling — flamegraph pass on `App::update` + render pipeline.

## Recent Changes (most recent first)

- 2026-02-26: **D1 + D3 completed; TASK017 now 100% complete** — Verified branch overlay
  affordances with an explicit presenter interaction test (Up/Down focus + Enter selection),
  and replaced graph overlay linear rendering with a tree-row ASCII topology renderer in
  `ui/graph.rs` (depth-aware connectors, edge-kind markers, row-based viewport/mouse mapping).
  Regression run passed for both `fireside-tui` and `fireside-engine` crates.

- 2026-02-26: **C1–C5 wave implemented + QA-cleared** — Added engine block-level commands
  (`UpdateBlock`, `MoveBlock`) with undo/redo tests, implemented block-level validation
  warnings (`validate_content_block`) and graph integration, and wired selected-block editing
  flow in TUI (`i` edits selected block field, Esc/Enter commit, Ctrl+C cancel). Added
  block selection/reorder keybindings (`Ctrl+j/k`, `Alt+j/k`), selected-block highlighting,
  and warning display in editor details. Independent QA follow-up fixes: warning filtering
  to actionable fields, block selection reset on node changes, and help text alignment.

- 2026-02-26: **B1–B3 + D2 wave implemented + QA-cleared** — Added reusable AppHarness test
  utility (`tests/harness.rs`), added golden integration tests for full hello traversal and
  branch choose behavior (`tests/harness_golden.rs`), and implemented a block-type picker with
  8 type options and one-line synopsis rows in editor mode. Independent QA found one blocker
  in picker mouse row mapping for two-line options; fixed with row-span aware index mapping.
- 2026-02-26: **H6/H7/H8 wave implemented + QA-cleared** — Implemented graph edge colour
  coding + bottom legend row, added new timeline module (`Ctrl+H` toggle), and added new
  breadcrumb module with branch-point jump action (`Ctrl+←`). Independent QA found one blocker
  (jump target resolving to branch destination instead of branch-point node), which was fixed by
  selecting the last visited node that is itself a branch point.
- 2026-02-26: **H4/H5 wave implemented + QA-cleared** — Added zen mode (`Ctrl+F`) with
  presenter chrome gating, added pace guide pipeline (`--target-minutes` CLI → session →
  app → presenter → progress bar), added pace thresholds/tests, and completed A2 traversal
  regression test validation. Independent QA blocker (target duration hidden by timer/progress
  settings) was fixed by forcing timer/progress visibility when target duration is provided.
- 2026-02-26: **TUI codebase audit + implementation plan** — Full audit of 8 source files
  (`theme.rs`, `design/tokens.rs`, `chrome.rs`, `progress.rs`, `event.rs`, `keybindings.rs`,
  `app.rs`, `config/keybindings.rs`) against the TUI implementation guidelines. Key findings:
  `Theme::default()` and `DesignTokens::default()` are still One Dark; `ModeBadgeKind` missing
  `Branch` variant and prefix icons; progress bar uses wrong dot characters and missing UX-02
  features; `Action` enum missing `ToggleZenMode`/`ToggleTimeline`/`JumpToBranchPoint`.
  Produced 6-phase implementation plan (H0a through H8) tracked as TASK017 subtasks. Plan
  consolidates guidelines §9 priority ordering with existing TASK017 A–D phases.
- 2026-02-26: **Penpot design system consolidation** — Complete recolour from Atom One Dark
  to Rosé Pine across all 46 boards (~1100 colour changes). Deleted 15 redundant boards
  (old Explorations superseded by UX boards, duplicate Library boards). Reorganized all
  content into 9 cohesive sections with dividers (Design Foundation, Components, Screen
  Layouts, Navigation & State Machine, Responsive Breakpoints, Accessibility, UX
  Improvements, New UX Proposals, Rose Pine Palette Reference). Fixed text clipping on UX-12
  and UX-16 boards. Repositioned 31 component instances. Created comprehensive TUI
  Implementation Guidelines document (`memory-bank/tui-implementation-guidelines.md`) with
  Rose Pine colour mapping, component specs, UX improvement priorities, and architecture
  rules for coding agents.
- 2026-02-25: **Rose Pine palette + 17 UX boards in Penpot** — Major Penpot design system
  expansion session. Fetched all 3 Rose Pine variants (Main, Moon, Dawn) including highlight
  colors. Created 45 library colors (`rp-{variant}/{role}`) and 3 token sets
  (`rosepine/main`, `rosepine/moon`, `rosepine/dawn`, 15 tokens each). Built 7 Rose Pine
  visual boards (overview, 3 palette swatches, TUI colour mapping, 2 TUI previews). Created
  4 new Penpot pages (Screens & Layouts, UX Proposals & Explorations, Rose Pine Palette,
  Flows & Architecture) — note: cross-page API limitation means all content remains on the
  Design System page. Implemented 9 original UX proposals as mockup boards (01–09:
  Persistent Mode Indicator, Progress Bar Upgrade, Branch Overlay Affordance, Metadata
  Selectors, Context-Preserving Help, Graph Edge Colour Coding, GotoNode Visual Feedback,
  Undo/Redo Visual State, Compact Breakpoint). Generated 8 new UX proposals from TUI
  codebase audit (10–17: Breadcrumb Navigation Trail, Node Preview on Hover, Command
  Palette, Session Timeline, Focus/Zen Mode, Presenter Timer & Pace Guide, Content Block
  Minimap, Micro-Interactions & Polish). Fixed critical z-order bug across all 17 boards
  (area-based sorting: large rects→back, text→front). All boards visually verified via
  export.
- 2026-02-24: **Penpot design system overhaul** — Deep audit and rebuild of the entire
  Penpot component library. Fixed all 8 typographies (were all 14px; now correct at
  48/32/24/20/16/14/12/14). Fixed color naming inconsistencies (6 renames, 1 value fix:
  `surface` from #282C34 to #21252B). Added `accent-cyan` (#56B6C2) and `foreground`
  (#DCDFE4) colors. Added 2 new tokens (36 total). Deleted duplicate empty board. Created
  **31 reusable components** across 10 categories: Button (4), Mode Badge (4), Status Chip
  (3), Keybinding Chip (1), Input (3), Progress Bar (1), Block Type (8), Content Block (3),
  Footer Bar (1), Branch Option (3). All components use flex layouts, proper font references,
  and One Dark palette. Built Component Showcase board organizing all 31 components by
  category with labels and dividers. All 31 components visually verified via export.
- 2026-02-23: **Penpot design system expansion (TASK017 Phase E)** — 4 new exploration boards
  (08 Block Edit Widgets, 09 Graph Tree View, 10 GotoNode Input, 11 Undo/Redo Chip States),
  2 new library boards (06 UI Components, 07 Typography & Color), fixed Board 03 branch
  overlay focus visibility bug, added 8 typography styles and 22 design tokens to Penpot
  library, corrected Token Catalog font reference (JetBrains Mono). All boards visually
  verified via export. TASK017 Phase E is complete; Phases A–D (code) remain.
- 2026-02-22: **Second-pass UX audit (TASK017)** — Ten issues reported directly by the
  author after TASK016. Confirmed mouse double-fire bug root cause in `app.rs` lines
  675–683 (both `MouseEventKind::Down` and `MouseEventKind::Up` dispatch
  `Action::MouseClick`). Created three UX research artifacts in `memory-bank/ux/`:
  `tui-second-pass-jtbd.md`, `tui-second-pass-journey.md`, `tui-second-pass-flow.md`.
  Nine implementation-ready flows (F1–F9) defined. Penpot board handoff notes prepared.
  P0 bugs: mouse double-fire (F1), branch-loop investigation (F2).
