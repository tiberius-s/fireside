use std::path::Path;

use anyhow::{Context, Result};

/// Scaffold a new presentation file from a template.
pub fn scaffold_presentation(name: &str, dir: &Path) -> Result<()> {
    let filename = if name.ends_with(".json") {
        name.to_owned()
    } else {
        format!("{name}.json")
    };

    let path = dir.join(&filename);

    if path.exists() {
        anyhow::bail!("File already exists: {}", path.display());
    }

    let date = today_iso_date();
    let template = serde_json::json!({
        "$schema": "https://fireside.dev/schemas/graph.schema.json",
        "title": name,
        "author": "Your Name",
        "date": date,
        "theme": "default",
        "defaults": {
            "layout": "top",
            "transition": "fade"
        },
        "nodes": [
            {
                "id": "title",
                "layout": "center",
                "content": [
                    { "kind": "heading", "level": 1, "text": name },
                    { "kind": "text", "body": "Your presentation starts here" }
                ]
            },
            {
                "content": [
                    { "kind": "heading", "level": 2, "text": "Node 2" },
                    {
                        "kind": "list",
                        "ordered": false,
                        "items": [
                            { "text": "First point" },
                            { "text": "Second point" },
                            { "text": "Third point" }
                        ]
                    }
                ]
            },
            {
                "content": [
                    { "kind": "heading", "level": 2, "text": "Code Example" },
                    {
                        "kind": "code",
                        "language": "rust",
                        "source": "fn main() {\n    println!(\"Hello from Fireside!\");\n}"
                    }
                ]
            },
            {
                "layout": "center",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Thank You" },
                    { "kind": "text", "body": "Questions?" }
                ]
            }
        ]
    });

    std::fs::create_dir_all(dir).context("creating output directory")?;
    let json_str = serde_json::to_string_pretty(&template).context("serializing template")?;
    std::fs::write(&path, json_str).context("writing presentation file")?;

    println!("Created new presentation: {}", path.display());
    Ok(())
}

/// Scaffold a new project directory with fireside.json and an initial presentation.
pub fn scaffold_project(name: &str, dir: &Path) -> Result<()> {
    let project_dir = dir.join(name);

    if project_dir.exists() {
        anyhow::bail!("Directory already exists: {}", project_dir.display());
    }

    std::fs::create_dir_all(&project_dir).context("creating project directory")?;

    let config = serde_json::json!({
        "name": name,
        "nodes": ["nodes/main.json"],
        "theme": "default"
    });
    let config_json =
        serde_json::to_string_pretty(&config).context("serializing project config")?;
    std::fs::write(project_dir.join("fireside.json"), config_json)
        .context("writing project config")?;

    let nodes_dir = project_dir.join("nodes");
    std::fs::create_dir_all(&nodes_dir).context("creating nodes directory")?;
    scaffold_presentation("main", &nodes_dir)?;

    std::fs::create_dir_all(project_dir.join("themes")).context("creating themes directory")?;

    println!("Created new project: {}", project_dir.display());
    Ok(())
}

fn today_iso_date() -> String {
    let now = time::OffsetDateTime::now_utc();
    let date = now.date();
    format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        u8::from(date.month()),
        date.day()
    )
}
