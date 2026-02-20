use fireside_engine::{PresentationSession, load_graph};
use fireside_tui::render::blocks::render_node_content_with_base;
use fireside_tui::{Action, App, Theme};

fn hello_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/examples/hello.json")
}

#[test]
fn hello_example_loads_and_branches() {
    let graph = load_graph(&hello_path()).expect("hello example should load");
    let branch_index = graph
        .index_of("branch-demo")
        .expect("branch-demo node should exist");
    let themes_index = graph.index_of("themes").expect("themes node should exist");
    let blocks_index = graph.index_of("blocks").expect("blocks node should exist");

    let session_a = PresentationSession::new(graph.clone(), 0);
    let mut app_a = App::new(session_a, Theme::default());

    app_a.update(Action::GoToNode(branch_index));
    app_a.update(Action::ChooseBranch('a'));

    assert_eq!(app_a.session.current_node_index(), themes_index);

    let session_b = PresentationSession::new(graph, 0);
    let mut app_b = App::new(session_b, Theme::default());

    app_b.update(Action::GoToNode(branch_index));
    app_b.update(Action::ChooseBranch('b'));

    assert_eq!(app_b.session.current_node_index(), blocks_index);
}

#[test]
fn hello_example_transition_animation_ticks_to_completion() {
    let graph = load_graph(&hello_path()).expect("hello example should load");
    let session = PresentationSession::new(graph, 0);
    let mut app = App::new(session, Theme::default());

    for node_id in [
        "code-demo",
        "image-success",
        "image-fallback",
        "container-splits",
        "extension-known",
        "extension-unknown",
        "branch-demo",
        "themes",
        "blocks",
        "thanks",
    ] {
        let node_index = app
            .session
            .graph
            .index_of(node_id)
            .expect("expected transition node to exist");

        app.update(Action::GoToNode(node_index));
        assert!(
            app.is_animating(),
            "transition should start after navigation to {node_id}"
        );

        for _ in 0..16 {
            app.update(Action::Tick);
        }

        assert!(
            !app.is_animating(),
            "transition should complete after ticks for {node_id}"
        );
    }
}

#[test]
fn hello_example_image_success_and_fallback_render() {
    let graph = load_graph(&hello_path()).expect("hello example should load");
    let image_ok_index = graph
        .index_of("image-success")
        .expect("image-success should exist");
    let image_missing_index = graph
        .index_of("image-fallback")
        .expect("image-fallback should exist");

    let base_dir = hello_path()
        .parent()
        .expect("hello example should have parent")
        .to_path_buf();

    let ok_lines = render_node_content_with_base(
        &graph.nodes[image_ok_index].content,
        &Theme::default(),
        80,
        Some(&base_dir),
    );
    let missing_lines = render_node_content_with_base(
        &graph.nodes[image_missing_index].content,
        &Theme::default(),
        80,
        Some(&base_dir),
    );

    let ok_text = ok_lines
        .iter()
        .flat_map(|line| line.spans.iter().map(|span| span.content.as_ref()))
        .collect::<Vec<_>>()
        .join("\n");
    let missing_text = missing_lines
        .iter()
        .flat_map(|line| line.spans.iter().map(|span| span.content.as_ref()))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        ok_text.contains("size: 8×8"),
        "expected detected image dimensions"
    );
    assert!(
        missing_text.contains("fallback:"),
        "expected graceful fallback output for missing image"
    );
}

#[test]
fn hello_example_each_node_renders_non_empty_content_lines() {
    let graph = load_graph(&hello_path()).expect("hello example should load");
    let base_dir = hello_path()
        .parent()
        .expect("hello example should have parent")
        .to_path_buf();

    for node in &graph.nodes {
        let lines =
            render_node_content_with_base(&node.content, &Theme::default(), 80, Some(&base_dir));
        assert!(
            !lines.is_empty(),
            "expected non-empty rendered output for node {:?}",
            node.id
        );
    }
}
