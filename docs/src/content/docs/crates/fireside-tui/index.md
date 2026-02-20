---
title: 'fireside-tui'
description: 'Architecture overview of the Fireside terminal frontend — TEA event loop, rendering pipeline, design system, and configuration.'
---

`fireside-tui` is the terminal frontend for Fireside. It translates
`fireside-engine` session state into ratatui draw calls and maps crossterm
keyboard events into typed `Action` intents that mutate the application state.
It is the largest and most complex crate in the workspace.

## Crate responsibilities

| Owns                                          | Explicitly excluded             |
| --------------------------------------------- | ------------------------------- |
| `App` struct — all application state          | Graph parsing or protocol types |
| `AppMode` state machine and `update()` loop   | File I/O beyond session save    |
| `Action` intent model and keybinding dispatch | Engine traversal logic          |
| Rendering pipeline (`render/*`)               | CLI argument parsing            |
| Theme model, `DesignTokens`, layout templates | External network or OS calls    |
| `Settings` and editor UI preferences          |                                 |

## Module structure

```text
fireside-tui/src/
├── app.rs                 App struct, AppMode, update(), view()
├── error.rs               TuiError; RenderError
├── event.rs               Action enum; MouseScrollDirection
├── lib.rs                 public re-exports; run_presentation() entry point
├── theme.rs               Theme, ThemeFile, color parsing
├── config/
│   ├── mod.rs             theme resolution and config loading helpers
│   ├── keybindings.rs     map_key_to_action() — mode-aware dispatch
│   └── settings.rs        Settings, EditorUiPrefs, load/save helpers
├── design/
│   ├── mod.rs
│   ├── tokens.rs          DesignTokens — semantic color roles
│   ├── templates.rs       NodeTemplate — layout-to-area mapping
│   ├── fonts.rs           font discovery via font-kit
│   └── iterm2.rs          iTerm2 color scheme import
├── render/
│   ├── mod.rs             public interface; render_block()
│   ├── markdown.rs        ContentBlock → Vec<Line<'_>>
│   ├── code.rs            syntect + two-face syntax highlighting
│   └── layout.rs          multi-column container layout
└── ui/
    ├── mod.rs
    ├── presenter.rs       full-screen presenter frame composition
    ├── editor.rs          split-pane editor overlay
    ├── graph.rs           graph overview overlay
    ├── branch.rs          branch-point choice overlay
    ├── help.rs            scrollable help overlay
    └── progress.rs        footer progress bar and timer
```

## Architectural layers

The crate is organized in three layers that have strict dependency flow:

```text
┌────────────────────────────────────────────┐
│  ui/  — frame composition                   │
│  (presenter, editor, graph, branch, help)   │
│  Calls: render/* and App state              │
├────────────────────────────────────────────┤
│  render/ — block-level drawing              │
│  (markdown, code, layout)                   │
│  Calls: design/tokens, syntect              │
├────────────────────────────────────────────┤
│  design/ — visual design system             │
│  (tokens, templates, fonts, iterm2)         │
│  Calls: theme, ratatui::style               │
└────────────────────────────────────────────┘
```

`App` sits above all three layers: `update()` mutates state; `view()` delegates
to `ui/` compositors which call into `render/` to produce `Line` buffers which
in turn call into `design/` for color resolution.

## TEA architecture

`fireside-tui` implements the **Elm Architecture** (TEA) as a terminal
application loop. The three concerns — model, update, view — map precisely:

**Model** — `App` owns all mutable state. No state lives anywhere else; no
`Arc<Mutex<…>>`, no channels, no global singletons.

**Update** — `App::update(&mut self, action: Action)` is the **sole mutation
point** for the entire application. Every key event and mouse event passes
through this function. Rendering functions receive `&App` (shared reference)
and produce draw calls; they never mutate state.

**View** — `App::view(&self, frame: &mut Frame)` calls the appropriate `ui/`
compositor based on `self.mode`. It takes no references beyond `self` and the
ratatui `Frame`.

The event loop in `run_presentation` drives this cycle:

```text
loop {
    terminal.draw(|f| app.view(f));     // view
    let event = crossterm::event::read();
    if let Some(action) = dispatch(event, &app.mode) {
        app.update(action);             // update
    }
    if app.mode == AppMode::Quitting { break; }
}
```

This is not quite purely functional — `App::update` mutates `self` in place
rather than returning a new model — but the invariant is maintained that no
function other than `update` may call `&mut self` on `App`.

## Further reading

The TUI crate is documented in detail across three focused articles:

- [App State Machine](/crates/fireside-tui/app-state-machine/) — `App`,
  `AppMode`, the `Action` enum, `update()` dispatch, and mode transitions.
- [Rendering Pipeline](/crates/fireside-tui/rendering-pipeline/) — how
  `ContentBlock` values become styled ratatui `Line`s; syntax highlighting;
  image rendering and path security; container layout.
- [Theme & Design System](/crates/fireside-tui/theme-design-system/) — `Theme`,
  `DesignTokens`, layout templates, iTerm2 color import, font discovery, and
  the `Settings` configuration chain.
