//! Keybinding definitions.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{AppMode, EditorPaneFocus};
use crate::event::Action;

/// Map a key event to an action based on the current app mode.
///
/// `editor_focus` is used to route keys differently when the detail pane
/// has focus (e.g. `j/k` scroll the preview instead of selecting nodes).
///
/// Returns `None` for unbound keys.
#[must_use]
pub fn map_key_to_action(
    key: KeyEvent,
    mode: &AppMode,
    editor_focus: EditorPaneFocus,
) -> Option<Action> {
    match mode {
        AppMode::GotoNode { .. } => return map_goto_mode_key(key),
        AppMode::Editing => return map_edit_mode_key(key, editor_focus),
        AppMode::Presenting | AppMode::Quitting => {}
    }

    match key.code {
        // Timeline + breadcrumb navigation
        KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::ToggleTimeline)
        }
        KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::JumpToBranchPoint)
        }

        // Navigation
        KeyCode::Right | KeyCode::Char(' ') | KeyCode::Enter => Some(Action::NextNode),
        KeyCode::Char('l') => Some(Action::NextNode),
        KeyCode::Left => Some(Action::PrevNode),
        KeyCode::Char('h') => Some(Action::PrevNode),

        // Go to node
        KeyCode::Char('g') => Some(Action::EnterGotoMode),

        // Help
        KeyCode::Char('?') => Some(Action::ToggleHelp),

        // Speaker notes
        KeyCode::Char('s') => Some(Action::ToggleSpeakerNotes),

        // Zen mode
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::ToggleZenMode)
        }

        // Mode
        KeyCode::Char('e') => Some(Action::EnterEditMode),

        // Quit (must come before branch selection range)
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Esc => Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Quit),

        // Branch selection (a-f keys when at a branch point)
        KeyCode::Char(c @ 'a'..='f') => Some(Action::ChooseBranch(c)),

        _ => None,
    }
}

fn map_edit_mode_key(key: KeyEvent, focus: EditorPaneFocus) -> Option<Action> {
    if key.modifiers.contains(KeyModifiers::ALT) {
        return match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::EditorMoveBlockUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::EditorMoveBlockDown),
            _ => None,
        };
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::EditorSelectPrevBlock),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::EditorSelectNextBlock),
            KeyCode::Char('s') => Some(Action::EditorSaveGraph),
            KeyCode::Char('c') => Some(Action::Quit),
            _ => None,
        };
    }

    // When the detail (WYSIWYG preview) pane is focused, j/k and ↑/↓ scroll
    // the preview content rather than selecting nodes in the list.
    if focus == EditorPaneFocus::NodeDetail {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => return Some(Action::EditorDetailScrollDown),
            KeyCode::Char('k') | KeyCode::Up => return Some(Action::EditorDetailScrollUp),
            _ => {}
        }
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Action::EditorSelectNextNode),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::EditorSelectPrevNode),
        KeyCode::Char(',') => Some(Action::EditorSelectPrevBlock),
        KeyCode::Char('.') => Some(Action::EditorSelectNextBlock),
        KeyCode::Char('b') => Some(Action::EditorSelectNextBlock),
        KeyCode::Char('B') => Some(Action::EditorSelectPrevBlock),
        KeyCode::Char('J') | KeyCode::Char('<') => Some(Action::EditorMoveBlockUp),
        KeyCode::Char('K') | KeyCode::Char('>') => Some(Action::EditorMoveBlockDown),
        KeyCode::PageDown => Some(Action::EditorPageDown),
        KeyCode::PageUp => Some(Action::EditorPageUp),
        KeyCode::Home => Some(Action::EditorJumpTop),
        KeyCode::End => Some(Action::EditorJumpBottom),
        KeyCode::Char('/') => Some(Action::EditorStartNodeSearch),
        KeyCode::Char('[') => Some(Action::EditorSearchPrevHit),
        KeyCode::Char(']') => Some(Action::EditorSearchNextHit),
        KeyCode::Char('g') => Some(Action::EditorStartIndexJump),
        KeyCode::Tab => Some(Action::EditorToggleFocus),
        KeyCode::Char('i') => Some(Action::EditorStartInlineEdit),
        KeyCode::Char('m') => Some(Action::EditorStartInlineMetaEdit),
        KeyCode::Char('o') => Some(Action::EditorStartNotesEdit),
        KeyCode::Char('l') => Some(Action::EditorOpenLayoutPicker),
        KeyCode::Char('L') => Some(Action::EditorCycleLayoutPrev),
        KeyCode::Char('t') => Some(Action::EditorOpenTransitionPicker),
        KeyCode::Char('T') => Some(Action::EditorCycleTransitionPrev),
        KeyCode::Char('a') => Some(Action::EditorAppendTextBlock),
        KeyCode::Char('x') => Some(Action::EditorRemoveBlock),
        KeyCode::Char('n') => Some(Action::EditorAddNode),
        KeyCode::Char('d') => Some(Action::EditorRemoveNode),
        KeyCode::Char('v') => Some(Action::EditorToggleGraphView),
        KeyCode::Char('w') => Some(Action::EditorSaveGraph),
        KeyCode::Char('u') => Some(Action::EditorUndo),
        KeyCode::Char('r') => Some(Action::EditorRedo),
        KeyCode::Esc => Some(Action::ExitEditMode),
        KeyCode::Char('?') => Some(Action::ToggleHelp),
        KeyCode::Char('q') => Some(Action::Quit),
        _ => None,
    }
}

/// Map keys in go-to-node mode.
///
/// Accepts digits for numeric index jumps AND word characters / hyphens for
/// ID-prefix searches. The `GotoConfirm` handler resolves the buffer as a
/// 1-based numeric index first, then falls back to node-ID prefix matching.
fn map_goto_mode_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let digit = usize::from((c as u8) - b'0');
            Some(Action::GotoDigit(digit))
        }
        KeyCode::Char(c) if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' => {
            Some(Action::GotoChar(c))
        }
        KeyCode::Backspace => Some(Action::GotoBackspace),
        KeyCode::Enter => Some(Action::GotoConfirm),
        KeyCode::Esc => Some(Action::GotoCancel),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::map_key_to_action;
    use crate::app::{AppMode, EditorPaneFocus};
    use crate::event::Action;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn presenting_ctrl_h_toggles_timeline() {
        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL);
        let action = map_key_to_action(key, &AppMode::Presenting, EditorPaneFocus::NodeList);
        assert_eq!(action, Some(Action::ToggleTimeline));
    }

    #[test]
    fn presenting_ctrl_left_jumps_to_branch_point() {
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL);
        let action = map_key_to_action(key, &AppMode::Presenting, EditorPaneFocus::NodeList);
        assert_eq!(action, Some(Action::JumpToBranchPoint));
    }

    #[test]
    fn editing_b_selects_next_block() {
        let key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        let action = map_key_to_action(key, &AppMode::Editing, EditorPaneFocus::NodeList);
        assert_eq!(action, Some(Action::EditorSelectNextBlock));
    }

    #[test]
    fn editing_shift_b_selects_prev_block() {
        let key = KeyEvent::new(KeyCode::Char('B'), KeyModifiers::SHIFT);
        let action = map_key_to_action(key, &AppMode::Editing, EditorPaneFocus::NodeList);
        assert_eq!(action, Some(Action::EditorSelectPrevBlock));
    }

    #[test]
    fn editing_comma_and_period_select_blocks() {
        let prev_key = KeyEvent::new(KeyCode::Char(','), KeyModifiers::NONE);
        let next_key = KeyEvent::new(KeyCode::Char('.'), KeyModifiers::NONE);

        assert_eq!(
            map_key_to_action(prev_key, &AppMode::Editing, EditorPaneFocus::NodeList),
            Some(Action::EditorSelectPrevBlock)
        );
        assert_eq!(
            map_key_to_action(next_key, &AppMode::Editing, EditorPaneFocus::NodeList),
            Some(Action::EditorSelectNextBlock)
        );
    }

    #[test]
    fn editing_shift_jk_move_blocks() {
        let up_key = KeyEvent::new(KeyCode::Char('J'), KeyModifiers::SHIFT);
        let down_key = KeyEvent::new(KeyCode::Char('K'), KeyModifiers::SHIFT);

        assert_eq!(
            map_key_to_action(up_key, &AppMode::Editing, EditorPaneFocus::NodeList),
            Some(Action::EditorMoveBlockUp)
        );
        assert_eq!(
            map_key_to_action(down_key, &AppMode::Editing, EditorPaneFocus::NodeList),
            Some(Action::EditorMoveBlockDown)
        );
    }

    #[test]
    fn editing_m_starts_metadata_edit() {
        let key = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE);
        let action = map_key_to_action(key, &AppMode::Editing, EditorPaneFocus::NodeList);
        assert_eq!(action, Some(Action::EditorStartInlineMetaEdit));
    }

    #[test]
    fn detail_focus_jk_scrolls_preview() {
        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);

        // With NodeDetail focus, j/k should scroll the preview, not navigate nodes.
        assert_eq!(
            map_key_to_action(j_key, &AppMode::Editing, EditorPaneFocus::NodeDetail),
            Some(Action::EditorDetailScrollDown)
        );
        assert_eq!(
            map_key_to_action(k_key, &AppMode::Editing, EditorPaneFocus::NodeDetail),
            Some(Action::EditorDetailScrollUp)
        );
    }

    #[test]
    fn list_focus_jk_selects_nodes() {
        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);

        // With NodeList focus, j/k navigate nodes as before.
        assert_eq!(
            map_key_to_action(j_key, &AppMode::Editing, EditorPaneFocus::NodeList),
            Some(Action::EditorSelectNextNode)
        );
        assert_eq!(
            map_key_to_action(k_key, &AppMode::Editing, EditorPaneFocus::NodeList),
            Some(Action::EditorSelectPrevNode)
        );
    }
}
