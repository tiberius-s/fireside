---
title: 'fireside-cli'
description: 'Entry-point binary — argument parsing, terminal lifecycle management, the event loop, hot-reload, and the scaffolding system.'
---

`fireside-cli` is the binary crate that users run directly. It owns exactly
three things: CLI argument parsing (clap), terminal lifecycle management
(crossterm), and the main event loop. All other logic — graph loading,
validation, rendering, state — lives in the library crates below it.

## Crate boundaries

| Responsibility           | `fireside-cli` | Delegated to                                  |
| ------------------------ | -------------- | --------------------------------------------- |
| Argument parsing         | ✓              | —                                             |
| Terminal enter/exit      | ✓              | —                                             |
| Event loop               | ✓              | —                                             |
| Hot-reload file watching | ✓              | —                                             |
| Graph loading            | ✗              | `fireside-engine::load_graph`                 |
| Validation               | ✗              | `fireside-engine::validation::validate_graph` |
| State machine            | ✗              | `fireside-tui::App`                           |
| Rendering                | ✗              | `fireside-tui::App::view`                     |
| Theme resolution         | ✗              | `fireside-tui::config::resolve_theme`         |
| Scaffolding templates    | ✗              | `commands/scaffold.rs`                        |

This boundary means `main.rs` is under 160 lines and contains no business logic.

## Subcommand register

The `Command` enum is parsed by `clap::Parser`:

```text
fireside present <file.json> [--theme <name>] [--start <n>]
fireside open    <dir>       [--theme <name>]
fireside edit    [path]
fireside new     <name>      [--project] [--dir <path>]
fireside validate <file.json>
fireside fonts
fireside import-theme <file.itermcolors> [--name <name>]
```

`None` (no subcommand) prints a compact usage summary and exits successfully.

Clap's `--help` and `--version` flags are generated automatically from the
`#[command(name, version, about)]` attributes.

## Terminal lifecycle

Both `run_presentation` and `run_editor` follow the same lifecycle pattern:

```text
enable_raw_mode()
execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
Terminal::new(CrosstermBackend::new(stdout))
  └─► run_event_loop(terminal, app, settings.poll_timeout_ms, watch_path)
disable_raw_mode()
execute!(backend, LeaveAlternateScreen, DisableMouseCapture)
terminal.show_cursor()
```

All crossterm setup and teardown calls are wrapped with `.context(...)` to
produce clean `anyhow::Error` chains if the terminal is unavailable (e.g.,
not attached to a TTY).

Teardown runs unconditionally after `run_event_loop` returns, whether it
succeeded or failed. This prevents the terminal being left in raw mode if
the TUI panics or returns early with an error.

## The event loop

`run_event_loop` is `fireside-cli`'s most important function. It is the
bridge between crossterm events and `App::update`:

```rust
fn run_event_loop(
    terminal:         &mut Terminal<CrosstermBackend<io::Stdout>>,
    app:              &mut App,
    idle_poll_timeout_ms: u64,
    watch_path:       Option<&Path>,
) -> Result<()>
```

### Frame draw gate

Frames are drawn lazily, only when `app.take_needs_redraw()` returns `true`.
The `App` sets its redraw flag whenever state changes (any `Action` processed
by `update` that could alter the visual output). This avoids redundant redraws,
which matters on large terminals where `terminal.draw` is the hot path.

Quit detection follows draw: `app.should_quit()` is checked immediately after
draw, before any blocking poll.

### Poll timeout adaptation

Two poll durations are in play:

```rust
let poll_duration = if app.is_animating() {
    Duration::from_millis(50)   // ~20 fps for smooth transitions
} else {
    Duration::from_millis(idle_poll_timeout_ms.max(10))  // user-configured idle
};
```

When transitions are active, the loop drives at approximately 20 fps via
`Action::Tick` injections if no real event arrived. When idle, the loop
blocks on `event::poll` for the configured duration (default: 50–200 ms
depending on settings). This prevents CPU spinning during idle presentation
while maintaining smooth animation during transitions.

### Input dispatch

When an event is available:

```rust
app.handle_event(ev);
```

`App::handle_event` maps the `crossterm::event::Event` to an `Action` via the
keybinding configuration and calls `App::update(action)`. The CLI does not
interpret events directly; it delivers raw crossterm events to the TUI layer.

### Hot-reload

If `watch_path` is set (only in presentation mode, not edit mode), the event
loop polls for file modification on every iteration:

```rust
if app.can_hot_reload()
    && let Some(path) = watch_path
    && let Some(updated_modified) = file_modified_time(path)
    && last_modified.is_none_or(|prev| updated_modified > prev)
{
    if let Ok(graph) = load_graph(path) {
        app.reload_graph(graph);
    }
    last_modified = Some(updated_modified);
}
```

`file_modified_time(path)` reads `fs::metadata(path)?.modified()` and returns
`None` on any error. Modification time is compared to the last-known mtime stored
in `last_modified: Option<SystemTime>`.

`app.can_hot_reload()` is a guard that the TUI sets false during edit mode and
certain fragile states. On a successful reload, `load_graph` is called directly
(not through the engine session); if it fails the stale graph remains visible
and no error is shown (the next file-write will be retried).

## `present` — `run_presentation`

```rust
pub fn run_presentation(file: &Path, theme_name: Option<&str>, start_node: usize)
    -> Result<()>
```

Sequence:

1. `load_graph(file)` — parse and index the graph via `fireside-engine`
2. `load_settings()` — read `~/.config/fireside/config.json`
3. Resolve effective theme: `--theme` flag → document `meta.theme` → settings → default
4. `PresentationSession::new(graph, start_node - 1)` — 0-indexed internally
5. `App::new(session, theme)` — construct TUI state machine
6. Apply settings: `set_show_progress_bar`, `set_show_elapsed_timer`
7. Enter raw mode + alternate screen → `run_event_loop(…, Some(file))` → cleanup

The `start_node` argument is 1-indexed on the CLI (`--start 3` means the third
node listed in the `nodes` array) and converted to 0-indexed before passing to
the engine.

## `edit` — `run_editor`

```rust
pub fn run_editor(target: &Path) -> Result<()>
```

Accepts either a file path or a project directory. If `target.is_dir()`,
`resolve_project_entry(target)` opens `fireside.json` in that directory and
finds the primary entry point (the first file in `nodes`).

The key difference from `run_presentation`:

- `app.enter_edit_mode()` is called before the event loop, setting
  `AppMode::Editing` as the initial mode.
- `app.set_editor_target_path(graph_path)` configures the path that
  `Action::SaveGraph` will write to.
- `watch_path` is passed as `None` to `run_event_loop`, disabling hot-reload
  (avoiding a collision between the editor writing and the watcher reloading).

## `validate` — `run_validate`

```rust
pub fn run_validate(file: &Path) -> Result<()>
```

Non-interactive. Sequence:

1. `load_graph(file)` — errors here (malformed JSON, unknown fields) are printed as `anyhow` chain errors.
2. `validate_graph(&graph)` — returns `Vec<Diagnostic>` with error/warning severity.
3. Print all diagnostics to stdout in a `cargo`-compatible format (see example below).
4. If any `Severity::Error` diagnostics exist, `anyhow::bail!` with a summary,
   returning exit code 1.

Output example:

```text
error (node 'intro'): node references unknown target 'missing-node'
warning: node 'orphan' is unreachable from the start node
```

This format is intentionally similar to compiler output so it can be read by
editor integrations or CI log parsers.

## `new` — scaffolding

### `scaffold_presentation`

Creates a single `.json` file from a hard-coded template. The template is
constructed via `serde_json::json!` (not a string template) to guarantee
well-formed JSON output. It includes:

- `$schema` reference to the published JSON Schema
- `title`, `author`, `date` (populated from the name and `today_iso_date()`)
- `defaults` with `layout: "top"` and `transition: "fade"`
- 4 example nodes: title slide, bullet list, code block, closing node

An existing file at the target path is rejected with `anyhow::bail!` before
any write, providing safe re-run behavior.

### `scaffold_project`

Creates a named project directory structure:

```text
<name>/
├── fireside.json    ← project manifest with name and nodes list
├── nodes/
│   └── main.json   ← delegates to scaffold_presentation("main", nodes_dir)
└── themes/         ← empty directory, ready for .json/.itermcolors themes
```

The `fireside.json` manifest is also built via `serde_json::json!`. Directory
existence is checked before creation; an existing directory is rejected.

## `import-theme` — `import_iterm2_theme`

Wraps `fireside_tui::design::iterm2::Iterm2Scheme::from_file()`. The imported
theme name defaults to the file stem if `--name` is not provided. The resulting
`DesignTokens` are converted to `ThemeFile` JSON and written to
`~/.config/fireside/themes/<name>.json`.

## `fonts` — `list_fonts`

Calls into `fireside_tui::design::fonts` to enumerate monospace system fonts via
`font-kit`. Prints each discovered family name to stdout, one per line. Intended
to help users identify font names for `config.json`.

## Error handling strategy

`main` returns `anyhow::Result<()>`. All errors bubble up through `?` with
`.context(...)` annotations at each step. Clap prints usage errors itself
(to stderr) and exits with code 2. `anyhow` prints runtime errors as
`Error: <message>\nCaused by: <chain>` to stderr and exits with code 1.

No `unwrap()` or `expect()` is permitted in this crate's library-facing code;
the rule is relaxed only inside `main` itself for infallible operations where
`Result` would be noise.
