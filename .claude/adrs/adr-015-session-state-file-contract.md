---
title: 'ADR-015: Session-state file contract for the dual-screen presenter view'
status: 'accepted'
date: '2026-07-20'
deciders: ['@tiberius']
---

# ADR-015: Session-state file contract for the dual-screen presenter view

## Status

Accepted. Depends on ADR-014 (the scope extension this file exists to
support).

## Context

`fireside notes <deck>` (ADR-014) needs to learn, from a separate process,
which node the presenter is currently on, how far into a reveal sequence it
is, how long the talk has run, and whether the presenter is even still
running — polled roughly 4 times a second (the same 250ms cadence
`watch.rs` already uses for deck-file live reload) so the follower feels
live rather than laggy.

Fireside already has one host-local, path-keyed persistence file:
`resume.json` (`contracts/resume-state-format.md`, spec 007). The obvious
first idea is extending that record with the new fields instead of adding
a second file. This was considered and rejected:

- `resume.json` is a **single JSON object holding every deck the user has
  ever presented**, read with a full parse and rewritten with a full
  `serde_json::to_string_pretty` of the whole map on every `ResumeStore::save`
  call (`resume.rs`). A session heartbeat firing 4 times a second would mean
  4 rewrites per second of a file containing every *other* deck's resume
  record too — a large, unnecessary blast radius for a value that is
  relevant to exactly one deck.
- Two decks presented back-to-back in one working session (a common
  workflow: rehearse deck A, then deck B) would have their heartbeats race
  into the *same* file's read-modify-write cycle. `resume.json`'s own
  save path was written and tested assuming infrequent, uncontended writes
  (once per navigation move, never concurrent); making it hot in this way
  is a behavior change to an already-shipped, already-tested contract, for
  a feature that doesn't need it to be shared at all.
- The two files have fundamentally different write frequency and audience:
  `resume.json` is a cold cache read once at the next launch of a specific
  deck; the session file is a hot, single-deck heartbeat with a live
  reader watching it in near-real-time. Conflating them would make the
  colder file behave like the hotter one for every deck, forever.

## Decision

Live session state gets its **own file per deck**, never a
`resume.json` field. Full contract:
`specs/012-presenter-view/contracts/session-state-format.md`. Summary:

- **Location**: `$XDG_STATE_HOME/fireside/sessions/<key>.json`, falling
  back to `~/.local/state/...` — same base-directory resolution
  `resume_path()` already uses, one segment deeper.
- **Filename key**: the FNV-1a 64-bit hash (hex-encoded) of the deck's
  canonicalized absolute path — the same path `resume::resume_key` already
  computes, hashed rather than used verbatim because it must become a
  filename. FNV-1a chosen over the alternatives considered:
  - `std::collections::hash_map::DefaultHasher` is explicitly documented as
    **not** guaranteed stable across compiler versions or process
    invocations — unacceptable when the presenter and a separately
    launched follower process must independently derive the *same*
    filename from the same path.
  - `watch::fingerprint`'s `(mtime, len)` pair is the wrong kind of value
    entirely — a staleness check, not a stable identifier; it changes on
    every edit, which would silently orphan the session file's identity on
    the deck's first live edit of the talk.
  - FNV-1a 64 is ~6 lines of dependency-free `u64` arithmetic, deterministic
    across processes/versions/platforms, and only needs to avoid
    accidental collision within one user's own deck set — exactly the
    profile it was designed for.
- **Schema**: `{"schema": 1, "deck-path", "node-id", "reveal-step",
  "reveal-total", "elapsed-secs", "heartbeat"}`, kebab-case (the workspace's
  current serde convention — this file has no legacy shape to preserve,
  unlike `resume.json`'s pre-convention snake_case).
- **Writer**: exactly one, the presenting process, on every event-loop
  tick (not only on navigation change — the heartbeat must advance even
  while sitting still on one slide, or a dead-but-motionless presenter
  would look alive).
- **Readers**: any number of `fireside notes` followers of the same deck,
  concurrently, for free — none of them write anything.
- **Atomicity**: temp file + rename within the same directory on every
  write, so a concurrent reader never observes a torn write. `resume.json`
  gets away with a direct `std::fs::write` because it has exactly one
  reader at a time (the next launch); this file does not have that luxury.
- **Staleness**: a heartbeat more than 2 seconds old is treated identically
  to a missing or corrupt file — all three collapse to "not running,"
  never a distinguished error, because a follower has no way to tell a
  crash from a clean exit from "never started" and should not pretend
  otherwise.
- **Lifecycle**: created on the presenter's first tick against a real
  backing file (no file for `fireside demo`, matching `resume.rs`'s
  "no key, no record" rule), deleted on a clean `q`-quit exit, left in
  place (and thus aged into staleness) on a crash — no crash-cleanup logic
  is attempted; staleness alone is the recovery mechanism, deliberately,
  to avoid adding signal-handler complexity for a self-healing failure
  mode.

Not protocol-versioned, not part of the portable deck format — same
governance posture as `resume.json` (Constitution Principle I is
unaffected; this is host-local cache, never written to or read from the
`.fireside.json` document itself).

## Consequences

### Positive

- `resume.json`'s existing behavior, tests, and performance characteristics
  are completely unaffected — this feature adds a file, it does not touch
  an existing one beyond the unrelated P1-1 rekeying already shipped.
- Two presenters of different decks (or the same deck relaunched) never
  contend on the same file; each gets its own hash-named file.
- The one-writer/N-reader shape means no locking, no coordination protocol,
  and no new dependency are needed — `std::fs::rename`'s platform-level
  atomicity is the entire correctness mechanism.

### Negative or Trade-offs

- A second host-local file format now exists alongside `resume.json`'s,
  with a different (kebab-case) key convention between them — a minor
  inconsistency for anyone reading both by hand. Accepted: matching
  `resume.json`'s snake\_case-ish keys would mean adopting a *legacy*
  convention on a brand-new file for no reason; the workspace's current
  convention is kebab-case everywhere else.
- `sessions/` accumulates one small file per distinct deck path ever
  presented, with no automatic pruning of very old, long-stale files.
  Accepted as out of scope: the same low-stakes disposable-cache posture
  `resume.json` already has (it too is never proactively pruned beyond
  path-existence checks on save), left for a future cleanup pass only if
  it's ever observed to matter.

### Neutral / Follow-up

- No Constitution Principle III (crate boundary) or allowlist amendment is
  needed — the new `fireside-cli::session` module uses only
  `std::fs`/`std::path`/`std::time` and the already-permitted
  `serde_json`, the same posture `resume.rs` already established.
- Proceed with `specs/012-presenter-view/tasks.md`'s Foundational phase
  (T003–T009): the `session.rs` module and the `SessionTick`/
  `SessionSnapshot`/`SessionStatus` types crossing into `fireside-tui`.
