//! Fireside TUI — the ratatui presenter.
//!
//! One job: present a validated [`fireside_core::Graph`] in the terminal so
//! well that someone who has never seen Fireside can run a deck. The state
//! machine lives in [`app`] (TEA: `App::update` is the sole mutation point),
//! drawing in [`render`], and every color in [`theme::Tokens`].

pub mod app;
pub mod error;
pub mod render;
pub mod theme;

use std::time::Duration;

use crossterm::event;
use fireside_core::Graph;
use fireside_engine::Session;

pub use app::{App, Msg};
pub use error::TuiError;

/// A live-reload source: polled on every event tick, it returns `Some`
/// when the deck changed on disk — a fresh graph, or a human-readable
/// message about why the changed file could not be loaded. The presenter
/// itself never touches the filesystem; the caller owns the I/O.
pub type ReloadSource<'a> = &'a mut dyn FnMut() -> Option<Result<Graph, String>>;

/// Present a graph: set up the terminal, run the event loop, and always
/// restore the terminal — even on error.
///
/// # Errors
///
/// Returns [`TuiError::Engine`] for an unpresentable graph and
/// [`TuiError::Io`] for terminal failures.
pub fn present(graph: Graph) -> Result<(), TuiError> {
    present_watching(graph, &mut || None)
}

/// Present a graph with live reload: while presenting, `source` is polled
/// a few times per second, and any deck it hands back is swapped in
/// without leaving the current slide.
///
/// # Errors
///
/// Returns [`TuiError::Engine`] for an unpresentable graph and
/// [`TuiError::Io`] for terminal failures.
pub fn present_watching(graph: Graph, source: ReloadSource<'_>) -> Result<(), TuiError> {
    let session = Session::new(graph)?;
    let mut app = App::new(session);
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &mut app, source);
    ratatui::restore();
    result
}

fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    source: ReloadSource<'_>,
) -> Result<(), TuiError> {
    while !app.should_quit() {
        if let Some(result) = source() {
            app.update(Msg::Reload(result));
        }
        terminal.draw(|frame| render::draw(frame, app))?;
        // The timeout lets expired flash messages clear without input; a
        // fading slide polls fast so it brightens on time.
        let timeout = if app.fading() {
            Duration::from_millis(30)
        } else {
            Duration::from_millis(250)
        };
        if event::poll(timeout)? {
            app.update(Msg::Terminal(event::read()?));
        }
    }
    Ok(())
}
