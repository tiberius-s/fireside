use std::io;
use std::path::Path;
use std::time::SystemTime;

use anyhow::{Context, Result};
use crossterm::event;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use fireside_engine::{PresentationSession, load_graph, load_graph_from_str};
use fireside_tui::config::resolve_theme;
use fireside_tui::config::settings::load_settings;
use fireside_tui::{Action, App};

use super::project::resolve_project_entry;

/// Export a plain-text, non-interactive representation of the graph to stdout.
/// Useful for CI, recording, or headless inspection.
pub fn run_presentation_plain(
    file: &Path,
    _theme_name: Option<&str>,
    start_node: usize,
    _start_in_edit: bool,
    _target_duration_secs: Option<u64>,
) -> Result<()> {
    let graph = load_graph(file).context("loading graph")?;

    // Header
    if let Some(title) = graph.metadata.title.as_deref() {
        println!("Title: {}", title);
    }
    if let Some(author) = graph.metadata.author.as_deref() {
        println!("Author: {}", author);
    }
    if let Some(desc) = graph.metadata.description.as_deref() {
        println!();
        println!("{}", desc);
    }
    println!();

    let start_idx = start_node.saturating_sub(1).min(graph.nodes.len());
    for (i, node) in graph.nodes.iter().enumerate().skip(start_idx) {
        let num = i + 1;
        let title = node.title.as_deref().unwrap_or("(no title)");
        println!("--- Node {}: {} ---", num, title);

        for block in &node.content {
            // Print a stable JSON representation of the block for headless export
            match serde_json::to_string_pretty(block) {
                Ok(s) => println!("{}", s),
                Err(e) => println!("<error serializing block: {}>", e),
            }
            println!();
        }
        println!();
    }

    Ok(())
}

/// Run the interactive presentation.
///
/// When `start_in_edit` is `true` the TUI opens directly in editor mode,
/// equivalent to `fireside edit` but with the presentation-style CLI entry
/// point (theme / start-node flags). In both cases the document path is set
/// so that pressing `e` to switch modes mid-session enables saving with `w`.
pub fn run_presentation(
    file: &Path,
    theme_name: Option<&str>,
    start_node: usize,
    start_in_edit: bool,
    target_duration_secs: Option<u64>,
) -> Result<()> {
    let graph = load_graph(file).context("loading graph")?;
    let settings = load_settings();

    let effective_theme = theme_name
        .or(graph.metadata.theme.as_deref())
        .or(settings.theme.as_deref());
    let theme = resolve_theme(effective_theme);

    let session = PresentationSession::new(graph, start_node.saturating_sub(1));
    let mut app = App::new(session, theme);
    app.set_document_path(file.to_path_buf());
    // Set the save target so that switching to editor mode mid-presentation
    // and pressing `w` works correctly — same as `run_editor()` does.
    app.set_editor_target_path(file.to_path_buf());
    app.set_show_progress_bar(settings.show_progress || target_duration_secs.is_some());
    app.set_show_elapsed_timer(settings.show_timer || target_duration_secs.is_some());
    app.set_target_duration_secs(target_duration_secs);
    if start_in_edit {
        app.enter_edit_mode();
    }

    enable_raw_mode().context("enabling raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("entering alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("creating terminal")?;

    let result = run_event_loop(
        &mut terminal,
        &mut app,
        settings.poll_timeout_ms,
        Some(file),
    );

    disable_raw_mode().context("disabling raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("leaving alternate screen")?;
    terminal.show_cursor().context("showing cursor")?;

    result
}

/// Open the TUI editor for a file or project.
pub fn run_editor(target: &Path) -> Result<()> {
    let (graph_path, project_theme) = if target.is_dir() {
        let (entry_path, theme) = resolve_project_entry(target)?;
        (entry_path, theme)
    } else {
        (target.to_path_buf(), None)
    };

    let graph = load_graph(&graph_path).context("loading graph")?;
    let settings = load_settings();
    let theme = resolve_theme(
        project_theme
            .as_deref()
            .or(graph.metadata.theme.as_deref())
            .or(settings.theme.as_deref()),
    );

    let session = PresentationSession::new(graph, 0);
    let mut app = App::new(session, theme);
    app.set_document_path(graph_path.clone());
    app.set_show_progress_bar(settings.show_progress);
    app.set_show_elapsed_timer(settings.show_timer);
    app.enter_edit_mode();
    app.set_editor_target_path(graph_path);

    enable_raw_mode().context("enabling raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("entering alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("creating terminal")?;

    let result = run_event_loop(&mut terminal, &mut app, settings.poll_timeout_ms, None);

    disable_raw_mode().context("disabling raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("leaving alternate screen")?;
    terminal.show_cursor().context("showing cursor")?;

    result
}

/// Run the in-memory welcome graph.
pub fn run_welcome(start_in_edit: bool) -> Result<()> {
    let settings = load_settings();
    let theme = resolve_theme(settings.theme.as_deref());
    let session = build_welcome_session().context("building welcome graph")?;
    let mut app = App::new(session, theme);
    app.set_show_progress_bar(settings.show_progress);
    app.set_show_elapsed_timer(settings.show_timer);
    if start_in_edit {
        app.enter_edit_mode();
    }

    enable_raw_mode().context("enabling raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("entering alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("creating terminal")?;

    let result = run_event_loop(&mut terminal, &mut app, settings.poll_timeout_ms, None);

    disable_raw_mode().context("disabling raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("leaving alternate screen")?;
    terminal.show_cursor().context("showing cursor")?;

    result
}

/// The main event loop implementing the TEA pattern.
fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    idle_poll_timeout_ms: u64,
    watch_path: Option<&Path>,
) -> Result<()> {
    let mut last_modified = watch_path.and_then(file_modified_time);

    if let Ok(size) = terminal.size() {
        app.update(Action::Resize(size.width, size.height));
    }

    loop {
        if app.take_needs_redraw() {
            terminal
                .draw(|frame| app.view(frame))
                .context("drawing frame")?;
        }

        if app.should_quit() {
            break;
        }

        let needs_periodic_tick = app.needs_periodic_tick();

        let poll_duration = if needs_periodic_tick {
            std::time::Duration::from_millis(50)
        } else {
            std::time::Duration::from_millis(idle_poll_timeout_ms.max(10))
        };

        if event::poll(poll_duration).context("polling events")? {
            let ev = event::read().context("reading event")?;
            app.handle_event(ev);
        } else if needs_periodic_tick {
            app.update(Action::Tick);
        }

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
    }

    Ok(())
}

fn file_modified_time(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

fn build_welcome_session() -> Result<PresentationSession> {
    const WELCOME_JSON: &str = r#"{
  "title": "Fireside Welcome",
  "fireside-version": "0.1.0",
  "description": "In-memory welcome flow",
  "tags": ["welcome"],
  "nodes": [
    {
      "id": "welcome",
      "title": "Welcome",
      "tags": ["welcome"],
      "speaker-notes": "Use 1/2/3 to choose a path, e to edit, ? for help.",
      "traversal": {
        "branch-point": {
          "id": "welcome-choices",
          "prompt": "Where would you like to start?",
          "options": [
            { "label": "Present an existing file", "key": "1", "target": "present-file" },
            { "label": "Open editor mode", "key": "2", "target": "edit-mode" },
            { "label": "Read quick-start tips", "key": "3", "target": "quick-start" }
          ]
        }
      },
      "content": [
        { "kind": "heading", "level": 1, "text": "Welcome to Fireside" },
        { "kind": "text", "body": "No file was provided, so this in-memory welcome presentation is running." },
        {
          "kind": "list",
          "ordered": false,
          "items": [
            "Press e to switch between presentation and editor modes",
            "Press n to advance, b to go back, and ? for keybindings",
            "Use fireside new demo to scaffold a file"
          ]
        }
      ]
    },
    {
      "id": "present-file",
      "title": "Present a File",
      "traversal": { "next": "welcome" },
      "content": [
        { "kind": "heading", "level": 2, "text": "Present a file" },
        { "kind": "text", "body": "Run: fireside present path/to/graph.json" }
      ]
    },
    {
      "id": "edit-mode",
      "title": "Editor Mode",
      "traversal": { "next": "welcome" },
      "content": [
        { "kind": "heading", "level": 2, "text": "Open editor mode" },
        { "kind": "text", "body": "Run: fireside edit path/to/graph.json" }
      ]
    },
    {
      "id": "quick-start",
      "title": "Quick Start",
      "traversal": { "next": "welcome" },
      "content": [
        { "kind": "heading", "level": 2, "text": "Quick start" },
        { "kind": "text", "body": "Run: fireside new demo, then fireside present demo.json" }
      ]
    }
  ]
}"#;

    let graph = load_graph_from_str(WELCOME_JSON).context("parsing welcome graph")?;
    Ok(PresentationSession::new(graph, 0))
}
