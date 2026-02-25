//! Presenter mode tests — reload, hot-reload, branch overlay, graph overlay.

use super::*;

#[test]
fn reload_graph_preserves_current_node_by_id() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 1);
    let mut app = App::new(session, Theme::default());

    app.reload_graph(graph_with_ids(&["x", "b", "y"]));

    assert_eq!(app.session.current_node_index(), 1);
    assert_eq!(app.session.current_node().id.as_deref(), Some("b"));
}

#[test]
fn reload_graph_falls_back_to_clamped_index_when_id_missing() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 2);
    let mut app = App::new(session, Theme::default());

    app.reload_graph(graph_with_ids(&["a", "b"]));

    assert_eq!(app.session.current_node_index(), 1);
    assert_eq!(app.session.current_node().id.as_deref(), Some("b"));
}

#[test]
fn hot_reload_is_only_enabled_in_presenter_mode() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b"]), 0);
    let mut app = App::new(session, Theme::default());

    assert!(app.can_hot_reload());
    app.enter_edit_mode();
    assert!(!app.can_hot_reload());
}

#[test]
fn presenter_branch_overlay_up_down_enter_chooses_focused_option() {
    let session = PresentationSession::new(branch_graph(), 0);
    let mut app = App::new(session, Theme::default());

    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)));
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));

    assert_eq!(app.session.current_node().id.as_deref(), Some("path-a"));
    assert_eq!(app.branch_focused_option, 0);
}

#[test]
fn graph_overlay_toggle_tracks_editor_selection() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 1);
    let mut app = App::new(session, Theme::default());

    app.enter_edit_mode();
    app.update(Action::EditorToggleGraphView);

    assert!(app.editor_graph_overlay);
    assert_eq!(app.editor_graph_selected_node, 1);

    app.update(Action::EditorToggleGraphView);

    assert!(!app.editor_graph_overlay);
}

#[test]
fn graph_overlay_keyboard_enter_jumps_to_selected_node() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
    let mut app = App::new(session, Theme::default());

    app.enter_edit_mode();
    app.update(Action::EditorToggleGraphView);
    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));

    assert!(!app.editor_graph_overlay);
    assert_eq!(app.editor_selected_node, 1);
    assert_eq!(app.session.current_node_index(), 1);
}

#[test]
fn graph_overlay_paged_navigation_advances_selection() {
    let ids = (1..=24).map(|i: u8| format!("n{i}")).collect::<Vec<_>>();
    let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    let session = PresentationSession::new(graph_with_ids(&id_refs), 0);
    let mut app = App::new(session, Theme::default());

    app.enter_edit_mode();
    app.update(Action::EditorToggleGraphView);

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::PageDown,
        KeyModifiers::NONE,
    )));
    let after_page = app.editor_graph_selected_node;

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::PageUp,
        KeyModifiers::NONE,
    )));
    let after_page_back = app.editor_graph_selected_node;

    assert!(after_page > 0, "PageDown should advance");
    assert!(after_page_back < after_page, "PageUp should retreat");
}

#[test]
fn graph_overlay_home_end_navigation_hits_bounds() {
    let ids = (1..=10).map(|i: u8| format!("n{i}")).collect::<Vec<_>>();
    let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    let session = PresentationSession::new(graph_with_ids(&id_refs), 0);
    let mut app = App::new(session, Theme::default());

    app.enter_edit_mode();
    app.update(Action::EditorToggleGraphView);

    app.handle_event(Event::Key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)));
    let at_end = app.editor_graph_selected_node;

    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)));
    let at_start = app.editor_graph_selected_node;

    assert_eq!(at_end, ids.len() - 1, "End should jump to last node");
    assert_eq!(at_start, 0, "Home should jump to first node");
}

#[test]
fn graph_overlay_present_shortcut_switches_to_presenting_mode() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
    let mut app = App::new(session, Theme::default());

    app.enter_edit_mode();
    app.update(Action::EditorToggleGraphView);
    assert!(app.editor_graph_overlay);

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('p'),
        KeyModifiers::NONE,
    )));

    assert_eq!(app.mode, AppMode::Presenting);
    assert!(!app.editor_graph_overlay);
}

#[test]
fn presenter_enter_edit_mode_sets_breadcrumb_status() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 2);
    let mut app = App::new(session, Theme::default());

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('e'),
        KeyModifiers::NONE,
    )));

    assert_eq!(app.mode, AppMode::Editing);
    assert_eq!(app.editor_selected_node, 2);
    let status = app.editor_status.as_deref().unwrap_or_default();
    assert!(status.contains("Presenter"));
    assert!(status.contains("node #3"));
}

#[test]
fn editor_node_select_action_resets_block_selection() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
    let mut app = App::new(session, Theme::default());

    app.enter_edit_mode();
    app.editor_selected_block = 1;
    app.update(Action::EditorSelectNextNode);

    assert_eq!(app.editor_selected_node, 1);
    assert_eq!(app.editor_selected_block, 0);
    assert_eq!(app.session.current_node_index(), 1);
}
