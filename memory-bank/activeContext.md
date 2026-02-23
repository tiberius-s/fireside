# Active Context

## Current State (as of 2026-02-26)

The **Fireside Improvement Initiative** (6 phases) is **100% complete**. The project is in a clean, stable, well-documented state. All CI jobs are green. The **Penpot design system** has been fully recoloured to Rosé Pine, deduplicated, and reorganized into a single cohesive 9-section document. A comprehensive **TUI Implementation Guidelines** document (`memory-bank/tui-implementation-guidelines.md`) is ready for coding agents.

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

1. **Theme::default() → Rose Pine** — The `Theme::default()` still uses One Dark colours. Apply the mapping from `memory-bank/tui-implementation-guidelines.md` §1.4 to update it. This is the highest-priority code change.
2. **TASK017 Phases A–D (code)** — Mouse double-fire fix (A1), branch loop investigation (A2), test harness (B), content block editing (C), graph/branch visual improvements (D). All have implementation-ready flow specs (F1–F9).
3. **UX Improvements 01–17** — Full implementation specs in `memory-bank/tui-implementation-guidelines.md` §5, with priority ordering in §9.
4. **Protocol `0.2.0` planning** — What additive fields or new block kinds should go into the next minor?
5. **Performance profiling** — No profiling has been done. A flamegraph pass on `App::update` + render could surface regressions.

## Recent Changes (most recent first)

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
