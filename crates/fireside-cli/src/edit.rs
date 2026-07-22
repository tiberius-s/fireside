//! `fireside edit <deck>`: the full-screen authoring studio (spec 013).
//!
//! Foundational-phase scope (T024): the opening-rules chain from
//! `specs/013-authoring-editor/contracts/cli-edit-command.md`, then handing
//! off to `fireside_tui::editor::run` — mirroring `main.rs`'s `present()`:
//! CLI-side load/parse/create-if-missing first (no tty needed), then the
//! tty check lives inside the TUI entry point itself, exactly like
//! `present_impl`'s. The draft sidecar (US4, T059-T061) and save/
//! write-back (US1, T035) land in later waves — until then the studio
//! opens read-only-in-effect (its Save/Undo/Add-slide chips aren't wired).

use std::path::Path;

use anyhow::{Context, Result};
use fireside_core::{CoreError, Graph};
use fireside_tui::WriteBackError;

use crate::Template;
use crate::new::starter_deck;

/// Entry point for `fireside edit <file>`. Implements the opening-rules
/// chain: non-tty guard (inside `fireside_tui::editor::run`), hard refusal
/// on an unparseable deck, the `.md` import hint, create-if-missing
/// (reusing `new.rs`'s starter templates), and open-with-diagnostics-in-
/// the-status-banner for anything else — a deck with Layer-2 diagnostics
/// is never refused, since fixing those is the editor's job.
pub(crate) fn edit_deck(file: &Path) -> Result<()> {
    let graph = load_or_create(file)?;
    let mut sink = |g: &Graph| -> Result<(), WriteBackError> {
        let json = g
            .to_json_pretty()
            .map_err(|err| WriteBackError::Io(err.to_string()))?;
        atomic_write(file, &(json + "\n")).map_err(|err| WriteBackError::Io(err.to_string()))
    };
    let mut art_generator = |phrase: &str| -> Result<String, String> {
        crate::art::render_text_banner(phrase).map_err(|err| err.to_string())
    };
    crate::exit_on_not_a_tty(fireside_tui::editor::run(
        graph,
        &mut sink,
        Some(&mut art_generator),
    ))?;
    Ok(())
}

/// Writes `contents` to `path` atomically (spec 013 FR-022): a temp file in
/// the same directory, then a same-filesystem rename, so a reader (or a
/// crash mid-write) never observes a partially written deck — the same
/// technique `fireside-cli::session.rs::write` already uses for its own
/// state file.
fn atomic_write(path: &Path, contents: &str) -> std::io::Result<()> {
    let dir = path.parent().filter(|p| !p.as_os_str().is_empty());
    let tmp_name = format!(
        ".tmp-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default()
    );
    let tmp_path = match dir {
        Some(dir) => dir.join(tmp_name),
        None => Path::new(&tmp_name).to_path_buf(),
    };
    std::fs::write(&tmp_path, contents)?;
    std::fs::rename(&tmp_path, path)
}

/// Loads `file` as a deck, offering to create one if it doesn't exist yet.
/// Exits the process directly (matching `main.rs::load`'s convention) for
/// every refusal case, so the tty check inside `fireside_tui::editor::run`
/// is only ever reached once a real, parseable deck is in hand.
fn load_or_create(file: &Path) -> Result<Graph> {
    let text = match std::fs::read_to_string(file) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            if crate::is_markdown_path(file) {
                markdown_hint(file);
            }
            return create_deck(file);
        }
        Err(err) => {
            return Err(err).with_context(|| format!("could not read {}", file.display()));
        }
    };
    match Graph::from_json(&text) {
        Ok(graph) => Ok(graph),
        Err(CoreError::Parse(err)) => {
            if crate::is_markdown_path(file) {
                markdown_hint(file);
            } else {
                eprintln!("{}", crate::report::parse_report(file, &text, &err));
                eprintln!(
                    "Fix the file first \u{2014} \"fireside validate {}\" shows the full report.",
                    file.display()
                );
            }
            std::process::exit(1);
        }
    }
}

/// The `.md`/`.markdown` import hint, printed then exited — never the
/// create-if-missing flow, even for a path that "doesn't exist" as a deck
/// (contract rule 3, which takes priority over rule 4).
fn markdown_hint(file: &Path) -> ! {
    eprintln!(
        "This is a Markdown file \u{2014} run \"fireside import {}\" first, then \"fireside edit {}\"",
        file.display(),
        file.with_extension("fireside.json").display()
    );
    std::process::exit(1);
}

/// Creates a starter deck at the exact path requested (unlike `fireside
/// new`, which derives its own path from a slugified title) and opens it —
/// an actual flow, not a pointer to run a different command (contract
/// rule 4).
fn create_deck(file: &Path) -> Result<Graph> {
    let title = crate::deck_stem(file).replace(['-', '_'], " ");
    println!(
        "No deck at {} yet \u{2014} creating a starter deck to edit.",
        file.display()
    );
    let graph = starter_deck(&title, Template::Branching, None)
        .context("could not build the starter deck")?;
    let json = graph
        .to_json_pretty()
        .context("could not serialize the starter deck")?;
    if let Some(parent) = file.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("could not create {}", parent.display()))?;
    }
    std::fs::write(file, json + "\n")
        .with_context(|| format!("could not write {}", file.display()))?;
    Ok(graph)
}
