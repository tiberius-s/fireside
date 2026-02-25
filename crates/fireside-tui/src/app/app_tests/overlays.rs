//! Overlay tests — help overlay and goto mode.

use super::*;

// ── Help overlay ─────────────────────────────────────────────────────────────

#[test]
fn help_overlay_scroll_keys_adjust_offset() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
    let mut app = App::new(session, Theme::default());
    app.terminal_size = (80, 24);

    app.update(Action::ToggleHelp);
    let start = app.help_scroll_offset;

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::PageDown,
        KeyModifiers::NONE,
    )));
    assert!(app.help_scroll_offset >= start);

    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
    assert!(app.help_scroll_offset >= start);

    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)));
    assert_eq!(app.help_scroll_offset, 0);
}

#[test]
fn help_overlay_section_jump_key_moves_scroll() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
    let mut app = App::new(session, Theme::default());
    app.terminal_size = (80, 24);

    app.update(Action::ToggleHelp);
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('4'),
        KeyModifiers::NONE,
    )));

    assert!(app.help_scroll_offset > 0);
}

#[test]
fn help_overlay_consumes_navigation_keys() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
    let mut app = App::new(session, Theme::default());

    app.update(Action::ToggleHelp);
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Right,
        KeyModifiers::NONE,
    )));

    assert!(app.show_help);
    assert_eq!(app.session.current_node_index(), 0);
}

// ── Goto mode ─────────────────────────────────────────────────────────────────

#[test]
fn goto_mode_confirm_uses_one_based_index() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
    let mut app = App::new(session, Theme::default());

    app.update(Action::EnterGotoMode);
    app.update(Action::GotoDigit(2));
    app.update(Action::GotoConfirm);

    assert_eq!(app.mode, AppMode::Presenting);
    assert_eq!(app.session.current_node_index(), 1);
}

#[test]
fn goto_mode_cancel_preserves_current_index() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 2);
    let mut app = App::new(session, Theme::default());

    app.update(Action::EnterGotoMode);
    app.update(Action::GotoDigit(1));
    app.update(Action::GotoCancel);

    assert_eq!(app.mode, AppMode::Presenting);
    assert_eq!(app.session.current_node_index(), 2);
}

#[test]
fn goto_mode_text_id_prefix_jumps_to_matching_node() {
    let session = PresentationSession::new(graph_with_ids(&["alpha", "beta", "gamma"]), 0);
    let mut app = App::new(session, Theme::default());

    app.update(Action::EnterGotoMode);
    app.update(Action::GotoChar('b'));
    app.update(Action::GotoChar('e'));
    app.update(Action::GotoConfirm);

    assert_eq!(app.mode, AppMode::Presenting);
    assert_eq!(app.session.current_node_index(), 1);
}

#[test]
fn goto_mode_backspace_removes_last_char() {
    let session = PresentationSession::new(graph_with_ids(&["alpha", "beta"]), 0);
    let mut app = App::new(session, Theme::default());

    app.update(Action::EnterGotoMode);
    // Type "bx", then backspace → "b" → confirm jumps to "beta"
    app.update(Action::GotoChar('b'));
    app.update(Action::GotoChar('x'));
    app.update(Action::GotoBackspace);
    app.update(Action::GotoConfirm);

    assert_eq!(app.mode, AppMode::Presenting);
    assert_eq!(app.session.current_node_index(), 1);
}
