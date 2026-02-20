//! Engine-level error types.
//!
//! Covers validation failures, traversal errors, and session errors.
//! Protocol-level (parse/format) errors are in `fireside-core`.

/// Errors produced by the Fireside engine.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// A core-level error (parsing, deserialization).
    #[error(transparent)]
    Core(#[from] fireside_core::error::CoreError),

    /// A node references a non-existent node ID.
    #[error("dangling node reference: {0}")]
    DanglingReference(String),

    /// The graph has no start node (empty graph).
    #[error("graph has no start node")]
    NoStartNode,

    /// An invalid traversal was attempted.
    #[error("invalid traversal: {0}")]
    InvalidTraversal(String),

    /// A command could not be applied to the current session.
    #[error("command error: {0}")]
    CommandError(String),

    /// A path traversal attempt was detected.
    #[error("path traversal detected: {0}")]
    PathTraversal(String),
}
