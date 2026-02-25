use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use fireside_core::model::branch::{BranchOption, BranchPoint};
use fireside_core::model::content::ContentBlock;
use fireside_core::model::graph::{Graph, GraphFile};
use fireside_core::model::node::Node;
use fireside_core::model::traversal::Traversal;
use fireside_engine::PresentationSession;

use super::app_helpers::{update_block_from_inline_text, update_block_metadata_from_inline_text};
use super::{App, AppMode, Theme};
use crate::event::Action;

fn graph_with_ids(ids: &[&str]) -> Graph {
    let file = GraphFile {
        title: None,
        fireside_version: None,
        author: None,
        date: None,
        description: None,
        version: None,
        tags: Vec::new(),
        theme: None,
        font: None,
        defaults: None,
        extensions: Vec::new(),
        nodes: ids
            .iter()
            .map(|id| Node {
                id: Some((*id).to_string()),
                title: None,
                tags: Vec::new(),
                duration: None,
                layout: None,
                transition: None,
                speaker_notes: None,
                traversal: None,
                content: Vec::new(),
            })
            .collect(),
    };

    Graph::from_file(file).expect("graph should be valid")
}

fn branch_graph() -> Graph {
    let mut start = Node {
        id: Some("start".to_string()),
        title: None,
        tags: Vec::new(),
        duration: None,
        layout: None,
        transition: None,
        speaker_notes: None,
        traversal: None,
        content: Vec::new(),
    };
    start.traversal = Some(Traversal {
        next: None,
        after: None,
        branch_point: Some(BranchPoint {
            id: Some("branch-0".to_string()),
            prompt: Some("Choose path".to_string()),
            options: vec![
                BranchOption {
                    label: "Path A".to_string(),
                    key: '1',
                    target: "path-a".to_string(),
                },
                BranchOption {
                    label: "Path B".to_string(),
                    key: '2',
                    target: "path-b".to_string(),
                },
            ],
        }),
    });

    let path_a = Node {
        id: Some("path-a".to_string()),
        title: None,
        tags: Vec::new(),
        duration: None,
        layout: None,
        transition: None,
        speaker_notes: None,
        traversal: None,
        content: Vec::new(),
    };
    let path_b = Node {
        id: Some("path-b".to_string()),
        title: None,
        tags: Vec::new(),
        duration: None,
        layout: None,
        transition: None,
        speaker_notes: None,
        traversal: None,
        content: Vec::new(),
    };

    Graph::from_file(GraphFile {
        title: None,
        fireside_version: None,
        author: None,
        date: None,
        description: None,
        version: None,
        tags: Vec::new(),
        theme: None,
        font: None,
        defaults: None,
        extensions: Vec::new(),
        nodes: vec![start, path_a, path_b],
    })
    .expect("branch graph should be valid")
}

fn graph_with_content_blocks() -> Graph {
    let file = GraphFile {
        title: None,
        fireside_version: None,
        author: None,
        date: None,
        description: None,
        version: None,
        tags: Vec::new(),
        theme: None,
        font: None,
        defaults: None,
        extensions: Vec::new(),
        nodes: vec![Node {
            id: Some("node-1".to_string()),
            title: None,
            tags: Vec::new(),
            duration: None,
            layout: None,
            transition: None,
            speaker_notes: None,
            traversal: None,
            content: vec![
                ContentBlock::Heading {
                    level: 1,
                    text: "Title".to_string(),
                },
                ContentBlock::Text {
                    body: "Body paragraph".to_string(),
                },
                ContentBlock::Code {
                    language: Some("rust".to_string()),
                    source: "fn main() {}".to_string(),
                    highlight_lines: Vec::new(),
                    show_line_numbers: false,
                },
            ],
        }],
    };

    Graph::from_file(file).expect("graph with content blocks should be valid")
}

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
    let ids = (1..=24)
        .map(|index| format!("node-{index}"))
        .collect::<Vec<_>>();
    let id_refs = ids.iter().map(String::as_str).collect::<Vec<_>>();

    let session = PresentationSession::new(graph_with_ids(&id_refs), 0);
    let mut app = App::new(session, Theme::default());
    app.terminal_size = (100, 28);

    app.enter_edit_mode();
    app.update(Action::EditorToggleGraphView);
    let page = app.editor_graph_visible_rows().max(1);

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::PageDown,
        KeyModifiers::NONE,
    )));
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));

    assert_eq!(app.editor_selected_node, page.min(23));
}

#[test]
fn graph_overlay_home_end_navigation_hits_bounds() {
    let ids = (1..=12)
        .map(|index| format!("node-{index}"))
        .collect::<Vec<_>>();
    let id_refs = ids.iter().map(String::as_str).collect::<Vec<_>>();

    let session = PresentationSession::new(graph_with_ids(&id_refs), 5);
    let mut app = App::new(session, Theme::default());
    app.enter_edit_mode();
    app.update(Action::EditorToggleGraphView);

    app.handle_event(Event::Key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)));
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert_eq!(app.editor_selected_node, 11);

    app.update(Action::EditorToggleGraphView);
    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)));
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert_eq!(app.editor_selected_node, 0);
}

#[test]
fn graph_overlay_present_shortcut_switches_to_presenting_mode() {
    let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
    let mut app = App::new(session, Theme::default());

    app.enter_edit_mode();
    app.update(Action::EditorToggleGraphView);
    app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('p'),
        KeyModifiers::NONE,
    )));

    assert_eq!(app.mode, AppMode::Presenting);
    assert!(!app.editor_graph_overlay);
    assert_eq!(app.session.current_node_index(), 1);
    assert_eq!(app.editor_selected_node, 1);
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

#[test]
fn update_block_from_inline_text_keeps_heading_level() {
    let updated = update_block_from_inline_text(
        ContentBlock::Heading {
            level: 3,
            text: "old".to_string(),
        },
        "new".to_string(),
    );

    assert_eq!(
        updated,
        ContentBlock::Heading {
            level: 3,
            text: "new".to_string(),
        }
    );
}

#[test]
fn update_block_from_inline_text_list_inserts_first_item_when_empty() {
    let updated = update_block_from_inline_text(
        ContentBlock::List {
            ordered: false,
            items: vec![],
        },
        "first".to_string(),
    );

    let ContentBlock::List { items, .. } = updated else {
        panic!("expected list block");
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].text, "first");
}

#[test]
fn update_block_from_inline_text_container_blank_clears_layout() {
    let updated = update_block_from_inline_text(
        ContentBlock::Container {
            layout: Some("row".to_string()),
            children: vec![],
        },
        "   ".to_string(),
    );

    let ContentBlock::Container { layout, .. } = updated else {
        panic!("expected container block");
    };
    assert!(layout.is_none());
}

#[test]
fn update_block_metadata_from_inline_text_sets_code_language() {
    let updated = update_block_metadata_from_inline_text(
        ContentBlock::Code {
            language: Some("rust".to_string()),
            source: "fn main() {}".to_string(),
            highlight_lines: Vec::new(),
            show_line_numbers: false,
        },
        "python".to_string(),
    )
    .expect("code language should update");

    let ContentBlock::Code { language, .. } = updated else {
        panic!("expected code block");
    };
    assert_eq!(language.as_deref(), Some("python"));
}

#[test]
fn update_block_metadata_from_inline_text_rejects_invalid_heading_level() {
    let err = update_block_metadata_from_inline_text(
        ContentBlock::Heading {
            level: 2,
            text: "Title".to_string(),
        },
        "9".to_string(),
    )
    .expect_err("heading level must be rejected");

    assert_eq!(err, "Heading level must be between 1 and 6");
}

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
    let target_row = first_block_row + 2;
    let target_col = detail_area.x + 4;

    app.handle_event(Event::Mouse(crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: target_col,
        row: target_row,
        modifiers: KeyModifiers::NONE,
    }));

    assert_eq!(app.editor_selected_block, 2);
    assert_eq!(app.editor_focus, super::EditorPaneFocus::NodeDetail);
    assert_eq!(app.editor_status.as_deref(), Some("Selected block #3"));
}

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
