//! Application state machine and main update loop.
//!
//! Implements the TEA (Model-View-Update) pattern: the `App` struct holds
//! all state, `update()` processes actions, and `view()` renders the UI.

use std::time::Instant;

use crossterm::event::{Event, KeyEventKind};
use ratatui::Frame;

use fireside_engine::PresentationSession;

use crate::config::keybindings::map_key_to_action;
use crate::event::Action;
use crate::theme::Theme;
use crate::ui::presenter::render_presenter;

/// The current mode of the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    /// Normal presentation mode.
    Presenting,
    /// Waiting for node number input.
    GotoNode {
        /// The digits entered so far.
        buffer: String,
    },
    /// Application is quitting.
    Quitting,
}

/// Main application state.
pub struct App {
    /// The presentation session (graph + traversal state).
    pub session: PresentationSession,
    /// Current application mode.
    pub mode: AppMode,
    /// Whether the help overlay is visible.
    pub show_help: bool,
    /// The active theme.
    pub theme: Theme,
    /// When the presentation started (for elapsed time display).
    pub start_time: Instant,
}

impl App {
    /// Create a new application with the given session and theme.
    #[must_use]
    pub fn new(session: PresentationSession, theme: Theme) -> Self {
        Self {
            session,
            mode: AppMode::Presenting,
            show_help: false,
            theme,
            start_time: Instant::now(),
        }
    }

    /// Returns `true` if the application should quit.
    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.mode == AppMode::Quitting
    }

    /// Process a single action, updating application state.
    pub fn update(&mut self, action: Action) {
        match action {
            Action::NextNode => {
                self.session.traversal.next(&self.session.graph);
            }
            Action::PrevNode => {
                self.session.traversal.back();
            }
            Action::GoToNode(idx) => {
                let _ = self.session.traversal.goto(idx, &self.session.graph);
            }
            Action::ChooseBranch(key) => {
                let _ = self.session.traversal.choose(key, &self.session.graph);
            }
            Action::ToggleHelp => self.show_help = !self.show_help,
            Action::Quit => self.mode = AppMode::Quitting,
            Action::EnterGotoMode => {
                self.mode = AppMode::GotoNode {
                    buffer: String::new(),
                };
            }
            Action::GotoDigit(digit) => {
                if let AppMode::GotoNode { ref mut buffer } = self.mode {
                    buffer.push_str(&digit.to_string());
                }
            }
            Action::GotoConfirm => {
                if let AppMode::GotoNode { ref buffer } = self.mode
                    && let Ok(num) = buffer.parse::<usize>()
                {
                    // User enters 1-based, we use 0-based
                    let idx = num.saturating_sub(1);
                    let _ = self.session.traversal.goto(idx, &self.session.graph);
                }
                self.mode = AppMode::Presenting;
            }
            Action::GotoCancel => {
                self.mode = AppMode::Presenting;
            }
            Action::Resize(_, _) => {
                // Terminal resize is handled by ratatui automatically
            }
            Action::Tick => {
                // Used for animations and timer updates
            }
        }
    }

    /// Render the current application state to the terminal frame.
    pub fn view(&self, frame: &mut Frame) {
        let elapsed = self.start_time.elapsed().as_secs();
        render_presenter(frame, &self.session, &self.theme, self.show_help, elapsed);
    }

    /// Handle a crossterm event and map it to an action.
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let in_goto = matches!(self.mode, AppMode::GotoNode { .. });
                if let Some(action) = map_key_to_action(key, in_goto) {
                    self.update(action);
                }
            }
            Event::Resize(w, h) => {
                self.update(Action::Resize(w, h));
            }
            _ => {}
        }
    }
}
