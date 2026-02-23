# Progress

## What Works (as of 2026-02-25)

### Protocol layer

- TypeSpec model (`models/main.tsp`) defines all 18 types; `npm run build` generates all JSON Schemas cleanly.
- Protocol `0.1.0` is fully spec-documented in 6 normative chapters + 3 appendices.
- Extended fields (`Node.title`, `Node.tags`, `Node.duration`, `Graph.fireside-version`, `Graph.extensions`) are additive and in the schema.

### Rust crates

- `fireside-core`: All `ContentBlock` variants, `Graph`/`GraphFile`, traversal types, `CoreError`. Full serde round-trip test coverage.
- `fireside-engine`: Loader, `validate_graph` with `Diagnostic` severity model, `TraversalEngine` (next/choose/goto/back), `CommandHistory` (undo/redo), `PresentationSession`. Fixture test suite + history invariant tests.
- `fireside-tui`: Full `App` state machine, all 5 `AppMode` transitions, `DesignTokens` / `Breakpoint` / `Spacing` / `NodeTemplate` design system, `render_block` pipeline for all 8 block kinds, iTerm2 scheme import, hot-reload, graph view overlay, help overlay. Smoke test passes.
- `fireside-cli`: All 7 subcommands (`present`, `open`, `edit`, `new`, `validate`, `fonts`, `import-theme`), event loop with frame gating and hot-reload, terminal lifecycle management. CLI e2e tests pass.

### DX & CI

- `cargo nextest run --workspace` is green.
- `cargo clippy --workspace -- -D warnings` is clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` is clean.
- CI: `rust.yml` (lint/test/MSRV 1.88), `docs.yml`, `models.yml`, `audit.yml` all green.
- `deny.toml` enforces license allowlist and advisory ignores.
- Git hooks: pre-commit (`fmt --check`), pre-push (clippy + nextest).

### Documentation

- Docs site builds cleanly at 45 pages (Astro + Starlight).
- Sections: Spec, Schemas, Reference, Guides, Crates deep-dives, Explanation.
- Full crate deep-dive set: `fireside-core`, `fireside-engine`, `fireside-tui` (4 articles), `fireside-cli`.
- Learn Rust with Fireside: 9 pages (overview + 8 chapters).

### Penpot Design System

- **Palette**: Rosé Pine (all One Dark traces removed). 45 library colors (`rp-main/*`, `rp-moon/*`, `rp-dawn/*`, 15 each).
- **81 design tokens**: 36 `fireside/core` + 15 `rosepine/main` + 15 `rosepine/moon` + 15 `rosepine/dawn`.
- **8 typographies**: Display/H1/H2/H3/Body/Small/Caption/Code (Source Sans Pro + JetBrains Mono).
- **31 reusable components**: Button (4), Mode Badge (4), Status Chip (3), Keybinding Chip (1), Input (3), Progress Bar (1), Block Type (8), Content Block (3), Footer Bar (1), Branch Option (3).
- **17 UX proposal boards**: 9 implementation boards (01–09) + 8 new proposals (10–17), all Rosé Pine.
- **7 Rose Pine visual boards**: Overview, 3 palette swatches, TUI colour mapping, 2 TUI previews.
- **9-section layout**: Design Foundation, Components, Screen Layouts, Navigation & State, Responsive Breakpoints, Accessibility, UX Improvements, New UX Proposals, Rose Pine Palette Reference.
- **TUI Implementation Guidelines**: `memory-bank/tui-implementation-guidelines.md` — comprehensive coding agent reference with colour mapping, component specs, UX priorities, architecture rules.

## What's Left / Known Gaps

- **TASK017 Phases A–D (code)** — Mouse double-fire fix, branch loop investigation, test harness, content block editing, graph/branch visual improvements. All have implementation-ready flow specs (F1–F9).
- **Protocol `0.2.0` planning** — No timeline set. Candidates: export block types, richer extension API, audio/media blocks.
- **Export formats (HTML/PDF)** — Explicitly deferred to `1.0.0` horizon.
- **WCAG contrast enforcement on import** — `contrast_ratio` exists in `DesignTokens` but no validation warning on theme import.
- **Profiling** — No flamegraph/perf analysis has been done. Render pipeline and `App::update` are candidates.
- **Penpot prototype flows** — 17 UX boards exist but no interactive prototype linking them.
- **Theme::default() → Rose Pine** — Code still uses One Dark. Mapping ready in `memory-bank/tui-implementation-guidelines.md` §1.4.
- **17 UX proposals (01–17)** — Designed in Penpot with full specs in implementation guidelines. Priority order: Theme update → UX-01 → UX-02 → UX-08 → UX-03 → UX-17 → UX-14 → UX-15 → UX-06 → UX-07 → UX-05 → UX-09 → UX-04 → UX-13 → UX-10 → UX-11 → UX-12 → UX-16.
