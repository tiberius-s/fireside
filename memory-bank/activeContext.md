# Active Context

## Current Focus

All release gates are green. Phase 6 documentation and tutorial expansion is complete.

Phase 1 of `plan-fireside-improvement-initiative` is now implemented end-to-end:

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

## Next Workstream

- Continue protocol-vocabulary alignment across remaining legacy wording in
  implementation docs and UX copy.
- Keep task tracking explicitly phase-aligned so progress is easy to audit.

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
