//! Fireside core — the protocol data model, nothing more.
//!
//! This crate is a faithful Rust mirror of the Fireside 0.1.0 protocol
//! (`protocol/main.tsp` and its generated JSON schemas): parsing,
//! serialization, and small read-time helpers (traversal accessors and
//! default resolution). It performs no I/O, holds no state, and contains
//! no rendering or validation logic — semantic validation lives in
//! `fireside-engine`, presentation in `fireside-tui`.

pub mod error;
pub mod model;

pub use error::CoreError;
pub use model::{
    BranchOption, BranchPoint, ContainerLayout, ContentBlock, Graph, Node, NodeDefaults, NodeId,
    Transition, Traversal, TraversalSpec, ViewMode,
};
