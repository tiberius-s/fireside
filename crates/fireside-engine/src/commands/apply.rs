//! Pure command application logic.
//!
//! [`apply_command`] is the single point where a [`Command`] is executed
//! against a mutable [`Graph`].  It returns the *inverse* command needed
//! to undo the operation, making the function the foundation of the
//! undo/redo stack in [`super::history::CommandHistory`].
//!
//! The function is intentionally `pub(super)` — callers must go through
//! `CommandHistory` to keep history tracking consistent.

use fireside_core::model::content::ContentBlock;
use fireside_core::model::graph::Graph;
use fireside_core::model::node::Node;
use fireside_core::model::traversal::Traversal;

use crate::error::EngineError;

use super::types::Command;

/// Apply `command` to `graph` and return the inverse command for undo.
///
/// # Errors
///
/// Returns [`EngineError::CommandError`] when the command references a
/// node or block index that does not exist, or when the operation is
/// structurally invalid (e.g. removing the last node).
pub(super) fn apply_command(graph: &mut Graph, command: &Command) -> Result<Command, EngineError> {
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
        Command::RemoveBlock {
            node_id,
            block_index,
        } => {
            let idx = graph.index_of(node_id).ok_or_else(|| {
                EngineError::CommandError(format!("node id '{node_id}' not found"))
            })?;

            let node = graph
                .nodes
                .get_mut(idx)
                .ok_or_else(|| EngineError::CommandError("node index out of bounds".into()))?;

            if *block_index >= node.content.len() {
                return Err(EngineError::CommandError(format!(
                    "block index {block_index} out of bounds for node '{node_id}'"
                )));
            }

            let removed = node.content.remove(*block_index);

            // Inverse: re-insert the removed block at the same position.
            Ok(Command::InsertBlock {
                node_id: node_id.clone(),
                block_index: *block_index,
                block: removed,
            })
        }
        Command::InsertBlock {
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

            let insert_at = (*block_index).min(node.content.len());
            node.content.insert(insert_at, block.clone());

            // Inverse: remove the block we just inserted.
            Ok(Command::RemoveBlock {
                node_id: node_id.clone(),
                block_index: insert_at,
            })
        }
    }
}
