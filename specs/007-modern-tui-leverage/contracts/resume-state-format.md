# Contract: Resume-state file format

Host-local, not part of the portable deck format, not covered by protocol
0.1.x versioning. Owned entirely by `fireside-cli` (`fireside-tui` performs
no file I/O — see research.md §2).

**Location**: `$XDG_STATE_HOME/fireside/resume.json`, falling back to
`~/.local/state/fireside/resume.json` when `XDG_STATE_HOME` is unset. Built
with `std::env`/`std::path` only — no new dependency (flagged in
research.md §2 for review: a `dirs`-crate path would be more correct on
Windows/macOS but is a new dependency and wasn't added by default).

**Shape**: a JSON object mapping a deck's canonicalized absolute path to its
last known position.

```json
{
  "<canonicalized absolute path>": {
    "node_id": "<string>",
    "updated": "<epoch seconds>",
    "fingerprint": "<mtime>:<len>"
  }
}
```

- The key is the deck's absolute path after `std::fs::canonicalize`
  (`crate::resume::resume_key`), string-encoded — **not** a content
  fingerprint (revised by P1-1: fingerprint-keying orphaned the resume
  record on any edit to the file, silently, which is exactly the "night
  before the talk" moment resume exists for).
- `fingerprint` is the keyed path's current (mtime, length) pair at the
  time of the write, string-encoded. It is a staleness *annotation* only —
  never compared during lookup today, reserved for a future "deck changed
  since you left" toast.
- Unknown top-level keys and any additional per-entry fields MUST be
  tolerated on read (forward compatible, same posture as the deck format's
  Layer 1 unknown-field tolerance).
- A missing file, an unparseable file, or a lookup miss are all the same
  outcome: no resume record — present from the graph's normal entry node.
  This file is disposable local cache, not a source of truth; corruption or
  absence is never an error the presenter surfaces to the user.
- **Migration**: legacy entries from the pre-P1-1 fingerprint-keyed format
  (bare `<mtime>.<nanos>-<len>` keys, which never begin with a path
  separator) are recognized as not-a-path and silently dropped on the next
  write, alongside any entry whose keyed path no longer exists on disk. No
  version field is needed — the key shape alone disambiguates the two
  formats.
