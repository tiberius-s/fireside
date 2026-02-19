use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

use super::session::run_presentation;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectConfig {
    #[serde(default)]
    nodes: Vec<String>,
    #[serde(default)]
    slides: Vec<String>,
    theme: Option<String>,
}

pub(crate) fn resolve_project_entry(dir: &Path) -> Result<(PathBuf, Option<String>)> {
    let config_path = dir.join("fireside.json");
    if !config_path.exists() {
        anyhow::bail!("no fireside.json found in {}", dir.display());
    }

    let content = std::fs::read_to_string(&config_path).context("reading fireside.json")?;

    let config: ProjectConfig = serde_json::from_str(&content).with_context(|| {
        format!(
            "parsing {} (allowed keys: nodes, slides, theme)",
            config_path.display()
        )
    })?;

    let paths: Vec<String> = if config.nodes.is_empty() {
        config.slides
    } else {
        config.nodes
    };

    let entry = paths.first().context("project has no presentation files")?;

    if entry.trim().is_empty() {
        anyhow::bail!("project entry path is empty");
    }

    let entry_path = dir.join(entry);
    if !entry_path.exists() {
        anyhow::bail!(
            "project entry file does not exist: {}",
            entry_path.display()
        );
    }

    Ok((entry_path, config.theme))
}

/// Open and present a project directory.
pub fn run_project(dir: &Path, theme_name: Option<&str>) -> Result<()> {
    let (entry_path, project_theme) = resolve_project_entry(dir)?;
    let effective_theme = theme_name.or(project_theme.as_deref());
    run_presentation(&entry_path, effective_theme, 1)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::resolve_project_entry;

    fn temp_dir(name: &str) -> PathBuf {
        let unique = format!(
            "fireside-cli-project-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be monotonic")
                .as_nanos()
        );
        std::env::temp_dir().join(unique)
    }

    #[test]
    fn resolve_project_entry_uses_nodes_over_slides() {
        let root = temp_dir("nodes-priority");
        fs::create_dir_all(root.join("nodes")).expect("temp dir should be creatable");

        let node_path = root.join("nodes").join("main.json");
        fs::write(&node_path, "{\"nodes\":[{\"content\":[]}]}")
            .expect("node file should be writable");

        fs::write(
            root.join("fireside.json"),
            "{\n  \"nodes\": [\"nodes/main.json\"],\n  \"slides\": [\"slides/ignored.json\"],\n  \"theme\": \"nord\"\n}\n",
        )
        .expect("config should be writable");

        let (entry, theme) = resolve_project_entry(&root).expect("project should resolve");
        assert_eq!(entry, node_path);
        assert_eq!(theme.as_deref(), Some("nord"));

        let _ = fs::remove_file(root.join("fireside.json"));
        let _ = fs::remove_file(node_path);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn resolve_project_entry_accepts_slides_when_nodes_missing() {
        let root = temp_dir("slides-fallback");
        fs::create_dir_all(root.join("slides")).expect("temp dir should be creatable");

        let slide_path = root.join("slides").join("deck.json");
        fs::write(&slide_path, "{\"nodes\":[{\"content\":[]}]}")
            .expect("slide file should be writable");

        fs::write(
            root.join("fireside.json"),
            "{\n  \"slides\": [\"slides/deck.json\"]\n}\n",
        )
        .expect("config should be writable");

        let (entry, theme) = resolve_project_entry(&root).expect("project should resolve");
        assert_eq!(entry, slide_path);
        assert!(theme.is_none());

        let _ = fs::remove_file(root.join("fireside.json"));
        let _ = fs::remove_file(slide_path);
        let _ = fs::remove_dir_all(root);
    }
}
