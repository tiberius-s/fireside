//! Fireside — present branching decks in the terminal.
//!
//! Four verbs, nothing else: `fireside <file>` presents, `validate`
//! checks, `new` scaffolds, `demo` shows off. Validation always runs
//! before presenting, so a broken deck fails loudly at the prompt instead
//! of during the show. While presenting, the deck file is watched and
//! live-reloaded on save.

use std::cell::RefCell;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use fireside_core::{CoreError, Graph};
use fireside_engine::{Diagnostic, Severity, validate};
use fireside_tui::WriteBackError;

mod import;
mod resume;

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
        (None, Some(Command::Validate { file, watch })) => validate_file(&file, watch),
        (
            None,
            Some(Command::New {
                name,
                template,
                author,
            }),
        ) => new_deck(name, template, author),
        (None, Some(Command::Demo)) => demo(),
        (None, Some(Command::Import { input, output })) => import_file(&input, output.as_deref()),
        (None, None) => {
            // No arguments: teach, don't error.
            println!("fireside — present branching decks in the terminal\n");
            println!("  fireside demo              see what a deck can do");
            println!("  fireside <file>            present a deck");
            println!("  fireside validate <file>   check a deck for problems");
            println!("  fireside new               create a deck (asks a few questions)");
            println!("  fireside new <name>        create a starter deck instantly");
            println!("  fireside import <file.md>  compile a Markdown talk into a deck");
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
    let watcher = RefCell::new(Watcher::new(path));

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

    /// Writes `graph` to the watched path, refusing if the file changed on
    /// disk since this watcher last observed it (a quick-edit save must
    /// never silently discard a concurrent external edit). Deliberately
    /// leaves `self.fingerprint` stale on a *successful* write: the next
    /// `poll()` must see that write as a change and reload, exactly like
    /// any external edit (FR-008) — updating it here would make `poll()`
    /// treat the save as a no-op and leave the old content on screen.
    ///
    /// On a *conflict*, the fingerprint is resynced to the file's current
    /// (externally-changed) state before returning the error. The presenter
    /// keeps their edit (the caller leaves the modal open on failure) and
    /// can retry: since the fingerprint is now current, a repeat save
    /// succeeds as a deliberate overwrite. This is the "choose to overwrite
    /// or abandon" FR-013 asks for — pressing save again is the overwrite
    /// choice; Esc is the abandon choice.
    fn write_back(&mut self, graph: &Graph) -> Result<(), WriteBackError> {
        let current = fingerprint(&self.path);
        if current != self.fingerprint {
            self.fingerprint = current;
            return Err(WriteBackError::Conflict);
        }
        let json = graph
            .to_json_pretty()
            .map_err(|err| WriteBackError::Io(err.to_string()))?;
        std::fs::write(&self.path, json + "\n")
            .map_err(|err| WriteBackError::Io(err.to_string()))?;
        Ok(())
    }
}

/// The file's (mtime, size) pair — enough to notice editor saves.
fn fingerprint(path: &Path) -> Option<(SystemTime, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

/// Render a validation result exactly as `validate` has always printed it:
/// a success line, or the full diagnostic list plus a summary. Shared by
/// the one-shot path and the watch loop so their output never drifts apart.
fn diagnostics_report(path: &Path, diags: &[Diagnostic]) -> String {
    if diags.is_empty() {
        return format!("✓ {} — no problems found", path.display());
    }

    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut lines: Vec<String> = diags
        .iter()
        .map(|d| {
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
            format!("  {icon} {}", d.message)
        })
        .collect();
    lines.push(format!(
        "\n{}: {errors} error(s), {warnings} warning(s), {} note(s)",
        path.display(),
        diags.len() - errors - warnings
    ));
    lines.join("\n")
}

fn validate_file(path: &Path, watch: bool) -> Result<()> {
    if watch {
        return watch_loop(path);
    }

    let graph = load(path)?;
    let diags = validate(&graph);
    let has_errors = diags.iter().any(|d| d.severity == Severity::Error);
    println!("{}", diagnostics_report(path, &diags));
    if has_errors {
        std::process::exit(1);
    }
    Ok(())
}

/// Check the file once and render the result — a success line, the
/// diagnostic list, a caret-pointed parse report, or a one-line message if
/// the file can't currently be read. Never exits the process, so it is
/// safe to call on every tick of the watch loop.
fn watch_report(path: &Path) -> String {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => return format!("✗ could not read {}: {err}", path.display()),
    };
    match Graph::from_json(&text) {
        Err(CoreError::Parse(err)) => parse_report(path, &text, &err),
        Ok(graph) => diagnostics_report(path, &validate(&graph)),
    }
}

/// Check `path` immediately, then keep re-checking on a short poll and
/// re-report whenever the file changes — the same cadence `present`'s
/// live reload already uses, so a save-and-look loop feels the same
/// whether you're authoring or presenting.
fn watch_loop(path: &Path) -> Result<()> {
    println!("{}", watch_report(path));
    let mut last = fingerprint(path);
    loop {
        std::thread::sleep(std::time::Duration::from_millis(250));
        let current = fingerprint(path);
        if current != last {
            last = current;
            println!("\n{}", watch_report(path));
        }
    }
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

fn new_deck(
    name: Option<String>,
    template: Option<Template>,
    author: Option<String>,
) -> Result<()> {
    let (name, template, author) = match name {
        Some(name) => (name, template.unwrap_or(Template::Branching), author),
        None => interactive_new()?,
    };

    let slug = slugify(&name);
    if slug.is_empty() {
        bail!("please give the deck a name with at least one letter or digit");
    }
    let path = PathBuf::from(format!("{slug}.fireside.json"));
    if path.exists() {
        bail!("{} already exists — pick another name", path.display());
    }

    let json = starter_deck(&name, template, author.as_deref())?
        .to_json_pretty()
        .context("could not serialize the starter deck")?;
    std::fs::write(&path, json + "\n")
        .with_context(|| format!("could not write {}", path.display()))?;

    println!("Created {}.", path.display());
    println!("\nPresent it:   fireside {}", path.display());
    println!("Check it:     fireside validate {}", path.display());
    Ok(())
}

/// Reads one line from stdin, printing `label` first as a prompt. `Ok(None)`
/// means stdin hit EOF — callers must stop asking, not loop forever.
fn prompt_line(stdin: &mut impl BufRead, label: &str) -> Result<Option<String>> {
    print!("{label}");
    io::stdout().flush().ok();
    let mut line = String::new();
    let read = stdin.read_line(&mut line).context("could not read stdin")?;
    if read == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim().to_string()))
}

/// Asks the three questions a new deck needs — title, template, author —
/// and returns sensible answers for whichever were skipped. Only reached
/// when `fireside new` is run without a name.
fn interactive_new() -> Result<(String, Template, Option<String>)> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let name = loop {
        match prompt_line(&mut stdin, "Deck title: ")? {
            None => bail!("no input received — pass a name directly: fireside new <name>"),
            Some(s) if s.is_empty() => println!("  a title is required."),
            Some(s) => break s,
        }
    };

    println!("\nTemplates:");
    println!("  1) linear     a straight-through talk, no branching");
    println!("  2) branching  a talk with one choice that rejoins (default)");
    println!("  3) workshop   an agenda that jumps into a sequence of exercises");
    let template = loop {
        match prompt_line(&mut stdin, "Pick a template [1-3, default 2]: ")? {
            None => break Template::Branching,
            Some(s) if s.is_empty() => break Template::Branching,
            Some(s) => match s.as_str() {
                "1" | "linear" => break Template::Linear,
                "2" | "branching" => break Template::Branching,
                "3" | "workshop" => break Template::Workshop,
                _ => println!("  please enter 1, 2, or 3."),
            },
        }
    };

    let author = prompt_line(&mut stdin, "Author (optional): ")?.filter(|s| !s.is_empty());

    Ok((name, template, author))
}

fn starter_deck(name: &str, template: Template, author: Option<&str>) -> Result<Graph> {
    let json = match template {
        Template::Linear => linear_template(name),
        Template::Branching => branching_template(name),
        Template::Workshop => workshop_template(name),
    };
    let mut graph: Graph =
        serde_json::from_value(json).context("the starter deck template is broken")?;
    graph.author = author.map(str::to_owned);
    Ok(graph)
}

/// A straight-through talk with no branching — the simplest possible deck,
/// for a presenter who just wants to get on stage.
fn linear_template(name: &str) -> serde_json::Value {
    serde_json::json!({
        "fireside-version": "0.1.0",
        "title": name,
        "nodes": [
            {
                "id": "welcome",
                "title": "Welcome",
                "traversal": "context",
                "speaker-notes": "This is your title slide. Edit the heading and subtitle below to fit your talk.",
                "content": [
                    { "kind": "container", "layout": "center", "children": [
                        { "kind": "heading", "level": 1, "text": name },
                        { "kind": "text", "body": "Press **Space** to move forward. Press **?** any time for help." }
                    ]}
                ]
            },
            {
                "id": "context",
                "title": "Context",
                "traversal": "example",
                "speaker-notes": "Replace this list with your own key points.",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Say something" },
                    { "kind": "text", "body": "Text blocks support **inline markdown**." },
                    { "kind": "list", "items": [
                        "One point per line",
                        "Keep it short — the audience is listening, not reading",
                        "Add as many nodes as your talk needs"
                    ]}
                ]
            },
            {
                "id": "example",
                "title": "Example",
                "traversal": "closing",
                "speaker-notes": "Swap this code sample for a snippet from your own project.",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Show something" },
                    { "kind": "divider" },
                    { "kind": "code", "language": "json", "source": "{ \"kind\": \"text\", \"body\": \"like this\" }" }
                ]
            },
            {
                "id": "closing",
                "title": "Closing",
                "content": [
                    { "kind": "container", "layout": "center", "children": [
                        { "kind": "heading", "level": 1, "text": "Thanks!" },
                        { "kind": "text", "body": "Edit the .fireside.json file to make it yours." }
                    ]}
                ]
            }
        ]
    })
}

/// A three-slide starter that demonstrates the one Fireside idea people
/// need: explicit edges, including a branch that rejoins. The default
/// template.
fn branching_template(name: &str) -> serde_json::Value {
    serde_json::json!({
        "fireside-version": "0.1.0",
        "title": name,
        "nodes": [
            {
                "id": "welcome",
                "title": "Welcome",
                "traversal": "pick-a-path",
                "speaker-notes": "This is your title slide. Edit the heading and subtitle below to fit your talk.",
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
                "speaker-notes": "This is a branch point — presenters see a menu here. Add or remove options in traversal.branch-point.options.",
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
    })
}

/// An agenda that lets the presenter jump into any exercise, then flows
/// forward through the rest in order — the hub-and-spoke pattern a workshop
/// needs, without looping back through the menu.
fn workshop_template(name: &str) -> serde_json::Value {
    serde_json::json!({
        "fireside-version": "0.1.0",
        "title": name,
        "nodes": [
            {
                "id": "welcome",
                "title": "Welcome",
                "traversal": "agenda",
                "speaker-notes": "This is your title slide. Edit the heading and subtitle below to fit your workshop.",
                "content": [
                    { "kind": "container", "layout": "center", "children": [
                        { "kind": "heading", "level": 1, "text": name },
                        { "kind": "text", "body": "Press **Space** to begin. Press **?** any time for help." }
                    ]}
                ]
            },
            {
                "id": "agenda",
                "title": "Agenda",
                "traversal": { "branch-point": {
                    "prompt": "Where should we start?",
                    "options": [
                        { "label": "Setup", "key": "a", "target": "setup" },
                        { "label": "Exercise 1", "key": "b", "target": "exercise-1" },
                        { "label": "Exercise 2", "key": "c", "target": "exercise-2" }
                    ]
                }},
                "speaker-notes": "Presenters can jump to any section from here; each section still flows into the next when they press Space. Add sections by adding an option here and a node below.",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Agenda" },
                    { "kind": "list", "items": [
                        "Setup",
                        "Exercise 1",
                        "Exercise 2"
                    ]}
                ]
            },
            {
                "id": "setup",
                "title": "Setup",
                "traversal": "exercise-1",
                "speaker-notes": "Walk through environment or prerequisite steps here.",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Setup" },
                    { "kind": "list", "ordered": true, "items": [
                        "Clone the repository",
                        "Install dependencies",
                        "Confirm everyone is ready"
                    ]}
                ]
            },
            {
                "id": "exercise-1",
                "title": "Exercise 1",
                "traversal": "exercise-2",
                "speaker-notes": "Replace this code sample with the first exercise.",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Exercise 1" },
                    { "kind": "code", "language": "json", "source": "{ \"kind\": \"text\", \"body\": \"like this\" }" }
                ]
            },
            {
                "id": "exercise-2",
                "title": "Exercise 2",
                "traversal": "wrap-up",
                "speaker-notes": "Replace this list with the second exercise's steps.",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Exercise 2" },
                    { "kind": "list", "items": [
                        "Step one",
                        "Step two",
                        "Step three"
                    ]}
                ]
            },
            {
                "id": "wrap-up",
                "title": "Wrap-up",
                "content": [
                    { "kind": "container", "layout": "center", "children": [
                        { "kind": "heading", "level": 1, "text": "That's it" },
                        { "kind": "text", "body": "Edit the .fireside.json file to make it yours." }
                    ]}
                ]
            }
        ]
    })
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

    #[test]
    fn every_starter_template_validates_clean() {
        for template in [Template::Linear, Template::Branching, Template::Workshop] {
            let graph = starter_deck("Test Deck", template, None)
                .unwrap_or_else(|e| panic!("{template:?} template builds: {e}"));
            let diags = validate(&graph);
            let serious: Vec<_> = diags
                .iter()
                .filter(|d| d.severity >= Severity::Warning)
                .collect();
            assert!(
                serious.is_empty(),
                "{template:?} template must be spotless: {serious:?}"
            );
        }
    }

    #[test]
    fn every_starter_template_carries_speaker_note_hints() {
        for template in [Template::Linear, Template::Branching, Template::Workshop] {
            let graph = starter_deck("Test Deck", template, None).expect("template builds");
            assert!(
                graph.nodes.iter().any(|n| n.speaker_notes.is_some()),
                "{template:?} template should hint the author via speaker notes"
            );
        }
    }

    #[test]
    fn starter_deck_embeds_the_given_author() {
        let graph = starter_deck("Test Deck", Template::Branching, None)
            .expect("branching template builds");
        assert_eq!(graph.author, None);

        let graph = starter_deck("Test Deck", Template::Branching, Some("Ada Lovelace"))
            .expect("branching template builds");
        assert_eq!(graph.author.as_deref(), Some("Ada Lovelace"));
    }

    #[test]
    fn parse_report_points_at_the_line_with_a_caret() {
        let text = "{\n  \"fireside-version\": \"0.1.0\",\n  \"nodes\": [}\n}";
        let err = Graph::from_json(text).expect_err("invalid JSON");
        let CoreError::Parse(err) = err;
        let report = parse_report(Path::new("broken.json"), text, &err);
        assert!(
            report.contains("broken.json is not a valid deck"),
            "{report}"
        );
        assert!(
            report.contains("3 │   \"nodes\": [}"),
            "offending line shown: {report}"
        );
        assert!(report.contains('^'), "caret shown: {report}");
        assert!(
            !report.contains("at line"),
            "no duplicated position: {report}"
        );
    }

    /// A single terminal node with no traversal and no content — the
    /// smallest deck that produces zero diagnostics of any severity, so
    /// `diagnostics_report` takes its empty-diagnostics branch.
    const SPOTLESS_DECK: &str = r#"{"nodes":[{"id":"a","content":[]}]}"#;

    #[test]
    fn watch_report_confirms_a_valid_deck() {
        let temp = tempfile::tempdir().expect("temp dir");
        let deck = temp.path().join("deck.json");
        std::fs::write(&deck, SPOTLESS_DECK).expect("write fixture");

        let report = watch_report(&deck);
        assert!(report.contains("no problems found"), "{report}");
    }

    #[test]
    fn watch_report_shows_diagnostics_for_a_dangling_target() {
        let temp = tempfile::tempdir().expect("temp dir");
        let deck = temp.path().join("broken.json");
        std::fs::write(
            &deck,
            r#"{"nodes":[{"id":"a","traversal":"ghost","content":[]}]}"#,
        )
        .expect("write fixture");

        let report = watch_report(&deck);
        assert!(
            report.contains("no node has that id"),
            "expected the dangling-target diagnostic: {report}"
        );
        assert!(
            report.contains("error(s)"),
            "expected the summary line: {report}"
        );
    }

    #[test]
    fn watch_report_shows_a_caret_report_for_malformed_json() {
        let temp = tempfile::tempdir().expect("temp dir");
        let deck = temp.path().join("broken.json");
        std::fs::write(&deck, "{\n  \"nodes\": [}\n}").expect("write fixture");

        let report = watch_report(&deck);
        assert!(report.contains("is not a valid deck"), "{report}");
        assert!(report.contains('^'), "expected a caret: {report}");
    }

    #[test]
    fn watch_report_names_a_missing_file_without_panicking() {
        let temp = tempfile::tempdir().expect("temp dir");
        let missing = temp.path().join("nope.json");

        let report = watch_report(&missing);
        assert!(
            report.contains("could not read"),
            "expected a missing-file message: {report}"
        );
    }

    #[test]
    fn watch_report_recovers_after_a_file_is_deleted_and_recreated() {
        let temp = tempfile::tempdir().expect("temp dir");
        let deck = temp.path().join("deck.json");
        std::fs::write(&deck, SPOTLESS_DECK).expect("write fixture");
        assert!(watch_report(&deck).contains("no problems found"));

        std::fs::remove_file(&deck).expect("delete fixture");
        let missing_report = watch_report(&deck);
        assert!(
            missing_report.contains("could not read"),
            "{missing_report}"
        );

        std::fs::write(&deck, SPOTLESS_DECK).expect("recreate fixture");
        let recovered_report = watch_report(&deck);
        assert!(
            recovered_report.contains("no problems found"),
            "{recovered_report}"
        );
    }

    #[test]
    fn write_back_succeeds_when_the_file_is_unchanged_since_load() {
        let temp = tempfile::tempdir().expect("temp dir");
        let deck = temp.path().join("deck.json");
        std::fs::write(&deck, SPOTLESS_DECK).expect("write fixture");

        let mut watcher = Watcher::new(&deck);
        let graph = Graph::from_json(SPOTLESS_DECK).expect("fixture parses");
        watcher.write_back(&graph).expect("save should succeed");

        let saved = std::fs::read_to_string(&deck).expect("read back");
        let reparsed = Graph::from_json(&saved).expect("saved file still parses");
        assert_eq!(reparsed, graph, "save must round-trip the graph exactly");
    }

    #[test]
    fn write_back_refuses_a_save_when_the_file_changed_on_disk() {
        let temp = tempfile::tempdir().expect("temp dir");
        let deck = temp.path().join("deck.json");
        std::fs::write(&deck, SPOTLESS_DECK).expect("write fixture");

        let mut watcher = Watcher::new(&deck);
        // Simulate an external edit the watcher hasn't polled yet.
        std::fs::write(
            &deck,
            r#"{"nodes":[{"id":"a","title":"changed externally","content":[]}]}"#,
        )
        .expect("external write");

        let graph = Graph::from_json(SPOTLESS_DECK).expect("fixture parses");
        let err = watcher
            .write_back(&graph)
            .expect_err("conflicting save must be refused");
        assert_eq!(err, WriteBackError::Conflict);

        let on_disk = std::fs::read_to_string(&deck).expect("read back");
        assert!(
            on_disk.contains("changed externally"),
            "the external edit must survive a refused save: {on_disk}"
        );

        // FR-013: the presenter can choose to overwrite. A retry — the
        // same call again — now succeeds, because the conflict resynced
        // the watcher's fingerprint to the file's current state.
        watcher
            .write_back(&graph)
            .expect("a retry after a conflict must succeed (the presenter's overwrite choice)");
        let on_disk = std::fs::read_to_string(&deck).expect("read back");
        let reparsed = Graph::from_json(&on_disk).expect("saved file still parses");
        assert_eq!(reparsed, graph, "the retried save must win");
    }

    #[test]
    fn write_back_reports_io_failure_without_panicking() {
        let temp = tempfile::tempdir().expect("temp dir");
        // A directory can't be written to as a file. The fingerprint check
        // still passes (the directory itself is unchanged since `Watcher::new`),
        // so this exercises the write failing for a reason other than a
        // conflict — write_back must report it, not panic or misreport it
        // as a `Conflict`.
        let deck = temp.path().join("not-a-file");
        std::fs::create_dir(&deck).expect("make a directory in place of the deck file");
        let mut watcher = Watcher::new(&deck);

        let graph = Graph::from_json(SPOTLESS_DECK).expect("fixture parses");
        let err = watcher
            .write_back(&graph)
            .expect_err("writing to a directory must fail, not panic");
        assert!(matches!(err, WriteBackError::Io(_)), "{err:?}");
    }

    const HELLO: &str = include_str!("../../../docs/examples/hello.json");

    #[test]
    fn write_back_round_trips_a_multi_node_branching_deck_with_one_field_changed() {
        let temp = tempfile::tempdir().expect("temp dir");
        let deck = temp.path().join("hello.json");
        std::fs::write(&deck, HELLO).expect("write fixture");

        let mut watcher = Watcher::new(&deck);
        let mut graph = Graph::from_json(HELLO).expect("hello parses");
        let node = graph
            .nodes
            .iter_mut()
            .find(|n| n.id == "features")
            .expect("features node");
        if let fireside_core::ContentBlock::Heading { text, .. } = &mut node.content[0] {
            *text = "Core Features (edited)".to_owned();
        }

        watcher.write_back(&graph).expect("save should succeed");

        let saved = std::fs::read_to_string(&deck).expect("read back");
        let reparsed = Graph::from_json(&saved).expect("saved file still parses");
        assert_eq!(
            reparsed, graph,
            "save must round-trip exactly, formatting aside"
        );

        // Whole-file reformat on save is accepted (ADR-005); every other
        // node's meaning must survive regardless.
        let original = Graph::from_json(HELLO).expect("hello parses");
        for original_node in &original.nodes {
            if original_node.id == "features" {
                continue;
            }
            let saved_node = reparsed
                .node(&original_node.id)
                .unwrap_or_else(|| panic!("node {} must survive the save", original_node.id));
            assert_eq!(
                saved_node, original_node,
                "node {} must be unchanged",
                original_node.id
            );
        }
    }
}
