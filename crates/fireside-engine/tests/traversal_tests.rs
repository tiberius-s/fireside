use fireside_engine::{PresentationSession, load_graph_from_str};

#[test]
fn next_reaches_branch_node_without_skip_or_loop() {
    let graph_json = r#"{
        "nodes": [
            { "id": "a", "content": [{ "kind": "text", "body": "A" }] },
            { "id": "b", "content": [{ "kind": "text", "body": "B" }] },
            {
                "id": "c",
                "content": [{ "kind": "text", "body": "C" }],
                "traversal": {
                    "branch-point": {
                        "options": [
                            { "label": "Take D", "key": "d", "target": "d" },
                            { "label": "Take E", "key": "e", "target": "e" }
                        ]
                    }
                }
            },
            { "id": "d", "content": [{ "kind": "text", "body": "D" }] },
            { "id": "e", "content": [{ "kind": "text", "body": "E" }] }
        ]
    }"#;

    let graph = load_graph_from_str(graph_json).expect("graph fixture should parse");
    let mut session = PresentationSession::new(graph, 0);

    assert_eq!(session.current_node().id.as_deref(), Some("a"));

    let _ = session.traversal.next(&session.graph);
    assert_eq!(session.current_node().id.as_deref(), Some("b"));

    let _ = session.traversal.next(&session.graph);
    assert_eq!(session.current_node().id.as_deref(), Some("c"));

    let _ = session.traversal.next(&session.graph);
    assert_eq!(session.current_node().id.as_deref(), Some("d"));

    let _ = session.traversal.next(&session.graph);
    assert_eq!(session.current_node().id.as_deref(), Some("e"));

    let _ = session.traversal.next(&session.graph);
    assert_eq!(session.current_node().id.as_deref(), Some("e"));
}
