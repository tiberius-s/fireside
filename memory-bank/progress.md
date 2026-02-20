# Progress

## Completed

- Protocol baseline normalized to `0.1.0`.
- TypeSpec versioning setup integrated.
- Docs site configured for static output with explicit spec chapter ordering.
- Core model updated to `container` + typed `extension` shape.
- Key spec, guide, schema-reference, ADR, and memory-bank pages updated.
- Root `specs/` quick-reference duplication removed; content moved to
  `docs/src/content/docs/reference/`.
- Runtime traversal now supports `after` branch-rejoin semantics in
  `TraversalEngine::next` with tests.
- CLI project flow now uses JSON project config (`fireside.json`) for
  open/edit resolution and scaffold generation.
- Presenter hot-reload loop and graph-reload safety behavior were validated with
  targeted tests, CLI tests, and crate-level clippy checks.
- Mermaid extension rendering was hardened (payload normalization, truncation
  safeguards, and preview overflow messaging) with focused tests.
- Settings handling was polished (key aliases, nested settings support,
  timeout bounds, and XDG-aware config path resolution) with tests.
- Editor mode now includes a graph-view overlay with ASCII edge summaries,
  keyboard/mouse navigation, and Enter/click jump-to-node behavior.
- Graph-view overlay now includes richer boxed edge-map rows, a mini-map
  side panel, and viewport navigation controls (`PgUp/PgDn/Home/End`).
- Graph-view edge map now renders multi-line branch fan-out edges per node,
  with paging and click-to-jump behavior aligned to variable row heights.
- Graph-view overlay now supports direct graph-to-presenter handoff (`p`) at
  the selected node, preserving traversal/editor selection sync.
- Presenter mode now hands off to editor mode (`e`) with a status breadcrumb
  that confirms current-node context (`Presenter → editor @ node #N`).
- Graph fan-out rows now align edge-kind and target labels with explicit
  connector glyphs (`├╼`/`└╼`) for clearer branch topology scanning.
- Help overlay now provides categorized, mode-aware shortcut sections with
  active/dimmed entries across presenter and editor modes.
- Help overlay now supports scroll navigation and section jumps (`1-6`) so
  full shortcut docs remain usable on compact terminals.
- Help overlay now includes a footer section index legend and scroll indicator
  to improve discoverability of jump/navigation controls.
- Help footer section legend now reflects mode context by emphasizing active
  sections and dimming out-of-mode sections.
- Holistic release/usability audit now passes end-to-end for Rust workspace
  checks and TypeSpec schema generation; docs static build is clean after
  resolving Starlight 404 entry handling.
- Crate READMEs written for all four workspace members (`fireside-core`,
  `fireside-engine`, `fireside-tui`, `fireside-cli`): teaching-quality Rust
  documentation with architecture rationale, annotated code examples, module
  maps, and dependency tables.
- Root `README.md` updated with full feature set documentation: all CLI
  subcommands with examples, complete keybinding tables for presenter and
  editor modes, iTerm2 theme import flow, and cross-links to crate READMEs.
- Phase 1 of `plan-fireside-improvement-initiative` completed end-to-end:
  additive protocol fields landed in TypeSpec and `fireside-core`, schema/spec
  docs updated, and `docs/examples/hello.json` updated with
  `fireside-version` and node-level `title` metadata.
- Phase 1 verification gates are green:
  - `cd models && npm run build`
  - `cargo build && cargo test --workspace`
  - `cd docs && npm run build`
- Phase 2 of `plan-fireside-improvement-initiative` completed end-to-end:
  - Removed unused workspace dependencies (`serde_yaml`, `toml`)
  - Added `.cargo/config.toml` with Apple Silicon linker optimization
  - Added `[profile.dev.package."*"] opt-level = 2`
  - Evaluated and adopted pure-Rust syntect feature set
    (`default-syntaxes`, `default-themes`, `regex-fancy`)
  - Added Rust CI workflow with `cargo nextest run --workspace`
  - Updated root README build/test section with nextest command
- Phase 2 verification gates are green:
  - `cargo build`
  - `cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`
- Phase 3 of `plan-fireside-improvement-initiative` completed end-to-end:
  - Added canonical `Graph::rebuild_index()` API and switched engine command
    mutations to use it.
  - Cached syntect assets in `fireside-tui` (`LazyLock` for syntax and themes)
    to avoid per-render reinitialization.
  - Added redraw gating (`needs_redraw`) so terminal draw calls only occur when
    state changes or animation ticks.
  - Capped traversal history at 256 entries with `VecDeque`.
- Phase 3 verification gates are green:
  - `cargo build`
  - `cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`
- Phase 4 of `plan-fireside-improvement-initiative` completed end-to-end:
  - Added `EngineError::PathTraversal` in `fireside-engine`
  - Hardened image-path sanitization with base-dir confinement and
    parent-traversal rejection in markdown renderer
  - Added iTerm2 plist pre-parse file-size guard (1 MB)
  - Added normative extension payload safety section in extensibility spec
  - Added unit/integration coverage for path sanitization and iTerm2
    oversized-file rejection
- Phase 4 verification gates are green:
  - `cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`
- Phase 5 of `plan-fireside-improvement-initiative` completed end-to-end:
  - Added `fireside-core` integration coverage for `ContentBlock` round trips
    across all variants and edge cases (`tests/content_roundtrip.rs`)
  - Added engine fixture validation suite and command-history invariant tests
    (`tests/fixtures/*`, `validation_fixtures.rs`, `command_history.rs`)
  - Added CLI end-to-end command tests for validate and scaffold flows
    (`crates/fireside-cli/tests/cli_e2e.rs`)
  - Added TUI smoke extension asserting non-empty render output per node
    (`crates/fireside-tui/tests/hello_smoke.rs`)
  - Added CLI test-only dependencies (`assert_cmd`, `predicates`, `tempfile`)
- Phase 5 verification gates are green:
  - `cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`
- Phase 6 of `plan-fireside-improvement-initiative` completed end-to-end:
  - Added theme authoring guide (`docs/src/content/docs/guides/theme-authoring.md`)
  - Added extension authoring guide (`docs/src/content/docs/guides/extension-authoring.md`)
  - Added keybindings reference (`docs/src/content/docs/reference/keybindings.md`)
  - Added migration placeholder (`docs/src/content/docs/spec/migration.md`)
  - Added full Learn Rust tutorial series (`docs/src/content/docs/guides/learn-rust/`):
    `_index.md` plus chapters 1-8
  - Updated docs sidebar in `docs/astro.config.mjs` for explicit ordering and
    navigation of all new pages
- Phase 6 verification gate is green:
  - `cd docs && npm run build`

## In Progress

- Bring Rust reference implementation vocabulary fully in line with protocol
  naming where legacy terms remain.

## Known Follow-Up

- Validate generated schema pages and examples against the latest TypeSpec
  output after each protocol model change.
- Keep competitive analysis as internal context in memory-bank unless a public
  docs publication is explicitly requested.
- Keep export formats (HTML/PDF) deferred while TUI usability milestones are
  still in progress.
