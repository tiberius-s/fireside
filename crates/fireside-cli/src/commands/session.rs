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

use fireside_engine::{PresentationSession, load_graph};
use fireside_tui::config::resolve_theme;
use fireside_tui::config::settings::load_settings;
use fireside_tui::{Action, App};

use super::project::resolve_project_entry;

/// Run the interactive presentation.
pub fn run_presentation(file: &Path, theme_name: Option<&str>, start_node: usize) -> Result<()> {
    let graph = load_graph(file).context("loading graph")?;
    let settings = load_settings();

    let effective_theme = theme_name
        .or(graph.metadata.theme.as_deref())
        .or(settings.theme.as_deref());
    let theme = resolve_theme(effective_theme);

    let session = PresentationSession::new(graph, start_node.saturating_sub(1));
    let mut app = App::new(session, theme);
    app.set_document_path(file.to_path_buf());
    app.set_show_progress_bar(settings.show_progress);
    app.set_show_elapsed_timer(settings.show_timer);

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

        let poll_duration = if app.is_animating() {
            std::time::Duration::from_millis(50)
        } else {
            std::time::Duration::from_millis(idle_poll_timeout_ms.max(10))
        };

        if event::poll(poll_duration).context("polling events")? {
            let ev = event::read().context("reading event")?;
            app.handle_event(ev);
        } else if app.is_animating() {
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
