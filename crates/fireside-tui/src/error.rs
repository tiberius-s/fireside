//! Typed errors for the TUI crate.

use thiserror::Error;

/// Errors produced while running the presenter.
#[derive(Debug, Error)]
pub enum TuiError {
    /// The session could not be created.
    #[error(transparent)]
    Engine(#[from] fireside_engine::EngineError),

    /// Terminal I/O failed.
    #[error("terminal error: {0}")]
    Io(#[from] std::io::Error),

    /// stdin/stdout is not an interactive terminal (piped, redirected, or
    /// otherwise non-tty) — presenting needs a real terminal to read keys
    /// and draw frames.
    #[error(
        "fireside needs an interactive terminal to present — run it directly in your terminal, not through a pipe."
    )]
    NotATty,
}
