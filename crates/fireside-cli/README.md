# fireside-cli

The binary entry point for Fireside. This crate wires together `fireside-engine`
and `fireside-tui` behind a `clap`-parsed command-line interface, handles the
terminal lifecycle (raw mode, alternate screen), and dispatches to the right
mode based on the subcommand the user invokes.

## Design Philosophy

`fireside-cli` is deliberately thin. The interesting logic lives in the engine
and TUI crates; this crate's job is to:

1. **Parse arguments** with `clap`.
2. **Bootstrap the terminal** (enable raw mode, alternate screen, mouse
   capture) before handing control to the TUI event loop.
3. **Tear the terminal down cleanly** even when the event loop exits with an
   error — restoring the user's shell to a usable state.
4. **Dispatch to command handlers** in `src/commands/`.

```text
main.rs
  └── match cli.command
       ├── Present     → commands/session.rs :: run_presentation
       ├── Open        → commands/project.rs :: run_project
       ├── Edit        → commands/session.rs :: run_editor
       ├── New         → commands/scaffold.rs :: scaffold_presentation / scaffold_project
       ├── Validate    → commands/validate.rs :: run_validate
       ├── Fonts       → commands/fonts.rs :: list_fonts
       └── ImportTheme → commands/theme.rs :: import_iterm2_theme
```

## Commands

### `fireside present <file>`

Loads a Fireside JSON file and launches the interactive terminal presenter.

```bash
fireside present talk.json
fireside present talk.json --theme nord
fireside present talk.json --start 3   # start at node 3 (1-indexed)
```

Theme resolution order:

1. `--theme` argument (name or path to `.itermcolors` / `.json`)
2. `theme` field in the document's metadata
3. User settings file (`~/.config/fireside/config.json`)
4. Built-in default

### `fireside open <dir>`

Opens a Fireside project directory. The directory must contain a `fireside.json`
project manifest. The manifest identifies the entry-point graph file and any
project-level settings.

```bash
fireside open ./my-course/
fireside open ./my-course/ --theme catppuccin
```

### `fireside edit [path]`

Opens the interactive node editor. If `path` is a directory (or omitted,
defaulting to `.`), the editor resolves the project entry. If `path` is a
`.json` file, it opens that file directly.

```bash
fireside edit                   # edit the project in the current directory
fireside edit talk.json         # edit a specific file
fireside edit ./my-course/      # edit the project entry in ./my-course/
```

The editor provides a two-pane view: a node list on the left and a detail view
on the right. Changes are held in memory until you save with `w` or `Ctrl-S`.
Undo and redo are available with `u` and `r`.

### `fireside new <name>`

Scaffolds a new presentation with a single starter node and writes it to
`<name>.json` in the current directory (or the directory specified by `--dir`).

```bash
fireside new my-talk
fireside new my-talk --dir ~/presentations/
```

With `--project`, scaffolds a full project directory:

```bash
fireside new my-course --project
# Creates my-course/
#   fireside.json      ← project manifest
#   intro.json         ← starter graph
```

### `fireside validate <file>`

Loads a graph and runs structural validation. Prints all diagnostics
(errors and warnings) and exits with a non-zero code if any errors are found.

```bash
fireside validate talk.json
# ✓ talk.json is valid

fireside validate broken.json
# error (node 'intro'): traversal.next references unknown node 'missing'
# 1 error(s), 0 warning(s) in broken.json
```

This is useful in CI pipelines:

```bash
fireside validate talk.json || exit 1
```

### `fireside fonts`

Lists monospace fonts installed on the system. Helps you identify font names
to use in theme configuration.

```bash
fireside fonts
```

### `fireside import-theme <file>`

Converts an iTerm2 `.itermcolors` file to a Fireside JSON theme and writes it
to `~/.config/fireside/themes/`. Thereafter the theme name can be passed to
`--theme`.

```bash
fireside import-theme ~/Downloads/Catppuccin.itermcolors --name catppuccin
fireside present talk.json --theme catppuccin
```

## Terminal Lifecycle

The `run_presentation` and `run_editor` functions in `commands/session.rs`
demonstrate the correct crossterm lifecycle pattern:

```rust
// Enter raw mode and alternate screen.
enable_raw_mode()?;
execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

// Hand off to the TUI event loop.
let result = run_event_loop(&mut terminal, &mut app, poll_timeout_ms, ...);

// Always restore the terminal, even on error.
disable_raw_mode()?;
execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
terminal.show_cursor()?;

// Propagate the event loop result.
result?
```

The restore block runs unconditionally before `result?` propagates, ensuring
the user's terminal is never left in raw mode even if the application panics or
returns an error.

This pattern is important to understand when targeting new frontends: the TUI
event loop itself (`run_event_loop`) is isolated from the lifecycle setup,
making it straightforward to test without a real terminal.

## Event Loop Architecture

The main event loop in `session.rs` follows the TEA discipline from
`fireside-tui`:

```rust
loop {
    // 1. Draw the current state.
    terminal.draw(|f| app.view(f))?;

    // 2. Poll for input.
    if event::poll(Duration::from_millis(poll_timeout_ms))? {
        let raw_event = event::read()?;

        // 3. Translate raw event → Action.
        if let Some(action) = translate_event(raw_event, &app.mode) {
            // 4. Update state.
            app.update(action);
        }
    }

    // 5. Check for transitions (animation) or quit.
    if app.is_quitting() {
        break;
    }
}
```

The loop polls with a timeout to allow the application to tick animations
(transitions, elapsed timer updates) even when the user is idle.

## Configuration Files

| Path                             | Purpose                                   |
| -------------------------------- | ----------------------------------------- |
| `~/.config/fireside/config.json` | User-level settings (theme, timers, etc.) |
| `~/.config/fireside/themes/`     | Imported theme JSON files                 |
| `fireside.json` in project dir   | Project-level settings and entry point    |

## Building

```bash
# Debug build (development)
cargo build -p fireside-cli

# Release build (distribution)
cargo build --release -p fireside-cli

# Run directly from the workspace
cargo run -- present docs/examples/hello.json
```

The compiled binary is named `fireside`.

## Dependencies

| Crate                            | Purpose                                                    |
| -------------------------------- | ---------------------------------------------------------- |
| `fireside-engine`                | `load_graph`, `PresentationSession`, `validate_graph`      |
| `fireside-tui`                   | `App`, `Action`, `Theme`, `resolve_theme`, `load_settings` |
| `clap`                           | Argument parsing with `derive` macros                      |
| `anyhow`                         | Error context for user-facing messages                     |
| `crossterm`                      | Raw mode, alternate screen, mouse capture                  |
| `ratatui`                        | `Terminal` and `CrosstermBackend` for the draw surface     |
| `serde` / `serde_json`           | Settings and scaffold file serialization                   |
| `tracing` / `tracing-subscriber` | Structured logging (warnings to stderr)                    |
| `time`                           | Timestamps in scaffolded presentation metadata             |
