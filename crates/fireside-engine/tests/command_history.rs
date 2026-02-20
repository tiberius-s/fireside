use std::path::Path;

use fireside_core::model::content::ContentBlock;
use fireside_engine::{Command, PresentationSession, load_graph};

fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn add_update_remove_undo_restores_original_graph_nodes() {
    let graph = load_graph(&fixture_path("valid_linear.json")).expect("fixture should load");
    let snapshot = graph.nodes.clone();

    let mut session = PresentationSession::new(graph, 0);

    session
        .execute_command(Command::AddNode {
            node_id: "n6".to_string(),
            after_index: Some(4),
        })
        .expect("add node should succeed");

    session
        .execute_command(Command::UpdateNodeContent {
            node_id: "n2".to_string(),
            content: vec![ContentBlock::Text {
                body: "updated node 2".to_string(),
            }],
        })
        .expect("update node should succeed");

    session
        .execute_command(Command::RemoveNode {
            node_id: "n3".to_string(),
        })
        .expect("remove node should succeed");

    assert!(session.undo().expect("undo remove should succeed"));
    assert!(session.undo().expect("undo update should succeed"));
    assert!(session.undo().expect("undo add should succeed"));

    assert_eq!(session.graph.nodes, snapshot);
}
