use std::path::Path;

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

/// Open and present a project directory.
pub fn run_project(dir: &Path, theme_name: Option<&str>) -> Result<()> {
    let config_path = dir.join("fireside.yml");
    if !config_path.exists() {
        anyhow::bail!("no fireside.yml found in {}", dir.display());
    }

    let content = std::fs::read_to_string(&config_path).context("reading fireside.yml")?;

    let config: ProjectConfig = serde_yaml::from_str(&content).with_context(|| {
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

    let effective_theme = theme_name.or(config.theme.as_deref());
    run_presentation(&entry_path, effective_theme, 1)
}
