//! Core error types for protocol-level failures.
//!
//! These cover deserialization, malformed fields, and type invariant violations.
//! Application-level and validation errors live in `fireside-engine`.

use std::path::PathBuf;

/// Errors that can occur when parsing a Fireside document.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// The file could not be read from disk.
    #[error("failed to read file: {path}")]
    FileRead {
        /// Path that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The JSON document could not be deserialized.
    #[error("invalid JSON: {0}")]
    InvalidJson(String),

    /// The document contains no nodes.
    #[error("graph contains no nodes")]
    EmptyGraph,

    /// A duplicate node ID was found.
    #[error("duplicate node id: {0}")]
    DuplicateNodeId(String),
}
