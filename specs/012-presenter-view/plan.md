# Implementation Plan: Dual-Screen Presenter View

**Branch**: `012-presenter-view` | **Date**: 2026-07-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/012-presenter-view/spec.md`

## Summary

A presenter running `fireside <deck>` fullscreen on a projector needs a second,
read-only window on their own laptop that shows the current slide's speaker
notes, what's next, reveal progress, and elapsed time — never visible to the
audience. The approach: the presenting process (unchanged in every other way)
gains a new sibling output — a small, host-local, per-deck **session-state
file** that it writes on every event-loop tick (a heartbeat) and on every
navigation/reveal move. A new `fireside notes <deck>` command starts a second,
independent TUI (`fireside-tui`, rendering only, zero file I/O of its own)
that loads the same deck, watches it for live edits the same way the
presenter does, and polls the session file at the same cadence to resolve
"where is the presenter right now" against its own copy of the graph. No
protocol change, no new dependency, no change to `fireside-engine` (the one
piece of state the follower needs, `Session::reveal_progress`, already
exists) — this is new CLI-owned I/O plumbing plus one new `fireside-tui`
rendering surface.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.88 (`resolver = "3"`, 2024 edition) — unchanged.

**Primary Dependencies**: No new dependency for any crate. `fireside-tui` gains one new module using only `ratatui`/`crossterm`/`fireside-core`/`fireside-engine` (all already permitted). `fireside-cli` gains one new module using only `std::fs`/`std::path`/`std::time`/`serde_json` (already permitted, same posture as `resume.rs`).

**Storage**: A new host-local file per deck at `$XDG_STATE_HOME/fireside/sessions/<fnv1a64-hex>.json` (falling back to `~/.local/state`, matching `resume.rs`'s existing fallback). Not protocol-versioned, not part of the portable deck format — a disposable local cache, same class as `resume.json`, contract documented in `contracts/session-state-format.md`.

**Testing**: `cargo test --workspace` (unit tests for the new session-state read/write module, `fireside-tui/src/render/tests.rs` TestBackend scenarios for the follower's rendering states), `fireside-cli/tests/cli_e2e.rs` for the new `notes` subcommand's CLI-level behavior (non-tty guard, missing-file guard), and a tmux smoke extension to `scripts/smoke.sh` per constitution Principle VII's fourth bullet (two panes: presenter + follower).

**Target Platform**: Same as today — any terminal Fireside already supports (this feature runs two terminal processes on one host; it is not a network feature).

**Project Type**: CLI + TUI (existing 4-crate workspace: `fireside-core`, `fireside-engine`, `fireside-tui`, `fireside-cli`) — no new crate.

**Performance Goals**: Session-state write ≤ one small JSON file per tick (~4 Hz, matching the existing 250 ms poll cadence) — negligible compared to the deck-reload fingerprint check already done every tick. Follower-observable update latency ≤ ~500 ms (spec SC-001); stale-detection latency ≤ ~2 s (spec SC-002), both driven by the same 250 ms poll cadence `watch.rs` already uses elsewhere.

**Constraints**: Zero protocol change (Principle I) — the session-state file is host-local, like `resume.json`, not a wire format. Zero new dependency. Crate boundaries unchanged (Principle III) — all file I/O for both the presenter's write side and the follower's read side stays in `fireside-cli`, injected into `fireside-tui` via the existing closure-injection pattern (`ReloadSource`, `WriteBackSink`, `PositionSink` precedent). The presenter itself performs zero file I/O, and so does the follower.

**Scale/Scope**: One presenter process, N read-only follower processes per deck (spec assumption: exactly one presenter is the source of truth at a time; multiple simultaneous followers of the same deck is supported for free since none of them write).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Spec Is the Source of Truth)**: PASS. No field, enum, or traversal behavior changes in the protocol. The session-state file is host-local cache (same class as `resume.json`, which already sets this precedent) — no `protocol/main.tsp` or schema change, no ADR needed *for the protocol*. An ADR is still required for the file's own contract (see below) because it's a new *cross-process* contract, not because it touches the protocol.
- **Principle II (Presenter-First Experience)**: PASS, with a recorded scope extension. `present`/`validate`/`new` is the ADR-004 baseline; this feature adds a fourth verb (`notes`) and a launch flag (`--fullscreen`). This is a scope addition — but the audit plan records that the user explicitly asked for it on 2026-07-19 (promoted addendum A-2 to scoped work), which is exactly the gate ADR-004 requires before scope grows. **Action**: write an ADR recording this scope extension and the user request that satisfies the gate, per the audit plan's "Constitution flags" section — do not re-ask the user. The follower's own footer must still teach its keys (`q` to quit) — same posture as every other screen.
- **Principle III (Crate Boundary Discipline)**: PASS. No new dependency anywhere. The follower's rendering lives in `fireside-tui` (permitted: `ratatui`, `crossterm`, `fireside-core`, `fireside-engine`); all file I/O (session-file read/write, deck load/watch) lives in `fireside-cli`, injected via closures — the same shape as `ReloadSource`/`WriteBackSink`/`PositionSink` today. `fireside-tui` performs no file I/O; `fireside-cli` performs no rendering.
- **Principle IV (Mandatory Code Idioms)**: PASS, to be enforced during implementation — no `unwrap()`/`expect()` outside `main()`/tests, `#[must_use]` on new public functions, TEA-shape kept for the follower's own tiny state (see research.md §3 for why it is a new, separate state type rather than a graft onto `App`).
- **Principle V (Stratified Error Handling)**: PASS. The new `fireside-cli::session` module is CLI-boundary code (`anyhow`/plain `Result`, same as `resume.rs`); nothing new touches `fireside-core`/`fireside-engine`, so no new `thiserror` variants needed there. `fireside-tui` gets one new small, focused error path (non-tty guard) reusing `TuiError::NotATty`.
- **Principle VI (MSRV 1.88)**: PASS. FNV-1a 64 is hand-rolled (`u64` arithmetic only, stable since Rust 1.0); atomic temp-file-then-rename uses only `std::fs`, already used the same way nowhere yet in this codebase but well within MSRV (rename has been stable forever). No new crate, so no MSRV risk to evaluate.
- **Principle VII (Test Discipline)**: PASS, planned: engine-layer — none needed (no engine change); TUI-layer — new `fireside-tui/src/render/tests.rs` scenarios for the follower's rendering states (has-notes, no-notes, branch, final-slide, waiting-for-presenter, stale); CLI-layer — `cli_e2e.rs` cases for the `notes` subcommand's non-tty guard and missing-deck guard, plus unit tests in the new `fireside-cli::session` module mirroring `resume.rs`'s test style; smoke-layer — tmux extension to `scripts/smoke.sh` per the audit plan's W4-DS-5.

**Two ADRs required before implementation** (per the audit plan's Wave 4 section and Constitution Flags):
1. **ADR-014 (proposed): ADR-004 scope extension — dual-screen presenter view.** Records that the user explicitly requested this 2026-07-19 (promoted addendum A-2 to scoped Wave 4 work), satisfying Principle II's scope-addition gate for the new `notes` verb and `--fullscreen` flag.
2. **ADR-015 (proposed): session-state file contract.** Records the decision that live session state is a separate per-deck file under `sessions/`, not a `resume.json` extension — rationale: `resume.json` is a shared read-modify-write map across every deck the user has ever presented, while the session file is a single-writer, high-frequency (~4 Hz), single-deck heartbeat; sharing the file would mean two concurrent presentations racing a last-writer-wins rewrite of the *entire* map, and would put heartbeat churn into a file every other code path treats as a cold, occasionally-touched cache. Also records the location, schema, atomicity (temp file + rename), staleness threshold (>2s), and the FNV-1a 64 choice over `DefaultHasher` (not stable across Rust versions) or reusing `watch::fingerprint`'s `(mtime, len)` pair (not a hash, wrong shape for a filename).

Both ADRs are the first tasks `/speckit-tasks` generates in Phase 2 — governance artifacts, not code, so they land in the same PR as the first implementation slice, before any file-writing code is merged.

## Project Structure

### Documentation (this feature)

```text
specs/012-presenter-view/
├── plan.md                          # This file
├── research.md                      # Phase 0 output
├── data-model.md                    # Phase 1 output
├── quickstart.md                    # Phase 1 output
├── contracts/
│   └── session-state-format.md      # Phase 1 output — the new file's contract
└── tasks.md                         # Phase 2 output (/speckit-tasks — not this command)
```

### Source Code (repository root)

Existing 4-crate Cargo workspace; no new crate. Changed/added files only:

```text
crates/fireside-cli/
├── src/
│   ├── main.rs        # + `notes` subcommand, `--fullscreen` flag on present,
│   │                  #   wiring the new SessionTickSink into present()
│   ├── session.rs     # NEW — session-state file: write (presenter side),
│   │                  #   read + staleness check (follower side), FNV-1a 64,
│   │                  #   atomic temp+rename, mirrors resume.rs's shape/tests
│   └── watch.rs        # unchanged — Watcher reused as-is for the follower's
│                        #   own deck-file watch (same pattern present() uses)
└── tests/
    └── cli_e2e.rs      # + notes-subcommand non-tty guard, missing-file guard

crates/fireside-tui/
├── src/
│   ├── lib.rs          # + `SessionTick` struct, `SessionTickSink` type,
│   │                   #   `SessionSnapshot`/`SessionStatus`, `SessionSource`
│   │                   #   type; `present_authoring` gains a tick-sink param;
│   │                   #   new `pub fn follow(...)` entry point
│   ├── follower.rs     # NEW — the follower's own tiny read-only state
│   │                   #   (current graph, latest SessionStatus, quit flag);
│   │                   #   no TEA `App`/`Msg` reuse (see research.md §3)
│   └── render/
│       ├── mod.rs      # + draw entry point for the follower screen
│       ├── notes.rs    # NEW — follower rendering: notes/next/branch/reveal/
│       │               #   timer/stale layout, using theme::Tokens throughout
│       └── tests.rs    # + TestBackend scenarios for every follower state
```

**Structure Decision**: No new crate, no change to the four-crate boundary.
The follower is additive: one new `fireside-tui` module pair (state +
render) that never touches `App`/`Screen`/`Msg` (the presenter's existing
TEA machine is untouched — lower risk than threading a "read-only mode"
through it), and one new `fireside-cli` module (`session.rs`) that mirrors
`resume.rs`'s already-reviewed shape (host-local JSON, `std`-only, no new
dependency, unit-tested the same way).

## Complexity Tracking

*No unjustified Constitution Check violations — this section is empty by design. The one deliberate scope addition (Principle II) is not a violation; it is a recorded, user-requested extension per ADR-014 above, which is exactly the mechanism Principle II specifies for growing scope.*
