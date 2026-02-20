//! Application settings.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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

/// Load application settings from config files.
///
/// Resolution order (later wins):
/// 1. User config: `~/.config/fireside/config.json`
/// 2. Project config: `fireside.json` in current working directory
#[must_use]
pub fn load_settings() -> Settings {
    let user_path = user_settings_path();
    let project_path = project_settings_path();
    load_settings_from_paths(project_path.as_deref(), user_path.as_deref())
}

fn load_settings_from_paths(project_path: Option<&Path>, user_path: Option<&Path>) -> Settings {
    let mut settings = Settings::default();

    if let Some(path) = user_path {
        merge_settings_from_file(path, &mut settings);
    }

    if let Some(path) = project_path {
        merge_settings_from_file(path, &mut settings);
    }

    settings
}

fn merge_settings_from_file(path: &Path, settings: &mut Settings) {
    let Ok(raw) = fs::read_to_string(path) else {
        return;
    };

    let Ok(parsed) = serde_json::from_str::<PartialSettings>(&raw) else {
        return;
    };

    parsed.apply(settings);
}

impl PartialSettings {
    fn apply(self, settings: &mut Settings) {
        if let Some(nested) = self.settings {
            nested.apply(settings);
        }

        if let Some(theme) = self.theme {
            let trimmed = theme.trim();
            if trimmed.is_empty() {
                settings.theme = None;
            } else {
                settings.theme = Some(trimmed.to_string());
            }
        }
        if let Some(timeout) = self.poll_timeout_ms {
            settings.poll_timeout_ms = timeout.clamp(10, 10_000);
        }
        if let Some(show_progress) = self.show_progress {
            settings.show_progress = show_progress;
        }
        if let Some(show_timer) = self.show_timer {
            settings.show_timer = show_timer;
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct PartialSettings {
    #[serde(alias = "default_theme")]
    theme: Option<String>,
    #[serde(alias = "poll-timeout-ms")]
    poll_timeout_ms: Option<u64>,
    #[serde(alias = "show-progress", alias = "show_progress_bar")]
    show_progress: Option<bool>,
    #[serde(alias = "show-timer", alias = "show_elapsed_timer")]
    show_timer: Option<bool>,
    settings: Option<Box<PartialSettings>>,
}

fn user_settings_path() -> Option<PathBuf> {
    config_base_dir().map(|base| base.join("fireside").join("config.json"))
}

fn project_settings_path() -> Option<PathBuf> {
    std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join("fireside.json"))
}

/// Persisted editor UI preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorUiPrefs {
    /// Last focused editor pane (`node-list` or `node-detail`).
    pub last_focus: String,
    /// Last selected layout picker index.
    pub last_layout_picker: usize,
    /// Last selected transition picker index.
    pub last_transition_picker: usize,
    /// Last node list scroll offset for virtualized list rendering.
    pub last_list_scroll_offset: usize,
}

impl Default for EditorUiPrefs {
    fn default() -> Self {
        Self {
            last_focus: "node-list".to_string(),
            last_layout_picker: 0,
            last_transition_picker: 0,
            last_list_scroll_offset: 0,
        }
    }
}

/// Load persisted editor UI preferences.
#[must_use]
pub fn load_editor_ui_prefs() -> EditorUiPrefs {
    let path = editor_prefs_path();
    let Ok(raw) = fs::read_to_string(&path) else {
        return EditorUiPrefs::default();
    };

    serde_json::from_str::<EditorUiPrefs>(&raw).unwrap_or_default()
}

/// Save persisted editor UI preferences.
pub fn save_editor_ui_prefs(prefs: &EditorUiPrefs) -> std::io::Result<()> {
    let path = editor_prefs_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(prefs).unwrap_or_else(|_| {
        String::from(
            "{\n  \"last_focus\": \"node-list\",\n  \"last_layout_picker\": 0,\n  \"last_transition_picker\": 0,\n  \"last_list_scroll_offset\": 0\n}\n",
        )
    });
    fs::write(path, content)
}

fn editor_prefs_path() -> PathBuf {
    if let Some(base) = config_base_dir() {
        return base.join("fireside").join("editor-ui.json");
    }

    PathBuf::from(".fireside-editor-ui.json")
}

fn config_base_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME")
        && !xdg.trim().is_empty()
    {
        return Some(PathBuf::from(xdg));
    }

    std::env::var("HOME")
        .ok()
        .filter(|home| !home.trim().is_empty())
        .map(|home| PathBuf::from(home).join(".config"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_default_values_are_stable() {
        let settings = Settings::default();
        assert_eq!(settings.poll_timeout_ms, 250);
        assert!(settings.show_progress);
        assert!(settings.show_timer);
        assert!(settings.theme.is_none());
    }

    #[test]
    fn project_settings_override_user_settings() {
        let unique = format!(
            "fireside-settings-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be monotonic")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&root).expect("temp dir should be creatable");

        let user_path = root.join("user.json");
        let project_path = root.join("project.json");

        std::fs::write(
            &user_path,
            "{\n  \"theme\": \"nord\",\n  \"poll_timeout_ms\": 200,\n  \"show_progress\": true,\n  \"show_timer\": true\n}\n",
        )
        .expect("user settings should be writable");
        std::fs::write(
            &project_path,
            "{\n  \"theme\": \"dracula\",\n  \"poll_timeout_ms\": 120,\n  \"show_progress\": false,\n  \"show_timer\": false\n}\n",
        )
        .expect("project settings should be writable");

        let settings = load_settings_from_paths(Some(&project_path), Some(&user_path));

        assert_eq!(settings.theme.as_deref(), Some("dracula"));
        assert_eq!(settings.poll_timeout_ms, 120);
        assert!(!settings.show_progress);
        assert!(!settings.show_timer);

        let _ = std::fs::remove_file(&user_path);
        let _ = std::fs::remove_file(&project_path);
        let _ = std::fs::remove_dir(&root);
    }

    #[test]
    fn nested_settings_block_is_supported() {
        let unique = format!(
            "fireside-settings-nested-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be monotonic")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&root).expect("temp dir should be creatable");

        let project_path = root.join("project.json");
        std::fs::write(
            &project_path,
            "{\n  \"settings\": {\n    \"theme\": \"  nord  \",\n    \"poll-timeout-ms\": 1,\n    \"show-progress\": false\n  }\n}\n",
        )
        .expect("project settings should be writable");

        let settings = load_settings_from_paths(Some(&project_path), None);

        assert_eq!(settings.theme.as_deref(), Some("nord"));
        assert_eq!(settings.poll_timeout_ms, 10);
        assert!(!settings.show_progress);

        let _ = std::fs::remove_file(&project_path);
        let _ = std::fs::remove_dir(&root);
    }
}
