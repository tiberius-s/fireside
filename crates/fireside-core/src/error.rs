//! Typed errors for the core crate.

use thiserror::Error;

/// Errors produced while reading a Fireside document.
#[derive(Debug, Error)]
pub enum CoreError {
    /// The text is not valid JSON, or its shape does not match the
    /// protocol data model.
    #[error("not a valid Fireside document: {0}")]
    Parse(#[from] serde_json::Error),
}
