//! Host-local resume-state storage (contracts/resume-state-format.md):
//! remembers each deck's last position, keyed by content fingerprint, so an
//! interrupted presentation reopens where it left off. Not part of the
//! portable deck format, not protocol-versioned — disposable local cache.
//! Uses only `std::env`/`std::path` and the already-permitted `serde_json`
//! (via its `Value`/`Map`, so no new `serde` dependency is needed) per
//! research.md §2.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

use crate::fingerprint;

/// The resume-state file: a fingerprint-keyed map of last positions.
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
    /// when there is no fingerprint at all (no backing file — a demo/one-off
    /// presentation never has one), or when there is no record for this
    /// fingerprint. A caller ignoring `Some(id)` because the graph has
    /// changed shape gets the same safe fallback for free: `Session::goto`
    /// on an unknown id is already a guarded no-op (FR-008).
    #[must_use]
    pub(crate) fn resolve_initial_node(&self, key: Option<&str>, restart: bool) -> Option<String> {
        if restart {
            return None;
        }
        self.node_for(key?)
    }

    /// Record `node_id` as the current position for `key`, persisting
    /// immediately — a resume record must survive a crash on the very next
    /// instruction, not just a clean exit.
    pub(crate) fn set(&mut self, key: String, node_id: &str) {
        self.entries.insert(
            key,
            serde_json::json!({ "node_id": node_id, "updated_at": epoch_seconds() }),
        );
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

    fn save(&self) {
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

/// A deck's content fingerprint as a resume-state map key: the same
/// (mtime, length) pair `fingerprint()` already uses for reload/write-back
/// conflict detection, string-encoded.
#[must_use]
pub(crate) fn fingerprint_key(path: &Path) -> Option<String> {
    let (modified, len) = fingerprint(path)?;
    let since_epoch = modified.duration_since(UNIX_EPOCH).ok()?;
    Some(format!(
        "{}.{}-{len}",
        since_epoch.as_secs(),
        since_epoch.subsec_nanos()
    ))
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

    #[test]
    fn missing_or_corrupt_file_yields_an_empty_store() {
        let store = ResumeStore::load_from(Some(PathBuf::from("/nonexistent/does-not-exist.json")));
        assert_eq!(store.node_for("anything"), None);
    }

    #[test]
    fn set_then_load_round_trips_the_node_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("resume.json");
        let mut store = ResumeStore::load_from(Some(path.clone()));
        store.set("fp-1".to_owned(), "features");
        let reloaded = ResumeStore::load_from(Some(path));
        assert_eq!(reloaded.node_for("fp-1").as_deref(), Some("features"));
    }

    #[test]
    fn clear_removes_the_record() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("resume.json");
        let mut store = ResumeStore::load_from(Some(path.clone()));
        store.set("fp-1".to_owned(), "features");
        store.clear("fp-1");
        let reloaded = ResumeStore::load_from(Some(path));
        assert_eq!(reloaded.node_for("fp-1"), None);
    }

    #[test]
    fn unrelated_fingerprints_do_not_collide() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("resume.json");
        let mut store = ResumeStore::load_from(Some(path));
        store.set("fp-1".to_owned(), "a");
        store.set("fp-2".to_owned(), "b");
        assert_eq!(store.node_for("fp-1").as_deref(), Some("a"));
        assert_eq!(store.node_for("fp-2").as_deref(), Some("b"));
    }

    #[test]
    fn restart_bypasses_the_lookup_without_deleting_the_record() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("resume.json");
        let mut store = ResumeStore::load_from(Some(path.clone()));
        store.set("fp-1".to_owned(), "features");

        assert_eq!(
            store.resolve_initial_node(Some("fp-1"), true),
            None,
            "restart wins"
        );

        let reloaded = ResumeStore::load_from(Some(path));
        assert_eq!(
            reloaded.node_for("fp-1").as_deref(),
            Some("features"),
            "the record survives a --restart run for next time"
        );
    }

    #[test]
    fn no_fingerprint_means_no_initial_node() {
        // A presentation with no backing file (e.g. `fireside demo`) never
        // has a fingerprint to look up — structurally cannot resume (FR-009).
        let store = ResumeStore::load_from(None);
        assert_eq!(store.resolve_initial_node(None, false), None);
    }

    #[test]
    fn missing_record_for_a_known_fingerprint_means_no_initial_node() {
        let store = ResumeStore::load_from(None);
        assert_eq!(store.resolve_initial_node(Some("never-seen"), false), None);
    }
}
