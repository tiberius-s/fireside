//! Slideways project configuration.
//!
//! A Slideways project is a directory containing a `slideways.yml` config
//! file that maps to a collection of markdown slide files.
//!
//! ## Example `slideways.yml`
//!
//! ```yaml
//! name: "My Conference Talk"
//! author: "Jane Developer"
//! date: "2026-02-14"
//! theme: "one-dark"
//! font: "JetBrains Mono"
//!
//! slides:
//!   - title.md
//!   - intro.md
//!   - demo.md
//!   - conclusion.md
//!
//! options:
//!   auto_advance: false
//!   show_progress: true
//!   show_timer: true
//! ```
//!
//! ## Single-file fallback
//!
//! When a single `.md` file is provided instead of a project directory,
//! Slideways treats it as a simple deck (current behavior) with no
//! project-level config.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Errors when loading a project configuration.
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    /// The project directory does not contain a `slideways.yml`.
    #[error("no slideways.yml found in {0}")]
    NoConfig(PathBuf),

    /// Could not read the config file.
    #[error("failed to read slideways.yml: {0}")]
    Io(#[from] std::io::Error),

    /// Could not parse the YAML config.
    #[error("failed to parse slideways.yml: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

/// A Slideways project configuration loaded from `slideways.yml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    /// Project display name.
    pub name: String,
    /// Author name.
    pub author: Option<String>,
    /// Date string (ISO 8601 recommended).
    pub date: Option<String>,
    /// Theme name or path to `.itermcolors` / `.toml` theme file.
    pub theme: Option<String>,
    /// Recommended monospace font family name.
    pub font: Option<String>,
    /// Ordered list of slide files (relative to project dir).
    pub slides: Vec<String>,
    /// Presentation options.
    pub options: ProjectOptions,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: String::from("Untitled Presentation"),
            author: None,
            date: None,
            theme: None,
            font: None,
            slides: Vec::new(),
            options: ProjectOptions::default(),
        }
    }
}

/// Presentation behavior options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectOptions {
    /// Auto-advance slides after a delay (seconds). 0 = disabled.
    pub auto_advance: u16,
    /// Show the progress bar footer.
    pub show_progress: bool,
    /// Show elapsed time in the progress bar.
    pub show_timer: bool,
    /// Enable mouse interaction.
    pub mouse_enabled: bool,
}

impl Default for ProjectOptions {
    fn default() -> Self {
        Self {
            auto_advance: 0,
            show_progress: true,
            show_timer: true,
            mouse_enabled: true,
        }
    }
}

/// Resolved project — a loaded config with its root directory.
#[derive(Debug, Clone)]
pub struct Project {
    /// The project root directory.
    pub root: PathBuf,
    /// The loaded configuration.
    pub config: ProjectConfig,
}

impl Project {
    /// Load a project from a directory containing `slideways.yml`.
    ///
    /// # Errors
    ///
    /// Returns `ProjectError` if the config file is missing or malformed.
    pub fn load(dir: &Path) -> Result<Self, ProjectError> {
        let config_path = dir.join("slideways.yml");
        if !config_path.exists() {
            return Err(ProjectError::NoConfig(dir.to_path_buf()));
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: ProjectConfig = serde_yaml::from_str(&content)?;

        Ok(Self {
            root: dir.to_path_buf(),
            config,
        })
    }

    /// Get the absolute paths to all slide files in order.
    #[must_use]
    pub fn slide_paths(&self) -> Vec<PathBuf> {
        self.config
            .slides
            .iter()
            .map(|s| self.root.join(s))
            .collect()
    }

    /// Check if a path is a Slideways project directory.
    #[must_use]
    pub fn is_project_dir(path: &Path) -> bool {
        path.is_dir() && path.join("slideways.yml").exists()
    }

    /// Scaffold a new project directory with default config.
    ///
    /// # Errors
    ///
    /// Returns IO error if directory/file creation fails.
    pub fn scaffold(dir: &Path, name: &str) -> Result<Self, ProjectError> {
        std::fs::create_dir_all(dir)?;

        let config = ProjectConfig {
            name: name.to_owned(),
            slides: vec!["slides/01-title.md".to_owned()],
            ..Default::default()
        };

        let yaml = serde_yaml::to_string(&config).map_err(ProjectError::Yaml)?;
        std::fs::write(dir.join("slideways.yml"), yaml)?;

        // Create slides directory and initial slide
        let slides_dir = dir.join("slides");
        std::fs::create_dir_all(&slides_dir)?;

        let title_slide =
            format!("---\ntemplate: title\n---\n\n## {name}\n\nYour presentation starts here\n");
        std::fs::write(slides_dir.join("01-title.md"), title_slide)?;

        // Create themes directory
        std::fs::create_dir_all(dir.join("themes"))?;

        Ok(Self {
            root: dir.to_path_buf(),
            config,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid_yaml() {
        let config = ProjectConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("Untitled Presentation"));
    }

    #[test]
    fn roundtrip_config() {
        let config = ProjectConfig {
            name: "Test Talk".into(),
            author: Some("Test Author".into()),
            theme: Some("gruvbox-dark".into()),
            font: Some("JetBrains Mono".into()),
            slides: vec!["intro.md".into(), "demo.md".into()],
            ..Default::default()
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: ProjectConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.name, "Test Talk");
        assert_eq!(parsed.slides.len(), 2);
        assert_eq!(parsed.font.as_deref(), Some("JetBrains Mono"));
    }

    #[test]
    fn is_not_project_dir_for_file() {
        assert!(!Project::is_project_dir(Path::new("/tmp/nonexistent")));
    }
}
