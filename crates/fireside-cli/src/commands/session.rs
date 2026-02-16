use std::io;
use std::path::Path;

use anyhow::{Context, Result};
use crossterm::event;
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use fireside_engine::{PresentationSession, load_graph};
use fireside_tui::App;
use fireside_tui::config::resolve_theme;

/// Run the interactive presentation.
pub fn run_presentation(file: &Path, theme_name: Option<&str>, start_node: usize) -> Result<()> {
    let graph = load_graph(file).context("loading graph")?;

    let effective_theme = theme_name.or(graph.metadata.theme.as_deref());
    let theme = resolve_theme(effective_theme);

    let session = PresentationSession::new(graph, start_node.saturating_sub(1));
    let mut app = App::new(session, theme);

    enable_raw_mode().context("enabling raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("entering alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("creating terminal")?;

    let result = run_event_loop(&mut terminal, &mut app);

    disable_raw_mode().context("disabling raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).context("leaving alternate screen")?;
    terminal.show_cursor().context("showing cursor")?;

    result
}

/// Open the TUI editor for a file or project.
pub fn run_editor(_target: &Path) -> Result<()> {
    anyhow::bail!("The node editor is not yet implemented")
}

/// The main event loop implementing the TEA pattern.
fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal
            .draw(|frame| app.view(frame))
            .context("drawing frame")?;

        if app.should_quit() {
            break;
        }

        if event::poll(std::time::Duration::from_millis(250)).context("polling events")? {
            let ev = event::read().context("reading event")?;
            app.handle_event(ev);
        }
    }

    Ok(())
}
