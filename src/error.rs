//! Error types for the Slideways application.
//!
//! Uses `thiserror` for library-level error types with meaningful variants.
//! Application boundaries use `anyhow` for ergonomic error propagation.

use std::path::PathBuf;

/// Errors that can occur during slide deck parsing.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// The slide file could not be read from disk.
    #[error("failed to read slide file: {path}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// The JSON presentation file could not be deserialized.
    #[error("invalid presentation JSON: {0}")]
    InvalidJson(String),

    /// The slide file contains no slides.
    #[error("presentation contains no slides")]
    EmptyDeck,

    /// A slide references a non-existent slide ID.
    #[error("unknown slide id: {0}")]
    UnknownSlideId(String),

    /// Duplicate slide ID found.
    #[error("duplicate slide id: {0}")]
    DuplicateSlideId(String),
}

/// Errors that can occur during rendering.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// An I/O error occurred during terminal rendering.
    #[error("terminal I/O error")]
    Io(#[from] std::io::Error),

    /// An image file could not be loaded.
    #[error("failed to load image: {path}")]
    ImageLoad {
        path: PathBuf,
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
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// A theme file contains invalid TOML.
    #[error("invalid theme file")]
    InvalidTheme(#[from] toml::de::Error),

    /// The specified theme was not found.
    #[error("unknown theme: {0}")]
    UnknownTheme(String),
}
