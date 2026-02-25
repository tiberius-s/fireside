//! Graph mutation commands for editor support.
//!
//! Commands represent atomic operations on a graph that can be applied,
//! undone, and redone.  The module is split into three focused submodules:
//!
//! - [`types`] — the [`Command`] enum (all variants with their fields)
//! - [`apply`] — the pure `apply_command` function (private implementation)
//! - [`history`] — [`CommandHistory`], the undo/redo stack

mod apply;
mod history;
mod types;

pub use history::CommandHistory;
pub use types::Command;

#[cfg(test)]
mod tests {
    use crate::loader::load_graph_from_str;

    use super::*;

    fn graph_with_ids() -> fireside_core::model::graph::Graph {
        load_graph_from_str(
            r#"{
            "nodes": [
              { "id": "n1", "content": [{"kind":"text","body":"one"}] },
              { "id": "n2", "content": [{"kind":"text","body":"two"}] }
            ]
          }"#,
        )
        .expect("graph should parse")
    }

    #[test]
    fn update_content_roundtrips_with_undo_redo() {
        let mut graph = graph_with_ids();
        let mut history = CommandHistory::new();

        history
            .apply_command(
                &mut graph,
                Command::UpdateNodeContent {
                    node_id: "n1".to_string(),
                    content: vec![fireside_core::model::content::ContentBlock::Text {
                        body: "updated".to_string(),
                    }],
                },
            )
            .expect("update should succeed");

        let current = &graph.nodes[0].content;
        assert_eq!(
            current,
            &vec![fireside_core::model::content::ContentBlock::Text {
                body: "updated".to_string()
            }]
        );

        assert!(history.undo(&mut graph).expect("undo should succeed"));
        assert_eq!(
            graph.nodes[0].content,
            vec![fireside_core::model::content::ContentBlock::Text {
                body: "one".to_string()
            }]
        );

        assert!(history.redo(&mut graph).expect("redo should succeed"));
        assert_eq!(
            graph.nodes[0].content,
            vec![fireside_core::model::content::ContentBlock::Text {
                body: "updated".to_string()
            }]
        );
    }

    #[test]
    fn add_node_roundtrips_with_undo_redo() {
        let mut graph = graph_with_ids();
        let mut history = CommandHistory::new();

        history
            .apply_command(
                &mut graph,
                Command::AddNode {
                    node_id: "n3".to_string(),
                    after_index: Some(0),
                },
            )
            .expect("add should succeed");

        assert_eq!(graph.nodes.len(), 3);
        assert!(graph.index_of("n3").is_some());

        assert!(history.undo(&mut graph).expect("undo should succeed"));
        assert_eq!(graph.nodes.len(), 2);
        assert!(graph.index_of("n3").is_none());

        assert!(history.redo(&mut graph).expect("redo should succeed"));
        assert_eq!(graph.nodes.len(), 3);
        assert!(graph.index_of("n3").is_some());
    }

    #[test]
    fn update_block_roundtrips_with_undo_redo() {
        let mut graph = graph_with_ids();
        let mut history = CommandHistory::new();

        history
            .apply_command(
                &mut graph,
                Command::UpdateBlock {
                    node_id: "n1".to_string(),
                    block_index: 0,
                    block: fireside_core::model::content::ContentBlock::Text {
                        body: "changed".to_string(),
                    },
                },
            )
            .expect("update block should succeed");

        assert_eq!(
            graph.nodes[0].content,
            vec![fireside_core::model::content::ContentBlock::Text {
                body: "changed".to_string()
            }]
        );

        assert!(history.undo(&mut graph).expect("undo should succeed"));
        assert_eq!(
            graph.nodes[0].content,
            vec![fireside_core::model::content::ContentBlock::Text {
                body: "one".to_string()
            }]
        );
    }

    #[test]
    fn move_block_roundtrips_with_undo_redo() {
        let mut graph = load_graph_from_str(
            r#"{
            "nodes": [
              {
                "id": "n1",
                "content": [
                  {"kind":"text","body":"one"},
                  {"kind":"text","body":"two"}
                ]
              }
            ]
          }"#,
        )
        .expect("graph should parse");
        let mut history = CommandHistory::new();

        history
            .apply_command(
                &mut graph,
                Command::MoveBlock {
                    node_id: "n1".to_string(),
                    from_index: 0,
                    to_index: 1,
                },
            )
            .expect("move block should succeed");

        let first_body = match &graph.nodes[0].content[0] {
            fireside_core::model::content::ContentBlock::Text { body } => body,
            _ => panic!("expected text block"),
        };
        assert_eq!(first_body, "two");

        assert!(history.undo(&mut graph).expect("undo should succeed"));
        let first_body_after_undo = match &graph.nodes[0].content[0] {
            fireside_core::model::content::ContentBlock::Text { body } => body,
            _ => panic!("expected text block"),
        };
        assert_eq!(first_body_after_undo, "one");
    }

    #[test]
    fn remove_block_roundtrips_with_undo_redo() {
        let mut graph = load_graph_from_str(
            r#"{
            "nodes": [
              {
                "id": "n1",
                "content": [
                  {"kind":"text","body":"first"},
                  {"kind":"text","body":"second"}
                ]
              }
            ]
          }"#,
        )
        .expect("graph should parse");
        let mut history = CommandHistory::new();

        // Remove the first block.
        history
            .apply_command(
                &mut graph,
                Command::RemoveBlock {
                    node_id: "n1".to_string(),
                    block_index: 0,
                },
            )
            .expect("remove block should succeed");

        assert_eq!(graph.nodes[0].content.len(), 1);
        let remaining = match &graph.nodes[0].content[0] {
            fireside_core::model::content::ContentBlock::Text { body } => body,
            _ => panic!("expected text block"),
        };
        assert_eq!(remaining, "second");

        // Undo restores both blocks.
        assert!(history.undo(&mut graph).expect("undo should succeed"));
        assert_eq!(graph.nodes[0].content.len(), 2);
        let first = match &graph.nodes[0].content[0] {
            fireside_core::model::content::ContentBlock::Text { body } => body,
            _ => panic!("expected text block"),
        };
        assert_eq!(first, "first");

        // Redo removes it again.
        assert!(history.redo(&mut graph).expect("redo should succeed"));
        assert_eq!(graph.nodes[0].content.len(), 1);
    }
}
