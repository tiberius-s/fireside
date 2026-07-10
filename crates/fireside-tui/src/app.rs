//! The presenter application state machine.
//!
//! TEA-style: [`App::update`] is the **only** place state mutates. It
//! receives terminal events (keys, resizes) and applies them; rendering
//! reads the state and draws. Every keypress that cannot act produces a
//! flash message — the presenter is never left wondering whether a key
//! "worked".

use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use fireside_core::{Graph, Transition, ViewMode};
use fireside_engine::{Outcome, Session, Severity, validate};

use crate::render;

/// How long feedback messages stay on screen.
const FLASH_DURATION: Duration = Duration::from_millis(3000);

/// How long a slide's fade-in lasts: one dim beat, then full brightness.
const FADE_DURATION: Duration = Duration::from_millis(90);

/// A message into the state machine: terminal input, or a fresh read of
/// the deck source while presenting (live reload).
#[derive(Debug)]
pub enum Msg {
    /// A terminal event (key press, resize).
    Terminal(Event),
    /// The deck file changed on disk and was re-read: a new graph, or a
    /// human-readable message about why it could not be loaded.
    Reload(Result<Graph, String>),
}

/// Which screen the presenter is looking at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// The slide itself.
    Present,
    /// The key reference overlay.
    Help,
    /// The map: every slide, visited markers, jump on Enter.
    Map {
        /// Index of the highlighted node.
        selected: usize,
    },
}

/// The tone of a flash message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashKind {
    /// Neutral guidance.
    Info,
    /// Something was refused.
    Error,
}

/// A transient feedback message shown in the footer.
#[derive(Debug, Clone)]
pub struct Flash {
    /// The message text.
    pub text: String,
    /// Its tone.
    pub kind: FlashKind,
    expires: Instant,
}

/// All presenter state.
#[derive(Debug)]
pub struct App {
    session: Session,
    screen: Screen,
    branch_selected: usize,
    scroll: u16,
    view_override: Option<ViewMode>,
    show_notes: bool,
    show_timer: bool,
    started: Instant,
    flash: Option<Flash>,
    fade_started: Option<Instant>,
    viewport: (u16, u16),
    quit: bool,
}

impl App {
    /// Create the app over a live session.
    #[must_use]
    pub fn new(session: Session) -> Self {
        Self {
            session,
            screen: Screen::Present,
            branch_selected: 0,
            scroll: 0,
            view_override: None,
            show_notes: false,
            show_timer: false,
            started: Instant::now(),
            flash: None,
            fade_started: None,
            viewport: (80, 24),
            quit: false,
        }
    }

    /// The live session.
    #[must_use]
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// The active screen.
    #[must_use]
    pub fn screen(&self) -> Screen {
        self.screen
    }

    /// Index of the highlighted branch option.
    #[must_use]
    pub fn branch_selected(&self) -> usize {
        self.branch_selected
    }

    /// Current content scroll offset in lines.
    #[must_use]
    pub fn scroll(&self) -> u16 {
        self.scroll
    }

    /// Whether the speaker-notes panel is open.
    #[must_use]
    pub fn show_notes(&self) -> bool {
        self.show_notes
    }

    /// Whether the elapsed timer is on screen.
    #[must_use]
    pub fn show_timer(&self) -> bool {
        self.show_timer
    }

    /// Time since the presentation started.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// The active flash message, if it has not expired.
    #[must_use]
    pub fn flash(&self) -> Option<&Flash> {
        self.flash.as_ref().filter(|f| f.expires > Instant::now())
    }

    /// Whether the event loop should exit.
    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.quit
    }

    /// Whether the current slide is inside its brief fade-in window. The
    /// renderer dims the slide while this holds; the event loop polls fast
    /// so the brighten lands on time.
    #[must_use]
    pub fn fading(&self) -> bool {
        self.fade_started
            .is_some_and(|started| started.elapsed() < FADE_DURATION)
    }

    /// The view mode in effect: the presenter's runtime toggle wins over the
    /// document (spec: the node-level value is a suggestion, not a
    /// constraint).
    #[must_use]
    pub fn view_mode(&self) -> ViewMode {
        self.view_override.unwrap_or_else(|| {
            self.session
                .current()
                .resolved_view_mode(self.session.defaults())
        })
    }

    /// Apply one message. The sole mutation point.
    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Terminal(Event::Resize(w, h)) => self.viewport = (w, h),
            Msg::Terminal(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                self.on_key(key);
            }
            Msg::Terminal(_) => {}
            Msg::Reload(result) => self.on_reload(result),
        }
    }

    /// Swap in a re-read deck without losing the presenter's place. A save
    /// that broke the deck never replaces the working one — the presenter
    /// keeps the old slides and a footer message says what happened.
    fn on_reload(&mut self, result: Result<Graph, String>) {
        let graph = match result {
            Ok(graph) => graph,
            Err(message) => {
                self.set_flash(&message, FlashKind::Error);
                return;
            }
        };
        let errors = validate(&graph)
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        if errors > 0 {
            let word = if errors == 1 { "problem" } else { "problems" };
            self.set_flash(
                &format!("Reload skipped — the saved deck has {errors} {word}; fix and save again"),
                FlashKind::Error,
            );
            return;
        }
        let here = self.session.current().id.clone();
        let Ok(mut session) = Session::new(graph) else {
            self.set_flash("Reload skipped — the saved deck has no slides", FlashKind::Error);
            return;
        };
        let survived = session.graph().node(&here).is_some();
        if survived && session.current().id != here {
            let _ = session.goto(&here);
        }
        self.session = session;
        self.scroll = 0;
        self.branch_selected = 0;
        self.fade_started = None;
        if survived {
            self.set_flash("Reloaded", FlashKind::Info);
        } else {
            self.set_flash(
                &format!("Reloaded — \"{here}\" is gone, back at the start"),
                FlashKind::Info,
            );
        }
    }

    fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        match self.screen {
            Screen::Help => self.screen = Screen::Present,
            Screen::Map { selected } => self.on_map_key(key.code, selected),
            Screen::Present => self.on_present_key(key.code),
        }
    }

    fn on_map_key(&mut self, code: KeyCode, selected: usize) {
        let count = self.session.graph().nodes.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.screen = Screen::Map { selected: selected.saturating_sub(1) };
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.screen = Screen::Map { selected: (selected + 1).min(count.saturating_sub(1)) };
            }
            KeyCode::Enter => {
                let id = self.session.graph().nodes[selected].id.clone();
                self.screen = Screen::Present;
                if id != self.session.current().id {
                    let outcome = self.session.goto(&id);
                    self.apply(&outcome);
                }
            }
            KeyCode::Esc | KeyCode::Char('m' | 'g' | 'q') => self.screen = Screen::Present,
            _ => {}
        }
    }

    fn on_present_key(&mut self, code: KeyCode) {
        let at_branch = self.session.branch_point().is_some();
        match code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('?' | 'h') => self.screen = Screen::Help,
            KeyCode::Char('m' | 'g') => {
                let current = self.session.current().id.clone();
                let selected = self
                    .session
                    .graph()
                    .nodes
                    .iter()
                    .position(|n| n.id == current)
                    .unwrap_or(0);
                self.screen = Screen::Map { selected };
            }
            KeyCode::Char('f') => {
                let next = match self.view_mode() {
                    ViewMode::Default => ViewMode::Fullscreen,
                    ViewMode::Fullscreen => ViewMode::Default,
                };
                self.view_override = Some(next);
                self.set_flash(
                    match next {
                        ViewMode::Fullscreen => "Fullscreen — press f to exit",
                        ViewMode::Default => "Standard view",
                    },
                    FlashKind::Info,
                );
            }
            KeyCode::Char('s') => {
                if self.session.current().speaker_notes.is_some() {
                    self.show_notes = !self.show_notes;
                } else {
                    self.set_flash("This slide has no speaker notes", FlashKind::Info);
                }
            }
            KeyCode::Char('t') => self.show_timer = !self.show_timer,
            _ if at_branch => self.on_branch_key(code),
            _ => self.on_flow_key(code),
        }
    }

    /// Keys while the current node presents a choice.
    fn on_branch_key(&mut self, code: KeyCode) {
        let count = self
            .session
            .branch_point()
            .map(|bp| bp.options.len())
            .unwrap_or(0);
        match code {
            KeyCode::Up | KeyCode::Char('k') if count > 0 => {
                self.branch_selected = (self.branch_selected + count - 1) % count;
            }
            KeyCode::Down | KeyCode::Char('j') if count > 0 => {
                self.branch_selected = (self.branch_selected + 1) % count;
            }
            KeyCode::Enter => {
                let outcome = self.session.choose(self.branch_selected);
                self.apply(&outcome);
            }
            KeyCode::Char(c @ '1'..='9') => {
                let idx = (c as usize) - ('1' as usize);
                if idx < count {
                    let outcome = self.session.choose(idx);
                    self.apply(&outcome);
                } else {
                    self.set_flash(
                        &format!("There are only {count} choices"),
                        FlashKind::Error,
                    );
                }
            }
            KeyCode::Char(' ') | KeyCode::Right | KeyCode::PageDown | KeyCode::Char('n') => {
                let outcome = self.session.next();
                self.apply(&outcome);
            }
            KeyCode::Left | KeyCode::Backspace | KeyCode::PageUp | KeyCode::Char('p') => {
                let outcome = self.session.back();
                self.apply(&outcome);
            }
            KeyCode::Char(c) if c.is_alphanumeric() => match self.option_for_key(c) {
                Some(idx) => {
                    let outcome = self.session.choose(idx);
                    self.apply(&outcome);
                }
                None => self.set_flash(&format!("No choice on key '{c}'"), FlashKind::Error),
            },
            _ => {}
        }
    }

    /// Keys on an ordinary (non-branch) node.
    fn on_flow_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(' ' | 'n')
            | KeyCode::Right
            | KeyCode::Enter
            | KeyCode::PageDown => {
                let outcome = self.session.next();
                self.apply(&outcome);
            }
            KeyCode::Left | KeyCode::Backspace | KeyCode::PageUp | KeyCode::Char('p') => {
                let outcome = self.session.back();
                self.apply(&outcome);
            }
            KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::Down => self.scroll = (self.scroll + 1).min(self.max_scroll()),
            _ => {}
        }
    }

    /// Match a typed character against the options' author-declared keys
    /// (first character, case-insensitive).
    fn option_for_key(&self, c: char) -> Option<usize> {
        let bp = self.session.branch_point()?;
        bp.options.iter().position(|opt| {
            opt.key
                .as_deref()
                .and_then(|k| k.chars().next())
                .is_some_and(|k| k.eq_ignore_ascii_case(&c))
        })
    }

    /// Turn a traversal outcome into presenter feedback.
    fn apply(&mut self, outcome: &Outcome) {
        match outcome {
            Outcome::Moved => {
                self.scroll = 0;
                self.branch_selected = 0;
                self.flash = None;
                let fades = self
                    .session
                    .current()
                    .resolved_transition(self.session.defaults())
                    == Transition::Fade;
                self.fade_started = fades.then(Instant::now);
            }
            Outcome::BlockedByBranch => {
                self.set_flash("This slide asks for a choice — ↑↓ then Enter", FlashKind::Info);
            }
            Outcome::EndOfPath => {
                self.set_flash("End of this path — ← goes back", FlashKind::Info);
            }
            Outcome::HistoryEmpty => {
                self.set_flash("Already at the first slide", FlashKind::Info);
            }
            Outcome::InvalidChoice => {
                self.set_flash("That choice does not exist", FlashKind::Error);
            }
            Outcome::UnknownNode(id) => {
                self.set_flash(&format!("No slide is called \"{id}\""), FlashKind::Error);
            }
        }
    }

    fn set_flash(&mut self, text: &str, kind: FlashKind) {
        self.flash = Some(Flash {
            text: text.to_owned(),
            kind,
            expires: Instant::now() + FLASH_DURATION,
        });
    }

    /// The largest useful scroll offset for the current node at the current
    /// viewport, derived from the same line flow the renderer draws.
    fn max_scroll(&self) -> u16 {
        let (w, h) = self.viewport;
        render::max_scroll(self, w, h)
    }
}
