//! Traversal state machine — implements Next, Choose, Goto, Back operations.
//!
//! The `TraversalEngine` maintains the current position within a graph
//! and a history stack for backtracking.

use fireside_core::model::graph::Graph;

use crate::error::EngineError;

/// The traversal state machine for navigating a Fireside graph.
///
/// Maintains the current node index and a history stack for `Back` operations.
#[derive(Debug)]
pub struct TraversalEngine {
    /// Index of the currently active node (0-based).
    current: usize,
    /// Navigation history stack for backtracking.
    history: Vec<usize>,
}

/// Result of a traversal operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraversalResult {
    /// Successfully moved to a new node.
    Moved { from: usize, to: usize },
    /// Already at the boundary (first/last node); no movement.
    AtBoundary,
}

impl TraversalEngine {
    /// Create a new traversal engine starting at the given node index.
    #[must_use]
    pub fn new(start: usize) -> Self {
        Self {
            current: start,
            history: Vec::new(),
        }
    }

    /// Returns the current node index (0-based).
    #[must_use]
    pub fn current(&self) -> usize {
        self.current
    }

    /// Returns the navigation history stack.
    #[must_use]
    pub fn history(&self) -> &[usize] {
        &self.history
    }

    /// Advance to the next node.
    ///
    /// Respects traversal overrides (`traversal.next`) on the current node.
    /// Falls back to sequential advance if no override exists.
    pub fn next(&mut self, graph: &Graph) -> TraversalResult {
        let node = &graph.nodes[self.current];

        // Check for traversal next override
        if let Some(target_id) = node.next_override()
            && let Some(idx) = graph.index_of(target_id)
        {
            let from = self.current;
            self.history.push(from);
            self.current = idx;
            return TraversalResult::Moved { from, to: idx };
        }

        // Sequential advance
        if self.current + 1 < graph.len() {
            let from = self.current;
            self.history.push(from);
            self.current = from + 1;
            TraversalResult::Moved { from, to: from + 1 }
        } else {
            TraversalResult::AtBoundary
        }
    }

    /// Go back to the previous node (pop history stack).
    ///
    /// If the history is empty, tries sequential backward movement.
    pub fn back(&mut self) -> TraversalResult {
        if let Some(prev) = self.history.pop() {
            let from = self.current;
            self.current = prev;
            TraversalResult::Moved { from, to: prev }
        } else if self.current > 0 {
            let from = self.current;
            self.current = from - 1;
            TraversalResult::Moved { from, to: from - 1 }
        } else {
            TraversalResult::AtBoundary
        }
    }

    /// Jump to a specific node by index.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::InvalidTraversal` if the index is out of bounds.
    pub fn goto(&mut self, index: usize, graph: &Graph) -> Result<TraversalResult, EngineError> {
        if index >= graph.len() {
            return Err(EngineError::InvalidTraversal(format!(
                "node index {index} out of bounds (graph has {} nodes)",
                graph.len()
            )));
        }

        let from = self.current;
        self.history.push(from);
        self.current = index;
        Ok(TraversalResult::Moved { from, to: index })
    }

    /// Choose a branch option by key character.
    ///
    /// Looks up the branch point on the current node and navigates to the
    /// target of the matching option.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::InvalidTraversal` if the current node has no
    /// branch point, or if no option matches the given key.
    pub fn choose(&mut self, key: char, graph: &Graph) -> Result<TraversalResult, EngineError> {
        let node = &graph.nodes[self.current];
        let bp = node.branch_point().ok_or_else(|| {
            EngineError::InvalidTraversal("current node has no branch point".into())
        })?;

        let option = bp.options.iter().find(|o| o.key == key).ok_or_else(|| {
            EngineError::InvalidTraversal(format!("no branch option with key '{key}'"))
        })?;

        let target_idx = graph.index_of(&option.target).ok_or_else(|| {
            EngineError::DanglingReference(format!("branch target '{}' not found", option.target))
        })?;

        let from = self.current;
        self.history.push(from);
        self.current = target_idx;
        Ok(TraversalResult::Moved {
            from,
            to: target_idx,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::load_graph_from_str;

    fn test_graph() -> Graph {
        let json = r#"{
            "nodes": [
                { "id": "a", "content": [{ "kind": "text", "body": "A" }] },
                { "id": "b", "content": [{ "kind": "text", "body": "B" }] },
                { "id": "c", "content": [{ "kind": "text", "body": "C" }] }
            ]
        }"#;
        load_graph_from_str(json).unwrap()
    }

    #[test]
    fn sequential_next() {
        let graph = test_graph();
        let mut engine = TraversalEngine::new(0);
        assert_eq!(
            engine.next(&graph),
            TraversalResult::Moved { from: 0, to: 1 }
        );
        assert_eq!(engine.current(), 1);
    }

    #[test]
    fn next_at_end_is_boundary() {
        let graph = test_graph();
        let mut engine = TraversalEngine::new(2);
        assert_eq!(engine.next(&graph), TraversalResult::AtBoundary);
    }

    #[test]
    fn back_pops_history() {
        let graph = test_graph();
        let mut engine = TraversalEngine::new(0);
        engine.next(&graph);
        engine.next(&graph);
        assert_eq!(engine.current(), 2);
        assert_eq!(engine.back(), TraversalResult::Moved { from: 2, to: 1 });
    }

    #[test]
    fn goto_with_valid_index() {
        let graph = test_graph();
        let mut engine = TraversalEngine::new(0);
        let result = engine.goto(2, &graph).unwrap();
        assert_eq!(result, TraversalResult::Moved { from: 0, to: 2 });
    }

    #[test]
    fn goto_out_of_bounds_errors() {
        let graph = test_graph();
        let mut engine = TraversalEngine::new(0);
        assert!(engine.goto(10, &graph).is_err());
    }

    #[test]
    fn next_respects_traversal_override() {
        let json = r#"{
            "nodes": [
                { "id": "a", "content": [], "traversal": { "next": "c" } },
                { "id": "b", "content": [] },
                { "id": "c", "content": [] }
            ]
        }"#;
        let graph = load_graph_from_str(json).unwrap();
        let mut engine = TraversalEngine::new(0);
        let result = engine.next(&graph);
        assert_eq!(result, TraversalResult::Moved { from: 0, to: 2 });
    }

    #[test]
    fn choose_branch_option() {
        let json = r#"{
            "nodes": [
                {
                    "id": "start",
                    "content": [],
                    "traversal": {
                        "branch-point": {
                            "options": [
                                { "label": "Alpha", "key": "a", "target": "alpha" },
                                { "label": "Beta", "key": "b", "target": "beta" }
                            ]
                        }
                    }
                },
                { "id": "alpha", "content": [] },
                { "id": "beta", "content": [] }
            ]
        }"#;
        let graph = load_graph_from_str(json).unwrap();
        let mut engine = TraversalEngine::new(0);
        let result = engine.choose('b', &graph).unwrap();
        assert_eq!(result, TraversalResult::Moved { from: 0, to: 2 });
    }
}
