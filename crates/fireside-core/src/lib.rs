//! Fireside Core — protocol model, wire-format serialization, and type invariants.
//!
//! This crate defines the canonical Fireside data model types matching the
//! Fireside 0.1.0 protocol specification. All wire-format JSON uses kebab-case
//! property names and the `"kind"` discriminator for content blocks.
//!
//! # Crate Boundaries
//!
//! - **No UI dependencies** — this crate is intentionally UI-agnostic.
//! - **No validation logic** — graph integrity checks live in `fireside-engine`.
//! - **Serde only** — serialization targets JSON wire format.

pub mod error;
pub mod model;

pub use model::branch::{BranchOption, BranchPoint};
pub use model::content::{ContentBlock, ListItem};
pub use model::graph::{Graph, GraphFile, GraphMeta, NodeDefaults};
pub use model::layout::Layout;
pub use model::node::{Node, NodeId};
pub use model::transition::Transition;
pub use model::traversal::Traversal;
