# TASK006 - Phase 6 integration settings release polish

**Status:** In Progress
**Added:** 2026-02-14
**Updated:** 2026-02-19

## Original Request

Design and implement a complete design system for Fireside: color tokens from
iTerm2 schemes, monospace font detection, node layout templates, a TUI editor
with mouse support, and a directory-backed project structure. The app should be
a standalone TUI that can open without a target graph file.

## Thought Process

The user wants to evolve Fireside from a simple "present file.json" tool into a
full TUI application with:

1. **Design system** — color tokens mapped from iTerm2 `.itermcolors` palettes,
   restricted to monospace fonts, using Ratatui blocks/panels/outlines
2. **Node templates** — reusable layouts (Title, Two-column, Code, Quote, etc.)
   backed by frontmatter schemas and Ratatui component mappings
3. **Editor mode** — WYSIWYG-like block editing with keyboard+mouse, undo/redo,
   template selection, property panels, and preview toggle
4. **Project structure** — directory-backed projects with `fireside.json`
   config mapping to collections of graph files
5. **Standalone TUI** — open the app without arguments to get a dashboard/editor

Key technical decisions:

- iTerm2 `.itermcolors` files are XML plists → use `plist` crate to parse
- Monospace font detection → use `font-kit` crate (`is_monospace()`)
- Mouse support → crossterm `EnableMouseCapture` + ratatui stateful widgets
- Project config → `fireside.json` with serde_json
- Editor state machine → new `AppMode::Editing` variant in TEA loop

## Implementation Plan

### Phase 1: Design system foundations

- Define color token types and iTerm2-to-Ratatui mapping
- Add `plist` dependency for .itermcolors parsing
- Create design token module (`src/design/tokens.rs`)
- Create component library specs mapped to Ratatui primitives

### Phase 2: Font detection

- Add `font-kit` dependency
- Create font detection module (`src/design/fonts.rs`)
- Filter system fonts to monospace-only using `is_monospace()`
- Build font chooser widget

### Phase 3: Slide templates

- Define template enum and frontmatter schemas
- Implement 8 layout templates with Ratatui area calculations
- Create template selection UI widget

### Phase 4: Project structure

- Define `fireside.json` config schema
- Implement project loader alongside single-file loader
- Add `fireside open [dir]` CLI subcommand
- Create project dashboard view

### Phase 5: Editor mode

- Design editor state machine (modes, blocks, selection)
- Implement block CRUD (create, reorder, edit, delete)
- Add mouse event handling (click, drag, double-click)
- Implement property panel and preview toggle
- Add undo/redo stack

### Phase 6: Integration and polish

- Wire editor save → markdown + frontmatter serialization
- Theme preview in editor
- Keyboard shortcut reference
- Acceptance testing at multiple terminal sizes

## Progress Tracking

**Overall Status:** In Progress - 84%

### Subtasks

| ID  | Description                       | Status      | Updated    | Notes                                                     |
| --- | --------------------------------- | ----------- | ---------- | --------------------------------------------------------- |
| 6.1 | Design token types and iTerm2 map | Complete    | 2026-02-19 | Token system and iTerm2 parsing active                    |
| 6.2 | Font detection module             | Complete    | 2026-02-19 | Monospace detection module implemented                    |
| 6.3 | Slide template layouts            | Complete    | 2026-02-19 | Template areas wired into presenter                       |
| 6.4 | Project config and loader         | Complete    | 2026-02-19 | Project open/edit directory flow active (`fireside.json`) |
| 6.5 | Editor state machine              | In Progress | 2026-02-19 | Core editing mode/actions are active                      |
| 6.6 | Block editing with mouse          | In Progress | 2026-02-19 | Mouse click/drag/scroll handlers active                   |
| 6.7 | Property panel and preview        | In Progress | 2026-02-19 | Metadata panel and preview are active                     |
| 6.8 | Undo/redo and save                | In Progress | 2026-02-19 | Undo/redo + save flow implemented                         |
| 6.9 | Integration testing               | In Progress | 2026-02-19 | Mermaid/settings/hot-reload validations are green         |

## Progress Log

### 2026-02-14

- Researched Ratatui v0.30 layout/widget/style APIs via context7 docs
- Confirmed font-kit `is_monospace()` + `SystemSource::all_families()` API
- Confirmed crossterm mouse event support (`EnableMouseCapture`)
- Confirmed iTerm2 .itermcolors is XML plist format
- Surveyed current codebase: Layout enum, Theme struct, render pipeline
- Created task; beginning Phase 1 implementation

### 2026-02-19

- Updated this milestone to reflect current implementation state from the UX initiative plan
- Confirmed design tokens, templates, font tooling, and project open flow are in active code
- Confirmed editor mode now includes selection, inline edits, mouse interactions, undo/redo, and save
- Added project-directory edit support so `fireside edit <project-dir>` resolves `fireside.json` and opens the project entry graph

### 2026-02-19 (priority reset)

- Applied JSON-first project configuration (`fireside.json`) and removed YAML project-config dependency from active CLI flow.
- Kept export formats out of this phase to maintain focus on TUI usability and release polish.

### 2026-02-19 (hot-reload continuation)

- Implemented presenter-mode hot-reload loop in CLI session runner using file modification time checks.
- Added app-level graph reload behavior that preserves current node when IDs remain stable and clamps safely otherwise.
- Restricted hot-reload application to presenting mode to avoid editor-state conflicts.
- Resolved strict clippy warning in reload condition by using `Option::is_none_or`.
- Verified touched Rust files with diagnostics (`No errors found` in `session.rs` and `app.rs`) while broader smoke/lint runs continue.

### 2026-02-19 (resume after terminal crash)

- Re-ran targeted TUI hot-reload tests and confirmed they pass:
  - `cargo test -p fireside-tui reload_graph`
  - `cargo test -p fireside-tui hot_reload_is_only_enabled_in_presenter_mode`
- Re-ran crate lint gate for touched crates and confirmed clean output:
  - `cargo clippy -p fireside-cli -p fireside-tui -- -D warnings`
- Re-ran CLI test suite and confirmed green status:
  - `cargo test -p fireside-cli --no-fail-fast`
- Confirmed this hot-reload continuation slice is validated and ready for next Phase 6 polish items.

### 2026-02-19 (Mermaid + settings completion)

- Completed Mermaid extension hardening in renderer:
  - Added robust extension-type detection helper for Mermaid variants.
  - Added payload extraction support for `code`, `diagram`, `source`, and string payloads.
  - Added fenced-code normalization and preview truncation safeguards for long diagrams.
  - Added preview overflow messaging for clipped lines and payload truncation notice.
- Completed settings polish:
  - Added alias support for config keys (`poll-timeout-ms`, `show-progress`, `show-timer`, etc.).
  - Added nested `settings` block support and bounded poll-timeout clamping.
  - Normalized theme values by trimming whitespace and handling empty theme strings.
  - Added XDG-aware config base-dir resolution for settings and editor UI preferences.
- Verified with sequential checks:
  - `cargo test -p fireside-tui extension_mermaid --no-fail-fast`
  - `cargo test -p fireside-tui nested_settings_block_is_supported --no-fail-fast`
  - `cargo clippy -p fireside-tui -p fireside-cli -- -D warnings`
  - `cargo test -p fireside-tui --no-fail-fast`
