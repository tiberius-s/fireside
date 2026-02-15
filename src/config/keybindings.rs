//! Keybinding definitions.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::event::Action;

/// Map a key event to an action based on the current app mode.
///
/// Returns `None` for unbound keys.
#[must_use]
pub fn map_key_to_action(key: KeyEvent, in_goto_mode: bool) -> Option<Action> {
    if in_goto_mode {
        return map_goto_mode_key(key);
    }

    match key.code {
        // Navigation
        KeyCode::Right | KeyCode::Char(' ') | KeyCode::Enter => Some(Action::NextSlide),
        KeyCode::Char('l') => Some(Action::NextSlide),
        KeyCode::Left => Some(Action::PrevSlide),
        KeyCode::Char('h') => Some(Action::PrevSlide),

        // Go to slide
        KeyCode::Char('g') => Some(Action::EnterGotoMode),

        // Help
        KeyCode::Char('?') => Some(Action::ToggleHelp),

        // Quit
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Esc => Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Quit),

        _ => None,
    }
}

/// Map keys in go-to-slide mode.
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
