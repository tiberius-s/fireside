use std::path::Path;

use anyhow::{Context, Result};

use fireside_engine::load_graph;
use fireside_engine::validation::{Severity, validate_graph};

/// Validate a graph file and print diagnostics.
pub fn run_validate(file: &Path) -> Result<()> {
    let graph = load_graph(file).context("loading graph for validation")?;
    let diagnostics = validate_graph(&graph);

    if diagnostics.is_empty() {
        println!("✓ {} is valid", file.display());
        return Ok(());
    }

    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    for d in &diagnostics {
        let prefix = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let location = d
            .node_id
            .as_deref()
            .map_or(String::new(), |id| format!(" (node '{id}')"));
        println!("{prefix}{location}: {}", d.message);
    }

    println!();
    println!(
        "{} error(s), {} warning(s) in {}",
        errors,
        warnings,
        file.display()
    );

    if errors > 0 {
        anyhow::bail!("validation failed with {errors} error(s)");
    }

    Ok(())
}
