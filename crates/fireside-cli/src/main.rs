//! Fireside — present branching decks in the terminal.
//!
//! Four verbs, nothing else: `fireside <file>` presents, `validate`
//! checks, `new` scaffolds, `demo` shows off. Validation always runs
//! before presenting, so a broken deck fails loudly at the prompt instead
//! of during the show. While presenting, the deck file is watched and
//! live-reloaded on save.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use fireside_core::{CoreError, Graph};
use fireside_engine::{Severity, validate};

mod art;
mod import;
mod new;
mod report;
mod resume;
mod templates;
mod watch;

// `resume.rs` names this at the crate root (`crate::fingerprint`); keep it
// resolvable there even though the implementation lives in `watch.rs`.
use watch::fingerprint;

/// The built-in showcase deck presented by `fireside demo`.
const DEMO_DECK: &str = include_str!("../assets/demo.fireside.json");

/// Present branching decks in the terminal.
#[derive(Debug, Parser)]
#[command(
    name = "fireside",
    version,
    about,
    args_conflicts_with_subcommands = true
)]
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

        /// Start from the beginning, ignoring any saved resume position for
        /// this deck.
        #[arg(long)]
        restart: bool,
    },

    /// Check a deck and report anything wrong, in plain language.
    Validate {
        /// Path to the deck file.
        file: PathBuf,

        /// Keep checking the file and re-report on every save.
        #[arg(long)]
        watch: bool,
    },

    /// Create a starter deck you can present immediately. Omit the name to
    /// be asked a few quick questions instead.
    New {
        /// A name for the deck, e.g. "team-onboarding". Omit to be asked.
        name: Option<String>,

        /// Starter template. Defaults to `branching`.
        #[arg(long, value_enum)]
        template: Option<Template>,

        /// Author name to embed in the deck.
        #[arg(long)]
        author: Option<String>,
    },

    /// See what Fireside can do — no file needed.
    Demo,

    /// Compile a Markdown file into a deck (headings become slides).
    Import {
        /// Path to the Markdown source file.
        input: PathBuf,

        /// Path for the generated deck. Defaults to `input` with its
        /// extension replaced by `.fireside.json`.
        output: Option<PathBuf>,
    },

    /// Generate ASCII art to paste into a deck.
    Art {
        #[command(subcommand)]
        mode: ArtMode,
    },
}

/// The two ways to generate ASCII art (spec 009): a stylized text banner,
/// or a conversion of a local image. Both print to stdout; neither edits
/// a deck file.
#[derive(Debug, Subcommand)]
enum ArtMode {
    /// Turn a short phrase into a stylized text banner.
    Text {
        /// The phrase to render.
        phrase: String,
    },

    /// Convert a local image file into ASCII art.
    Image {
        /// Path to the image file.
        path: PathBuf,

        /// Output width in columns. Defaults to a size that fits the
        /// standard presentation card.
        #[arg(long)]
        width: Option<u32>,
    },
}

/// The shape of deck `fireside new` scaffolds. Each demonstrates one
/// traversal pattern so the author has a working example to edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum Template {
    /// A straight-through talk: no branching, no choices.
    Linear,
    /// A talk with one choice that rejoins — the default.
    Branching,
    /// An agenda that jumps into a sequence of exercises.
    Workshop,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match (cli.file, cli.command) {
        (Some(file), _) => present(&file, false),
        (None, Some(Command::Present { file, restart })) => present(&file, restart),
        (None, Some(Command::Validate { file, watch })) => report::validate_file(&file, watch),
        (
            None,
            Some(Command::New {
                name,
                template,
                author,
            }),
        ) => new::new_deck(name, template, author),
        (None, Some(Command::Demo)) => demo(),
        (None, Some(Command::Import { input, output })) => import_file(&input, output.as_deref()),
        (None, Some(Command::Art { mode })) => match mode {
            ArtMode::Text { phrase } => art::art_text(&phrase),
            ArtMode::Image { path, width } => art::art_image(&path, width),
        },
        (None, None) => {
            // No arguments: teach, don't error.
            println!("fireside — present branching decks in the terminal\n");
            println!("  fireside demo              see what a deck can do");
            println!("  fireside <file>            present a deck");
            println!("  fireside validate <file>   check a deck for problems");
            println!("  fireside new               create a deck (asks a few questions)");
            println!("  fireside new <name>        create a starter deck instantly");
            println!("  fireside import <file.md>  compile a Markdown talk into a deck");
            println!("  fireside art text <phrase> generate a text banner to paste in");
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
            eprintln!("{}", report::parse_report(path, &text, &err));
            std::process::exit(1);
        }
    }
}

fn present(path: &Path, restart: bool) -> Result<()> {
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
    let watcher = RefCell::new(watch::Watcher::new(path));

    // Resume-from-fingerprint (spec 007): a resume position is host-local
    // cache, not part of the deck itself — `--restart` skips the lookup for
    // this run only, without touching the stored record.
    let key = resume::fingerprint_key(path);
    let mut store = resume::ResumeStore::load();
    let initial_node = store.resolve_initial_node(key.as_deref(), restart);
    let graph_for_resume = graph.clone();

    let result = fireside_tui::present_authoring(
        graph,
        &mut || watcher.borrow_mut().poll(),
        &mut |graph| watcher.borrow_mut().write_back(graph),
        initial_node.as_deref(),
        &mut |node_id| {
            let Some(key) = &key else { return };
            let terminal = graph_for_resume
                .node(node_id)
                .is_some_and(fireside_core::Node::is_terminal);
            if terminal {
                store.clear(key);
            } else {
                store.set(key.clone(), node_id);
            }
        },
    );
    result.context("the presenter hit a terminal error")
}

fn demo() -> Result<()> {
    let graph = Graph::from_json(DEMO_DECK).context("the built-in demo deck is broken")?;
    fireside_tui::present(graph).context("the presenter hit a terminal error")
}

/// What v1 Markdown import never carries over, restated after every
/// successful import so a presenter learns the boundary from the tool
/// itself rather than by omission (FR-023, ADR-006).
const IMPORT_LIMITATIONS_NOTE: &str = "Note: this v1 import doesn't carry over columns/containers, speaker notes, or per-slide view-mode/transition — hand-edit the JSON (or use quick-edit for headings/text) to add those.";

fn import_file(input: &Path, output: Option<&Path>) -> Result<()> {
    let default_output;
    let output = match output {
        Some(output) => output,
        None => {
            default_output = input.with_extension("fireside.json");
            &default_output
        }
    };
    if output.exists() {
        bail!("{} already exists — pick another name", output.display());
    }

    let source = std::fs::read_to_string(input)
        .with_context(|| format!("could not read {}", input.display()))?;
    let graph = match import::import(&source) {
        Ok(graph) => graph,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    let json = graph
        .to_json_pretty()
        .context("could not serialize the imported deck")?;
    std::fs::write(output, json + "\n")
        .with_context(|| format!("could not write {}", output.display()))?;

    println!("Imported {}.", output.display());
    println!("{IMPORT_LIMITATIONS_NOTE}");
    Ok(())
}

/// Turns arbitrary text into a lowercase, hyphen-separated identifier safe
/// for both filenames (`new_deck`) and node ids (`import`): lowercase,
/// non-alphanumeric runs collapse to a single `-`, leading/trailing `-`
/// trimmed.
pub(crate) fn slugify(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
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
        assert!(
            serious.is_empty(),
            "demo deck must be spotless: {serious:?}"
        );
    }

    #[test]
    fn demo_deck_shows_every_block_kind() {
        for kind in [
            "heading",
            "text",
            "code",
            "list",
            "image",
            "divider",
            "container",
        ] {
            assert!(
                DEMO_DECK.contains(&format!("\"kind\": \"{kind}\"")),
                "demo deck is missing a {kind} block"
            );
        }
    }
}
