//! Branching path types for interactive presentations.
//!
//! A branch point presents the audience with choices that determine
//! which node to traverse to next.

use serde::{Deserialize, Serialize};

use super::node::NodeId;

/// A branch point where the audience chooses a path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchPoint {
    /// Unique identifier for this branch point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Text displayed above the options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// The available options to choose from.
    pub options: Vec<BranchOption>,
}

/// A single choice option at a branch point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchOption {
    /// Display label for this option.
    pub label: String,
    /// Keyboard shortcut key (e.g. `'a'`, `'b'`, `'c'`).
    pub key: char,
    /// The node to navigate to when this option is selected.
    pub target: NodeId,
}
