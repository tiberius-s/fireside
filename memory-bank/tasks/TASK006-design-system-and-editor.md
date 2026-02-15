# TASK006 - Design system, editor UI, and project structure

**Status:** In Progress
**Added:** 2026-02-14
**Updated:** 2026-02-14

## Original Request

Design and implement a complete design system for Slideways: color tokens from
iTerm2 schemes, monospace font detection, slide layout templates, a TUI editor
with mouse support, and a directory-backed project structure. The app should be
a standalone TUI that can open without a target `.md` file.

## Thought Process

The user wants to evolve Slideways from a simple "present file.md" tool into a
full TUI application with:

1. **Design system** — color tokens mapped from iTerm2 `.itermcolors` palettes,
   restricted to monospace fonts, using Ratatui blocks/panels/outlines
2. **Slide templates** — reusable layouts (Title, Two-column, Code, Quote, etc.)
   backed by frontmatter schemas and Ratatui component mappings
3. **Editor mode** — WYSIWYG-like block editing with keyboard+mouse, undo/redo,
   template selection, property panels, and preview toggle
4. **Project structure** — directory-backed projects with `slideways.yml` config
   mapping to collections of markdown files, with single-file fallback
5. **Standalone TUI** — open the app without arguments to get a dashboard/editor

Key technical decisions:

- iTerm2 `.itermcolors` files are XML plists → use `plist` crate to parse
- Monospace font detection → use `font-kit` crate (`is_monospace()`)
- Mouse support → crossterm `EnableMouseCapture` + ratatui stateful widgets
- Project config → `slideways.yml` with serde_yaml
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

- Define `slideways.yml` config schema
- Implement project loader alongside single-file loader
- Add `slideways open [dir]` CLI subcommand
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

**Overall Status:** In Progress - 5%

### Subtasks

| ID  | Description                       | Status      | Updated    | Notes                          |
| --- | --------------------------------- | ----------- | ---------- | ------------------------------ |
| 6.1 | Design token types and iTerm2 map | In Progress | 2026-02-14 | Researched; next is implement  |
| 6.2 | Font detection module             | Not Started | 2026-02-14 | font-kit API confirmed         |
| 6.3 | Slide template layouts            | Not Started | 2026-02-14 | 8 templates planned            |
| 6.4 | Project config and loader         | Not Started | 2026-02-14 | slideways.yml schema designed  |
| 6.5 | Editor state machine              | Not Started | 2026-02-14 | TEA extension planned          |
| 6.6 | Block editing with mouse          | Not Started | 2026-02-14 | crossterm mouse events ready   |
| 6.7 | Property panel and preview        | Not Started | 2026-02-14 | Ratatui layout confirmed       |
| 6.8 | Undo/redo and save                | Not Started | 2026-02-14 | Command pattern planned        |
| 6.9 | Integration testing               | Not Started | 2026-02-14 | Multiple terminal size targets |

## Progress Log

### 2026-02-14

- Researched Ratatui v0.30 layout/widget/style APIs via context7 docs
- Confirmed font-kit `is_monospace()` + `SystemSource::all_families()` API
- Confirmed crossterm mouse event support (`EnableMouseCapture`)
- Confirmed iTerm2 .itermcolors is XML plist format
- Surveyed current codebase: Layout enum, Theme struct, render pipeline
- Created task; beginning Phase 1 implementation
