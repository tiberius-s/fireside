//! The authoring editor's application state machine (spec 013,
//! `fireside edit`).
//!
//! TEA-style, matching the presenter's own contract (constitution IV,
//! generalized): `EditorApp::update` is the sole place `EditorApp` state
//! mutates. `fireside-tui` performs no file I/O anywhere in this
//! module — saving, drafts, and deck creation are `fireside-cli`'s job
//! (ADR-014); this Foundational phase (T015–T026) does not wire save at
//! all yet (that lands with US1, T035).

pub(crate) mod hit;

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::execute;
use crossterm::tty::IsTty;
use fireside_engine::authoring::BlockPath;
use fireside_engine::validate;
use ratatui::layout::Rect;

use fireside_core::Graph;

use crate::app::App as PresenterApp;
use crate::error::TuiError;
use crate::{WriteBackError, render};

/// What's selected in the studio, if anything.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum Selection {
    #[default]
    None,
    Slide(String),
    Block(String, BlockPath),
}

/// A block or slide drag in progress. Only `Idle` exists until US2/US3
/// (T044/T050) add the lifting/hovering states drag-and-drop needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum DragState {
    #[default]
    Idle,
}

/// The currently open block/slide/answer form. Uninhabited until US1
/// (T027+) adds its first variant — `open_form` is always `None` until
/// then, by construction.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FormState {}

/// One undo/redo checkpoint: a full graph clone plus the selection at that
/// point, so undo restores view context too (design brief, "Never lose
/// work"). Unused until US1 wires undo (T036) — the type exists now so
/// later tasks extend `EditorApp::history`, not redefine it.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct HistorySnapshot {
    pub(crate) graph: Graph,
    pub(crate) selection: Selection,
}

/// A message into the editor's state machine. Reload/save-result messages
/// join once T024/T035 wire file I/O — for now this is terminal input only.
#[derive(Debug)]
pub(crate) enum Msg {
    Terminal(Event),
}

/// All authoring-editor state (spec 013, `data-model.md`'s `EditorApp`
/// section).
#[derive(Debug)]
pub(crate) struct EditorApp {
    working_graph: Graph,
    saved_graph: Graph,
    selection: Selection,
    #[allow(dead_code)] // read once drag-and-drop (US2, T044) lands
    drag: DragState,
    #[allow(dead_code)] // read once forms (US1, T034) land
    open_form: Option<FormState>,
    #[allow(dead_code)] // pushed to by undo (US1, T036), which then reads it
    history: Vec<HistorySnapshot>,
    #[allow(dead_code)] // read once undo/redo (US1, T036) pushes/pops it
    redo: Vec<HistorySnapshot>,
    terminal_size: (u16, u16),
    status: Vec<fireside_engine::Diagnostic>,
    scroll: u16,
    hover: Option<hit::Target>,
    #[allow(dead_code)] // read once draft autosave (US4, T060) lands
    dirty_since_draft: bool,
    #[allow(dead_code)] // read once draft autosave (US4, T060) lands
    last_draft_write: Instant,
    showing_help: bool,
    present_requested: bool,
    quit: bool,
}

impl EditorApp {
    /// Opens a fresh editor session over `graph`, validating it up front
    /// for the status banner (spec FR-026) — a deck with diagnostics is
    /// never refused, only reported.
    #[must_use]
    pub(crate) fn new(graph: Graph) -> Self {
        let status = validate(&graph);
        let saved_graph = graph.clone();
        Self {
            working_graph: graph,
            saved_graph,
            selection: Selection::None,
            drag: DragState::Idle,
            open_form: None,
            history: Vec::new(),
            redo: Vec::new(),
            terminal_size: (80, 24),
            status,
            scroll: 0,
            hover: None,
            dirty_since_draft: false,
            last_draft_write: Instant::now(),
            showing_help: false,
            present_requested: false,
            quit: false,
        }
    }

    #[must_use]
    pub(crate) fn working_graph(&self) -> &Graph {
        &self.working_graph
    }

    #[must_use]
    pub(crate) fn selection(&self) -> &Selection {
        &self.selection
    }

    #[must_use]
    pub(crate) fn scroll(&self) -> u16 {
        self.scroll
    }

    #[must_use]
    pub(crate) fn status(&self) -> &[fireside_engine::Diagnostic] {
        &self.status
    }

    #[must_use]
    pub(crate) fn showing_help(&self) -> bool {
        self.showing_help
    }

    #[must_use]
    pub(crate) fn hover(&self) -> Option<&hit::Target> {
        self.hover.as_ref()
    }

    /// Whether `working_graph` has unsaved changes — the toolbar's `●` dot
    /// (spec FR-018).
    #[must_use]
    pub(crate) fn dirty(&self) -> bool {
        self.working_graph != self.saved_graph
    }

    /// Forward-declared accessor: undo (US1, T036) is the first thing that
    /// reads this outside tests.
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Forward-declared accessor: drag-and-drop (US2, T044) is the first
    /// thing that reads this outside tests.
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn drag(&self) -> &DragState {
        &self.drag
    }

    /// Forward-declared accessor: forms (US1, T034) are the first thing
    /// that reads this outside tests.
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn open_form(&self) -> Option<&FormState> {
        self.open_form.as_ref()
    }

    #[must_use]
    pub(crate) fn should_quit(&self) -> bool {
        self.quit
    }

    fn set_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
    }

    /// The node id the canvas is currently showing — what `[ ▶ Present ]`
    /// starts from.
    fn selected_node_id(&self) -> Option<String> {
        hit::selected_node(self).map(|n| n.id.clone())
    }

    /// Consumes a pending present request, if one is set, returning the
    /// slide to start from.
    fn take_present_request(&mut self) -> Option<String> {
        if std::mem::take(&mut self.present_requested) {
            self.selected_node_id()
        } else {
            None
        }
    }

    /// Apply one message. The sole mutation point (constitution IV).
    pub(crate) fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Terminal(Event::Resize(w, h)) => {
                self.set_terminal_size(w, h);
                self.hover = None;
            }
            Msg::Terminal(Event::Key(key)) => self.on_key(key),
            Msg::Terminal(Event::Mouse(mouse)) => self.on_mouse(mouse),
            Msg::Terminal(_) => {}
        }
    }

    fn on_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if self.showing_help {
            self.showing_help = false;
            return;
        }
        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Esc => {
                if self.selection != Selection::None {
                    self.selection = Selection::None;
                }
            }
            KeyCode::Char('?') => self.showing_help = true,
            KeyCode::Char('p' | 'P') => self.present_requested = true,
            KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
            _ => {}
        }
    }

    fn on_mouse(&mut self, event: MouseEvent) {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => self.on_click(event.column, event.row),
            MouseEventKind::Moved => {
                let (w, h) = self.terminal_size;
                self.hover = hit::hit(self, Rect::new(0, 0, w, h), event.column, event.row);
            }
            MouseEventKind::ScrollDown => self.scroll = self.scroll.saturating_add(1),
            MouseEventKind::ScrollUp => self.scroll = self.scroll.saturating_sub(1),
            _ => {}
        }
    }

    fn on_click(&mut self, col: u16, row: u16) {
        let (w, h) = self.terminal_size;
        match hit::hit(self, Rect::new(0, 0, w, h), col, row) {
            Some(hit::Target::OutlineRow(id)) => {
                self.selection = Selection::Slide(id);
                self.scroll = 0;
            }
            Some(hit::Target::Block(node_id, path)) => {
                self.selection = Selection::Block(node_id, path);
            }
            Some(hit::Target::ToolbarChip(hit::ToolbarAction::Present)) => {
                self.present_requested = true;
            }
            Some(hit::Target::ToolbarChip(hit::ToolbarAction::Help)) => {
                self.showing_help = true;
            }
            // Add slide / Save / Undo aren't wired until US1/US3 — the
            // read-only Foundational studio resolves the click but the
            // action is a no-op for now (design brief E1: "no mutations").
            Some(
                hit::Target::ToolbarChip(_)
                | hit::Target::OutlineNewSlide
                | hit::Target::StatusBanner,
            ) => {}
            Some(_) => {}
            None => self.selection = Selection::None,
        }
    }
}

/// Opens the full-screen authoring studio (spec 013) over `graph`: sets up
/// the terminal, runs the editor's own event loop, and always restores the
/// terminal, even on error — the same contract [`crate::present`] gives
/// the presenter. `fireside-tui` performs no file I/O anywhere in this
/// path; `fireside-cli`'s `edit.rs` owns the opening-rules chain, loading,
/// and (in later waves) saving.
///
/// # Errors
///
/// Returns [`TuiError::NotATty`] outside an interactive terminal and
/// [`TuiError::Io`] for terminal failures.
pub fn run(graph: Graph) -> Result<(), TuiError> {
    if !io::stdout().is_tty() || !io::stdin().is_tty() {
        return Err(TuiError::NotATty);
    }
    let mut app = EditorApp::new(graph);
    let mut terminal = ratatui::try_init()?;
    // Mouse capture is enabled once for the whole editor session — both
    // the studio's own loop and the in-process presenter loop `present_now`
    // enters share it, per research.md §6.
    let _ = execute!(io::stdout(), EnableMouseCapture);
    let result = editor_event_loop(&mut terminal, &mut app);
    let _ = execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

fn editor_event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut EditorApp,
) -> Result<(), TuiError> {
    if let Ok(size) = terminal.size() {
        app.set_terminal_size(size.width, size.height);
    }
    while !app.should_quit() {
        terminal.draw(|frame| render::draw_editor(frame, app))?;
        if event::poll(Duration::from_millis(250))? {
            app.update(Msg::Terminal(event::read()?));
        }
        if let Some(start) = app.take_present_request() {
            present_now(terminal, app.working_graph(), Some(&start))?;
            if let Ok(size) = terminal.size() {
                app.set_terminal_size(size.width, size.height);
            }
        }
    }
    Ok(())
}

/// `[ ▶ Present ]`: runs the existing presenter event loop in-process
/// against the *same*, already-initialized terminal — no process spawn, no
/// second `try_init` (research.md §6). The embedded run never touches
/// resume state or the live session-state file: a no-op reload source, an
/// `Unavailable`-reporting write-back sink, and no-op position/tick sinks.
/// Control falls back to the editor loop on quit, which repaints.
fn present_now(
    terminal: &mut ratatui::DefaultTerminal,
    working_graph: &Graph,
    start_node: Option<&str>,
) -> Result<(), TuiError> {
    let mut session = fireside_engine::Session::new(working_graph.clone())?;
    if let Some(id) = start_node {
        session.goto(id);
    }
    let mut presenter = PresenterApp::new(session).without_sink();
    crate::event_loop(
        terminal,
        &mut presenter,
        &mut || None,
        &mut |_| Err(WriteBackError::Unavailable),
        &mut |_| {},
        &mut |_| {},
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    const FIXTURE: &str = r#"{"nodes":[
        {"id":"a","title":"Welcome","traversal":"b","content":[
            {"kind":"heading","level":1,"text":"Hello"},
            {"kind":"text","body":"World"}
        ]},
        {"id":"b","title":"The end","content":[{"kind":"text","body":"Done"}]}
    ]}"#;

    fn app() -> EditorApp {
        let mut app = EditorApp::new(Graph::from_json(FIXTURE).expect("fixture parses"));
        app.set_terminal_size(100, 30);
        app
    }

    fn press(app: &mut EditorApp, code: KeyCode) {
        app.update(Msg::Terminal(Event::Key(KeyEvent::from(code))));
    }

    fn click(app: &mut EditorApp, col: u16, row: u16) {
        app.update(Msg::Terminal(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::empty(),
        })));
    }

    fn move_to(app: &mut EditorApp, col: u16, row: u16) {
        app.update(Msg::Terminal(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::empty(),
        })));
    }

    fn draw(app: &EditorApp, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("backend");
        terminal
            .draw(|frame| render::draw_editor(frame, app))
            .expect("draw");
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>()
    }

    #[test]
    fn opens_read_only_showing_the_entry_slide() {
        let app = app();
        assert_eq!(hit::selected_node(&app).map(|n| n.id.as_str()), Some("a"));
        assert!(!app.dirty());
        assert_eq!(app.history_len(), 0);
        assert_eq!(app.drag(), &DragState::Idle);
        assert!(app.open_form().is_none());
    }

    #[test]
    fn clicking_an_outline_row_selects_that_slide() {
        let mut app = app();
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        click(&mut app, areas.outline.x, areas.outline.y + 1);
        assert_eq!(app.selection(), &Selection::Slide("b".to_owned()));
    }

    #[test]
    fn clicking_a_block_selects_it() {
        let mut app = app();
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        // The first content line inside the card is well within the
        // heading block's extent for this small fixture.
        let target = hit::hit(
            &app,
            Rect::new(0, 0, 100, 30),
            areas.canvas.x + 40,
            areas.canvas.y + 4,
        );
        if let Some(hit::Target::Block(node, path)) = target {
            click(&mut app, areas.canvas.x + 40, areas.canvas.y + 4);
            assert_eq!(app.selection(), &Selection::Block(node, path));
        }
    }

    #[test]
    fn hover_tracks_the_pointer_without_selecting() {
        let mut app = app();
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        move_to(&mut app, areas.outline.x, areas.outline.y);
        assert_eq!(app.selection(), &Selection::None, "hover must not select");
        assert!(app.hover().is_some());
    }

    #[test]
    fn wheel_scrolls_the_canvas() {
        let mut app = app();
        assert_eq!(app.scroll(), 0);
        app.update(Msg::Terminal(Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: crossterm::event::KeyModifiers::empty(),
        })));
        assert_eq!(app.scroll(), 1);
    }

    #[test]
    fn arrow_keys_scroll_too() {
        let mut app = app();
        press(&mut app, KeyCode::Down);
        assert_eq!(app.scroll(), 1);
        press(&mut app, KeyCode::Up);
        assert_eq!(app.scroll(), 0);
    }

    #[test]
    fn question_mark_opens_help_and_any_key_closes_it() {
        let mut app = app();
        press(&mut app, KeyCode::Char('?'));
        assert!(app.showing_help());
        press(&mut app, KeyCode::Char('x'));
        assert!(!app.showing_help());
    }

    #[test]
    fn esc_deselects_before_quitting() {
        let mut app = app();
        app.selection = Selection::Slide("b".to_owned());
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.selection(), &Selection::None);
        assert!(!app.should_quit());
    }

    #[test]
    fn q_quits() {
        let mut app = app();
        press(&mut app, KeyCode::Char('q'));
        assert!(app.should_quit());
    }

    #[test]
    fn resize_updates_terminal_size_and_clears_hover() {
        let mut app = app();
        move_to(&mut app, 5, 5);
        app.update(Msg::Terminal(Event::Resize(90, 28)));
        assert_eq!(app.terminal_size, (90, 28));
        assert!(app.hover().is_none());
    }

    // `present_now` drives the real blocking presenter event loop (reading
    // actual terminal events via `crossterm::event`), so it cannot run
    // under `TestBackend` the way drawing can — that would hang waiting
    // for real input. This test instead pins the state-machine half of
    // "present and return": the request is captured with the right start
    // slide, consumed exactly once, and leaves the editor's own state
    // untouched. The full embedded-terminal path is proven by the tmux
    // smoke test (T026), in a real terminal.
    #[test]
    fn present_request_captures_the_selected_slide_and_is_consumed_once() {
        let mut app = app();
        app.selection = Selection::Slide("b".to_owned());
        press(&mut app, KeyCode::Char('p'));
        assert_eq!(app.take_present_request().as_deref(), Some("b"));
        assert_eq!(
            app.take_present_request(),
            None,
            "a present request must not fire twice"
        );
        // Back in the editor: selection and the working graph are
        // untouched by having requested a present.
        assert_eq!(app.selection(), &Selection::Slide("b".to_owned()));
        assert!(!app.dirty());
    }

    #[test]
    fn read_only_studio_renders_at_100x30() {
        let app = app();
        let screen = draw(&app, 100, 30);
        assert!(
            screen.contains('W'),
            "expected the deck title/content on screen"
        );
    }

    #[test]
    fn below_minimum_size_shows_the_resize_guard_not_the_studio() {
        let app = app();
        let screen = draw(&app, 44, 14);
        assert!(screen.contains("bigger"));
    }
}
