//! Graph mutation commands for editor support.
//!
//! Commands represent atomic operations on a graph that can be applied,
//! undone, and redone. This module defines the command types and the
//! application logic.
//!
use fireside_core::model::content::ContentBlock;
use fireside_core::model::graph::Graph;
use fireside_core::model::node::{Node, NodeId};
use fireside_core::model::traversal::Traversal;

use crate::error::EngineError;

/// A command that mutates the graph within a session.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// Update the content blocks of a node.
    UpdateNodeContent {
        /// Target node ID.
        node_id: NodeId,
        /// New content blocks.
        content: Vec<ContentBlock>,
    },

    /// Update a specific content block in a node.
    UpdateBlock {
        /// Target node ID.
        node_id: NodeId,
        /// Zero-based block index.
        block_index: usize,
        /// New block value.
        block: ContentBlock,
    },

    /// Move a content block within a node.
    MoveBlock {
        /// Target node ID.
        node_id: NodeId,
        /// Source zero-based block index.
        from_index: usize,
        /// Destination zero-based block index.
        to_index: usize,
    },

    /// Add a new node to the graph.
    AddNode {
        /// The node ID for the new node.
        node_id: NodeId,
        /// Insert after this node index (None = append).
        after_index: Option<usize>,
    },

    /// Restore a previously removed node at an index.
    RestoreNode {
        /// Full node data to restore.
        node: Node,
        /// Index at which to restore.
        index: usize,
    },

    /// Remove a node from the graph.
    RemoveNode {
        /// The node ID to remove.
        node_id: NodeId,
    },

    /// Set the traversal next override for a node.
    SetTraversalNext {
        /// Source node ID.
        node_id: NodeId,
        /// Target node ID for the next override.
        target: NodeId,
    },

    /// Clear the traversal next override for a node.
    ClearTraversalNext {
        /// Node ID to clear.
        node_id: NodeId,
    },
}

#[derive(Debug, Clone)]
struct HistoryEntry {
    command: Command,
    inverse: Command,
}

/// History entry for undo/redo support.
#[derive(Debug)]
pub struct CommandHistory {
    /// Applied commands with their inverses (for undo).
    applied: Vec<HistoryEntry>,
    /// Undone commands with their inverses (for redo).
    undone: Vec<HistoryEntry>,
}

impl CommandHistory {
    /// Create an empty command history.
    #[must_use]
    pub fn new() -> Self {
        Self {
            applied: Vec::new(),
            undone: Vec::new(),
        }
    }

    /// Apply a command to the graph and record it for undo.
    ///
    /// # Errors
    ///
    /// Returns an `EngineError::CommandError` when the command is invalid
    /// for the current graph state.
    pub fn apply_command(
        &mut self,
        graph: &mut Graph,
        command: Command,
    ) -> Result<(), EngineError> {
        let inverse = apply_command(graph, &command)?;
        self.applied.push(HistoryEntry { command, inverse });
        self.undone.clear();
        Ok(())
    }

    /// Undo the most recent applied command.
    ///
    /// Returns `Ok(true)` if a command was undone, `Ok(false)` if there is
    /// nothing to undo.
    ///
    /// # Errors
    ///
    /// Returns an `EngineError` if applying the inverse command fails.
    pub fn undo(&mut self, graph: &mut Graph) -> Result<bool, EngineError> {
        let Some(entry) = self.applied.pop() else {
            return Ok(false);
        };

        apply_command(graph, &entry.inverse)?;
        self.undone.push(entry);
        Ok(true)
    }

    /// Redo the most recently undone command.
    ///
    /// Returns `Ok(true)` if a command was redone, `Ok(false)` if there is
    /// nothing to redo.
    ///
    /// # Errors
    ///
    /// Returns an `EngineError` if applying the command fails.
    pub fn redo(&mut self, graph: &mut Graph) -> Result<bool, EngineError> {
        let Some(entry) = self.undone.pop() else {
            return Ok(false);
        };

        apply_command(graph, &entry.command)?;
        self.applied.push(entry);
        Ok(true)
    }

    /// Returns `true` if there are commands to undo.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.applied.is_empty()
    }

    /// Returns `true` if there are commands to redo.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

fn apply_command(graph: &mut Graph, command: &Command) -> Result<Command, EngineError> {
    match command {
        Command::UpdateNodeContent { node_id, content } => {
            let idx = graph.index_of(node_id).ok_or_else(|| {
                EngineError::CommandError(format!("node id '{node_id}' not found"))
            })?;

            let node = graph
                .nodes
                .get_mut(idx)
                .ok_or_else(|| EngineError::CommandError("node index out of bounds".into()))?;

            let previous = node.content.clone();
            node.content = content.clone();

            Ok(Command::UpdateNodeContent {
                node_id: node_id.clone(),
                content: previous,
            })
        }
        Command::UpdateBlock {
            node_id,
            block_index,
            block,
        } => {
            let idx = graph.index_of(node_id).ok_or_else(|| {
                EngineError::CommandError(format!("node id '{node_id}' not found"))
            })?;

            let node = graph
                .nodes
                .get_mut(idx)
                .ok_or_else(|| EngineError::CommandError("node index out of bounds".into()))?;

            let Some(previous) = node.content.get(*block_index).cloned() else {
                return Err(EngineError::CommandError(format!(
                    "block index {} out of bounds for node '{node_id}'",
                    block_index
                )));
            };

            node.content[*block_index] = block.clone();

            Ok(Command::UpdateBlock {
                node_id: node_id.clone(),
                block_index: *block_index,
                block: previous,
            })
        }
        Command::MoveBlock {
            node_id,
            from_index,
            to_index,
        } => {
            let idx = graph.index_of(node_id).ok_or_else(|| {
                EngineError::CommandError(format!("node id '{node_id}' not found"))
            })?;

            let node = graph
                .nodes
                .get_mut(idx)
                .ok_or_else(|| EngineError::CommandError("node index out of bounds".into()))?;

            let len = node.content.len();
            if *from_index >= len || *to_index >= len {
                return Err(EngineError::CommandError(format!(
                    "move indices out of bounds for node '{node_id}'"
                )));
            }

            if from_index != to_index {
                let block = node.content.remove(*from_index);
                node.content.insert(*to_index, block);
            }

            Ok(Command::MoveBlock {
                node_id: node_id.clone(),
                from_index: *to_index,
                to_index: *from_index,
            })
        }
        Command::AddNode {
            node_id,
            after_index,
        } => {
            if graph.index_of(node_id).is_some() {
                return Err(EngineError::CommandError(format!(
                    "node id '{node_id}' already exists"
                )));
            }

            let index = after_index.map_or(graph.nodes.len(), |i| {
                i.saturating_add(1).min(graph.nodes.len())
            });

            graph.nodes.insert(
                index,
                Node {
                    id: Some(node_id.clone()),
                    title: None,
                    tags: Vec::new(),
                    duration: None,
                    layout: None,
                    transition: None,
                    speaker_notes: None,
                    traversal: None,
                    content: vec![ContentBlock::Text {
                        body: String::new(),
                    }],
                },
            );
            graph.rebuild_index().map_err(EngineError::CommandError)?;

            Ok(Command::RemoveNode {
                node_id: node_id.clone(),
            })
        }
        Command::RestoreNode { node, index } => {
            if let Some(id) = node.id.as_deref()
                && graph.index_of(id).is_some()
            {
                return Err(EngineError::CommandError(format!(
                    "node id '{id}' already exists"
                )));
            }

            let insert_index = (*index).min(graph.nodes.len());
            graph.nodes.insert(insert_index, node.clone());
            graph.rebuild_index().map_err(EngineError::CommandError)?;

            let node_id = node
                .id
                .clone()
                .ok_or_else(|| EngineError::CommandError("restored node is missing id".into()))?;

            Ok(Command::RemoveNode { node_id })
        }
        Command::RemoveNode { node_id } => {
            if graph.nodes.len() <= 1 {
                return Err(EngineError::CommandError(
                    "cannot remove the last node in the graph".into(),
                ));
            }

            let idx = graph.index_of(node_id).ok_or_else(|| {
                EngineError::CommandError(format!("node id '{node_id}' not found"))
            })?;

            let removed = graph.nodes.remove(idx);
            graph.rebuild_index().map_err(EngineError::CommandError)?;

            Ok(Command::RestoreNode {
                node: removed,
                index: idx,
            })
        }
        Command::SetTraversalNext { node_id, target } => {
            if graph.index_of(target).is_none() {
                return Err(EngineError::CommandError(format!(
                    "target node id '{target}' not found"
                )));
            }

            let idx = graph.index_of(node_id).ok_or_else(|| {
                EngineError::CommandError(format!("node id '{node_id}' not found"))
            })?;

            let node = graph
                .nodes
                .get_mut(idx)
                .ok_or_else(|| EngineError::CommandError("node index out of bounds".into()))?;

            let previous_next = node.traversal.as_ref().and_then(|tr| tr.next.clone());

            let traversal = node.traversal.get_or_insert(Traversal {
                next: None,
                after: None,
                branch_point: None,
            });
            traversal.next = Some(target.clone());

            let inverse = match previous_next {
                Some(previous) => Command::SetTraversalNext {
                    node_id: node_id.clone(),
                    target: previous,
                },
                None => Command::ClearTraversalNext {
                    node_id: node_id.clone(),
                },
            };

            Ok(inverse)
        }
        Command::ClearTraversalNext { node_id } => {
            let idx = graph.index_of(node_id).ok_or_else(|| {
                EngineError::CommandError(format!("node id '{node_id}' not found"))
            })?;

            let node = graph
                .nodes
                .get_mut(idx)
                .ok_or_else(|| EngineError::CommandError("node index out of bounds".into()))?;

            let previous_next = node.traversal.as_ref().and_then(|tr| tr.next.clone());

            if let Some(traversal) = node.traversal.as_mut() {
                traversal.next = None;
                if traversal.after.is_none() && traversal.branch_point.is_none() {
                    node.traversal = None;
                }
            }

            let inverse = match previous_next {
                Some(previous) => Command::SetTraversalNext {
                    node_id: node_id.clone(),
                    target: previous,
                },
                None => Command::ClearTraversalNext {
                    node_id: node_id.clone(),
                },
            };

            Ok(inverse)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::loader::load_graph_from_str;

    use super::*;

    fn graph_with_ids() -> Graph {
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
                    content: vec![ContentBlock::Text {
                        body: "updated".to_string(),
                    }],
                },
            )
            .expect("update should succeed");

        let current = &graph.nodes[0].content;
        assert_eq!(
            current,
            &vec![ContentBlock::Text {
                body: "updated".to_string()
            }]
        );

        assert!(history.undo(&mut graph).expect("undo should succeed"));
        assert_eq!(
            graph.nodes[0].content,
            vec![ContentBlock::Text {
                body: "one".to_string()
            }]
        );

        assert!(history.redo(&mut graph).expect("redo should succeed"));
        assert_eq!(
            graph.nodes[0].content,
            vec![ContentBlock::Text {
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
                    block: ContentBlock::Text {
                        body: "changed".to_string(),
                    },
                },
            )
            .expect("update block should succeed");

        assert_eq!(
            graph.nodes[0].content,
            vec![ContentBlock::Text {
                body: "changed".to_string()
            }]
        );

        assert!(history.undo(&mut graph).expect("undo should succeed"));
        assert_eq!(
            graph.nodes[0].content,
            vec![ContentBlock::Text {
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
            ContentBlock::Text { body } => body,
            _ => panic!("expected text block"),
        };
        assert_eq!(first_body, "two");

        assert!(history.undo(&mut graph).expect("undo should succeed"));
        let first_body_after_undo = match &graph.nodes[0].content[0] {
            ContentBlock::Text { body } => body,
            _ => panic!("expected text block"),
        };
        assert_eq!(first_body_after_undo, "one");
    }
}
