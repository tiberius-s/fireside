//! Fireside — present branching decks in the terminal.
//!
//! Three verbs, nothing else: `fireside <file>` presents, `validate`
//! checks, `new` scaffolds. Validation always runs before presenting, so a
//! broken deck fails loudly at the prompt instead of during the show.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use fireside_core::Graph;
use fireside_engine::{Severity, validate};

/// Present branching decks in the terminal.
#[derive(Debug, Parser)]
#[command(name = "fireside", version, about, args_conflicts_with_subcommands = true)]
struct Cli {
    /// Path to a deck (.fireside.json) — shorthand for `fireside present <file>`.
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Present a deck in the terminal.
    Present {
        /// Path to the deck file.
        file: PathBuf,
    },

    /// Check a deck and report anything wrong, in plain language.
    Validate {
        /// Path to the deck file.
        file: PathBuf,
    },

    /// Create a starter deck you can present immediately.
    New {
        /// A name for the deck, e.g. "team-onboarding".
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match (cli.file, cli.command) {
        (Some(file), _) | (None, Some(Command::Present { file })) => present(&file),
        (None, Some(Command::Validate { file })) => validate_file(&file),
        (None, Some(Command::New { name })) => new_deck(&name),
        (None, None) => {
            // No arguments: teach, don't error.
            println!("fireside — present branching decks in the terminal\n");
            println!("  fireside <file>            present a deck");
            println!("  fireside validate <file>   check a deck for problems");
            println!("  fireside new <name>        create a starter deck");
            println!("\nTry: fireside new my-first-deck");
            Ok(())
        }
    }
}

/// Load and parse a deck with errors a person can act on.
fn load(path: &Path) -> Result<Graph> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("could not read {}", path.display()))?;
    Graph::from_json(&text)
        .with_context(|| format!("{} is not a valid Fireside deck", path.display()))
}

fn present(path: &Path) -> Result<()> {
    let graph = load(path)?;
    let diags = validate(&graph);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    if !errors.is_empty() {
        eprintln!("{} cannot be presented yet:\n", path.display());
        for d in &errors {
            eprintln!("  ✗ {}", d.message);
        }
        eprintln!("\nFix the above, or run `fireside validate` for the full report.");
        std::process::exit(1);
    }
    fireside_tui::present(graph).context("the presenter hit a terminal error")
}

fn validate_file(path: &Path) -> Result<()> {
    let graph = load(path)?;
    let diags = validate(&graph);

    if diags.is_empty() {
        println!("✓ {} — no problems found", path.display());
        return Ok(());
    }

    let mut errors = 0usize;
    let mut warnings = 0usize;
    for d in &diags {
        let icon = match d.severity {
            Severity::Error => {
                errors += 1;
                "✗"
            }
            Severity::Warning => {
                warnings += 1;
                "⚠"
            }
            Severity::Info => "ℹ",
        };
        println!("  {icon} {}", d.message);
    }
    println!(
        "\n{}: {errors} error(s), {warnings} warning(s), {} note(s)",
        path.display(),
        diags.len() - errors - warnings
    );
    if errors > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn new_deck(name: &str) -> Result<()> {
    let slug: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        bail!("please give the deck a name with at least one letter or digit");
    }
    let path = PathBuf::from(format!("{slug}.fireside.json"));
    if path.exists() {
        bail!("{} already exists — pick another name", path.display());
    }

    let json = starter_deck(name)?
        .to_json_pretty()
        .context("could not serialize the starter deck")?;
    std::fs::write(&path, json + "\n")
        .with_context(|| format!("could not write {}", path.display()))?;

    println!("Created {}.", path.display());
    println!("\nPresent it:   fireside {}", path.display());
    println!("Check it:     fireside validate {}", path.display());
    Ok(())
}

/// A three-slide starter that demonstrates the one Fireside idea people
/// need: explicit edges, including a branch that rejoins.
fn starter_deck(name: &str) -> Result<Graph> {
    let json = serde_json::json!({
        "fireside-version": "0.1.0",
        "title": name,
        "nodes": [
            {
                "id": "welcome",
                "title": "Welcome",
                "traversal": "pick-a-path",
                "content": [
                    { "kind": "container", "layout": "center", "children": [
                        { "kind": "heading", "level": 1, "text": name },
                        { "kind": "text", "body": "Press **Space** to move forward. Press **?** any time for help." }
                    ]}
                ]
            },
            {
                "id": "pick-a-path",
                "title": "Pick a path",
                "traversal": { "branch-point": {
                    "prompt": "Decks can branch. Where to?",
                    "options": [
                        { "label": "Show me content blocks", "key": "a", "target": "blocks" },
                        { "label": "Skip to the end", "key": "b", "target": "the-end" }
                    ]
                }},
                "content": [
                    { "kind": "heading", "level": 2, "text": "A choice" },
                    { "kind": "text", "body": "Use the arrow keys and press Enter." }
                ]
            },
            {
                "id": "blocks",
                "title": "Content blocks",
                "traversal": "the-end",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Blocks" },
                    { "kind": "list", "items": [
                        "Headings, text with **inline markdown**",
                        "Code with `highlight-lines`",
                        "Lists, images, dividers, containers"
                    ]},
                    { "kind": "divider" },
                    { "kind": "code", "language": "json", "source": "{ \"kind\": \"text\", \"body\": \"like this\" }" }
                ]
            },
            {
                "id": "the-end",
                "title": "The end",
                "content": [
                    { "kind": "container", "layout": "center", "children": [
                        { "kind": "heading", "level": 1, "text": "That's it" },
                        { "kind": "text", "body": "Edit the .fireside.json file to make it yours." }
                    ]}
                ]
            }
        ]
    });
    serde_json::from_value(json).context("the starter deck template is broken")
}
