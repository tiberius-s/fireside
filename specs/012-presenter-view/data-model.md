# Data Model: Dual-Screen Presenter View

No changes to the Fireside protocol data model (`fireside-core`). The
entities below are new, host-local, non-protocol types living in
`fireside-cli` (the on-disk record) and `fireside-tui` (the in-memory
snapshot and follower state) — the CLI's record and the TUI's snapshot are
the same information in two layers, per the existing `ReloadSource`/
`WriteBackSink` split (I/O shape in the CLI, plain data crossing into the
TUI).

## SessionRecord (on disk — `fireside-cli::session`)

The JSON shape written by the presenter and read by the follower. Full
contract in [`contracts/session-state-format.md`](./contracts/session-state-format.md).
Rust struct field names below are `snake_case` per normal Rust convention;
the wire (on-disk JSON) field names are `kebab-case` per the contract —
e.g. the struct field `node_id` serializes as `"node-id"`.

| Field (struct / wire)       | Type             | Notes                                                                 |
| ---------------------------- | ---------------- | ---------------------------------------------------------------------- |
| `schema` / `schema`           | `u32`            | `1`. A follower reading a future higher schema treats it as "not running" (see contract) rather than misparsing it. |
| `deck_path` / `deck-path`     | `string`         | The canonicalized absolute path, informational (the filename already keys it; this field lets a human `cat` the file and know what it's for). |
| `node_id` / `node-id`         | `string`         | The presenter's current node id, verbatim from `Session::current().id`. |
| `reveal_step` / `reveal-step` | `u32`            | From `Session::reveal_progress()`, or `0` when the node has no reveal steps. |
| `reveal_total` / `reveal-total` | `u32`          | From `Session::reveal_progress()`, or `0` when the node has no reveal steps. |
| `elapsed_secs` / `elapsed-secs` | `u64`          | From `App::elapsed()`, whole seconds.                                 |
| `heartbeat` / `heartbeat`     | `u64`            | Epoch seconds, refreshed every write (every event-loop tick).        |

**Validation rules**: A record failing to parse as this shape, a `schema`
value other than `1`, or a `heartbeat` more than 2 seconds in the past
(relative to the follower's own clock) are all the *same* outcome to a
reader — `SessionStatus::NotRunning` — never a distinguished error (spec
FR-004, edge case: "not running" covers never-started, exited-clean, and
crashed identically).

**Lifecycle**: Created on the first tick of `present`/`present_authoring`
against a real deck file (no session file for `fireside demo`, which has no
backing path — same "no key, no record" rule `resume.rs` already applies).
Overwritten atomically (temp file + rename in the same directory, so a
concurrent reader never observes a partial write) on every subsequent tick.
Deleted on a clean presenter exit (the `q` quit path returning from
`event_loop` normally).

## SessionSnapshot (in memory — `fireside-tui`)

The parsed, validated form of a fresh `SessionRecord`, or its absence.

```rust
pub struct SessionSnapshot {
    pub node_id: String,
    pub reveal_step: usize,
    pub reveal_total: usize,
    pub elapsed: Duration,
}

pub enum SessionStatus {
    Running(SessionSnapshot),
    NotRunning,
}
```

`fireside-tui` never parses JSON or touches a clock itself — `fireside-cli`
hands it an already-decided `SessionStatus` on each poll (staleness
comparison happens in the CLI, which owns `SystemTime`), matching the
"caller owns all I/O and all time-based decisions, TUI just renders"
posture `ReloadSource` already establishes for deck reload.

## SessionTick (in memory — `fireside-tui`, presenter → CLI direction)

What the *presenting* process's event loop hands to the CLI's write sink
every tick — the mirror image of `SessionSnapshot` above, flowing the other
way:

```rust
pub struct SessionTick {
    pub node_id: String,
    pub reveal_step: usize,
    pub reveal_total: usize,
    pub elapsed: Duration,
}
pub type SessionTickSink<'a> = &'a mut dyn FnMut(SessionTick);
```

## Follower (in memory — `fireside-tui::follower`)

The follower's own small TEA-shaped state (see research.md §3):

```rust
pub struct Follower {
    graph: Graph,       // reloaded in place on a deck-file change
    status: SessionStatus,
    quit: bool,
}

pub enum FollowerMsg {
    Terminal(crossterm::event::Event),
    Reload(Result<Graph, String>),
    SessionUpdate(SessionStatus),
}
```

Derived, per-render (not stored) view of "what to show", computed from
`graph.node(&snapshot.node_id)`:

- **Current**: node title + `speaker_notes` (or "No notes for this slide"
  when absent — FR-012).
- **Next**: `Node::branch_point()` → render the branch's `options`
  (`label`, `key`); else `Node::next_target()` resolved via
  `graph.node(...)` → its title; else (terminal, no branch) → "This is the
  last slide" (FR-013).
- **Reveal**: `reveal_total > 0` → `"{reveal_step}/{reveal_total}
  revealed"`; else omitted entirely (nothing to show on a node with no
  reveal steps).
- **Elapsed**: `SessionSnapshot::elapsed`, formatted the same
  `mm:ss` style `format_present_summary` already uses.
- **Node not found** (session reports a `node_id` the follower's current
  `Graph` doesn't have — a brief reload-skew race, FR-007): render
  "waiting for presenter…" instead of any of the above.
- **`SessionStatus::NotRunning`**: overrides everything above — render the
  plain "Presenter not running — start …" message (FR-004), regardless of
  whether a `Graph` and a stale snapshot would otherwise resolve.

No new entity is added to `fireside-core`; `Node`, `Graph`, `BranchPoint`,
`BranchOption` are read-only inputs to the follower, unchanged.
