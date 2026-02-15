//! Application state machine and main update loop.
//!
//! Implements the TEA (Model-View-Update) pattern: the `App` struct holds
//! all state, `update()` processes actions, and `view()` renders the UI.

use std::time::Instant;

use crossterm::event::{Event, KeyEventKind};
use ratatui::Frame;

use crate::config::keybindings::map_key_to_action;
use crate::event::Action;
use crate::model::SlideDeck;
use crate::model::theme::Theme;
use crate::ui::presenter::render_presenter;

/// The current mode of the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    /// Normal presentation mode.
    Presenting,
    /// Waiting for slide number input.
    GotoSlide {
        /// The digits entered so far.
        buffer: String,
    },
    /// Application is quitting.
    Quitting,
}

/// Main application state.
pub struct App {
    /// The loaded slide deck.
    pub deck: SlideDeck,
    /// Index of the currently displayed slide (0-based).
    pub current_slide: usize,
    /// Current application mode.
    pub mode: AppMode,
    /// Whether the help overlay is visible.
    pub show_help: bool,
    /// The active theme.
    pub theme: Theme,
    /// When the presentation started (for elapsed time display).
    pub start_time: Instant,
    /// Navigation history stack for branch backtracking.
    pub slide_history: Vec<usize>,
}

impl App {
    /// Create a new application with the given slide deck and theme.
    #[must_use]
    pub fn new(deck: SlideDeck, theme: Theme, start_slide: usize) -> Self {
        let current = start_slide.min(deck.slides.len().saturating_sub(1));
        Self {
            deck,
            current_slide: current,
            mode: AppMode::Presenting,
            show_help: false,
            theme,
            start_time: Instant::now(),
            slide_history: Vec::new(),
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
            Action::NextSlide => self.next_slide(),
            Action::PrevSlide => self.prev_slide(),
            Action::GoToSlide(idx) => self.goto_slide(idx),
            Action::ToggleHelp => self.show_help = !self.show_help,
            Action::Quit => self.mode = AppMode::Quitting,
            Action::EnterGotoMode => {
                self.mode = AppMode::GotoSlide {
                    buffer: String::new(),
                };
            }
            Action::GotoDigit(digit) => {
                if let AppMode::GotoSlide { ref mut buffer } = self.mode {
                    buffer.push_str(&digit.to_string());
                }
            }
            Action::GotoConfirm => {
                if let AppMode::GotoSlide { ref buffer } = self.mode
                    && let Ok(num) = buffer.parse::<usize>()
                {
                    // User enters 1-based, we use 0-based
                    let idx = num.saturating_sub(1);
                    self.goto_slide(idx);
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
        render_presenter(
            frame,
            &self.deck,
            self.current_slide,
            &self.theme,
            self.show_help,
            elapsed,
        );
    }

    /// Handle a crossterm event and map it to an action.
    ///
    /// Returns `Ok(())` after processing. Poll timeout is handled by the caller.
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let in_goto = matches!(self.mode, AppMode::GotoSlide { .. });
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

    fn next_slide(&mut self) {
        if self.current_slide + 1 < self.deck.slides.len() {
            self.slide_history.push(self.current_slide);

            // Check for next_override from navigation
            let slide = &self.deck.slides[self.current_slide];
            if let Some(target_id) = slide.next_override()
                && let Some(idx) = self.deck.index_of(target_id)
            {
                self.current_slide = idx;
                return;
            }

            self.current_slide += 1;
        }
    }

    fn prev_slide(&mut self) {
        if let Some(prev) = self.slide_history.pop() {
            self.current_slide = prev;
        } else if self.current_slide > 0 {
            self.current_slide -= 1;
        }
    }

    fn goto_slide(&mut self, idx: usize) {
        if idx < self.deck.slides.len() {
            self.slide_history.push(self.current_slide);
            self.current_slide = idx;
        }
    }
}
