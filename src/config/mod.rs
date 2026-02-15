//! Configuration system for Slideways.
//!
//! Manages settings, keybindings, and theme loading.

pub mod keybindings;
pub mod settings;

use std::path::Path;

use anyhow::Result;

use crate::error::ConfigError;
use crate::model::theme::{Theme, ThemeFile};

/// Load a theme from a TOML file path.
///
/// Parses the TOML and overlays it on top of the default theme.
///
/// # Errors
///
/// Returns `ConfigError::ThemeRead` if the file cannot be read,
/// or `ConfigError::InvalidTheme` if the TOML is malformed.
pub fn load_theme(path: &Path) -> Result<Theme> {
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::ThemeRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    let theme_file: ThemeFile = toml::from_str(&content).map_err(ConfigError::InvalidTheme)?;
    Ok(theme_file.apply_to(&Theme::default()))
}

/// Resolve a theme by name, looking in the themes/ directory relative to the binary.
///
/// Falls back to the default theme if the named theme is not found.
#[must_use]
pub fn resolve_theme(name: Option<&str>) -> Theme {
    let Some(name) = name else {
        return Theme::default();
    };

    // Try to find the theme file
    let candidates = [format!("themes/{name}.toml"), format!("themes/{name}")];

    for candidate in &candidates {
        let path = Path::new(candidate);
        if path.exists()
            && let Ok(theme) = load_theme(path)
        {
            return theme;
        }
    }

    tracing::warn!("Theme '{name}' not found, using default");
    Theme::default()
}
