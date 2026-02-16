//! Event system — defines actions that the TUI app can perform.

/// Actions that can be performed in the application.
///
/// These represent the intent behind user input, decoupled from the
/// specific keys that trigger them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Advance to the next node.
    NextNode,
    /// Go back to the previous node.
    PrevNode,
    /// Jump to a specific node by index (0-based).
    GoToNode(usize),
    /// Choose a branch option by key character.
    ChooseBranch(char),
    /// Toggle the help overlay.
    ToggleHelp,
    /// Quit the application.
    Quit,
    /// Enter go-to-node mode (waiting for number input).
    EnterGotoMode,
    /// A digit was entered in go-to mode.
    GotoDigit(usize),
    /// Confirm the go-to node number.
    GotoConfirm,
    /// Cancel go-to mode.
    GotoCancel,
    /// Terminal was resized.
    Resize(u16, u16),
    /// A tick event for animations and timers.
    Tick,
}
