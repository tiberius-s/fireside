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
use std::io;
use std::time::Duration;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate};
use fireside_core::Graph;
use fireside_engine::{Outcome, Session};

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

/// A position-changed sink: called with the new current node id every time
/// it changes (including once, immediately, with the starting node). The
/// presenter itself never touches the filesystem; a caller that wants to
/// persist "where the presenter is" (e.g. resume-on-relaunch) owns all I/O.
pub type PositionSink<'a> = &'a mut dyn FnMut(&str);

/// What a presentation session accomplished, returned on a graceful stop
/// (the `q` key or in-TUI Ctrl+C — both exit the event loop identically;
/// see `specs/010-presenter-polish/research.md` §3) so a caller can report
/// a rehearsal summary. `fireside-tui` never prints this itself — the
/// caller owns all terminal output outside the TUI's own frames.
#[derive(Debug, Clone, Copy)]
pub struct PresentSummary {
    /// Distinct slides visited this session.
    pub seen: usize,
    /// Total slides in the deck.
    pub total: usize,
    /// Wall-clock time since the presentation started.
    pub elapsed: Duration,
}

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
pub fn present(graph: Graph) -> Result<PresentSummary, TuiError> {
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
pub fn present_watching(
    graph: Graph,
    source: ReloadSource<'_>,
) -> Result<PresentSummary, TuiError> {
    present_authoring(
        graph,
        source,
        &mut |_| Err(WriteBackError::Unavailable),
        None,
        &mut |_| {},
    )
}

/// Present a graph with live reload and quick-edit write-back: on top of
/// `present_watching`'s reload polling, a presenter can quick-edit the
/// current node's heading/text blocks and save — the edited graph is
/// handed to `sink`, which owns all file I/O (`fireside-tui` performs
/// none), per ADR-005. `initial_node` (when it names a real node) opens the
/// presentation there instead of the graph's normal entry node — an unknown
/// id is a guarded no-op, per `Session::goto`, falling back to the entry
/// node exactly as an unrecognized `goto` always has. `on_position_changed`
/// is called with the current node id once at startup and again every time
/// it changes, for a caller that wants to persist "where the presenter is"
/// (e.g. resume-on-relaunch) — `fireside-tui` performs no file I/O itself.
///
/// # Errors
///
/// Returns [`TuiError::Engine`] for an unpresentable graph and
/// [`TuiError::Io`] for terminal failures.
pub fn present_authoring(
    graph: Graph,
    source: ReloadSource<'_>,
    sink: WriteBackSink<'_>,
    initial_node: Option<&str>,
    on_position_changed: PositionSink<'_>,
) -> Result<PresentSummary, TuiError> {
    let total = graph.nodes.len();
    let mut session = Session::new(graph)?;
    let resumed = initial_node.is_some_and(|id| matches!(session.goto(id), Outcome::Moved));
    let mut app = App::new(session);
    if resumed {
        app.set_flash(
            "Resumed where you left off — --restart starts over",
            app::FlashKind::Info,
        );
    }
    let mut terminal = ratatui::init();
    // Mouse is additive on top of the keyboard contract (constitution
    // Principle II) — enabled/disabled around the same window raw mode is,
    // so a panic or early return still leaves the terminal in mouse-off,
    // cooked-mode state via `ratatui::restore()`.
    let _ = execute!(io::stdout(), EnableMouseCapture);
    let result = event_loop(&mut terminal, &mut app, source, sink, on_position_changed);
    let _ = execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result.map(|()| PresentSummary {
        seen: app.session().visited().len(),
        total,
        elapsed: app.elapsed(),
    })
}

fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    source: ReloadSource<'_>,
    sink: WriteBackSink<'_>,
    on_position_changed: PositionSink<'_>,
) -> Result<(), TuiError> {
    let mut last_id = app.session().current().id.clone();
    on_position_changed(&last_id);
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
        // Synchronized output eliminates any visible tearing mid-transition;
        // it is just an escape-sequence pair a terminal either honors or
        // silently ignores (DEC private mode 2026), so no capability query
        // is needed — the same "invisible if unsupported" reasoning already
        // used for the `fade` transition's fallback (Appendix D).
        let _ = execute!(io::stdout(), BeginSynchronizedUpdate);
        terminal.draw(|frame| render::draw(frame, app))?;
        let _ = execute!(io::stdout(), EndSynchronizedUpdate);
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
        let current_id = &app.session().current().id;
        if *current_id != last_id {
            last_id = current_id.clone();
            on_position_changed(&last_id);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::Command;
    use crossterm::terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate};

    /// `event_loop` brackets every `terminal.draw` with these two exact
    /// escape sequences (DEC private mode 2026) via `execute!` — this pins
    /// down the byte-level contract `crossterm::terminal::{Begin,End}SynchronizedUpdate`
    /// promise to emit, which is what `event_loop` relies on being a no-op
    /// on a terminal that doesn't understand it (research.md §3). The full
    /// "no visible tearing" claim itself is a real-terminal property, not
    /// something a headless test can observe — proven in tmux instead
    /// (quickstart.md §3).
    #[test]
    fn synchronized_update_commands_are_the_expected_escape_sequences() {
        let mut begin = String::new();
        BeginSynchronizedUpdate
            .write_ansi(&mut begin)
            .expect("write_ansi");
        assert_eq!(begin, "\x1b[?2026h");

        let mut end = String::new();
        EndSynchronizedUpdate
            .write_ansi(&mut end)
            .expect("write_ansi");
        assert_eq!(end, "\x1b[?2026l");
    }
}
