//! Fireside — present branching decks in the terminal.
//!
//! Four verbs, nothing else: `fireside <file>` presents, `validate`
//! checks, `new` scaffolds, `demo` shows off. Validation always runs
//! before presenting, so a broken deck fails loudly at the prompt instead
//! of during the show. While presenting, the deck file is watched and
//! live-reloaded on save.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::Duration;

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

        /// Add an ASCII title banner (generated from the deck's title)
        /// to the first slide.
        #[arg(long)]
        banner: bool,
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
                banner,
            }),
        ) => match new::new_deck(name, template, author, banner)? {
            Some(path) => present(&path, false),
            None => Ok(()),
        },
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

/// Load and parse a deck with errors a person can act on: a missing file
/// gets one plain-language line with the fix, and a broken file prints the
/// offending line with a caret — neither shows a raw anyhow/serde chain.
fn load(path: &Path) -> Result<Graph> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "No deck named {} — \"fireside new {}\" creates one.",
                path.display(),
                deck_stem(path)
            );
            std::process::exit(1);
        }
        Err(err) => {
            return Err(err).with_context(|| format!("could not read {}", path.display()));
        }
    };
    match Graph::from_json(&text) {
        Ok(graph) => Ok(graph),
        Err(CoreError::Parse(err)) => {
            eprintln!("{}", report::parse_report(path, &text, &err));
            std::process::exit(1);
        }
    }
}

/// The name `fireside new` would take for this path: `nope.fireside.json`
/// becomes `nope`, matching what `new` itself writes back out
/// (`{slug}.fireside.json`).
fn deck_stem(path: &Path) -> String {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("deck");
    name.strip_suffix(".fireside.json")
        .map(str::to_owned)
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(name)
                .to_owned()
        })
}

/// Formats a graceful-quit rehearsal summary: `"Presented {seen}/{total}
/// slides in {mm}:{ss}."`, seconds zero-padded, no hours component.
#[must_use]
fn format_present_summary(seen: usize, total: usize, elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    format!(
        "Presented {seen}/{total} slides in {}:{:02}.",
        secs / 60,
        secs % 60
    )
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
    let summary = result.context("the presenter hit a terminal error")?;
    println!(
        "{}",
        format_present_summary(summary.seen, summary.total, summary.elapsed)
    );
    Ok(())
}

fn demo() -> Result<()> {
    let graph = Graph::from_json(DEMO_DECK).context("the built-in demo deck is broken")?;
    let summary = fireside_tui::present(graph).context("the presenter hit a terminal error")?;
    println!(
        "{}",
        format_present_summary(summary.seen, summary.total, summary.elapsed)
    );
    Ok(())
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
    fn format_present_summary_pads_seconds() {
        assert_eq!(
            format_present_summary(5, 7, Duration::from_secs(12 * 60 + 30)),
            "Presented 5/7 slides in 12:30."
        );
        assert_eq!(
            format_present_summary(2, 7, Duration::from_secs(12 * 60 + 5)),
            "Presented 2/7 slides in 12:05."
        );
    }

    #[test]
    fn format_present_summary_handles_first_slide_only() {
        assert_eq!(
            format_present_summary(1, 7, Duration::from_secs(0)),
            "Presented 1/7 slides in 0:00."
        );
    }

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
            "ascii-art",
        ] {
            assert!(
                DEMO_DECK.contains(&format!("\"kind\": \"{kind}\"")),
                "demo deck is missing a {kind} block"
            );
        }
    }
}
