//! Node types — a single node within a Fireside graph.
//!
//! Maps to the protocol `Node` type with kebab-case wire format.

use serde::{Deserialize, Serialize};

use super::content::ContentBlock;
use super::layout::Layout;
use super::transition::Transition;
use super::traversal::Traversal;

/// A unique identifier for a node, used for traversal targets.
pub type NodeId = String;

/// A single node within a Fireside graph.
///
/// Deserialized from the `nodes` array in a Fireside JSON document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Optional unique identifier for this node (used for traversal targets).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<NodeId>,

    /// Layout variant for this node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,

    /// Transition effect when entering this node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition: Option<Transition>,

    /// Speaker notes (not rendered in the audience view).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "speaker-notes"
    )]
    pub speaker_notes: Option<String>,

    /// Traversal overrides and branching for this node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub traversal: Option<Traversal>,

    /// The ordered list of content blocks that make up this node.
    pub content: Vec<ContentBlock>,
}

impl Node {
    /// Get the next-node traversal override, if any.
    #[must_use]
    pub fn next_override(&self) -> Option<&str> {
        self.traversal.as_ref().and_then(|t| t.next.as_deref())
    }

    /// Get the branch point, if any.
    #[must_use]
    pub fn branch_point(&self) -> Option<&super::branch::BranchPoint> {
        self.traversal
            .as_ref()
            .and_then(|t| t.branch_point.as_ref())
    }

    /// Get the after (backtrack) target, if any.
    #[must_use]
    pub fn after_target(&self) -> Option<&str> {
        self.traversal.as_ref().and_then(|t| t.after.as_deref())
    }
}
