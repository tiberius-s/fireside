# Contract: Resume-state file format

Host-local, not part of the portable deck format, not covered by protocol
0.1.x versioning. Owned entirely by `fireside-cli` (`fireside-tui` performs
no file I/O — see research.md §2).

**Location**: `$XDG_STATE_HOME/fireside/resume.json`, falling back to
`~/.local/state/fireside/resume.json` when `XDG_STATE_HOME` is unset. Built
with `std::env`/`std::path` only — no new dependency (flagged in
research.md §2 for review: a `dirs`-crate path would be more correct on
Windows/macOS but is a new dependency and wasn't added by default).

**Shape**: a JSON object mapping a deck's content fingerprint to its last
known position.

```json
{
  "<fingerprint>": {
    "node_id": "<string>",
    "updated_at": "<RFC 3339 timestamp>"
  }
}
```

- `fingerprint` matches the existing `main.rs::fingerprint` shape already
  used for reload/write-back conflict detection (mtime + length pair,
  string-encoded as the map key).
- Unknown top-level keys and any additional per-entry fields MUST be
  tolerated on read (forward compatible, same posture as the deck format's
  Layer 1 unknown-field tolerance).
- A missing file, an unparseable file, or a lookup miss are all the same
  outcome: no resume record — present from the graph's normal entry node.
  This file is disposable local cache, not a source of truth; corruption or
  absence is never an error the presenter surfaces to the user.
