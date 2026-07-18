//! The deck file watcher: cheap fingerprint polling for the presenting
//! live-reload loop, quick-edit write-back, and the `validate --watch`
//! authoring loop.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use fireside_core::{CoreError, Graph};
use fireside_tui::WriteBackError;

use crate::report::{strip_position, watch_report};

/// Watches the deck file while presenting: cheap fingerprint check per
/// poll, full re-read and re-parse only when the file actually changed.
pub(crate) struct Watcher {
    path: PathBuf,
    fingerprint: Option<(SystemTime, u64)>,
}

impl Watcher {
    pub(crate) fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            fingerprint: fingerprint(path),
        }
    }

    /// `None` while the file is unchanged (or briefly unreadable mid-save);
    /// otherwise the freshly parsed deck or a one-line footer message.
    pub(crate) fn poll(&mut self) -> Option<Result<Graph, String>> {
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
    pub(crate) fn write_back(&mut self, graph: &Graph) -> Result<(), WriteBackError> {
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
pub(crate) fn fingerprint(path: &Path) -> Option<(SystemTime, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

/// Check `path` immediately, then keep re-checking on a short poll and
/// re-report whenever the file changes — the same cadence `present`'s
/// live reload already uses, so a save-and-look loop feels the same
/// whether you're authoring or presenting.
pub(crate) fn watch_loop(path: &Path) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A single terminal node with no traversal and no content — the
    /// smallest deck that produces zero diagnostics of any severity.
    const SPOTLESS_DECK: &str = r#"{"nodes":[{"id":"a","content":[]}]}"#;

    const HELLO: &str = include_str!("../../../docs/examples/hello.json");

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

    /// Spec 008 US3: closes a coverage gap the single-malformed-write
    /// tests above don't reach — recovery must not depend on the invalid
    /// streak "settling" first. Drives `Watcher::poll()` directly (rather
    /// than through `watch_report`, which builds a fresh `Watcher` per
    /// call) through valid → truncated → still-invalid (different
    /// malformed payload) → valid again, asserting no panic at any step
    /// and that each poll's `Result` matches its file's actual state.
    #[test]
    fn watcher_recovers_after_a_rapid_invalid_then_valid_sequence() {
        let temp = tempfile::tempdir().expect("temp dir");
        let deck = temp.path().join("deck.json");
        std::fs::write(&deck, SPOTLESS_DECK).expect("write fixture");

        let mut watcher = Watcher::new(&deck);

        // A non-atomic editor save caught mid-write: truncated JSON.
        std::fs::write(&deck, "{\n  \"nodes\": [{\"id\"").expect("simulate truncated save");
        match watcher.poll() {
            Some(Err(_)) => {}
            other => panic!("expected a reload error for truncated JSON, got {other:?}"),
        }

        // A second, differently-broken write before a valid one lands —
        // recovery must not require the invalid streak to "settle".
        std::fs::write(&deck, "not json at all").expect("simulate a second broken save");
        match watcher.poll() {
            Some(Err(_)) => {}
            other => panic!("expected a reload error for the second broken save, got {other:?}"),
        }

        // The save completes: valid JSON, different content than the
        // original so the fingerprint is guaranteed to differ.
        let recovered = r#"{"nodes":[{"id":"a","title":"recovered","content":[]}]}"#;
        std::fs::write(&deck, recovered).expect("simulate the completed save");
        match watcher.poll() {
            Some(Ok(graph)) => {
                assert_eq!(graph.nodes[0].title.as_deref(), Some("recovered"));
            }
            other => panic!("expected a successful reload once the file is valid, got {other:?}"),
        }
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
