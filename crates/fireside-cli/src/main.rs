//! Fireside — present branching decks in the terminal.
//!
//! Four verbs, nothing else: `fireside <file>` presents, `validate`
//! checks, `new` scaffolds, `demo` shows off. Validation always runs
//! before presenting, so a broken deck fails loudly at the prompt instead
//! of during the show. While presenting, the deck file is watched and
//! live-reloaded on save.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use fireside_core::{CoreError, Graph};
use fireside_engine::{Severity, validate};

/// The built-in showcase deck presented by `fireside demo`.
const DEMO_DECK: &str = include_str!("../assets/demo.fireside.json");

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

    /// See what Fireside can do — no file needed.
    Demo,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match (cli.file, cli.command) {
        (Some(file), _) | (None, Some(Command::Present { file })) => present(&file),
        (None, Some(Command::Validate { file })) => validate_file(&file),
        (None, Some(Command::New { name })) => new_deck(&name),
        (None, Some(Command::Demo)) => demo(),
        (None, None) => {
            // No arguments: teach, don't error.
            println!("fireside — present branching decks in the terminal\n");
            println!("  fireside demo              see what a deck can do");
            println!("  fireside <file>            present a deck");
            println!("  fireside validate <file>   check a deck for problems");
            println!("  fireside new <name>        create a starter deck");
            println!("\nTry: fireside demo");
            Ok(())
        }
    }
}

/// Load and parse a deck with errors a person can act on: a broken file
/// prints the offending line with a caret, not a serde one-liner.
fn load(path: &Path) -> Result<Graph> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("could not read {}", path.display()))?;
    match Graph::from_json(&text) {
        Ok(graph) => Ok(graph),
        Err(CoreError::Parse(err)) => {
            eprintln!("{}", parse_report(path, &text, &err));
            std::process::exit(1);
        }
    }
}

/// A parse failure the author can act on: the line before, the offending
/// line, and a caret under the exact column.
fn parse_report(path: &Path, text: &str, err: &serde_json::Error) -> String {
    let line = err.line();
    let column = err.column().max(1);
    let mut out = format!("✗ {} is not a valid deck\n", path.display());

    let lines: Vec<&str> = text.lines().collect();
    if line >= 1 && line <= lines.len() {
        let gutter = line.to_string().len();
        out.push('\n');
        if line >= 2 {
            out.push_str(&format!("  {:>gutter$} │ {}\n", line - 1, lines[line - 2]));
        }
        out.push_str(&format!("  {:>gutter$} │ {}\n", line, lines[line - 1]));
        out.push_str(&format!(
            "  {:>gutter$} │ {}^ {}\n",
            "",
            " ".repeat(column - 1),
            strip_position(err),
        ));
    } else {
        out.push_str(&format!("\n  {err}\n"));
    }
    out.push_str("\nFix the file and try again.");
    out
}

/// serde_json appends " at line L column C" to every message; the report
/// and the reload flash show the position themselves, so drop it.
fn strip_position(err: &serde_json::Error) -> String {
    let full = err.to_string();
    full.split(" at line ").next().unwrap_or(&full).to_owned()
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
    let mut watcher = Watcher::new(path);
    fireside_tui::present_watching(graph, &mut || watcher.poll())
        .context("the presenter hit a terminal error")
}

fn demo() -> Result<()> {
    let graph = Graph::from_json(DEMO_DECK).context("the built-in demo deck is broken")?;
    fireside_tui::present(graph).context("the presenter hit a terminal error")
}

/// Watches the deck file while presenting: cheap fingerprint check per
/// poll, full re-read and re-parse only when the file actually changed.
struct Watcher {
    path: PathBuf,
    fingerprint: Option<(SystemTime, u64)>,
}

impl Watcher {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            fingerprint: fingerprint(path),
        }
    }

    /// `None` while the file is unchanged (or briefly unreadable mid-save);
    /// otherwise the freshly parsed deck or a one-line footer message.
    fn poll(&mut self) -> Option<Result<Graph, String>> {
        let current = fingerprint(&self.path)?;
        if Some(current) == self.fingerprint {
            return None;
        }
        self.fingerprint = Some(current);
        let name = self
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.path.display().to_string());
        Some(match std::fs::read_to_string(&self.path) {
            Err(err) => Err(format!("Reload failed — could not read {name}: {err}")),
            Ok(text) => Graph::from_json(&text).map_err(|CoreError::Parse(err)| {
                format!(
                    "Reload failed — {name}:{}:{} — {}",
                    err.line(),
                    err.column(),
                    strip_position(&err),
                )
            }),
        })
    }
}

/// The file's (mtime, size) pair — enough to notice editor saves.
fn fingerprint(path: &Path) -> Option<(SystemTime, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_deck_parses_and_validates_clean() {
        let graph = Graph::from_json(DEMO_DECK).expect("demo deck parses");
        let diags = validate(&graph);
        let serious: Vec<_> = diags
            .iter()
            .filter(|d| d.severity >= Severity::Warning)
            .collect();
        assert!(serious.is_empty(), "demo deck must be spotless: {serious:?}");
    }

    #[test]
    fn demo_deck_shows_every_block_kind() {
        for kind in ["heading", "text", "code", "list", "image", "divider", "container"] {
            assert!(
                DEMO_DECK.contains(&format!("\"kind\": \"{kind}\"")),
                "demo deck is missing a {kind} block"
            );
        }
    }

    #[test]
    fn parse_report_points_at_the_line_with_a_caret() {
        let text = "{\n  \"fireside-version\": \"0.1.0\",\n  \"nodes\": [}\n}";
        let err = Graph::from_json(text).expect_err("invalid JSON");
        let CoreError::Parse(err) = err;
        let report = parse_report(Path::new("broken.json"), text, &err);
        assert!(report.contains("broken.json is not a valid deck"), "{report}");
        assert!(report.contains("3 │   \"nodes\": [}"), "offending line shown: {report}");
        assert!(report.contains('^'), "caret shown: {report}");
        assert!(!report.contains("at line"), "no duplicated position: {report}");
    }
}
