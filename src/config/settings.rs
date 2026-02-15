//! Application settings.

use serde::Deserialize;

/// Top-level application settings, loaded from config file or defaults.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Default theme name.
    pub theme: Option<String>,
    /// Poll timeout for event loop in milliseconds.
    pub poll_timeout_ms: u64,
    /// Whether to show the progress bar.
    pub show_progress: bool,
    /// Whether to show elapsed time in the progress bar.
    pub show_timer: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: None,
            poll_timeout_ms: 250,
            show_progress: true,
            show_timer: true,
        }
    }
}
