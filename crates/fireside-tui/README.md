# fireside-tui

The terminal user interface for Fireside. This crate renders a `Graph` to the
terminal, handles keyboard and mouse input, and delegates navigation and editing
to `fireside-engine`. It depends on `ratatui` for layout and widgets and
`crossterm` for cross-platform terminal control.

## Design Philosophy: TEA (The Elm Architecture)

The TUI is structured as a strict **Model-View-Update** loop, commonly known as
TEA:

```text
  ┌────────────────────────────────────────────────────────┐
  │                     Event Loop                         │
  │                                                        │
  │  crossterm::Event                                      │
  │       │                                                │
  │       ▼                                                │
  │  map_key_to_action(key, &app.mode) → Option<Action>    │
  │       │                                                │
  │       ▼                                                │
  │  App::update(&mut self, action)  ← sole mutation point │
  │       │                                                │
  │       ▼                                                │
  │  terminal.draw(|f| app.view(f))  ← pure render        │
  └────────────────────────────────────────────────────────┘
```

The key discipline: **`App::update` is the only function that mutates `App`
state.** Rendering functions receive immutable references and produce `ratatui`
widgets as output. This makes the render path trivially testable and ensures
that you can always reason about state by reading `update` alone.

## Module Map

```text
fireside-tui/src/
├── lib.rs              — public re-exports (App, Action, Theme)
├── app.rs              — App struct, AppMode, update loop (2200+ lines)
├── error.rs            — TuiError
├── event.rs            — Action enum (~35 variants)
├── theme.rs            — Theme, ThemeFile, parse_color
├── config/
│   ├── keybindings.rs  — map_key_to_action (mode-aware dispatch)
│   └── settings.rs     — Settings, load_settings (user + project merge)
├── design/
│   ├── tokens.rs       — spacing, border styles, color constants
│   ├── templates.rs    — reusable layout compositions
│   ├── fonts.rs        — font enumeration and selection
│   └── iterm2.rs       — iTerm2 .itermcolors → ThemeFile conversion
├── render/
│   ├── markdown.rs     — inline Markdown → ratatui Span list
│   ├── code.rs         — syntect-powered syntax highlighting
│   └── layout.rs       — ContentBlock → Lines, layout-aware placement
├── ui/
│   ├── presenter.rs    — presentation mode view
│   ├── editor.rs       — editor mode view
│   ├── graph.rs        — graph overview overlay
│   ├── branch.rs       — branch choice overlay
│   ├── help.rs         — help overlay
│   └── progress.rs     — footer progress bar
```

## Application Modes

`AppMode` controls which keybindings are active and which view is rendered:

```rust
pub enum AppMode {
    /// Normal presentation mode — navigate through nodes.
    Presenting,
    /// Node editor — browse and edit the graph structure.
    Editing,
    /// Waiting for digit input to jump to a node.
    GotoNode { buffer: String },
    /// Application is shutting down.
    Quitting,
}
```

Mode transitions are explicit `Action` variants:

| Action | Transition |
| --- | --- |
| `EnterEditMode` | `Presenting` → `Editing` |
| `ExitEditMode` | `Editing` → `Presenting` |
| `EnterGotoMode` | `Presenting` → `GotoNode { buffer: "" }` |
| `GotoConfirm` | `GotoNode` → `Presenting` (after jumping) |
| `Quit` | any → `Quitting` |

## `App` State

`App` holds all mutable state for the running application. Its fields cluster
into logical groups:

```rust
pub struct App {
    // --- Core session ---
    pub session: PresentationSession,  // graph + traversal
    pub mode: AppMode,
    pub theme: Theme,
    pub start_time: Instant,

    // --- Presentation overlays ---
    pub show_help: bool,
    pub show_speaker_notes: bool,

    // --- Editor state ---
    pub editor_selected_node: usize,
    pub editor_focus: EditorPaneFocus,   // NodeList | NodeDetail
    pub editor_target_path: Option<PathBuf>,
    pub editor_text_input: Option<String>,
    pub editor_picker: Option<EditorPickerOverlay>,  // layout | transition picker
    pub editor_graph_overlay: bool,

    // --- Animation ---
    active_transition: Option<ActiveTransition>,
```

Private fields (e.g., `editor_inline_target`, `pending_exit_action`) are
implementation details of `update` — callers and renderers never need them
directly.

## The `Action` Enum

Actions are the vocabulary of intent. Every user interaction translates into
exactly one `Action` before reaching `update`. This indirection means
keybindings, mouse events, and programmatic triggers all speak the same
language:

```rust
pub enum Action {
    // Presentation navigation
    NextNode,
    PrevNode,
    GoToNode(usize),
    ChooseBranch(char),

    // Overlays
    ToggleHelp,
    ToggleSpeakerNotes,

    // Mode switches
    EnterEditMode,
    ExitEditMode,

    // Editor — node list navigation
    EditorSelectNextNode,
    EditorSelectPrevNode,
    EditorPageDown / EditorPageUp,
    EditorJumpTop / EditorJumpBottom,

    // Editor — search and jump
    EditorStartNodeSearch,
    EditorSearchPrevHit / EditorSearchNextHit,
    EditorStartIndexJump,

    // Editor — content mutation
    EditorStartInlineEdit,
    EditorStartNotesEdit,
    EditorAppendTextBlock,
    EditorAddNode,
    EditorRemoveNode,
    EditorUndo,
    EditorRedo,

    // Editor — metadata
    EditorOpenLayoutPicker,
    EditorCycleLayoutNext / EditorCycleLayoutPrev,
    EditorOpenTransitionPicker,
    EditorCycleTransitionNext / EditorCycleTransitionPrev,

    // Editor — persistence and view
    EditorSaveGraph,
    EditorToggleGraphView,

    // System
    Quit,
    Resize(u16, u16),
    MouseClick { column: u16, row: u16 },
    // ...
}
```

## Keybindings

### Presentation Mode

| Key | Action |
| --- | --- |
| `→` / `Space` / `Enter` / `l` | Next node |
| `←` / `h` | Previous node |
| `g` | Enter go-to-node mode (type a number, confirm with Enter) |
| `a`–`f` | Choose branch option (when a branch point is active) |
| `s` | Toggle speaker notes |
| `?` | Toggle help overlay |
| `e` | Enter editor mode |
| `q` / `Esc` / `Ctrl-C` | Quit |

### Editor Mode

| Key | Action |
| --- | --- |
| `j` / `↓` | Select next node |
| `k` / `↑` | Select previous node |
| `PgDn` / `PgUp` | Page through node list |
| `Home` / `End` | Jump to first/last node |
| `/` | Start node-ID search |
| `[` / `]` | Previous/next search hit |
| `g` | Jump to node by index |
| `Tab` | Toggle focus between node list and detail pane |
| `i` | Edit selected node text content inline |
| `o` | Edit selected node speaker notes inline |
| `a` | Append a text block to the selected node |
| `n` | Add a new node after the selected node |
| `d` | Remove the selected node |
| `l` | Open layout picker overlay |
| `t` | Open transition picker overlay |
| `w` / `Ctrl-S` | Save graph to file |
| `v` | Toggle graph overview overlay |
| `u` | Undo last command |
| `r` | Redo last undone command |
| `?` | Toggle help overlay |
| `Esc` | Exit editor mode |

## Theme System

`Theme` holds `ratatui::style::Color` values for every named UI element. The
default theme adapts to whatever the terminal's own background and foreground
colors are by using `Color::Reset` for the base palette:

```rust
pub struct Theme {
    pub background: Color,    // Color::Reset by default
    pub foreground: Color,    // Color::Reset by default
    pub heading_h1: Color,
    pub heading_h2: Color,
    pub heading_h3: Color,
    pub code_background: Color,
    pub code_foreground: Color,
    pub code_border: Color,
    pub block_quote: Color,
    pub footer: Color,
    pub syntax_theme: String, // syntect theme name
}
```

Themes are loaded from JSON files. `ThemeFile` holds `Option<String>` for every
field so that partial theme files only override what they specify, merging with
defaults for the rest.

Color strings support named colors (`"cyan"`, `"darkgray"`, etc.), hex values
(`"#1e1e2e"`), and `"reset"`/`"default"` for terminal pass-through.

### iTerm2 Themes

The `design/iterm2.rs` module converts iTerm2 `.itermcolors` files (XML plist
format) into `ThemeFile` JSON. This lets you reuse any iTerm2 color scheme in
Fireside without manual color transcription. The `fireside import-theme`
command exposes this to the user.

## Rendering Pipeline

All render functions in the `ui/` and `render/` modules are **pure**: they
receive `&App` (or sub-slices of it) and a `ratatui::Frame`, and produce
output. They do not mutate state.

```text
App::view(frame: &mut Frame)
    └── render_presenter(frame, area, &app)  ← presentation mode
         ├── render::layout::render_content_blocks(blocks, area, frame, &theme)
         │    ├── render::markdown::render_text(body)   → Vec<Span>
         │    └── render::code::render_code(src, lang)  → Vec<Line>
         ├── ui::progress::render_progress(frame, footer_area, &app)
         └── (overlays)
              ├── ui::help::render_help(frame, area, &app)
              ├── ui::branch::render_branch_overlay(frame, area, &app)
              └── ui::graph::render_graph_overlay(frame, area, &app)
```

### Markdown Rendering

`render/markdown.rs` parses inline Markdown (bold, italic, code spans,
strikethrough) and converts it to a `Vec<ratatui::text::Span>`, each with the
appropriate `Style`. Block-level structure (headings, lists, blockquotes) is
handled by the layout renderer which maps `ContentBlock` variants directly to
ratatui `Line` sequences.

### Syntax Highlighting

`render/code.rs` uses `syntect` with the `two-face` theme pack for syntax
highlighting. The active `syntax_theme` field from `Theme` selects the
`syntect` theme (default: `"base16-ocean.dark"`). Each highlighted line becomes
a `ratatui::text::Line` composed of styled `Span`s.

## Settings and Configuration

Settings are loaded from two places, with project config winning over user
config:

1. `~/.config/fireside/config.json` — user-level defaults
2. `fireside.json` in the current working directory — project-level overrides

```json
{
  "theme": "nord",
  "show_progress": true,
  "show_timer": true,
  "poll_timeout_ms": 250
}
```

`EditorUiPrefs` (the editor's specific UI state like last-used pane focus) are
stored separately and round-tripped on each editor session, so the editor
remembers your preferred panel layout between invocations.

## Testing

```bash
cargo test -p fireside-tui
```

The smoke test in `tests/hello_smoke.rs` loads `docs/examples/hello.json`,
constructs an `App`, and asserts initialization succeeds — catching panics from
bad defaults or missing required fields before any UI code runs.

Render functions are pure and take `&App`, making snapshot-style testing
tractable: construct the state you want, call the renderer, inspect the output
buffer.

## Dependencies

| Crate | Purpose |
| --- | --- |
| `fireside-core` | Protocol types |
| `fireside-engine` | `PresentationSession`, `Command`, `TraversalEngine` |
| `ratatui` | Terminal UI framework (widgets, layout, styling) |
| `crossterm` | Cross-platform terminal control (raw mode, events) |
| `syntect` | Syntax highlighting engine |
| `two-face` | Extended syntect theme pack |
| `unicode-width` | Correct terminal column width for Unicode characters |
| `textwrap` | Word wrapping for prose content |
| `plist` | XML plist parsing for iTerm2 `.itermcolors` import |
| `font-kit` | Font enumeration for the `fireside fonts` command |
| `image` | Image decoding for PPM/PNG rendering in the TUI |
| `serde` / `serde_json` | Theme file and settings deserialization |
| `anyhow` | Error context at application boundaries |
| `thiserror` | `TuiError` derivation |
| `tracing` | Structured logging |
