# Research: Dual-Screen Presenter View

## §1: Where does session state live, and how is it keyed?

**Decision**: A dedicated file per deck at
`$XDG_STATE_HOME/fireside/sessions/<fnv1a64-hex-of-canonical-path>.json`
(falling back to `~/.local/state` exactly as `resume.rs::resume_path` does).

**Rationale**: `resume.json` is a single JSON object holding every deck the
user has ever presented, read-modify-write on each save (`ResumeStore::save`
rewrites the whole map). The session file is written on every event-loop
tick — roughly 4 times a second while presenting — and must support a
follower reading it concurrently without ever seeing a torn write. Putting
heartbeat traffic into `resume.json` would mean: (a) every tick rewrites a
map containing every other deck's resume record too, multiplying blast
radius for a bug, (b) two decks presented back-to-back in the same session
would race each other's heartbeat writes into one file, and (c) the file
`resume.rs`'s own tests and every other read path treat as a cold, rarely-
touched cache would become hot — a behavior change to an already-shipped,
already-tested contract for no benefit. A separate file removes all three
problems for the cost of one small new module.

**Filename derivation — why FNV-1a 64, not the canonical path itself, not
`DefaultHasher`, not the existing `(mtime, len)` fingerprint**:
- The canonical path itself can't be the filename (contains `/`, arbitrary
  length, arbitrary characters).
- `std::collections::hash_map::DefaultHasher` is explicitly documented as
  *not* guaranteed stable across Rust compiler versions or even process
  runs with different `RUSTFLAGS` — the presenter and a separately-launched
  follower process must derive the *same* filename from the same path, so a
  hash that can silently change between builds is disqualified.
  `resume.rs` avoids this same trap by keying on the literal canonicalized
  path string, but a literal path can't be a filename here.
- `watch::fingerprint` (`(SystemTime, u64)`, i.e. mtime+length) is the wrong
  *kind* of value — it's a staleness check, not a stable identifier; it
  changes every time the deck file is edited, which would silently orphan
  the session file's identity on the very first live edit.
- FNV-1a 64-bit is ~6 lines of `u64` multiply/xor over the path's UTF-8
  bytes, deterministic across processes, versions, and platforms, needs no
  new dependency, and only needs to avoid *accidental* collision within one
  user's own deck set (not cryptographic collision resistance) — this is
  exactly the profile FNV was designed for.

**Alternatives considered**:
- *Extend `resume.json`'s record with heartbeat fields* — rejected in the
  audit plan itself (see plan.md's Constitution Check) for the
  race/blast-radius reasons above; superseded before this research phase
  even started.
- *A single global session file with a path-keyed map, like `resume.json`* —
  rejected for the same read-modify-write race under concurrent presenters,
  plus it reintroduces exactly the shared-file contention this feature
  exists to avoid.
- *A directory of files named by a percent-encoded path* — rejected: path
  length/character edge cases (very long paths, non-UTF-8 filesystem
  entries) reintroduce filesystem portability concerns FNV hashing avoids
  outright, for no behavioral benefit over the hash.

## §2: How does the presenter report reveal step and elapsed time, when `PositionSink` only carries a node id?

**Decision**: Add a new sibling sink, not a widened `PositionSink`.

**Rationale**: `PositionSink` (`&mut dyn FnMut(&str)`) is called only when
the current node id *changes* — that's exactly right for resume-on-relaunch,
which only cares about the last node reached. The session file's heartbeat
must be refreshed on *every* tick regardless of whether the node changed
(per spec FR-004/SC-002: a follower must detect a dead presenter within ~2s
even if the presenter died sitting still on one slide) — a sink that only
fires on change cannot drive a heartbeat. Rather than overload
`PositionSink` with change-detection semantics it doesn't have today, add:

```rust
pub struct SessionTick {
    pub node_id: String,
    pub reveal_step: usize,
    pub reveal_total: usize,
    pub elapsed: Duration,
}
pub type SessionTickSink<'a> = &'a mut dyn FnMut(SessionTick);
```

called once per `event_loop` iteration (same place `on_position_changed` is
currently invoked, but unconditionally rather than only on change), sourced
from `Session::current().id`, `Session::reveal_progress()` (already
returns `Option<(usize, usize)>` — `(0, 0)` when the node has no reveal
steps, matching the "notes window shows 3/5 revealed" spec language when
present and simply omitting reveal progress when absent), and
`App::elapsed()` (already exists). `present_authoring` gains this as a new
parameter; its only caller (`main.rs::present`) is updated in the same
change. This keeps `PositionSink` exactly as it is (resume continues to
work unmodified) and adds one small, single-purpose sink alongside it,
consistent with the "a caller that wants X owns all I/O for X" pattern
already used three times in this file (`ReloadSource`, `WriteBackSink`,
`PositionSink`).

**Alternatives considered**:
- *Widen `PositionSink` to `FnMut(&str, usize, usize, Duration)`* — rejected:
  changes semantics (must now fire every tick, not just on change), which
  would make resume's own logic re-derive "did the node change" itself for
  no reason, and produces an unreadable five-argument closure at the one
  call site.
- *Have the CLI poll `Session` state directly from outside `fireside-tui`* —
  impossible without breaking the crate boundary: `Session` lives inside the
  TUI's event loop; `fireside-cli` never gets a reference to it.

## §3: Should the follower reuse `App`/`Msg`/`Screen` (the presenter's TEA machine)?

**Decision**: No — a new, separate, much smaller state type
(`fireside-tui::follower`), with its own tiny update function and its own
render module, sharing no code with `App` beyond `theme::Tokens` and model
types (`Graph`, `Node`, `BranchOption`).

**Rationale**: `App` is built around a live `Session` that *mutates* via
traversal (`next`, `back`, `choose`, reveal-stepping) — the follower does
none of that; it only ever resolves a `node_id` it was told about against
its own loaded `Graph` with a single `Graph::node(id)` lookup, which needs
no `Session` at all (`Session` exists to enforce traversal invariants the
follower has no reason to re-derive, since it never navigates). Grafting a
"read-only mode" onto `App` would mean auditing every one of `on_key`'s
~15 branches, `on_reload`, `on_save_result`, quick-edit, branch-selection,
and map-navigation code paths to make sure none of them can fire in
follower mode — a much larger surface for a bug to hide in than writing
~100 lines of new, obviously-simple state. The follower's own state is
small enough to fit in one screen:

```rust
pub struct Follower {
    graph: Graph,
    status: SessionStatus,     // Running(SessionSnapshot) | NotRunning
    quit: bool,
}
```

with an `update(&mut self, msg: FollowerMsg)` taking only `Terminal(Event)`
(quit on `q`/Ctrl+C, nothing else is interactive), `Reload(Result<Graph,
String>)` (deck file changed — same shape as `App`'s `Msg::Reload`, reused
verbatim as a type, not behavior), and `SessionUpdate(SessionStatus)` (the
next poll of the session file). This keeps the TEA invariant (Principle IV:
one function mutates state) without touching the presenter's existing,
already-tested machine at all — zero regression risk to `present`/`demo`.

**Alternatives considered**:
- *Add a `read_only: bool` flag to `App`* — rejected per the audit above:
  large surface, high regression risk to a heavily-tested existing type,
  for a feature that needs none of `App`'s actual capabilities.
- *Put the follower in `fireside-cli` directly* — impossible: Principle III
  forbids rendering outside `fireside-tui`, and the follower is 100%
  rendering (it has no traversal, no write-back, no local mutation logic
  beyond "did the poll return something new").

## §4: How does the follower resolve "next slide" at a branch, and at the final slide?

**Decision**: Resolve directly from the loaded `Graph`, no `Session` needed
(confirms §3): `graph.node(&snapshot.node_id)` gives the current `Node`;
`Node::branch_point()` gives `Some(&BranchPoint)` when the presenter is at a
choice (render its `options` — label + key — instead of a single "next"
line); otherwise `Node::next_target()` gives the next node id, which is
looked up via `graph.node(...)` for its title; `Node::is_terminal()` true
with no branch point means "final slide" (render that explicitly, per spec
edge case, rather than an empty next field). Reveal progress comes from the
`SessionSnapshot`'s `reveal_step`/`reveal_total` fields (sourced from
`Session::reveal_progress()` on the presenter side, per §2) — the follower
never recomputes it from the graph itself, since only the live `Session`
knows which reveal step is currently showing.

**Rationale**: This is exactly the "resolves the session file's `node_id`
against its own loaded graph" data flow the audit plan specifies (W4-DS-4).
It requires nothing from `fireside-engine` beyond types already public
(`Node`, `BranchPoint`, `BranchOption`) and needs no new engine method.

**Alternatives considered**:
- *Have the presenter also write the next-node title and branch options into
  the session file* — rejected: redundant with data the follower already
  has for free from its own loaded `Graph`, and would make the session file
  grow every time a node's content model grows, coupling a host-local cache
  file to protocol content in a way `resume.json` deliberately never has.

## §5: Non-tty guard for `fireside notes`

**Decision**: Reuse `TuiError::NotATty` and the same `is_tty()` check
`present_impl` already performs, at the top of the new `follow()` entry
point in `fireside-tui`, before any terminal init — identical shape to
P0-3's fix. `main.rs::exit_on_not_a_tty` is currently monomorphic over
`Result<PresentSummary, TuiError>`; it needs a small signature
generalization (`fn exit_on_not_a_tty<T>(result: Result<T, TuiError>) ->
Result<T>`) so both `present`/`demo` and the new `notes` command share it
rather than duplicating the match. A one-line, behavior-preserving change,
covered by the existing `cli_e2e.rs` non-tty tests continuing to pass.

**Rationale**: Consistency (spec FR-010) — a second command with different
non-tty behavior than the first would be a new inconsistency of exactly the
kind the audit plan's P0-3/CH-3 already closed once.
