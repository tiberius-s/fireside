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

use std::fmt;
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

/// A write-back sink: called with an edited graph when the presenter saves
/// a quick edit. The presenter itself never touches the filesystem; the
/// caller owns the I/O and reports back whether the save succeeded.
pub type WriteBackSink<'a> = &'a mut dyn FnMut(&Graph) -> Result<(), WriteBackError>;

/// Why a quick-edit save could not be applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteBackError {
    /// No file backs this presentation (e.g. the built-in demo deck).
    Unavailable,
    /// The on-disk file changed since it was last loaded; the save was
    /// refused rather than risk silently discarding either version.
    Conflict,
    /// The write failed for a reason other than a conflict (permissions,
    /// disk full, etc.).
    Io(String),
}

impl fmt::Display for WriteBackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => write!(f, "Can't save — this deck has no file to save to"),
            Self::Conflict => write!(
                f,
                "Save skipped — the file changed on disk; Ctrl+S again to overwrite, Esc to discard your edit"
            ),
            Self::Io(message) => write!(f, "Save failed — {message}"),
        }
    }
}

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
    present_authoring(graph, source, &mut |_| Err(WriteBackError::Unavailable))
}

/// Present a graph with live reload and quick-edit write-back: on top of
/// `present_watching`'s reload polling, a presenter can quick-edit the
/// current node's heading/text blocks and save — the edited graph is
/// handed to `sink`, which owns all file I/O (`fireside-tui` performs
/// none), per ADR-005.
///
/// # Errors
///
/// Returns [`TuiError::Engine`] for an unpresentable graph and
/// [`TuiError::Io`] for terminal failures.
pub fn present_authoring(
    graph: Graph,
    source: ReloadSource<'_>,
    sink: WriteBackSink<'_>,
) -> Result<(), TuiError> {
    let session = Session::new(graph)?;
    let mut app = App::new(session);
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &mut app, source, sink);
    ratatui::restore();
    result
}

fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    source: ReloadSource<'_>,
    sink: WriteBackSink<'_>,
) -> Result<(), TuiError> {
    while !app.should_quit() {
        // A pending save is handled before any reload check, in the very
        // next iteration after the save keypress. The keypress that sets
        // it also flips `screen` from `Edit` back to `Present` in the same
        // `app.update` call, so if reload ran first here it would treat the
        // now-`Present` screen as license to poll, resync the watcher's
        // fingerprint to match any external change, and only then let the
        // save through — silently overwriting that external change and
        // defeating the conflict check below. Handling the save first
        // means write-back always compares against the fingerprint as it
        // stood before this tick touched anything. See research.md §4 in
        // specs/002-quick-edit-modal/.
        if let Some(graph) = app.take_pending_save() {
            let result = sink(&graph).map_err(|err| err.to_string());
            app.update(Msg::SaveResult(result));
        }
        // Reload is paused while the quick-edit modal is open: otherwise an
        // external edit lands mid-edit, `on_reload` silently swaps the
        // session out from under the open modal, and the eventual save
        // both overwrites the external edit and desyncs the write-back
        // conflict check above.
        if !matches!(app.screen(), app::Screen::Edit { .. })
            && let Some(result) = source()
        {
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
