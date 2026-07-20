//! The `fireside notes` follower's state (spec 012).
//!
//! Deliberately separate from [`crate::app::App`]: the follower never
//! navigates, edits, or mutates anything — it only ever resolves a
//! presenter-reported node id against its own loaded [`Graph`], which needs
//! no [`fireside_engine::Session`] at all. Grafting a "read-only mode" onto
//! `App` would mean auditing every one of its many key-handling branches to
//! confirm none of them can fire here; a new, small, obviously-simple state
//! type has no such surface. TEA invariant preserved: [`Follower::update`]
//! is the only place this state mutates.

use crossterm::event::{Event, KeyCode, KeyEventKind};
use fireside_core::Graph;

use crate::SessionStatus;

/// A message into the follower's state machine: terminal input, a fresh
/// read of the deck file (live reload, same shape as the presenter's own
/// `Msg::Reload`), or the next poll of the presenter's session file.
#[derive(Debug)]
pub(crate) enum FollowerMsg {
    /// A terminal event (key press, resize).
    Terminal(Event),
    /// The deck file changed on disk and was re-read: a new graph, or a
    /// human-readable message about why it could not be loaded.
    Reload(Result<Graph, String>),
    /// The latest poll of the presenter's live session-state file.
    SessionUpdate(SessionStatus),
}

/// What the follower currently has to show, derived fresh on every render
/// from `Follower`'s state (data-model.md's "Follower" section) — never
/// stored, so there is exactly one place ("not running" wins over
/// everything else) that decides precedence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FollowerView<'a> {
    /// No presenter has reported in (never started, exited cleanly, or
    /// crashed — indistinguishable by design).
    NotRunning,
    /// The presenter reported a node id this follower's current graph
    /// doesn't have — a brief reload-skew race between the two processes.
    Waiting,
    /// The presenter is running and its reported node resolved cleanly.
    Tracking {
        /// The current node's title, if it has one.
        title: Option<&'a str>,
        /// The current node's speaker notes, if any.
        notes: Option<&'a str>,
        /// What comes after this node.
        next: NextView<'a>,
        /// `(revealed, total)`, when the node has any reveal steps.
        reveal: Option<(usize, usize)>,
        /// Wall-clock time since the presentation started, in whole
        /// seconds, as last reported by the presenter.
        elapsed_secs: u64,
    },
}

/// What a follower shows for "what's next" — a single title, the branch
/// options at a choice point, or an explicit end-of-path marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NextView<'a> {
    /// The next node's title (or its id, if it has no title).
    Title(&'a str),
    /// The presenter is at a branch point: label + optional key per option.
    Branch(Vec<(&'a str, Option<&'a str>)>),
    /// This node is terminal with no branch — there is no next slide.
    LastSlide,
}

/// All follower state.
#[derive(Debug)]
pub(crate) struct Follower {
    graph: Graph,
    status: SessionStatus,
    quit: bool,
}

impl Follower {
    /// Create the follower over its own loaded copy of the deck.
    #[must_use]
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            status: SessionStatus::NotRunning,
            quit: false,
        }
    }

    /// The only place this state mutates (TEA, Constitution Principle IV).
    pub fn update(&mut self, msg: FollowerMsg) {
        match msg {
            FollowerMsg::Terminal(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                    || (key.code == KeyCode::Char('c')
                        && key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL))
                {
                    self.quit = true;
                }
            }
            FollowerMsg::Terminal(_) => {}
            FollowerMsg::Reload(Ok(graph)) => self.graph = graph,
            FollowerMsg::Reload(Err(_)) => {
                // A reload failure (e.g. a transient truncated save) leaves
                // the follower's last-known-good graph in place — same
                // posture as a momentary parse error self-healing on the
                // next successful poll; there is nothing actionable for a
                // read-only follower to do with the message itself.
            }
            FollowerMsg::SessionUpdate(status) => self.status = status,
        }
    }

    /// Whether the event loop should exit.
    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.quit
    }

    /// The current, derived view — resolves the latest session status
    /// against the currently loaded graph. `NotRunning` wins over
    /// everything else; a `Running` snapshot naming a node absent from
    /// this follower's graph (reload skew) is `Waiting`, never a panic.
    #[must_use]
    pub fn view(&self) -> FollowerView<'_> {
        let SessionStatus::Running(snapshot) = &self.status else {
            return FollowerView::NotRunning;
        };
        let Some(node) = self.graph.node(&snapshot.node_id) else {
            return FollowerView::Waiting;
        };
        let next = if let Some(branch) = node.branch_point() {
            NextView::Branch(
                branch
                    .options
                    .iter()
                    .map(|option| (option.label.as_str(), option.key.as_deref()))
                    .collect(),
            )
        } else if let Some(target) = node.next_target() {
            NextView::Title(
                self.graph
                    .node(target)
                    .and_then(|n| n.title.as_deref())
                    .unwrap_or(target),
            )
        } else {
            NextView::LastSlide
        };
        FollowerView::Tracking {
            title: node.title.as_deref(),
            notes: node.speaker_notes.as_deref(),
            next,
            reveal: (snapshot.reveal_total > 0)
                .then_some((snapshot.reveal_step, snapshot.reveal_total)),
            elapsed_secs: snapshot.elapsed.as_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SessionSnapshot;
    use std::time::Duration;

    const HELLO: &str = include_str!("../../../docs/examples/hello.json");

    fn follower() -> Follower {
        Follower::new(Graph::from_json(HELLO).expect("hello parses"))
    }

    fn running(node_id: &str) -> SessionStatus {
        SessionStatus::Running(SessionSnapshot {
            node_id: node_id.to_owned(),
            reveal_step: 0,
            reveal_total: 0,
            elapsed: Duration::from_secs(90),
        })
    }

    #[test]
    fn starts_not_running_before_any_session_update() {
        assert_eq!(follower().view(), FollowerView::NotRunning);
    }

    #[test]
    fn q_key_quits() {
        let mut f = follower();
        f.update(FollowerMsg::Terminal(Event::Key(
            crossterm::event::KeyEvent::new(
                KeyCode::Char('q'),
                crossterm::event::KeyModifiers::NONE,
            ),
        )));
        assert!(f.should_quit());
    }

    #[test]
    fn a_running_snapshot_for_an_unknown_node_id_is_waiting_not_a_panic() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running("no-such-node")));
        assert_eq!(f.view(), FollowerView::Waiting);
    }

    #[test]
    fn a_running_snapshot_for_a_known_node_resolves_to_tracking() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running("features")));
        match f.view() {
            FollowerView::Tracking { elapsed_secs, .. } => assert_eq!(elapsed_secs, 90),
            other => panic!("expected Tracking, got {other:?}"),
        }
    }

    #[test]
    fn reload_swaps_the_graph_and_is_reflected_on_the_next_view() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running("features")));
        let edited = {
            let mut graph = Graph::from_json(HELLO).expect("hello parses");
            let node = graph
                .nodes
                .iter_mut()
                .find(|n| n.id == "features")
                .expect("features node");
            node.speaker_notes = Some("edited live".to_owned());
            graph
        };
        f.update(FollowerMsg::Reload(Ok(edited)));
        match f.view() {
            FollowerView::Tracking { notes, .. } => assert_eq!(notes, Some("edited live")),
            other => panic!("expected Tracking, got {other:?}"),
        }
    }

    #[test]
    fn a_failed_reload_keeps_the_last_known_good_graph() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running("features")));
        f.update(FollowerMsg::Reload(Err("truncated".to_owned())));
        assert!(
            matches!(f.view(), FollowerView::Tracking { .. }),
            "a failed reload must not drop the last-known-good graph: {:?}",
            f.view()
        );
    }

    #[test]
    fn not_running_wins_over_a_previously_tracked_position() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running("features")));
        f.update(FollowerMsg::SessionUpdate(SessionStatus::NotRunning));
        assert_eq!(f.view(), FollowerView::NotRunning);
    }
}
