# Active Context

## Current State (as of 2026-02-23)

The **Fireside Improvement Initiative** (6 phases) is **100% complete**. The project is in a clean, stable, well-documented state. All CI jobs are green.

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

1. **Protocol `0.2.0` planning** — What additive fields or new block kinds should go into the next minor? Export capabilities (HTML/PDF), richer extension ecosystem, or audio/media block types are candidates.
2. **UX polish pass** — The branch fan-out layout in the graph overlay and presentation transition polish could use another pass before the `1.0.0` push.
3. **Performance profiling** — No profiling has been done. The `LazyLock` for syntect assets and redraw gating are the primary optimizations currently. A flamegraph pass on `App::update` + render could surface regressions.
4. **Accessibility** — The WCAG contrast check is implemented in `DesignTokens` but not enforced on import. A validation warning for insufficient contrast themes would be a good addition.

## Recent Changes (most recent first)

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
