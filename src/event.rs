//! Event system — defines actions and maps terminal events to application actions.

/// Actions that can be performed in the application.
///
/// These represent the intent behind user input, decoupled from the
/// specific keys that trigger them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Advance to the next slide.
    NextSlide,
    /// Go back to the previous slide.
    PrevSlide,
    /// Jump to a specific slide by index (0-based).
    GoToSlide(usize),
    /// Toggle the help overlay.
    ToggleHelp,
    /// Quit the application.
    Quit,
    /// Enter go-to-slide mode (waiting for number input).
    EnterGotoMode,
    /// A digit was entered in go-to mode.
    GotoDigit(usize),
    /// Confirm the go-to slide number.
    GotoConfirm,
    /// Cancel go-to mode.
    GotoCancel,
    /// Terminal was resized.
    Resize(u16, u16),
    /// A tick event for animations and timers.
    Tick,
}
