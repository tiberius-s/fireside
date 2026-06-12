//! The presentation session — the §3 Traversal state machine.
//!
//! A [`Session`] owns an immutable [`Graph`] plus the presenter's position:
//! the current node and a history stack of node IDs (never array indices).
//! The four operations — [`Session::next`], [`Session::choose`],
//! [`Session::goto`], [`Session::back`] — implement the spec's algorithms
//! exactly, and every call returns an [`Outcome`] so a UI can give the
//! presenter feedback for *every* keypress: nothing here is a silent no-op.
//!
//! History invariants (spec §3) upheld by construction:
//! 1. `choose` and `goto` push exactly one entry on success.
//! 2. A `next` that moves pushes exactly one entry.
//! 3. `back` pops one entry and pushes none.
//! 4. Failed operations never mutate history.

use std::collections::{HashMap, HashSet};

use fireside_core::{BranchPoint, Graph, Node, NodeDefaults, NodeId};

use crate::error::EngineError;

/// The result of a traversal operation, for UI feedback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The session moved to a new current node.
    Moved,
    /// `next` is blocked: the current node has a branch point awaiting a
    /// choice.
    BlockedByBranch,
    /// `next` at a terminal node: this path has ended; only `back` exits.
    EndOfPath,
    /// `back` with an empty history: already at the start of the path.
    HistoryEmpty,
    /// `choose` outside a branch point, or with an option that does not
    /// exist.
    InvalidChoice,
    /// The requested target node ID does not exist in the graph.
    UnknownNode(NodeId),
}

/// A live presentation over an immutable graph.
#[derive(Debug)]
pub struct Session {
    graph: Graph,
    /// Index of the current node in `graph.nodes`.
    current: usize,
    /// Stack of previously visited node IDs.
    history: Vec<NodeId>,
    /// Node ID → index lookup, built once at construction.
    index: HashMap<NodeId, usize>,
    /// Every node ID the presenter has seen this session.
    visited: HashSet<NodeId>,
}

impl Session {
    /// Start a session at the graph's entry point (the first node).
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::EmptyGraph`] when the graph has no nodes.
    pub fn new(graph: Graph) -> Result<Self, EngineError> {
        if graph.nodes.is_empty() {
            return Err(EngineError::EmptyGraph);
        }
        // First occurrence wins on (invalid) duplicate IDs; validation
        // reports duplicates as errors before a session should start.
        let mut index = HashMap::with_capacity(graph.nodes.len());
        for (i, node) in graph.nodes.iter().enumerate() {
            index.entry(node.id.clone()).or_insert(i);
        }
        let mut visited = HashSet::new();
        visited.insert(graph.nodes[0].id.clone());
        Ok(Self {
            graph,
            current: 0,
            history: Vec::new(),
            index,
            visited,
        })
    }

    /// The graph being presented.
    #[must_use]
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// The node the presenter is on.
    #[must_use]
    pub fn current(&self) -> &Node {
        &self.graph.nodes[self.current]
    }

    /// The graph's node defaults, if any.
    #[must_use]
    pub fn defaults(&self) -> Option<&NodeDefaults> {
        self.graph.defaults.as_ref()
    }

    /// The branch point at the current node, if any.
    #[must_use]
    pub fn branch_point(&self) -> Option<&BranchPoint> {
        self.current().branch_point()
    }

    /// Whether `back` would move (history is non-empty).
    #[must_use]
    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }

    /// The history stack, oldest first.
    #[must_use]
    pub fn history(&self) -> &[NodeId] {
        &self.history
    }

    /// Node IDs visited so far this session.
    #[must_use]
    pub fn visited(&self) -> &HashSet<NodeId> {
        &self.visited
    }

    /// Advance along the explicit next edge.
    ///
    /// Blocked at a branch point; reports the end of the path at a
    /// terminal node.
    // The spec names this operation `next()`; matching it beats Iterator
    // naming hygiene, and Session is not an iterator.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Outcome {
        if self.current().branch_point().is_some() {
            return Outcome::BlockedByBranch;
        }
        match self.current().next_target() {
            Some(target) => {
                let id = target.to_owned();
                self.move_to(&id)
            }
            None => Outcome::EndOfPath,
        }
    }

    /// Select a branch option by its position in the options array.
    pub fn choose(&mut self, option: usize) -> Outcome {
        let Some(bp) = self.current().branch_point() else {
            return Outcome::InvalidChoice;
        };
        let Some(opt) = bp.options.get(option) else {
            return Outcome::InvalidChoice;
        };
        let id = opt.target.clone();
        self.move_to(&id)
    }

    /// Jump directly to a node by ID. As an explicit command, `goto`
    /// bypasses branch-point gating.
    pub fn goto(&mut self, target: &str) -> Outcome {
        self.move_to(target)
    }

    /// Return to the previous node in the history stack.
    pub fn back(&mut self) -> Outcome {
        let Some(id) = self.history.last() else {
            return Outcome::HistoryEmpty;
        };
        // Always present: history entries were valid when pushed and the
        // graph is immutable for the life of the session.
        let Some(&idx) = self.index.get(id) else {
            return Outcome::HistoryEmpty;
        };
        self.history.pop();
        self.current = idx;
        Outcome::Moved
    }

    /// Navigate to `target`, pushing the current node onto history.
    /// Fails without mutating anything when the target is unknown.
    fn move_to(&mut self, target: &str) -> Outcome {
        let Some(&idx) = self.index.get(target) else {
            return Outcome::UnknownNode(target.to_owned());
        };
        self.history.push(self.current().id.clone());
        self.current = idx;
        self.visited.insert(self.graph.nodes[idx].id.clone());
        Outcome::Moved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELLO: &str = include_str!("../../../docs/examples/hello.json");

    fn hello_session() -> Session {
        let graph = Graph::from_json(HELLO).expect("hello.json parses");
        Session::new(graph).expect("non-empty")
    }

    #[test]
    fn empty_graph_is_rejected() {
        let graph = Graph::from_json(r#"{"nodes":[]}"#).expect("parse");
        assert!(matches!(Session::new(graph), Err(EngineError::EmptyGraph)));
    }

    #[test]
    fn starts_at_entry_node() {
        let s = hello_session();
        assert_eq!(s.current().id, "intro");
        assert!(!s.can_go_back());
        assert!(s.visited().contains("intro"));
    }

    #[test]
    fn next_follows_string_shorthand_and_object_form() {
        let mut s = hello_session();
        assert_eq!(s.next(), Outcome::Moved); // intro -> features
        assert_eq!(s.current().id, "features");
        assert_eq!(s.next(), Outcome::Moved); // features -> choose
        assert_eq!(s.current().id, "choose");
        assert_eq!(s.history(), ["intro", "features"]);
    }

    #[test]
    fn next_is_blocked_at_branch_point_without_mutating() {
        let mut s = hello_session();
        s.next();
        s.next(); // at "choose"
        let before = s.history().to_vec();
        assert_eq!(s.next(), Outcome::BlockedByBranch);
        assert_eq!(s.current().id, "choose");
        assert_eq!(s.history(), before, "failed ops must not touch history");
    }

    #[test]
    fn choose_navigates_to_option_target() {
        let mut s = hello_session();
        s.next();
        s.next(); // at "choose"
        assert_eq!(s.choose(0), Outcome::Moved);
        assert_eq!(s.current().id, "code-demo");
        assert_eq!(s.history(), ["intro", "features", "choose"]);
    }

    #[test]
    fn choose_is_invalid_outside_branch_point() {
        let mut s = hello_session();
        assert_eq!(s.choose(0), Outcome::InvalidChoice);
        assert!(s.history().is_empty());
    }

    #[test]
    fn choose_rejects_out_of_range_option() {
        let mut s = hello_session();
        s.next();
        s.next();
        assert_eq!(s.choose(99), Outcome::InvalidChoice);
        assert_eq!(s.current().id, "choose");
    }

    #[test]
    fn next_at_terminal_reports_end_of_path() {
        let mut s = hello_session();
        assert_eq!(s.goto("thanks"), Outcome::Moved);
        assert_eq!(s.next(), Outcome::EndOfPath);
        assert_eq!(s.current().id, "thanks");
        assert_eq!(s.history(), ["intro"]);
    }

    #[test]
    fn goto_unknown_node_is_a_guarded_no_op() {
        let mut s = hello_session();
        assert_eq!(s.goto("nope"), Outcome::UnknownNode("nope".into()));
        assert_eq!(s.current().id, "intro");
        assert!(s.history().is_empty());
    }

    #[test]
    fn back_pops_one_entry_and_pushes_none() {
        let mut s = hello_session();
        s.next();
        s.next();
        assert_eq!(s.back(), Outcome::Moved);
        assert_eq!(s.current().id, "features");
        assert_eq!(s.history(), ["intro"]);
        assert_eq!(s.back(), Outcome::Moved);
        assert_eq!(s.current().id, "intro");
        assert_eq!(s.back(), Outcome::HistoryEmpty);
        assert_eq!(s.current().id, "intro");
    }

    #[test]
    fn back_reflects_actual_path_after_branching() {
        let mut s = hello_session();
        s.next(); // features
        s.next(); // choose
        s.choose(1); // layout-demo
        s.next(); // thanks (explicit rejoin edge)
        assert_eq!(s.current().id, "thanks");
        s.back();
        assert_eq!(s.current().id, "layout-demo");
        s.back();
        assert_eq!(s.current().id, "choose");
        s.back();
        assert_eq!(s.current().id, "features");
    }

    #[test]
    fn visited_tracks_every_node_seen() {
        let mut s = hello_session();
        s.next();
        s.next();
        s.choose(2); // thanks
        let visited = s.visited();
        for id in ["intro", "features", "choose", "thanks"] {
            assert!(visited.contains(id), "missing {id}");
        }
        assert!(!visited.contains("code-demo"));
    }
}
