//! Core data model types for the Fireside protocol.
//!
//! This module defines the structural types that represent a Fireside graph:
//! nodes, content blocks, layouts, transitions, traversal, and branching.
//! All types are JSON-native with serde derives using kebab-case wire format.

pub mod branch;
pub mod content;
pub mod graph;
pub mod layout;
pub mod node;
pub mod transition;
pub mod traversal;
