//! Host-local session-state storage
//! (contracts/session-state-format.md, spec 012): a live heartbeat of
//! "where the presenter currently is," one file per deck, written by the
//! presenting process on every tick and read by any number of `fireside
//! notes` followers. Deliberately a separate file per deck rather than an
//! extension of `resume.json` — see ADR-015 and research.md §1: this file
//! is written ~4 times a second by a single writer, while `resume.json` is
//! a cold, shared, read-modify-write map across every deck ever presented.
//! Uses only `std::fs`/`std::path`/`std::time` and the already-permitted
//! `serde_json`, same posture as `resume.rs`.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fireside_tui::{SessionSnapshot, SessionStatus, SessionTick};
use serde_json::json;

/// How old a session file's `heartbeat` may be before a reader treats the
/// presenter as not running (contracts/session-state-format.md).
const STALE_AFTER: Duration = Duration::from_secs(2);

/// The schema version this build writes and accepts. A reader encountering
/// any other value treats the file as absent (contracts/session-state-format.md).
const SCHEMA_VERSION: u64 = 1;

/// `$XDG_STATE_HOME/fireside/sessions/<key>.json`, falling back to
/// `~/.local/state/fireside/sessions/<key>.json` — one path segment deeper
/// than `resume::resume_path`, same base-directory resolution.
fn sessions_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state"))
        })?;
    Some(base.join("fireside").join("sessions"))
}

/// The session file's path for a deck already keyed by
/// [`crate::resume::resume_key`] (its canonicalized absolute path,
/// string-encoded) — hashed into a filename with FNV-1a 64, since the key
/// itself can't be a filename (arbitrary length, contains `/`). `None`
/// when no state directory can be resolved at all (no `HOME`/
/// `XDG_STATE_HOME` — the same "cannot persist" case `resume_path` treats
/// as a no-op store).
#[must_use]
pub(crate) fn session_path_for(key: &str) -> Option<PathBuf> {
    Some(sessions_dir()?.join(format!("{:016x}.json", fnv1a64(key.as_bytes()))))
}

/// FNV-1a, 64-bit. Chosen over `DefaultHasher` (not guaranteed stable
/// across compiler versions — the presenter and a separately launched
/// follower process must derive the same filename from the same path) and
/// over `watch::fingerprint`'s `(mtime, len)` pair (a staleness check, not
/// a stable identifier — see ADR-015). `pub(crate)`: `edit.rs`'s draft
/// sidecar (spec 013 US4, T059) reuses this exact hash rather than
/// implementing a third copy (research.md §3).
#[must_use]
pub(crate) fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET_BASIS;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// Writes the presenter's current position to `path`, refreshing the
/// heartbeat — called once per event-loop tick regardless of whether the
/// position changed (a motionless-but-alive presenter must still look
/// alive). Atomic: a temp file in the same directory, then a rename, so a
/// concurrent reader never observes a partial write. Best-effort: a write
/// failure (disk full, permissions) is silently dropped, exactly like
/// `resume.rs::ResumeStore::save` — a follower losing sight of a presenter
/// for one tick is the same as it not running yet, never a crash.
pub(crate) fn write(path: &Path, deck_path: &str, tick: &SessionTick) {
    let Some(parent) = path.parent() else { return };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    let record = json!({
        "schema": SCHEMA_VERSION,
        "deck-path": deck_path,
        "node-id": tick.node_id,
        "reveal-step": tick.reveal_step,
        "reveal-total": tick.reveal_total,
        "elapsed-secs": tick.elapsed.as_secs(),
        "heartbeat": epoch_seconds(),
    });
    let Ok(text) = serde_json::to_string(&record) else {
        return;
    };
    let tmp_path = parent.join(format!(".tmp-{}-{}", std::process::id(), epoch_nanos()));
    if std::fs::write(&tmp_path, text).is_err() {
        return;
    }
    // Same-directory rename is atomic on every platform Fireside supports
    // — a reader observes either the complete old file or the complete
    // new one, never a partial write.
    let _ = std::fs::rename(&tmp_path, path);
}

/// Reads and staleness-checks the session file at `path`. Missing file,
/// unparseable JSON, an unrecognized `schema`, and a heartbeat older than
/// [`STALE_AFTER`] are all [`SessionStatus::NotRunning`] — never a
/// distinguished error (contracts/session-state-format.md's reader
/// contract).
#[must_use]
pub(crate) fn read(path: &Path) -> SessionStatus {
    let Some(snapshot) = read_fresh(path) else {
        return SessionStatus::NotRunning;
    };
    SessionStatus::Running(snapshot)
}

fn read_fresh(path: &Path) -> Option<SessionSnapshot> {
    let text = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    if value.get("schema")?.as_u64()? != SCHEMA_VERSION {
        return None;
    }
    let heartbeat = value.get("heartbeat")?.as_u64()?;
    if epoch_seconds().saturating_sub(heartbeat) > STALE_AFTER.as_secs() {
        return None;
    }
    Some(SessionSnapshot {
        node_id: value.get("node-id")?.as_str()?.to_owned(),
        reveal_step: usize::try_from(value.get("reveal-step")?.as_u64()?).ok()?,
        reveal_total: usize::try_from(value.get("reveal-total")?.as_u64()?).ok()?,
        elapsed: Duration::from_secs(value.get("elapsed-secs")?.as_u64()?),
    })
}

/// Removes the session file, called on a clean presenter exit so a
/// follower sees "not running" immediately rather than waiting out the
/// staleness window. Best-effort: a missing file (or one that can't be
/// removed) is not an error — the presenter is exiting either way.
pub(crate) fn delete(path: &Path) {
    let _ = std::fs::remove_file(path);
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn epoch_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_then_read_round_trips_the_snapshot() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("session.json");

        write(
            &path,
            "/decks/talk.fireside.json",
            &SessionTick {
                node_id: "features".to_owned(),
                reveal_step: 2,
                reveal_total: 5,
                elapsed: Duration::from_secs(42),
            },
        );

        match read(&path) {
            SessionStatus::Running(snapshot) => {
                assert_eq!(snapshot.node_id, "features");
                assert_eq!(snapshot.reveal_step, 2);
                assert_eq!(snapshot.reveal_total, 5);
                assert_eq!(snapshot.elapsed, Duration::from_secs(42));
            }
            SessionStatus::NotRunning => panic!("expected a running snapshot"),
        }
    }

    #[test]
    fn a_missing_file_reads_as_not_running() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("does-not-exist.json");
        assert_eq!(read(&path), SessionStatus::NotRunning);
    }

    #[test]
    fn a_corrupt_file_reads_as_not_running_without_panicking() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("session.json");
        std::fs::write(&path, "not json at all").expect("write corrupt fixture");
        assert_eq!(read(&path), SessionStatus::NotRunning);
    }

    #[test]
    fn a_truncated_write_never_leaves_a_partial_file_readable() {
        // write() always goes through a temp file + rename; simulate that
        // no reader can ever observe the intermediate state by checking
        // the temp file is gone after a successful write.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("session.json");
        write(
            &path,
            "/deck.json",
            &SessionTick {
                node_id: "a".to_owned(),
                reveal_step: 0,
                reveal_total: 0,
                elapsed: Duration::ZERO,
            },
        );

        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .expect("read dir")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(".tmp-"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "no temp file should remain after a successful write"
        );
    }

    #[test]
    fn a_stale_heartbeat_reads_as_not_running_even_though_the_file_parses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("session.json");
        let ancient = epoch_seconds().saturating_sub(STALE_AFTER.as_secs() + 10);
        std::fs::write(
            &path,
            json!({
                "schema": SCHEMA_VERSION,
                "deck-path": "/deck.json",
                "node-id": "a",
                "reveal-step": 0,
                "reveal-total": 0,
                "elapsed-secs": 0,
                "heartbeat": ancient,
            })
            .to_string(),
        )
        .expect("write stale fixture");

        assert_eq!(read(&path), SessionStatus::NotRunning);
    }

    #[test]
    fn an_unrecognized_schema_reads_as_not_running() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("session.json");
        std::fs::write(
            &path,
            json!({
                "schema": SCHEMA_VERSION + 1,
                "deck-path": "/deck.json",
                "node-id": "a",
                "reveal-step": 0,
                "reveal-total": 0,
                "elapsed-secs": 0,
                "heartbeat": epoch_seconds(),
            })
            .to_string(),
        )
        .expect("write future-schema fixture");

        assert_eq!(read(&path), SessionStatus::NotRunning);
    }

    #[test]
    fn delete_removes_the_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("session.json");
        write(
            &path,
            "/deck.json",
            &SessionTick {
                node_id: "a".to_owned(),
                reveal_step: 0,
                reveal_total: 0,
                elapsed: Duration::ZERO,
            },
        );
        assert!(path.exists());

        delete(&path);
        assert!(!path.exists());
    }

    #[test]
    fn delete_on_a_missing_file_does_not_panic() {
        let dir = tempfile::tempdir().expect("tempdir");
        delete(&dir.path().join("never-existed.json"));
    }

    #[test]
    fn session_path_for_is_deterministic_for_the_same_key() {
        let a = session_path_for("/decks/talk.fireside.json");
        let b = session_path_for("/decks/talk.fireside.json");
        assert_eq!(a, b);
    }

    #[test]
    fn session_path_for_differs_for_different_keys() {
        let a = session_path_for("/decks/a.fireside.json");
        let b = session_path_for("/decks/b.fireside.json");
        assert_ne!(a, b);
    }
}
