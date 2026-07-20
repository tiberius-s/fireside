//! Host-local resume-state storage (contracts/resume-state-format.md):
//! remembers each deck's last position, keyed by its canonicalized absolute
//! path, so an interrupted presentation reopens where it left off even after
//! the file has been edited. Not part of the portable deck format, not
//! protocol-versioned — disposable local cache. Uses only
//! `std::env`/`std::path` and the already-permitted `serde_json` (via its
//! `Value`/`Map`, so no new `serde` dependency is needed) per research.md §2.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

use crate::fingerprint;

/// The resume-state file: a path-keyed map of last positions.
/// A missing file, an unparseable file, or a lookup miss are all the same
/// outcome — "no record" — never an error surfaced to the presenter.
pub(crate) struct ResumeStore {
    path: Option<PathBuf>,
    entries: Map<String, Value>,
}

impl ResumeStore {
    /// Load the store from its default location. Corruption or absence
    /// yields an empty (but still writable) store.
    pub(crate) fn load() -> Self {
        Self::load_from(resume_path())
    }

    fn load_from(path: Option<PathBuf>) -> Self {
        let entries = path
            .as_deref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        Self { path, entries }
    }

    /// The node id last recorded for `key`, if any.
    #[must_use]
    pub(crate) fn node_for(&self, key: &str) -> Option<String> {
        self.entries
            .get(key)?
            .get("node_id")?
            .as_str()
            .map(str::to_owned)
    }

    /// Which node id (if any) a presentation should open at: `None` when
    /// `restart` was requested (the record is left untouched — `--restart`
    /// skips the lookup for this run only, per contracts/cli-flags.md),
    /// when there is no key at all (no backing file — a demo/one-off
    /// presentation never has one), or when there is no record for this
    /// path. A caller ignoring `Some(id)` because the graph has changed
    /// shape gets the same safe fallback for free: `Session::goto` on an
    /// unknown id is already a guarded no-op (FR-008).
    #[must_use]
    pub(crate) fn resolve_initial_node(&self, key: Option<&str>, restart: bool) -> Option<String> {
        if restart {
            return None;
        }
        self.node_for(key?)
    }

    /// Record `node_id` as the current position for `key` (a canonicalized
    /// absolute path), persisting immediately — a resume record must
    /// survive a crash on the very next instruction, not just a clean exit.
    /// The record also carries the file's current (mtime, length) as a
    /// staleness annotation; it is never compared during lookup today, only
    /// stored for a future "deck changed since you left" toast.
    pub(crate) fn set(&mut self, key: String, node_id: &str) {
        let mut record = serde_json::json!({ "node_id": node_id, "updated": epoch_seconds() });
        if let (Some(map), Some(fp)) = (record.as_object_mut(), fingerprint_annotation(&key)) {
            map.insert("fingerprint".to_owned(), Value::String(fp));
        }
        self.entries.insert(key, record);
        self.save();
    }

    /// Remove any resume record for `key` — called when a session reaches a
    /// normal end, so a completed run does not leave a stale mid-deck
    /// pointer for the next launch.
    pub(crate) fn clear(&mut self, key: &str) {
        if self.entries.remove(key).is_some() {
            self.save();
        }
    }

    /// Prunes legacy (pre-path-keyed) entries and entries whose path has
    /// since been deleted, then writes. Migration is mechanical, no version
    /// field needed: legacy keys are bare `<mtime>.<nanos>-<len>`
    /// fingerprints and never begin with a path separator, so any key that
    /// isn't an absolute path is a legacy entry.
    fn save(&mut self) {
        self.entries
            .retain(|key, _| Path::new(key).is_absolute() && Path::new(key).exists());

        let Some(path) = &self.path else { return };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&Value::Object(self.entries.clone())) {
            let _ = std::fs::write(path, json);
        }
    }
}

/// `$XDG_STATE_HOME/fireside/resume.json`, falling back to
/// `~/.local/state/fireside/resume.json`. Manual `std::env`/`std::path`
/// construction rather than a `dirs`-style crate — flagged in research.md
/// §2 as a reviewable, no-new-dependency default.
fn resume_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state"))
        })?;
    Some(base.join("fireside").join("resume.json"))
}

/// A deck's resume-state map key: its canonicalized absolute path,
/// string-encoded. `None` when the path cannot be canonicalized (e.g. it
/// doesn't exist), which structurally means "cannot resume" — the same
/// outcome as a lookup miss.
#[must_use]
pub(crate) fn resume_key(path: &Path) -> Option<String> {
    let canonical = std::fs::canonicalize(path).ok()?;
    Some(canonical.to_string_lossy().into_owned())
}

/// The `fingerprint` staleness annotation stored alongside a record: the
/// keyed path's current (mtime, length), string-encoded. Informational
/// only — never parsed back by any lookup logic today.
fn fingerprint_annotation(key: &str) -> Option<String> {
    let (modified, len) = fingerprint(Path::new(key))?;
    let since_epoch = modified.duration_since(UNIX_EPOCH).ok()?;
    Some(format!("{}:{len}", since_epoch.as_secs()))
}

/// Informational only (contracts/resume-state-format.md) — never parsed
/// back by any logic, so plain seconds-since-epoch is enough without a
/// datetime dependency.
fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real file to canonicalize a resume key against — `resume_key`
    /// requires the path to exist, and `save()` prunes any entry whose
    /// keyed path doesn't exist on disk.
    fn deck_file(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, "{}").expect("write fixture deck");
        path
    }

    #[test]
    fn missing_or_corrupt_file_yields_an_empty_store() {
        let store = ResumeStore::load_from(Some(PathBuf::from("/nonexistent/does-not-exist.json")));
        assert_eq!(store.node_for("anything"), None);
    }

    #[test]
    fn set_then_load_round_trips_the_node_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deck = deck_file(dir.path(), "deck.fireside.json");
        let key = resume_key(&deck).expect("canonicalize fixture");

        let store_path = dir.path().join("resume.json");
        let mut store = ResumeStore::load_from(Some(store_path.clone()));
        store.set(key.clone(), "features");

        let reloaded = ResumeStore::load_from(Some(store_path));
        assert_eq!(reloaded.node_for(&key).as_deref(), Some("features"));
    }

    #[test]
    fn clear_removes_the_record() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deck = deck_file(dir.path(), "deck.fireside.json");
        let key = resume_key(&deck).expect("canonicalize fixture");

        let store_path = dir.path().join("resume.json");
        let mut store = ResumeStore::load_from(Some(store_path.clone()));
        store.set(key.clone(), "features");
        store.clear(&key);

        let reloaded = ResumeStore::load_from(Some(store_path));
        assert_eq!(reloaded.node_for(&key), None);
    }

    #[test]
    fn unrelated_paths_do_not_collide() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deck_a = deck_file(dir.path(), "a.fireside.json");
        let deck_b = deck_file(dir.path(), "b.fireside.json");
        let key_a = resume_key(&deck_a).expect("canonicalize a");
        let key_b = resume_key(&deck_b).expect("canonicalize b");

        let mut store = ResumeStore::load_from(Some(dir.path().join("resume.json")));
        store.set(key_a.clone(), "a-node");
        store.set(key_b.clone(), "b-node");

        assert_eq!(store.node_for(&key_a).as_deref(), Some("a-node"));
        assert_eq!(store.node_for(&key_b).as_deref(), Some("b-node"));
    }

    #[test]
    fn restart_bypasses_the_lookup_without_deleting_the_record() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deck = deck_file(dir.path(), "deck.fireside.json");
        let key = resume_key(&deck).expect("canonicalize fixture");

        let store_path = dir.path().join("resume.json");
        let mut store = ResumeStore::load_from(Some(store_path.clone()));
        store.set(key.clone(), "features");

        assert_eq!(
            store.resolve_initial_node(Some(&key), true),
            None,
            "restart wins"
        );

        let reloaded = ResumeStore::load_from(Some(store_path));
        assert_eq!(
            reloaded.node_for(&key).as_deref(),
            Some("features"),
            "the record survives a --restart run for next time"
        );
    }

    #[test]
    fn no_key_means_no_initial_node() {
        // A presentation with no backing file (e.g. `fireside demo`) never
        // has a key to look up — structurally cannot resume (FR-009).
        let store = ResumeStore::load_from(None);
        assert_eq!(store.resolve_initial_node(None, false), None);
    }

    #[test]
    fn missing_record_for_a_known_path_means_no_initial_node() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deck = deck_file(dir.path(), "deck.fireside.json");
        let key = resume_key(&deck).expect("canonicalize fixture");

        let store = ResumeStore::load_from(None);
        assert_eq!(store.resolve_initial_node(Some(&key), false), None);
    }

    #[test]
    fn editing_the_file_does_not_orphan_the_resume_record() {
        // The headline bug (P1-1): a path-keyed record survives an edit
        // that changes the file's mtime/length, unlike the old
        // fingerprint-keyed scheme.
        let dir = tempfile::tempdir().expect("tempdir");
        let deck = deck_file(dir.path(), "deck.fireside.json");
        let key = resume_key(&deck).expect("canonicalize fixture");

        let store_path = dir.path().join("resume.json");
        let mut store = ResumeStore::load_from(Some(store_path.clone()));
        store.set(key.clone(), "features");

        std::fs::write(&deck, "{\"edited\": true}").expect("edit fixture");
        let key_after_edit = resume_key(&deck).expect("canonicalize fixture");
        assert_eq!(key, key_after_edit, "path key is stable across edits");

        let reloaded = ResumeStore::load_from(Some(store_path));
        assert_eq!(reloaded.node_for(&key).as_deref(), Some("features"));
    }

    #[test]
    fn legacy_fingerprint_keyed_entries_are_pruned_on_save() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store_path = dir.path().join("resume.json");
        std::fs::write(
            &store_path,
            r#"{"1700000000.0-42": {"node_id": "old", "updated_at": 1700000000}}"#,
        )
        .expect("seed legacy store");

        let deck = deck_file(dir.path(), "deck.fireside.json");
        let key = resume_key(&deck).expect("canonicalize fixture");

        let mut store = ResumeStore::load_from(Some(store_path.clone()));
        assert_eq!(
            store.node_for("1700000000.0-42").as_deref(),
            Some("old"),
            "legacy entry is still readable until the next save"
        );
        store.set(key, "features");

        let reloaded = ResumeStore::load_from(Some(store_path));
        assert_eq!(
            reloaded.node_for("1700000000.0-42"),
            None,
            "legacy non-path key is dropped on save"
        );
    }

    #[test]
    fn entries_for_deleted_paths_are_pruned_on_save() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deck_a = deck_file(dir.path(), "a.fireside.json");
        let deck_b = deck_file(dir.path(), "b.fireside.json");
        let key_a = resume_key(&deck_a).expect("canonicalize a");
        let key_b = resume_key(&deck_b).expect("canonicalize b");

        let store_path = dir.path().join("resume.json");
        let mut store = ResumeStore::load_from(Some(store_path.clone()));
        store.set(key_a.clone(), "a-node");
        store.set(key_b.clone(), "b-node");

        std::fs::remove_file(&deck_a).expect("delete a");
        // Any save (here, setting b again) prunes entries for paths that no
        // longer exist.
        store.set(key_b.clone(), "b-node-2");

        let reloaded = ResumeStore::load_from(Some(store_path));
        assert_eq!(reloaded.node_for(&key_a), None, "deleted path is pruned");
        assert_eq!(reloaded.node_for(&key_b).as_deref(), Some("b-node-2"));
    }
}
