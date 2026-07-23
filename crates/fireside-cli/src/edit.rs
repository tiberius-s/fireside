//! `fireside edit <deck>`: the full-screen authoring studio (spec 013).
//!
//! Owns every bit of I/O `fireside_tui::editor` itself refuses to touch
//! (ADR-014's "caller owns all I/O" contract, generalized to the editor):
//! the opening-rules chain from
//! `specs/013-authoring-editor/contracts/cli-edit-command.md`, atomic
//! save write-back with external-change conflict detection (US4, T062,
//! symmetric to `watch::Watcher`'s quick-edit guard), and the draft
//! sidecar's read/write/delete lifecycle (US4, T059-T061,
//! `data-model.md`'s Draft sidecar section).

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use fireside_core::{CoreError, Graph};
use fireside_tui::WriteBackError;
use fireside_tui::editor::DraftPrompt;

use crate::Template;
use crate::new::starter_deck;

/// Entry point for `fireside edit <file>`. Implements the opening-rules
/// chain: non-tty guard (inside `fireside_tui::editor::run`), hard refusal
/// on an unparseable deck, the `.md` import hint, create-if-missing
/// (reusing `new.rs`'s starter templates), open-with-diagnostics-in-the-
/// status-banner for anything else, and the open-time draft-vs-saved-file
/// prompt (spec 013 US4, FR-020) when a draft sidecar disagrees with the
/// file just loaded.
pub(crate) fn edit_deck(file: &Path) -> Result<()> {
    let graph = load_or_create(file)?;

    let draft_key = crate::resume::resume_key(file);
    let draft_path = draft_key.as_deref().and_then(draft_path_for);
    let draft_prompt = draft_path
        .as_deref()
        .and_then(|path| resolve_draft_prompt(path, file, &graph));

    let mut fingerprint_at_open = crate::watch::fingerprint(file);
    let mut sink = |g: &Graph| -> Result<(), WriteBackError> {
        let result = write_back(file, &mut fingerprint_at_open, g);
        if result.is_ok()
            && let Some(path) = &draft_path
        {
            delete_draft(path);
        }
        result
    };
    let deck_path_display = file.display().to_string();
    let mut draft_sink = |g: &Graph| {
        if let Some(path) = &draft_path {
            write_draft(path, &deck_path_display, g);
        }
    };
    let mut art_generator = |phrase: &str| -> Result<String, String> {
        crate::art::render_text_banner(phrase).map_err(|err| err.to_string())
    };
    let result = fireside_tui::editor::run(
        graph,
        draft_prompt,
        &mut sink,
        &mut draft_sink,
        Some(&mut art_generator),
    );
    // The editor only ever returns normally via a deliberate, voluntary
    // quit (nothing-to-save, an explicit save, or an explicit discard —
    // FR-019 never lets `run` return past a dirty, unresolved quit
    // prompt): a stale draft is never useful after that, so it is cleared
    // unconditionally. A crash or `kill -9` never reaches this line at
    // all, which is exactly how the draft survives for next time.
    if result.is_ok()
        && let Some(path) = &draft_path
    {
        delete_draft(path);
    }
    crate::exit_on_not_a_tty(result)?;
    Ok(())
}

/// Writes `graph` to `file`, refusing (spec 013 US4, T062) if the file
/// changed on disk since this session last observed it — the same
/// fingerprint-resync contract `watch::Watcher::write_back` already gives
/// the presenter's quick-edit save (FR-021). Kept separate from the
/// `sink` closure so it is directly unit-testable without a terminal.
fn write_back(
    file: &Path,
    fingerprint_at_open: &mut Option<(SystemTime, u64)>,
    graph: &Graph,
) -> Result<(), WriteBackError> {
    let current = crate::watch::fingerprint(file);
    if current != *fingerprint_at_open {
        *fingerprint_at_open = current;
        return Err(WriteBackError::Conflict);
    }
    let json = graph
        .to_json_pretty()
        .map_err(|err| WriteBackError::Io(err.to_string()))?;
    atomic_write(file, &(json + "\n")).map_err(|err| WriteBackError::Io(err.to_string()))?;
    *fingerprint_at_open = crate::watch::fingerprint(file);
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

// ─── Draft sidecar (spec 013 US4, T059-T061) ────────────────────────────

/// The schema version this build writes and accepts — a reader (this same
/// function, on the next open) encountering any other value treats the
/// file as absent, the same "no draft to offer" contract `session.rs`'s
/// and `resume.rs`'s own state files already keep.
const DRAFT_SCHEMA_VERSION: u64 = 1;

/// `$XDG_STATE_HOME/fireside/drafts/<fnv1a64-hex-of-canonicalized-deck-path>.json`,
/// falling back to `~/.local/state/fireside/drafts/...` — one path
/// segment deeper than `resume::resume_path`, same base-directory
/// resolution and the exact `fnv1a64` hash `session.rs::session_path_for`
/// already uses (`data-model.md`'s Draft sidecar section, research.md §3).
fn drafts_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state"))
        })?;
    Some(base.join("fireside").join("drafts"))
}

fn draft_path_for(key: &str) -> Option<PathBuf> {
    Some(drafts_dir()?.join(format!(
        "{:016x}.json",
        crate::session::fnv1a64(key.as_bytes())
    )))
}

/// If a draft sidecar exists at `draft_path` and its recovered graph
/// differs from `saved` (the deck just loaded from `file`), returns the
/// prompt `fireside_tui::editor::run` shows before drawing the studio
/// (`data-model.md`'s Draft sidecar section, spec FR-020). A missing,
/// corrupt, or matching draft is silently "nothing to offer" — the same
/// best-effort contract every other state file in this project keeps.
fn resolve_draft_prompt(draft_path: &Path, file: &Path, saved: &Graph) -> Option<DraftPrompt> {
    let (draft, draft_saved_at) = read_draft(draft_path)?;
    if &draft == saved {
        return None;
    }
    let now = SystemTime::now();
    let draft_touched = format_ago(now, UNIX_EPOCH + Duration::from_secs(draft_saved_at));
    let saved_touched = std::fs::metadata(file)
        .and_then(|meta| meta.modified())
        .map_or_else(|_| "an unknown time ago".to_owned(), |m| format_ago(now, m));
    Some(DraftPrompt {
        draft,
        draft_touched,
        saved_touched,
    })
}

/// Reads the draft sidecar at `path`, if present and parseable.
fn read_draft(path: &Path) -> Option<(Graph, u64)> {
    let text = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    if value.get("schema")?.as_u64()? != DRAFT_SCHEMA_VERSION {
        return None;
    }
    let saved_at = value.get("saved-at")?.as_u64()?;
    let graph: Graph = serde_json::from_value(value.get("deck")?.clone()).ok()?;
    Some((graph, saved_at))
}

/// Writes `graph` to the draft sidecar at `path` atomically (spec 013 US4,
/// T060) — best-effort: a write failure (disk full, permissions) must
/// never interrupt authoring, exactly like `session.rs::write`.
fn write_draft(path: &Path, deck_path: &str, graph: &Graph) {
    let Some(parent) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    let record = serde_json::json!({
        "schema": DRAFT_SCHEMA_VERSION,
        "deck-path": deck_path,
        "saved-at": epoch_seconds(),
        "deck": graph,
    });
    let Ok(text) = serde_json::to_string(&record) else {
        return;
    };
    let _ = atomic_write(path, &text);
}

/// Removes the draft sidecar — called on a successful save and on a clean
/// quit (spec 013 US4: "deleted on successful save and on clean quit with
/// no unsaved changes"). Best-effort: a missing file is not an error.
fn delete_draft(path: &Path) {
    let _ = std::fs::remove_file(path);
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A plain-language "how long ago" for the draft-choice prompt's two
/// timestamps (spec FR-020: "showing when each was last touched") —
/// `now < then` (a clock skew or a same-second race) reads as "just now"
/// rather than an underflow.
fn format_ago(now: SystemTime, then: SystemTime) -> String {
    let secs = now.duration_since(then).map(|d| d.as_secs()).unwrap_or(0);
    match secs {
        0..=4 => "just now".to_owned(),
        5..=59 => format!("{secs} seconds ago"),
        60..=3599 => plural(secs / 60, "minute"),
        3600..=86399 => plural(secs / 3600, "hour"),
        _ => plural(secs / 86400, "day"),
    }
}

fn plural(n: u64, unit: &str) -> String {
    if n == 1 {
        format!("1 {unit} ago")
    } else {
        format!("{n} {unit}s ago")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPOTLESS_DECK: &str = r#"{"nodes":[{"id":"a","content":[]}]}"#;

    // ─── write_back (T062) ────────────────────────────────────────────

    #[test]
    fn write_back_succeeds_when_the_file_is_unchanged_since_open() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deck = dir.path().join("deck.json");
        std::fs::write(&deck, SPOTLESS_DECK).expect("write fixture");
        let mut fp = crate::watch::fingerprint(&deck);

        let graph = Graph::from_json(SPOTLESS_DECK).expect("fixture parses");
        write_back(&deck, &mut fp, &graph).expect("save should succeed");

        let saved = std::fs::read_to_string(&deck).expect("read back");
        let reparsed = Graph::from_json(&saved).expect("saved file still parses");
        assert_eq!(reparsed, graph);
    }

    #[test]
    fn write_back_refuses_when_the_file_changed_externally() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deck = dir.path().join("deck.json");
        std::fs::write(&deck, SPOTLESS_DECK).expect("write fixture");
        let mut fp = crate::watch::fingerprint(&deck);

        // Simulate an external edit (e.g. quick-edit) after this session
        // opened the file but before it saved.
        std::thread::sleep(std::time::Duration::from_millis(10));
        std::fs::write(&deck, r#"{"nodes":[{"id":"b","content":[]}]}"#).expect("external edit");

        let graph = Graph::from_json(SPOTLESS_DECK).expect("fixture parses");
        let result = write_back(&deck, &mut fp, &graph);
        assert_eq!(result, Err(WriteBackError::Conflict));
    }

    #[test]
    fn write_back_leaves_no_temp_file_and_the_saved_content_is_whole() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deck = dir.path().join("deck.json");
        std::fs::write(&deck, SPOTLESS_DECK).expect("write fixture");
        let mut fp = crate::watch::fingerprint(&deck);

        let graph = Graph::from_json(SPOTLESS_DECK).expect("fixture parses");
        write_back(&deck, &mut fp, &graph).expect("save should succeed");

        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .expect("read dir")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(".tmp-"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "atomic_write must leave no temp file after a successful write (FR-022)"
        );
    }

    // ─── Draft sidecar round trip (T059) ────────────────────────────────

    #[test]
    fn draft_round_trips_via_write_read_delete() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("draft.json");
        let graph = Graph::from_json(SPOTLESS_DECK).expect("fixture parses");

        write_draft(&path, "/decks/talk.fireside.json", &graph);
        let (read_back, saved_at) = read_draft(&path).expect("draft reads back");
        assert_eq!(read_back, graph);
        assert!(saved_at > 0);

        delete_draft(&path);
        assert!(!path.exists());
    }

    #[test]
    fn a_missing_draft_reads_as_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(read_draft(&dir.path().join("nope.json")).is_none());
    }

    #[test]
    fn a_corrupt_draft_reads_as_none_without_panicking() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("draft.json");
        std::fs::write(&path, "not json at all").expect("write corrupt fixture");
        assert!(read_draft(&path).is_none());
    }

    #[test]
    fn an_unrecognized_draft_schema_reads_as_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("draft.json");
        let graph = Graph::from_json(SPOTLESS_DECK).expect("fixture parses");
        let record = serde_json::json!({
            "schema": DRAFT_SCHEMA_VERSION + 1,
            "deck-path": "/deck.json",
            "saved-at": epoch_seconds(),
            "deck": graph,
        });
        std::fs::write(&path, record.to_string()).expect("write future-schema fixture");
        assert!(read_draft(&path).is_none());
    }

    #[test]
    fn draft_path_for_is_deterministic_and_differs_by_key() {
        let a = draft_path_for("/decks/a.fireside.json");
        let b = draft_path_for("/decks/a.fireside.json");
        let c = draft_path_for("/decks/b.fireside.json");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // ─── Draft-choice decision (T059/T061) ──────────────────────────────

    #[test]
    fn resolve_draft_prompt_is_none_when_no_draft_exists() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deck = dir.path().join("deck.json");
        std::fs::write(&deck, SPOTLESS_DECK).expect("write fixture");
        let saved = Graph::from_json(SPOTLESS_DECK).expect("fixture parses");

        let missing = dir.path().join("drafts").join("none.json");
        assert!(resolve_draft_prompt(&missing, &deck, &saved).is_none());
    }

    #[test]
    fn resolve_draft_prompt_is_none_when_the_draft_matches_the_saved_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deck = dir.path().join("deck.json");
        std::fs::write(&deck, SPOTLESS_DECK).expect("write fixture");
        let saved = Graph::from_json(SPOTLESS_DECK).expect("fixture parses");

        let draft_path = dir.path().join("draft.json");
        write_draft(&draft_path, &deck.display().to_string(), &saved);

        assert!(resolve_draft_prompt(&draft_path, &deck, &saved).is_none());
    }

    #[test]
    fn resolve_draft_prompt_offers_the_draft_when_it_differs_with_both_timestamps() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deck = dir.path().join("deck.json");
        std::fs::write(&deck, SPOTLESS_DECK).expect("write fixture");
        let saved = Graph::from_json(SPOTLESS_DECK).expect("fixture parses");

        let draft = Graph::from_json(r#"{"nodes":[{"id":"a","title":"Recovered","content":[]}]}"#)
            .expect("draft fixture parses");
        let draft_path = dir.path().join("draft.json");
        write_draft(&draft_path, &deck.display().to_string(), &draft);

        let prompt =
            resolve_draft_prompt(&draft_path, &deck, &saved).expect("draft differs from saved");
        assert_eq!(prompt.draft, draft);
        assert!(!prompt.draft_touched.is_empty());
        assert!(!prompt.saved_touched.is_empty());
    }

    // ─── format_ago ──────────────────────────────────────────────────────

    #[test]
    fn format_ago_covers_the_plain_language_bands() {
        let now = SystemTime::now();
        assert_eq!(format_ago(now, now), "just now");
        assert_eq!(
            format_ago(now, now - Duration::from_secs(30)),
            "30 seconds ago"
        );
        assert_eq!(
            format_ago(now, now - Duration::from_secs(120)),
            "2 minutes ago"
        );
        assert_eq!(
            format_ago(now, now - Duration::from_secs(3600)),
            "1 hour ago"
        );
        assert_eq!(
            format_ago(now, now - Duration::from_secs(2 * 86400)),
            "2 days ago"
        );
    }
}
