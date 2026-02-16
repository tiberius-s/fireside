//! Per-node traversal overrides.
//!
//! Maps to the protocol `Traversal` type. Uses kebab-case wire format
//! (e.g., `"branch-point"` in JSON maps to `branch_point` in Rust).

use serde::{Deserialize, Serialize};

use super::branch::BranchPoint;
use super::node::NodeId;

/// Per-node traversal control.
///
/// Allows a node to override the default sequential traversal with
/// explicit next targets, backtrack points, or branch points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Traversal {
    /// Override the next node by ID (instead of sequential advance).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<NodeId>,

    /// Node ID to return to after completing a branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<NodeId>,

    /// Branch point definition, if this node presents choices.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "branch-point"
    )]
    pub branch_point: Option<BranchPoint>,
}
