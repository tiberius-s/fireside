//! Editor interaction tests — block ops, dirty-exit/quit, picker, search,
//! inline notes, block selection, mouse hit-test, and metadata editing.

use super::*;

// ── Block operations ──────────────────────────────────────────────────────────

#[test]
fn editor_remove_block_deletes_selected_block() {
    let session = PresentationSession::new(graph_with_content_blocks(), 0);
    let mut app = App::new(session, Theme::default());
    app.enter_edit_mode();

    // Select second block (index 1) and delete it.
    app.editor_selected_block = 1;
    let initial_count = app.session.graph.nodes[0].content.len();
    app.update(Action::EditorRemoveBlock);

    assert_eq!(app.session.graph.nodes[0].content.len(), initial_count - 1);
    // Selection clamped to last remaining block.
    assert!(app.editor_selected_block < app.session.graph.nodes[0].content.len());
}

// ── Dirty-state exit / quit ──────────────────────────────────────────────────

#[test]
fn exit_edit_mode_dirty_cancel_stays_in_editor() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 1);
    let mut app = App::new(session, Theme::default());

    app.enter_edit_mode();
    app.session.mark_dirty();

    app.update(Action::ExitEditMode);
    assert!(app.pending_exit_action.is_some());

    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));

    assert_eq!(app.mode, AppMode::Editing);
    assert!(app.pending_exit_action.is_none());
    assert_eq!(app.editor_status.as_deref(), Some("Stayed in editor"));
}

#[test]
fn quit_dirty_confirm_quits_app() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
    let mut app = App::new(session, Theme::default());

    app.session.mark_dirty();
    app.update(Action::Quit);
    assert!(app.pending_exit_action.is_some());

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('y'),
        KeyModifiers::NONE,
    )));

    assert!(app.should_quit());
    assert!(app.pending_exit_action.is_none());
}

#[test]
fn exit_edit_mode_dirty_n_discards_and_exits_editor() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 1);
    let mut app = App::new(session, Theme::default());

    app.enter_edit_mode();
    app.session.mark_dirty();

    app.update(Action::ExitEditMode);
    assert!(app.pending_exit_action.is_some());

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('n'),
        KeyModifiers::NONE,
    )));

    assert_eq!(app.mode, AppMode::Presenting);
    assert!(app.pending_exit_action.is_none());
}

#[test]
fn quit_dirty_n_discards_and_quits_app() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
    let mut app = App::new(session, Theme::default());

    app.session.mark_dirty();
    app.update(Action::Quit);
    assert!(app.pending_exit_action.is_some());

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('n'),
        KeyModifiers::NONE,
    )));

    assert!(app.should_quit());
    assert!(app.pending_exit_action.is_none());
}

// ── Editor UI interactions ────────────────────────────────────────────────────

#[test]
fn compact_editor_n_toggles_node_list_visibility() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
    let mut app = App::new(session, Theme::default());

    app.enter_edit_mode();
    app.terminal_size = (80, 24);
    let before = app.editor_node_list_visible;

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('n'),
        KeyModifiers::NONE,
    )));

    assert_ne!(app.editor_node_list_visible, before);
}

#[test]
fn picker_escape_closes_overlay() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
    let mut app = App::new(session, Theme::default());

    app.enter_edit_mode();
    app.update(Action::EditorOpenLayoutPicker);
    assert!(app.editor_picker.is_some());

    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));

    assert!(app.editor_picker.is_none());
    assert_eq!(app.editor_status.as_deref(), Some("Picker cancelled"));
}

#[test]
fn editor_search_and_repeat_hits_next_match() {
    let session = PresentationSession::new(graph_with_ids(&["alpha", "beta", "alpha-two"]), 0);
    let mut app = App::new(session, Theme::default());

    app.enter_edit_mode();
    app.update(Action::EditorStartNodeSearch);
    for ch in ['a', 'l', 'p', 'h', 'a'] {
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char(ch),
            KeyModifiers::NONE,
        )));
    }
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert_eq!(app.editor_selected_node, 0);

    app.update(Action::EditorSearchNextHit);
    assert_eq!(app.editor_selected_node, 2);
}

#[test]
fn inline_notes_edit_ctrl_c_cancels_without_mutation() {
    let mut graph = graph_with_ids(&["a"]);
    graph.nodes[0].speaker_notes = Some("Original notes".to_string());

    let session = PresentationSession::new(graph, 0);
    let mut app = App::new(session, Theme::default());
    app.enter_edit_mode();

    app.update(Action::EditorStartNotesEdit);
    assert!(app.editor_text_input.is_some());

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL,
    )));

    assert!(app.editor_text_input.is_none());
    assert_eq!(
        app.session.graph.nodes[0].speaker_notes.as_deref(),
        Some("Original notes")
    );
}

// ── Block selection ───────────────────────────────────────────────────────────

#[test]
fn editor_b_and_shift_b_select_blocks() {
    let session = PresentationSession::new(graph_with_content_blocks(), 0);
    let mut app = App::new(session, Theme::default());
    app.enter_edit_mode();

    assert_eq!(app.editor_selected_block, 0);

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('b'),
        KeyModifiers::NONE,
    )));
    assert_eq!(app.editor_selected_block, 1);

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('B'),
        KeyModifiers::SHIFT,
    )));
    assert_eq!(app.editor_selected_block, 0);
}

// ── Mouse hit-test ────────────────────────────────────────────────────────────

#[test]
fn editor_mouse_click_selects_block_in_detail_pane() {
    let session = PresentationSession::new(graph_with_content_blocks(), 0);
    let mut app = App::new(session, Theme::default());
    app.enter_edit_mode();
    app.terminal_size = (120, 40);

    let root = ratatui::layout::Rect::new(0, 0, app.terminal_size.0, app.terminal_size.1);
    let sections = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Min(1),
            ratatui::layout::Constraint::Length(3),
        ])
        .split(root);
    let body = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(30),
            ratatui::layout::Constraint::Percentage(70),
        ])
        .split(sections[0]);
    let detail_area = body[1];

    let first_block_row = detail_area.y + 1 + 9;
    // WYSIWYG block heights for graph_with_content_blocks():
    //   Block 0 (Heading level 1): header(1) + text(1) + underline(1) + sep(1) = 4 rows
    //   Block 1 (Text):            header(1) + line(1)                 + sep(1) = 3 rows
    //   Block 2 (Code) starts at offset 4 + 3 = 7 from first_block_row
    let target_row = first_block_row + 7;
    let target_col = detail_area.x + 4;

    app.handle_event(Event::Mouse(crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: target_col,
        row: target_row,
        modifiers: KeyModifiers::NONE,
    }));

    assert_eq!(app.editor_selected_block, 2);
    assert_eq!(app.editor_focus, crate::app::EditorPaneFocus::NodeDetail);
    assert_eq!(app.editor_status.as_deref(), Some("Selected block #3"));
}

// ── Block metadata editing ────────────────────────────────────────────────────

#[test]
fn editor_m_starts_metadata_edit_for_selected_block() {
    let session = PresentationSession::new(graph_with_content_blocks(), 0);
    let mut app = App::new(session, Theme::default());
    app.enter_edit_mode();
    app.editor_selected_block = 2;

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('m'),
        KeyModifiers::NONE,
    )));

    assert_eq!(app.editor_text_input.as_deref(), Some("rust"));
    let status = app.editor_status.as_deref().unwrap_or_default();
    assert!(status.contains("Code language"));
}

#[test]
fn editor_m_enter_commits_heading_level_metadata() {
    let session = PresentationSession::new(graph_with_content_blocks(), 0);
    let mut app = App::new(session, Theme::default());
    app.enter_edit_mode();
    app.editor_selected_block = 0;

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('m'),
        KeyModifiers::NONE,
    )));

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Backspace,
        KeyModifiers::NONE,
    )));
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('4'),
        KeyModifiers::NONE,
    )));
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));

    let ContentBlock::Heading { level, .. } = &app.session.graph.nodes[0].content[0] else {
        panic!("expected heading block");
    };
    assert_eq!(*level, 4);
}

#[test]
fn editor_m_invalid_heading_level_sets_error_and_preserves_block() {
    let session = PresentationSession::new(graph_with_content_blocks(), 0);
    let mut app = App::new(session, Theme::default());
    app.enter_edit_mode();
    app.editor_selected_block = 0;

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('m'),
        KeyModifiers::NONE,
    )));

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Backspace,
        KeyModifiers::NONE,
    )));
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('9'),
        KeyModifiers::NONE,
    )));
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));

    assert_eq!(
        app.editor_status.as_deref(),
        Some("Heading level must be between 1 and 6")
    );
    let ContentBlock::Heading { level, .. } = &app.session.graph.nodes[0].content[0] else {
        panic!("expected heading block");
    };
    assert_eq!(*level, 1);
}
