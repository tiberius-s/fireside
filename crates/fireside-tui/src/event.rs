//! Event system — defines actions that the TUI app can perform.

/// Scroll direction for mouse wheel input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseScrollDirection {
    /// Scroll up.
    Up,
    /// Scroll down.
    Down,
}

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
    /// Toggle speaker notes panel.
    ToggleSpeakerNotes,
    /// Enter editing mode.
    EnterEditMode,
    /// Exit editing mode and return to presenting mode.
    ExitEditMode,
    /// Append a quick text block in editor mode.
    EditorAppendTextBlock,
    /// Add a new node after the selected editor node.
    EditorAddNode,
    /// Remove the selected editor node.
    EditorRemoveNode,
    /// Select next node in editor node list.
    EditorSelectNextNode,
    /// Select previous node in editor node list.
    EditorSelectPrevNode,
    /// Move selection down by one viewport page in editor node list.
    EditorPageDown,
    /// Move selection up by one viewport page in editor node list.
    EditorPageUp,
    /// Jump selection to first node in editor node list.
    EditorJumpTop,
    /// Jump selection to last node in editor node list.
    EditorJumpBottom,
    /// Start node-id search in editor mode.
    EditorStartNodeSearch,
    /// Jump to previous node-id search hit in editor mode.
    EditorSearchPrevHit,
    /// Jump to next node-id search hit in editor mode.
    EditorSearchNextHit,
    /// Start numeric node-index jump in editor mode.
    EditorStartIndexJump,
    /// Toggle editor focus between panes.
    EditorToggleFocus,
    /// Start inline editing of selected node text content.
    EditorStartInlineEdit,
    /// Start inline editing of selected node speaker notes.
    EditorStartNotesEdit,
    /// Cycle selected node layout to next variant.
    EditorCycleLayoutNext,
    /// Cycle selected node layout to previous variant.
    EditorCycleLayoutPrev,
    /// Open layout picker overlay.
    EditorOpenLayoutPicker,
    /// Cycle selected node transition to next variant.
    EditorCycleTransitionNext,
    /// Cycle selected node transition to previous variant.
    EditorCycleTransitionPrev,
    /// Open transition picker overlay.
    EditorOpenTransitionPicker,
    /// Save current graph to its editor target path.
    EditorSaveGraph,
    /// Undo the last editor command.
    EditorUndo,
    /// Redo the last undone editor command.
    EditorRedo,
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
    /// Mouse click at a terminal cell.
    MouseClick { column: u16, row: u16 },
    /// Mouse drag at a terminal cell.
    MouseDrag { column: u16, row: u16 },
    /// Mouse scroll input.
    MouseScroll(MouseScrollDirection),
    /// A tick event for animations and timers.
    Tick,
}
