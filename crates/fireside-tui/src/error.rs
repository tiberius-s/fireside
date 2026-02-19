//! TUI error types for rendering and configuration.

use std::path::PathBuf;

/// Errors that can occur during rendering.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// An I/O error occurred during terminal rendering.
    #[error("terminal I/O error")]
    Io(#[from] std::io::Error),

    /// An image file could not be loaded.
    #[error("failed to load image: {path}")]
    ImageLoad {
        /// Path to the image file.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

/// Errors that can occur in the configuration system.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A theme file could not be read.
    #[error("failed to read theme file: {path}")]
    ThemeRead {
        /// Path to the theme file.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// A theme file contains invalid JSON.
    #[error("invalid theme file")]
    InvalidTheme(#[from] serde_json::Error),

    /// The specified theme was not found.
    #[error("unknown theme: {0}")]
    UnknownTheme(String),
}
