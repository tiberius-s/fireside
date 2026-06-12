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

pub use app::App;
pub use error::TuiError;

/// Present a graph: set up the terminal, run the event loop, and always
/// restore the terminal — even on error.
///
/// # Errors
///
/// Returns [`TuiError::Engine`] for an unpresentable graph and
/// [`TuiError::Io`] for terminal failures.
pub fn present(graph: Graph) -> Result<(), TuiError> {
    let session = Session::new(graph)?;
    let mut app = App::new(session);
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn event_loop(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<(), TuiError> {
    while !app.should_quit() {
        terminal.draw(|frame| render::draw(frame, app))?;
        // The timeout lets expired flash messages clear without input.
        if event::poll(Duration::from_millis(250))? {
            let ev = event::read()?;
            app.update(&ev);
        }
    }
    Ok(())
}
