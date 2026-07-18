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
    /// A reveal step was consumed: more of the current node's content
    /// became visible. The current node did NOT change — UI effects tied
    /// to real navigation (transitions, branch-selection reset) MUST NOT
    /// fire for this outcome.
    Revealed,
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
    /// The reveal threshold reached at the current node. Reset to `0` on
    /// every node entry (see `move_to` and `back`) — reveal progress is
    /// not history-aware.
    reveal_level: u32,
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
            reveal_level: 0,
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

    /// The reveal threshold currently reached at the current node. A
    /// block is visible when its own `reveal` value (or `0` if absent)
    /// is `<=` this.
    #[must_use]
    pub fn reveal_level(&self) -> u32 {
        self.reveal_level
    }

    /// Whether the current node has reveal steps not yet reached — while
    /// true, `next()` will reveal rather than navigate, and branch
    /// selection MUST be unavailable.
    #[must_use]
    pub fn has_pending_reveal(&self) -> bool {
        self.reveal_level < self.current().reveal_levels().last().copied().unwrap_or(0)
    }

    /// `(revealed, total)` distinct reveal steps for the current node.
    /// `None` when the node uses no reveal marks at all.
    #[must_use]
    pub fn reveal_progress(&self) -> Option<(usize, usize)> {
        let levels = self.current().reveal_levels();
        if levels.is_empty() {
            return None;
        }
        let revealed = levels.iter().filter(|&&l| l <= self.reveal_level).count();
        Some((revealed, levels.len()))
    }

    /// Advance along the explicit next edge — or, first, reveal more of
    /// the current node.
    ///
    /// If the current node has reveal steps not yet reached, this
    /// advances to the next one and stops: no branch-point or
    /// traversal-target check happens on this call. Only once every
    /// reveal step is exhausted does `next()` fall through to its
    /// pre-reveal behavior: blocked at a branch point, or reporting the
    /// end of the path at a terminal node.
    // The spec names this operation `next()`; matching it beats Iterator
    // naming hygiene, and Session is not an iterator.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Outcome {
        let levels = self.current().reveal_levels();
        if let Some(&next_level) = levels.iter().find(|&&l| l > self.reveal_level) {
            self.reveal_level = next_level;
            return Outcome::Revealed;
        }
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
    ///
    /// MUST NOT succeed while the current node has reveal steps not yet
    /// reached — a presenter cannot skip ahead to a choice by choosing
    /// early. Callers that route branch-selection keys through their own
    /// UI SHOULD additionally gate on [`Session::has_pending_reveal`]
    /// themselves so the same keypress continues revealing instead of
    /// simply doing nothing (the reference TUI does this).
    pub fn choose(&mut self, option: usize) -> Outcome {
        if self.has_pending_reveal() {
            return Outcome::InvalidChoice;
        }
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
        self.reveal_level = 0;
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
        self.reveal_level = 0;
        Outcome::Moved
    }
}

#[cfg(test)]
mod proptest_support {
    //! Test-only generators for session-invariant property tests, per
    //! `specs/008-protocol-workflow-hardening/research.md` §3. Written
    //! independently of `fireside-core`'s own `proptest_support` module
    //! (which is `#[cfg(test)]`-private to that crate) rather than shared
    //! via a test-utility crate, per Constitution Principle III (no new
    //! crate, no new dependency): each crate's tests own their generators,
    //! and this one is scoped to what session replay actually needs
    //! (navigable graphs), not the full wire-format generality
    //! `fireside-core`'s round-trip property requires.

    use proptest::collection::vec;
    use proptest::option;
    use proptest::prelude::*;

    use fireside_core::{
        BranchOption, BranchPoint, ContentBlock, Graph, Node, Traversal, TraversalSpec,
    };

    /// One step of a generated navigation sequence. `Choose` carries an
    /// option *index* (matching `Session::choose`'s actual signature, a
    /// `usize` position in the branch point's `options` array — not a key
    /// string), deliberately including out-of-range indices so illegal
    /// `choose` calls are exercised, not just legal ones.
    #[derive(Debug, Clone)]
    pub(super) enum SessionOp {
        Next,
        Choose(usize),
        Goto(String),
        Back,
    }

    fn arbitrary_op(ids: Vec<String>) -> impl Strategy<Value = SessionOp> {
        prop_oneof![
            3 => Just(SessionOp::Next),
            2 => (0usize..4).prop_map(SessionOp::Choose),
            2 => arbitrary_target(ids).prop_map(SessionOp::Goto),
            1 => Just(SessionOp::Back),
        ]
    }

    /// A target id: usually one that exists in the graph, occasionally a
    /// deliberately unknown one — `goto`/traversal targets must be
    /// exercised against both to prove `UnknownNode` never corrupts state.
    fn arbitrary_target(existing: Vec<String>) -> impl Strategy<Value = String> {
        if existing.is_empty() {
            return Just("missing".to_owned()).boxed();
        }
        prop_oneof![
            4 => proptest::sample::select(existing),
            1 => Just("does-not-exist".to_owned()),
        ]
        .boxed()
    }

    /// A small, navigable graph: `n` nodes with unique, predictable ids
    /// (`"n0".."n{n-1}"`), each with either no traversal (terminal), a
    /// `next` edge, or a branch point with 1-3 options — targets drawn
    /// from `arbitrary_target` so both resolvable and dangling edges
    /// appear. Content is deliberately empty: this generator exercises
    /// history/visited invariants, not reveal semantics (already covered
    /// by the hand-written reveal tests above).
    fn arbitrary_graph_of_size(n: usize) -> impl Strategy<Value = Graph> {
        let ids: Vec<String> = (0..n).map(|i| format!("n{i}")).collect();
        let node_strategies: Vec<_> = ids
            .iter()
            .map(|id| arbitrary_node(id.clone(), ids.clone()))
            .collect();
        node_strategies.prop_map(|nodes| Graph {
            fireside_version: None,
            title: None,
            author: None,
            date: None,
            description: None,
            version: None,
            defaults: None,
            nodes,
        })
    }

    fn arbitrary_node(id: String, ids: Vec<String>) -> impl Strategy<Value = Node> {
        let traversal = prop_oneof![
            1 => Just(None),
            3 => arbitrary_target(ids.clone())
                .prop_map(|target| Some(TraversalSpec::Target(target))),
            3 => arbitrary_target(ids.clone()).prop_map(|next| Some(TraversalSpec::Rules(
                Traversal { next: Some(next), branch_point: None }
            ))),
            2 => vec(arbitrary_branch_option(ids.clone()), 1..4).prop_map(|options| Some(
                TraversalSpec::Rules(Traversal {
                    next: None,
                    branch_point: Some(BranchPoint { prompt: None, options }),
                })
            )),
        ];
        traversal.prop_map(move |traversal| Node {
            id: id.clone(),
            title: None,
            view_mode: None,
            transition: None,
            speaker_notes: None,
            traversal,
            content: Vec::new(),
        })
    }

    fn arbitrary_branch_option(ids: Vec<String>) -> impl Strategy<Value = BranchOption> {
        arbitrary_target(ids).prop_map(|target| BranchOption {
            label: "option".to_owned(),
            key: None,
            target,
            description: None,
        })
    }

    /// An arbitrary valid `(Graph, Vec<SessionOp>)` pair: 1-8 nodes, 0-30
    /// operations drawn against that graph's actual (and occasionally
    /// fictitious) node ids.
    pub(super) fn arbitrary_graph_and_ops() -> impl Strategy<Value = (Graph, Vec<SessionOp>)> {
        (1usize..8).prop_flat_map(|n| {
            let ids: Vec<String> = (0..n).map(|i| format!("n{i}")).collect();
            (arbitrary_graph_of_size(n), vec(arbitrary_op(ids), 0..30))
        })
    }

    /// Like `arbitrary_node`, but with 0-3 leaf blocks each independently
    /// marked `reveal: None` or a small positive level — small range
    /// keeps `reveal_levels()` short so shrunk failures stay readable.
    /// Traversal/targets are unchanged from `arbitrary_node`; only
    /// content differs, since reveal-gating is a property of a node's
    /// content, not its edges.
    fn arbitrary_reveal_node(id: String, ids: Vec<String>) -> impl Strategy<Value = Node> {
        let content = vec(
            option::of(0u32..4).prop_map(|reveal| ContentBlock::Divider { reveal }),
            0..3,
        );
        (arbitrary_node(id, ids), content).prop_map(|(mut node, content)| {
            node.content = content;
            node
        })
    }

    /// Like `arbitrary_graph_and_ops`, but nodes carry reveal marks — for
    /// exercising reveal-gating invariants (`next` pausing on unrevealed
    /// content, `back` undoing a moving `next`) rather than the
    /// history/visited bookkeeping `arbitrary_graph_and_ops` targets.
    pub(super) fn arbitrary_reveal_graph_and_ops() -> impl Strategy<Value = (Graph, Vec<SessionOp>)>
    {
        (1usize..8).prop_flat_map(|n| {
            let ids: Vec<String> = (0..n).map(|i| format!("n{i}")).collect();
            let node_strategies: Vec<_> = ids
                .iter()
                .map(|id| arbitrary_reveal_node(id.clone(), ids.clone()))
                .collect();
            let graph = node_strategies.prop_map(|nodes| Graph {
                fireside_version: None,
                title: None,
                author: None,
                date: None,
                description: None,
                version: None,
                defaults: None,
                nodes,
            });
            (graph, vec(arbitrary_op(ids), 0..30))
        })
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

    fn session_from(json: &str) -> Session {
        Session::new(Graph::from_json(json).expect("fixture parses")).expect("non-empty")
    }

    #[test]
    fn next_reveals_one_distinct_step_at_a_time_before_moving() {
        let mut s = session_from(
            r#"{"nodes":[
                {"id":"a","traversal":"b","content":[
                    {"kind":"text","body":"x","reveal":1},
                    {"kind":"text","body":"y","reveal":2}
                ]},
                {"id":"b","content":[]}
            ]}"#,
        );
        assert_eq!(s.next(), Outcome::Revealed);
        assert_eq!(s.current().id, "a", "reveal does not navigate");
        assert_eq!(s.next(), Outcome::Revealed);
        assert_eq!(s.current().id, "a");
        assert_eq!(s.next(), Outcome::Moved);
        assert_eq!(s.current().id, "b");
    }

    #[test]
    fn next_skips_gaps_in_reveal_numbering_without_a_dead_step() {
        let mut s = session_from(
            r#"{"nodes":[
                {"id":"a","content":[
                    {"kind":"text","body":"x","reveal":1},
                    {"kind":"text","body":"y","reveal":5}
                ]}
            ]}"#,
        );
        assert_eq!(s.next(), Outcome::Revealed);
        assert_eq!(s.next(), Outcome::Revealed);
        assert_eq!(
            s.next(),
            Outcome::EndOfPath,
            "exactly two reveal steps, no dead step for the gap between 1 and 5"
        );
    }

    #[test]
    fn next_reveals_before_blocking_on_branch_point() {
        let mut s = session_from(
            r#"{"nodes":[
                {"id":"a","traversal":{"branch-point":{"options":[{"label":"x","target":"b"}]}},"content":[
                    {"kind":"text","body":"x","reveal":1}
                ]},
                {"id":"b","content":[]}
            ]}"#,
        );
        assert_eq!(s.next(), Outcome::Revealed);
        assert_eq!(s.next(), Outcome::BlockedByBranch);
    }

    #[test]
    fn next_reveals_before_reporting_end_of_path() {
        let mut s = session_from(
            r#"{"nodes":[
                {"id":"a","content":[
                    {"kind":"text","body":"x","reveal":1}
                ]}
            ]}"#,
        );
        assert_eq!(s.next(), Outcome::Revealed);
        assert_eq!(s.next(), Outcome::EndOfPath);
    }

    #[test]
    fn reveal_resets_on_every_node_entry_including_back() {
        let mut s = session_from(
            r#"{"nodes":[
                {"id":"a","traversal":"b","content":[
                    {"kind":"text","body":"x","reveal":1}
                ]},
                {"id":"b","content":[]}
            ]}"#,
        );
        s.next(); // reveal
        s.next(); // move to b
        assert_eq!(s.current().id, "b");
        s.back();
        assert_eq!(s.current().id, "a");
        assert!(
            s.has_pending_reveal(),
            "reveal is not remembered across visits"
        );
        assert_eq!(s.reveal_progress(), Some((0, 1)));
    }

    #[test]
    fn reveal_progress_is_none_for_ordinary_nodes() {
        let s = hello_session();
        assert_eq!(s.reveal_progress(), None);
        assert!(!s.has_pending_reveal());
    }

    #[test]
    fn choose_is_rejected_while_reveal_is_pending() {
        let mut s = session_from(
            r#"{"nodes":[
                {"id":"a","traversal":{"branch-point":{"options":[{"label":"x","target":"b"}]}},"content":[
                    {"kind":"text","body":"x","reveal":1}
                ]},
                {"id":"b","content":[]}
            ]}"#,
        );
        assert_eq!(
            s.choose(0),
            Outcome::InvalidChoice,
            "reveal not yet exhausted"
        );
        assert_eq!(s.current().id, "a");
        s.next(); // consume the reveal step
        assert_eq!(s.choose(0), Outcome::Moved, "now selectable");
    }

    proptest::proptest! {
        /// For any valid graph and any sequence of legal-or-illegal
        /// navigation operations, `Session::history()` always exactly
        /// matches the path actually walked so far *excluding* the
        /// current node (per `move_to`'s contract: history holds prior
        /// nodes, the ones `back()` can return to — `current()` is always
        /// one step ahead of `history().last()`), and every node ever
        /// reported as visited is a real node in the graph (spec 008 US1,
        /// FR-002/FR-003). Illegal ops (`choose` with no branch point,
        /// `goto` to a nonexistent id, `back` with empty history) are
        /// deliberately included in the generated sequence and asserted
        /// to leave both `history()` and `current()` untouched, per the
        /// "failed operations never mutate history" invariant documented
        /// at the top of this module.
        #[test]
        fn session_history_and_visited_stay_truthful(
            (graph, ops) in proptest_support::arbitrary_graph_and_ops()
        ) {
            let node_ids: std::collections::HashSet<String> =
                graph.nodes.iter().map(|n| n.id.clone()).collect();
            let mut session = Session::new(graph).expect("generator always produces >=1 node");

            // `path` mirrors the full sequence of nodes entered, including
            // the starting entry node — `history()` is always `path` minus
            // its last element, and `current()` is always `path`'s last.
            let mut path = vec![session.current().id.clone()];

            for op in ops {
                let before_history = session.history().to_vec();
                let before_current = session.current().id.clone();

                let moved = match op {
                    proptest_support::SessionOp::Next => session.next() == Outcome::Moved,
                    proptest_support::SessionOp::Choose(i) => session.choose(i) == Outcome::Moved,
                    proptest_support::SessionOp::Goto(ref target) => {
                        session.goto(target) == Outcome::Moved
                    }
                    proptest_support::SessionOp::Back => {
                        let was_back = session.back() == Outcome::Moved;
                        if was_back {
                            path.pop();
                        }
                        was_back
                    }
                };

                // `Back`'s path bookkeeping already happened above (it
                // pops rather than pushes); every other op that moved
                // pushes the new current node.
                if moved && !matches!(op, proptest_support::SessionOp::Back) {
                    path.push(session.current().id.clone());
                }

                if !moved {
                    proptest::prop_assert_eq!(
                        session.history(),
                        before_history.as_slice(),
                        "a non-moving op must not touch history"
                    );
                    proptest::prop_assert_eq!(
                        &session.current().id,
                        &before_current,
                        "a non-moving op must not change the current node"
                    );
                }

                let expected_history = &path[..path.len() - 1];
                proptest::prop_assert_eq!(session.history(), expected_history);
                proptest::prop_assert_eq!(&session.current().id, path.last().expect("non-empty"));

                for id in session.visited() {
                    proptest::prop_assert!(
                        node_ids.contains(id),
                        "visited node {id} is not a real node in the graph"
                    );
                }
            }
        }

        /// Reveal-gating invariants under an arbitrary op sequence on
        /// reveal-bearing content: (1) `reveal_level()` is always either
        /// `0` or one of the current node's own `reveal_levels()` — never
        /// a value the node doesn't actually declare, even across
        /// navigation that resets it; (2) calling `next()` while a
        /// reveal is pending returns `Revealed`, never `Moved`, and
        /// leaves `current()` unchanged — the node itself does not
        /// advance until every reveal step is exhausted (spec FR-007);
        /// (3) whenever `next()` does move (`Outcome::Moved`), an
        /// immediate `back()` returns to that exact node id — `back`
        /// inverts a moving `next`, though not necessarily its reveal
        /// progress, since re-entering any node always resets
        /// `reveal_level` to `0` by design.
        #[test]
        fn reveal_state_stays_valid_and_next_back_are_consistent(
            (graph, ops) in proptest_support::arbitrary_reveal_graph_and_ops()
        ) {
            let mut session = Session::new(graph).expect("generator always produces >=1 node");

            for op in ops {
                let pending_before = session.has_pending_reveal();
                let id_before = session.current().id.clone();

                if matches!(op, proptest_support::SessionOp::Next) {
                    let outcome = session.next();
                    if pending_before {
                        proptest::prop_assert_eq!(
                            outcome,
                            Outcome::Revealed,
                            "next() with a pending reveal must reveal, not do anything else"
                        );
                        proptest::prop_assert_eq!(
                            &session.current().id,
                            &id_before,
                            "a revealing next() must not change the current node"
                        );
                    } else if outcome == Outcome::Moved {
                        proptest::prop_assert_eq!(
                            session.back(),
                            Outcome::Moved,
                            "back() must undo a moving next()"
                        );
                        proptest::prop_assert_eq!(
                            &session.current().id,
                            &id_before,
                            "back() after a moving next() must return to the same node"
                        );
                    }
                } else {
                    match op {
                        proptest_support::SessionOp::Choose(i) => {
                            session.choose(i);
                        }
                        proptest_support::SessionOp::Goto(ref target) => {
                            session.goto(target);
                        }
                        proptest_support::SessionOp::Back => {
                            session.back();
                        }
                        proptest_support::SessionOp::Next => unreachable!("handled above"),
                    }
                }

                let levels = session.current().reveal_levels();
                proptest::prop_assert!(
                    session.reveal_level() == 0 || levels.contains(&session.reveal_level()),
                    "reveal_level {} is not 0 and not among the current node's own levels {levels:?}",
                    session.reveal_level()
                );
            }
        }
    }
}
