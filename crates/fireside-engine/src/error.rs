//! Typed errors for the engine crate.

use thiserror::Error;

/// Errors produced when constructing a presentation session.
#[derive(Debug, Error)]
pub enum EngineError {
    /// A session needs at least one node to present.
    #[error("graph has no nodes")]
    EmptyGraph,
}
