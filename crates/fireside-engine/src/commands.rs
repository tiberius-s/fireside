//! Graph mutation commands for editor support.
//!
//! Commands represent atomic operations on a graph that can be applied,
//! undone, and redone. This module defines the command types and the
//! application logic.
//!
//! **Note:** This is a scaffold for future editor functionality. The
//! command types are defined but not yet fully implemented.

use fireside_core::model::content::ContentBlock;
use fireside_core::model::node::NodeId;

/// A command that mutates the graph within a session.
#[derive(Debug, Clone)]
pub enum Command {
    /// Update the content blocks of a node.
    UpdateNodeContent {
        /// Target node ID.
        node_id: NodeId,
        /// New content blocks.
        content: Vec<ContentBlock>,
    },

    /// Add a new node to the graph.
    AddNode {
        /// The node ID for the new node.
        node_id: NodeId,
        /// Insert after this node index (None = append).
        after_index: Option<usize>,
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

/// History entry for undo/redo support.
#[derive(Debug)]
pub struct CommandHistory {
    /// Applied commands (for undo).
    applied: Vec<Command>,
    /// Undone commands (for redo).
    undone: Vec<Command>,
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

    /// Record a command that was applied.
    pub fn push(&mut self, command: Command) {
        self.applied.push(command);
        // Clear redo stack when a new command is applied
        self.undone.clear();
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
