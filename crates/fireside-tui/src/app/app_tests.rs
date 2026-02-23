use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use fireside_core::model::branch::{BranchOption, BranchPoint};
use fireside_core::model::content::ContentBlock;
use fireside_core::model::graph::{Graph, GraphFile};
use fireside_core::model::node::Node;
use fireside_core::model::traversal::Traversal;
use fireside_engine::PresentationSession;

use super::app_helpers::update_block_from_inline_text;
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
