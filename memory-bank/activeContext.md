# Active Context

## Current State (as of 2026-02-20)

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

- 2026-02-20: **Penpot UX design session (TASK014)** — Full design audit of TUI in Presenting and Editing modes. Created 7 Penpot boards: Design System (palette, typography, spacing, borders), Presenter Mode, Presenter + Branch Overlay, Presenter + Help Overlay, Editor Mode, Editor + Graph Overlay, and a 9-card UX Improvements catalog. Identified 9 actionable UX gaps. Registered Penpot MCP server and `penpot-uiux-design` skill in `copilot-instructions.md`.
- 2026-02-20: Fixed `_index.md` → `index.md` naming in `guides/learn-rust/` (was returning 404 from Starlight sidebar).
- 2026-02-20: Added 7 expert crate deep-dive docs under `docs/src/content/docs/crates/` and corresponding Starlight sidebar section.
- 2026-02-20: Completed Phase 6 — theme authoring + extension authoring guides, keybindings reference, migration page, full 9-chapter Learn Rust tutorial series.

## UX Improvement Proposals (from TASK014)

The following 9 improvements have been designed and documented in Penpot board "07 — UX Improvements".

Low complexity: proposals 1–4. Medium: 5–8. High: 9.

1. **Persistent Mode Badge** — `■ PRESENT` (blue) / `✎ EDITING` (purple) in footer top-right. Non-breaking, pure UI.
2. **Progress Bar Upgrade** — Add next-node ID preview; show `⎇ BRANCH` indicator in gold when current node has a branch-point.
3. **GotoNode Autocomplete** — Show filtered node list above footer as user types, not just naked buffer.
4. **Undo/Redo Chip State** — Render `[ Z undo ]` / `[ Y redo ]` as greyed-out chips when unavailable.
5. **Branch Button Affordance** — Box-drawing key chip `╔═╗ ║a║ ╚═╝` per option instead of plain `[a]` text.
6. **Metadata Selector Chip Row** — Replace `◀ [val] ▶` with a visible chip row showing neighbouring values.
7. **Help Slide-in Panel** — Right-side 55% panel (not full-modal); left 45% stays visible and dimmed.
8. **Graph Edge Colour Coding** — blue=next, gold=branch, green=after, red=goto.
9. **Compact Breakpoint Adaptation** — At ≤80 cols, collapse node list to overlay; right pane takes full width.

- 2026-02-19: CI workflows hardened — MSRV bumped 1.85 → 1.88, added `libfontconfig1-dev` apt step, `deny.toml` license/advisory policy.
- 2026-02-19: Git hooks installed (`.githooks/pre-commit`, `.githooks/pre-push`, `.githooks/install.sh`).
- 2026-02-19: Completed Phases 1–5 of the Fireside Improvement Initiative (protocol enhancements, dep cleanup, perf optimizations, security hardening, test suite).

- TypeSpec updated with additive protocol fields:
  - `Node.title`, `Node.tags`, `Node.duration`
  - `Graph.fireside-version`
  - `Graph.extensions` via `ExtensionDeclaration`
- `fireside-core` model updated to match new wire shape (`Node`, `GraphFile`,
  `GraphMeta`, and extension declaration mapping).
- Docs updated in spec and schema reference pages for Graph and Node.
- `docs/examples/hello.json` updated with `fireside-version` and node `title`.
- Verification completed: `models npm run build`, `cargo build`,
  `cargo test --workspace`, and `docs npm run build` all green.

Phase 2 is now implemented end-to-end:

- Removed unused workspace dependencies (`serde_yaml`, `toml`) from root
  `Cargo.toml`.
- Added linker optimization config at `.cargo/config.toml` for Apple Silicon
  (`-ld_prime`).
- Added dependency optimization in dev profile:
  `[profile.dev.package."*"] opt-level = 2`.
- Evaluated and adopted a pure-Rust syntect feature set:
  `default-syntaxes + default-themes + regex-fancy`.
- Added Rust CI workflow using `cargo-nextest`:
  `.github/workflows/rust.yml`.
- Updated root README build/test section to include
  `cargo nextest run --workspace`.
- Validation completed: `cargo build`, `cargo test --workspace`,
  `cargo clippy --workspace -- -D warnings`.

Phase 3 is now implemented end-to-end:

- Added `Graph::rebuild_index()` in `fireside-core` and switched engine command
  mutations to use the shared rebuild path.
- Replaced per-call syntect initialization with cached `LazyLock` statics for
  `SyntaxSet` and `ThemeSet` in `fireside-tui`.
- Added redraw gating using `App::needs_redraw` +
  `App::take_needs_redraw()` and updated the CLI event loop to draw only when
  required.
- Capped traversal history at 256 entries using `VecDeque` in
  `TraversalEngine`.
- Validation completed: `cargo build`, `cargo test --workspace`,
  `cargo clippy --workspace -- -D warnings`.

Phase 4 is now implemented end-to-end:

- Added `EngineError::PathTraversal` in `fireside-engine` for typed traversal
  sanitization errors.
- Hardened image path sanitization in `fireside-tui` markdown renderer:
  rejects parent traversal components, constrains image resolution to the
  presentation base directory, and logs `tracing::warn!` for rejections.
- Added iTerm2 import safety limit in `fireside-tui` (`>1 MB` files rejected
  before plist parsing).
- Added normative extension payload safety language to spec docs in
  `docs/src/content/docs/spec/extensibility.md`.
- Added/updated tests for image-path sanitization and iTerm2 file-size
  rejection, including a macOS-canonicalization-safe assertion.
- Validation completed: `cargo test --workspace`,
  `cargo clippy --workspace -- -D warnings`.

Phase 5 is now implemented end-to-end:

- Added `fireside-core` round-trip tests for all `ContentBlock` variants in
  `crates/fireside-core/tests/content_roundtrip.rs`, including edge cases for
  list bare-string decoding, nested container children, and extension fallback payloads.
- Added engine fixture suite under
  `crates/fireside-engine/tests/fixtures/` with valid/invalid graph fixtures,
  plus `validation_fixtures.rs` coverage for expected diagnostics and load-time
  failure modes.
- Added engine command-history invariant test in
  `crates/fireside-engine/tests/command_history.rs` to verify add/update/remove
  and undo restores the original node sequence.
- Added CLI e2e tests in `crates/fireside-cli/tests/cli_e2e.rs` for
  validate success/failure and scaffolding flows (single file + project directory).
- Added TUI smoke extension in `crates/fireside-tui/tests/hello_smoke.rs`
  asserting each node produces non-empty rendered output.
- Added CLI test dev dependencies in `crates/fireside-cli/Cargo.toml`:
  `assert_cmd`, `predicates`, and `tempfile`.
- Validation completed: `cargo test --workspace`,
  `cargo clippy --workspace -- -D warnings`.

- Crate READMEs written for all four Cargo workspace members:
  `fireside-core`, `fireside-engine`, `fireside-tui`, `fireside-cli`.
- Root `README.md` updated to reflect full feature set post-UX rehaul:
  editor mode, graph overlay, help overlay, all keybindings, project mode,
  iTerm2 theme import, `fireside fonts`, crate README cross-links.
- Writing style: expert technical writer teaching Rust — architecture
  rationale, annotated code examples, key design decisions, module maps.

Current execution mode is TUI usability first (unchanged):

- JSON-first configuration surfaces (`fireside.json`) for project config.
- No YAML/TOML expansion work for now unless explicitly re-approved.
- Export workflows (HTML/PDF) are deferred from active implementation scope.
- Competitive analysis is tracked in memory-bank, not user-facing docs.

## Recently Applied Direction

- Replaced `group` with `container` in protocol model and docs.
- Replaced `x-` prefix extension convention with explicit extension blocks:
  `kind: "extension"` + `type`.
- Standardized serialization guidance to `application/json`.
- Removed root `specs/` duplication by moving quick-reference docs into
  `docs/src/content/docs/reference/`.
- Enforced chapter ordering in docs sidebar: §1–§6 then appendices.

## Recent Session (2026-02-20) — .github/ Agentic Tooling Overhaul

Complete audit and improvement of all `.github/` agentic infrastructure:

- **Memory bank**: Rewrote `projectbrief.md` (PRD structure), `activeContext.md` (current-state snapshot), `progress.md` (status tables), `techContext.md` (full stack + CI table), `systemPatterns.md` (TEA diagram, AppMode FSM, all patterns). Fixed `tasks/_index.md` — moved TASK001-006 to Completed.
- **copilot-instructions.md**: Fixed docs structure reference (`decisions/` → `crates/`), added `GraphView` to AppMode transitions, added `nextest` as primary test command, added `.githooks/` reference, removed duplicate TypeSpec and "When Making Changes" sections, added Skills Registry + subagent routing guidance, removed redundant `typespec-build` entry, fixed sidebar docs convention.
- **New skills**: Created `.github/skills/adr/SKILL.md` (Nygard-style ADR format, numbering, filing), `.github/skills/protocol-change/SKILL.md` (5-phase cascade: TypeSpec → JSON Schema → Rust → docs → verification gates).
- **New agent**: Created `.github/agents/rust-expert.agent.md` (MSRV validation, crate boundary enforcement, Context7-first API verification, TEA/index-rebuild rule enforcement).

## Next Workstream

- No active feature work in progress. Project is in maintenance mode.
- If resuming feature work: read `memory-bank/progress.md` "What's Left" section for deferred items.
- For any significant architectural decision: invoke the `adr` skill before coding.
- For any protocol-format change: invoke the `protocol-change` skill (5-phase cascade).

## Current Milestone Execution

- Completed Phase 6 of `plan-fireside-improvement-initiative`:
  - Added new guides:
    - `docs/src/content/docs/guides/theme-authoring.md`
    - `docs/src/content/docs/guides/extension-authoring.md`
  - Added new reference page:
    - `docs/src/content/docs/reference/keybindings.md`
  - Added migration placeholder:
    - `docs/src/content/docs/spec/migration.md`
  - Added full 9-page tutorial series at
    `docs/src/content/docs/guides/learn-rust/` (`_index` + 8 chapters).
  - Updated Starlight sidebar in `docs/astro.config.mjs` to explicitly include
    migration, keybindings, new guides, and the full Learn Rust chapter order.
  - Verified docs build successfully (`cd docs && npm run build`), with one
    pre-existing warning about duplicate id in `spec/extensibility.md`.

- Implemented runtime handling for `traversal.after` in engine traversal to
  support branch rejoin behavior.
- Implemented project-directory edit support and shared project entry
  resolution using `fireside.json`.
- Implemented editor graph view overlay (`v`) with keyboard/mouse node
  navigation and jump-to-node integration.
- Expanded editor graph view with a richer ASCII edge-map topology view,
  mini-map side panel, and viewport controls (`PgUp`/`PgDn`, `Home`/`End`).
- Added multi-line branch fan-out edge rendering per node in graph overlay and
  synchronized overlay row hit-testing to variable-height entries.
- Added graph overlay shortcut to jump directly into presenter mode from the
  selected graph node (`p`).
- Added reverse handoff breadcrumb when entering editor from presenter (`e`),
  preserving current-node selection context in editor status.
- Refined graph fan-out topology rows with aligned branch connector labels and
  explicit per-edge connector glyphs for dense branch nodes.
- Upgraded in-app help overlay into categorized, mode-aware sections with
  active/dimmed shortcut states for presenter vs editor contexts.
- Added help overlay scrolling controls (`j/k`, arrows, page/home/end) and
  section jump keys (`1-6`) for small terminal sizes.
- Added compact help footer legend that maps section jump keys (`1-6`) and
  shows current scroll position for shortcut discoverability.
- Updated help footer section legend to be mode-sensitive so in-context
  section labels are emphasized and out-of-mode labels are dimmed.
- Completed a holistic release/usability audit across models, crates, and docs;
  full Rust checks and TypeSpec schema build are green, and docs build now
  uses a custom content-backed 404 route without Starlight missing-entry warnings.
- Next milestone slices: branch fan-out layout polish in graph map,
  graph-to-editor/presenter workflows, and release polish for TUI usability.
