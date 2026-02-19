//! Presentation session — the mutable runtime container for a loaded graph.
//!
//! The session is the single source of truth used by frontend modes
//! (present, edit). It combines the graph, traversal state, and a
//! dirty flag for unsaved changes.

use fireside_core::model::graph::Graph;
use fireside_core::model::node::NodeId;

use crate::commands::{Command, CommandHistory};
use crate::error::EngineError;

use crate::traversal::TraversalEngine;

/// A live presentation session combining graph data and traversal state.
#[derive(Debug)]
pub struct PresentationSession {
    /// The loaded graph document.
    pub graph: Graph,
    /// The traversal state machine.
    pub traversal: TraversalEngine,
    /// Whether the session has unsaved changes.
    pub dirty: bool,
    /// Undo/redo command history for editor operations.
    pub command_history: CommandHistory,
}

impl PresentationSession {
    /// Create a new session from a loaded graph.
    ///
    /// Starts traversal at the given node index (clamped to valid range).
    #[must_use]
    pub fn new(graph: Graph, start: usize) -> Self {
        let clamped = start.min(graph.len().saturating_sub(1));
        Self {
            traversal: TraversalEngine::new(clamped),
            graph,
            dirty: false,
            command_history: CommandHistory::new(),
        }
    }

    /// Returns the index of the currently active node.
    #[must_use]
    pub fn current_node_index(&self) -> usize {
        self.traversal.current()
    }

    /// Returns a reference to the currently active node.
    #[must_use]
    pub fn current_node(&self) -> &fireside_core::model::node::Node {
        &self.graph.nodes[self.traversal.current()]
    }

    /// Mark the session as having unsaved changes.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clear the dirty flag (e.g., after saving).
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Ensure a node has an ID and return it.
    ///
    /// # Errors
    ///
    /// Returns an error if `index` is out of bounds.
    pub fn ensure_node_id(&mut self, index: usize) -> Result<NodeId, EngineError> {
        if index >= self.graph.nodes.len() {
            return Err(EngineError::CommandError(format!(
                "node index {index} out of bounds"
            )));
        }

        if let Some(existing) = self.graph.nodes[index].id.clone() {
            return Ok(existing);
        }

        let mut suffix = index + 1;
        let node_id = loop {
            let candidate = format!("node-{suffix}");
            if self.graph.index_of(&candidate).is_none() {
                break candidate;
            }
            suffix += 1;
        };

        self.graph.nodes[index].id = Some(node_id.clone());
        self.rebuild_node_index();
        self.mark_dirty();

        Ok(node_id)
    }

    /// Execute a graph mutation command and record it for undo/redo.
    ///
    /// # Errors
    ///
    /// Returns an engine error when command application fails.
    pub fn execute_command(&mut self, command: Command) -> Result<(), EngineError> {
        self.command_history.apply_command(&mut self.graph, command)?;
        self.rebuild_node_index();
        self.traversal.clamp_to_graph(self.graph.len());
        self.mark_dirty();
        Ok(())
    }

    /// Undo the latest command.
    ///
    /// Returns `true` when a command was undone.
    ///
    /// # Errors
    ///
    /// Returns an engine error when undo application fails.
    pub fn undo(&mut self) -> Result<bool, EngineError> {
        let changed = self.command_history.undo(&mut self.graph)?;
        if changed {
            self.rebuild_node_index();
            self.traversal.clamp_to_graph(self.graph.len());
            self.mark_dirty();
        }
        Ok(changed)
    }

    /// Redo the latest undone command.
    ///
    /// Returns `true` when a command was redone.
    ///
    /// # Errors
    ///
    /// Returns an engine error when redo application fails.
    pub fn redo(&mut self) -> Result<bool, EngineError> {
        let changed = self.command_history.redo(&mut self.graph)?;
        if changed {
            self.rebuild_node_index();
            self.traversal.clamp_to_graph(self.graph.len());
            self.mark_dirty();
        }
        Ok(changed)
    }

    fn rebuild_node_index(&mut self) {
        self.graph.node_index = self
            .graph
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| node.id.as_ref().map(|id| (id.clone(), idx)))
            .collect();
    }
}
