//! Keybinding definitions.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::AppMode;
use crate::event::Action;

/// Map a key event to an action based on the current app mode.
///
/// Returns `None` for unbound keys.
#[must_use]
pub fn map_key_to_action(key: KeyEvent, mode: &AppMode) -> Option<Action> {
    match mode {
        AppMode::GotoNode { .. } => return map_goto_mode_key(key),
        AppMode::Editing => return map_edit_mode_key(key),
        AppMode::Presenting | AppMode::Quitting => {}
    }

    match key.code {
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

fn map_edit_mode_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Action::EditorSelectNextNode),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::EditorSelectPrevNode),
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
        KeyCode::Char('o') => Some(Action::EditorStartNotesEdit),
        KeyCode::Char('l') => Some(Action::EditorOpenLayoutPicker),
        KeyCode::Char('L') => Some(Action::EditorCycleLayoutPrev),
        KeyCode::Char('t') => Some(Action::EditorOpenTransitionPicker),
        KeyCode::Char('T') => Some(Action::EditorCycleTransitionPrev),
        KeyCode::Char('a') => Some(Action::EditorAppendTextBlock),
        KeyCode::Char('n') => Some(Action::EditorAddNode),
        KeyCode::Char('d') => Some(Action::EditorRemoveNode),
        KeyCode::Char('v') => Some(Action::EditorToggleGraphView),
        KeyCode::Char('w') => Some(Action::EditorSaveGraph),
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::EditorSaveGraph)
        }
        KeyCode::Char('u') => Some(Action::EditorUndo),
        KeyCode::Char('r') => Some(Action::EditorRedo),
        KeyCode::Esc => Some(Action::ExitEditMode),
        KeyCode::Char('?') => Some(Action::ToggleHelp),
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
        _ => None,
    }
}

/// Map keys in go-to-node mode.
fn map_goto_mode_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let digit = c.to_digit(10).expect("verified ascii digit") as usize;
            Some(Action::GotoDigit(digit))
        }
        KeyCode::Enter => Some(Action::GotoConfirm),
        KeyCode::Esc => Some(Action::GotoCancel),
        _ => None,
    }
}
