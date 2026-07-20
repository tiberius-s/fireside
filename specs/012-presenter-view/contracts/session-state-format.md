# Contract: Session-state file format

Host-local, not part of the portable deck format, not covered by protocol
0.1.x versioning — same posture as `contracts/resume-state-format.md`
(spec 007), and deliberately a **separate file per deck**, not an extension
of `resume.json` (see research.md §1 for the race/blast-radius rationale;
recorded formally in ADR-015). Owned entirely by `fireside-cli`
(`fireside-tui` performs no file I/O; it receives already-parsed
`SessionStatus` values — see research.md §2–3).

**Location**: `$XDG_STATE_HOME/fireside/sessions/<key>.json`, falling back
to `~/.local/state/fireside/sessions/<key>.json` when `XDG_STATE_HOME` is
unset — same base-directory resolution `resume.rs::resume_path` already
uses, one path segment deeper. `<key>` is the lowercase hex encoding of the
FNV-1a 64-bit hash of the deck's canonicalized absolute path (the same path
`resume::resume_key` computes) — a stable, deterministic, dependency-free
filename derived from an unbounded, unfilenameable path string. **One
writer** (the presenting process for that deck), **N readers** (any number
of `fireside notes` followers pointed at the same deck file).

**Shape**: one JSON object per file, describing the single live session for
the deck that hashes to this filename.

```json
{
  "schema": 1,
  "deck-path": "<canonicalized absolute path>",
  "node-id": "<string>",
  "reveal-step": 0,
  "reveal-total": 0,
  "elapsed-secs": 0,
  "heartbeat": 0
}
```

(Field names are written kebab-case, matching the workspace's existing
`rename_all = "kebab-case"` serde convention used everywhere else in this
codebase — unlike `resume.json`, which predates that convention with
snake\_case keys; this file has no legacy shape to stay compatible with, so
it starts on the current convention.)

- `schema` is `1`. A reader encountering any other value treats the file
  exactly as it would treat a missing or corrupt file — see "Reader
  contract" below. No migration logic exists yet because there is no prior
  version; this field exists so a future format change has somewhere to
  branch without breaking old followers mid-rollout.
- `deck-path` is informational only (a human `cat`-ing the file can tell
  what it's for); no reader logic compares it against anything — the
  filename hash is the actual identity.
- `node-id`, `reveal-step`, `reveal-total`, `elapsed-secs` mirror
  `fireside-tui`'s `SessionTick` (data-model.md) exactly, written on every
  event-loop tick, not only on navigation change.
- `heartbeat` is epoch seconds, refreshed on every write — the sole
  liveness signal. It is compared against the *reader's* clock, not stored
  as a duration, so clock skew within one host (both processes on the same
  machine) is a non-issue; cross-host use is out of scope (spec
  Assumptions).

**Atomicity**: every write creates a temp file in the same directory
(`sessions/.tmp-<key>-<random>` or equivalent) and renames it over the
target path. `std::fs::rename` within one filesystem is atomic on every
platform Fireside supports, so a concurrent reader observes either the
complete old content or the complete new content, never a partial write —
unlike `resume.rs`'s direct `std::fs::write`, which is acceptable there
because `resume.json` has exactly one reader at a time (the next launch of
that same deck) and no reader ever runs concurrently with a writer the way
a follower does here.

**Reader contract** (the "not running" determination, spec FR-004):
missing file, unparseable file, wrong `schema`, and a `heartbeat` more than
2 seconds older than the reader's current time are all the same outcome —
`SessionStatus::NotRunning`. A reader MUST NOT distinguish these cases in
its user-facing output (spec edge case: never-started, exited-clean, and
crashed all look identical to a follower, by design — the follower has no
way to know which one occurred, and pretending otherwise would be
misleading).

**Lifecycle**:
- Created on the first tick of a presentation against a real backing file
  (no file for `fireside demo`, which has no path to key by — matches
  `resume.rs`'s "no key, no record" rule).
- Rewritten atomically on every subsequent tick, whether or not the node
  changed (the heartbeat must advance even standing still).
- Deleted by the presenting process on a clean `q`-quit exit. Left in place
  (and thus naturally aged into `NotRunning` by the 2-second staleness
  check) on a crash or `kill -9` — no crash-cleanup logic is needed or
  attempted; staleness alone is the recovery mechanism.
- A leftover file from a session that ended hours or days ago is
  indistinguishable from a crashed one to a reader (both are just "stale");
  neither this contract nor the implementation prunes old session files
  automatically. `sessions/` accumulating one small file per distinct deck
  ever presented is the same low-stakes disposable-cache posture
  `resume.json` already has, and is left for a future cleanup pass if it
  ever matters (not scoped here — no observed problem to fix).
