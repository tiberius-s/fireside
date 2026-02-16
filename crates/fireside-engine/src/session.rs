//! Presentation session — the mutable runtime container for a loaded graph.
//!
//! The session is the single source of truth used by frontend modes
//! (present, edit). It combines the graph, traversal state, and a
//! dirty flag for unsaved changes.

use fireside_core::model::graph::Graph;

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
}
