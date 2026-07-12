# Phase 0 Research: `validate --watch`

## Decision: Reuse `fingerprint()`, not the `Watcher` struct or `load()`

**Rationale**: The spec's input pointed at "the existing file watcher built
for `fireside present`" as reuse material. Reading `crates/fireside-cli/src/main.rs`
shows two candidates, and neither is a straight drop-in:

- `Watcher` (struct, `main.rs:156`) wraps `fingerprint()` change-detection
  and, on change, parses the file and maps a parse failure to a **one-line**
  message via `strip_position()` — the format used for the TUI's reload
  flash. FR-006 requires the **full caret-block** report (line-before,
  offending line, caret, message) — the format `parse_report()` produces,
  used by `load()` for the initial `present`/`validate` parse. `Watcher`
  does not produce that format.
- `load()` (`main.rs:82`) produces the caret-block report via `parse_report()`,
  but calls `std::process::exit(1)` on parse failure. That is correct for a
  one-shot command; it would kill a watch loop on the first typo.

**Decision**: the watch loop reuses the free function `fingerprint()`
(`(SystemTime, u64)` change detection, `main.rs:196`) directly for polling,
and reuses `parse_report()` (`main.rs:124`) for error formatting on a parse
failure. It does not reuse `Watcher` or `load()` as-is. `validate_file`'s
diagnostic-printing loop (the `for d in &diags` block, `main.rs:202-230`) is
extracted into a small helper that returns rendered output instead of
printing-and-exiting, so both the one-shot and watch paths call the same
formatting code and stay behaviorally identical (satisfies SC-004).

**Alternatives considered**:
- Modify `Watcher` to carry the fuller report format: rejected — `Watcher`
  is also used by `present`'s live TUI reload, where a short flash message
  is correct (screen space is a single footer line); changing its output
  format would regress that caller.
- Make `load()` return a `Result` instead of exiting, and have both
  one-shot `validate` and `present` adapt: out of scope — touches two
  working call sites for a benefit this feature doesn't need; `validate_file`
  already has its own non-exiting diagnostic path to extract from instead.

## Decision: Own polling loop with `std::thread::sleep`, not `Watcher`'s cadence

**Rationale**: `present`'s poll cadence lives inside `fireside-tui`'s
`event_loop` (`crates/fireside-tui/src/lib.rs:56`), driven by
`crossterm::event::poll` timeouts (250ms idle / 30ms while fading) — that
loop exists to service terminal input and rendering, not just file-watching,
and pulling it in would violate crate boundaries (constitution Principle III:
`fireside-cli` may depend on `fireside-tui`, but this feature has no UI and
must not gain a `ratatui`/`crossterm` runtime dependency just to get a
sleep loop). A plain `loop { … ; std::thread::sleep(Duration::from_millis(250)) }`
in `fireside-cli` matches the same 250ms cadence users already experience in
`present`'s idle poll, with zero new dependencies.

**Alternatives considered**:
- OS-level file-change notification (e.g., a `notify`-style crate): rejected
  — new dependency, must clear the MSRV/crate-boundary bar (constitution
  Principle VI), and the existing codebase already established
  mtime/size polling as the house pattern for exactly this problem; no
  reason to diverge for one more command.

## Decision: Ctrl-C exits via default SIGINT, no new dependency

**Rationale**: FR-010 requires clean exit on interrupt. The watch loop holds
no terminal state to restore (unlike `present`, which calls `ratatui::init()`/
`ratatui::restore()`) and writes only to stdout/stderr — the process default
SIGINT handling (immediate termination) already satisfies "exit cleanly."
No `ctrlc`-style crate is needed, keeping the crate boundary and MSRV gates
trivially satisfied.

## Decision: Test the polling step as a pure function, not via a spawned infinite loop

**Rationale**: `cli_e2e.rs` already establishes the project's pattern:
`present`'s watch/reload behavior is not e2e-tested by spawning the binary
and timing a real file edit — it's covered by `fireside-tui`'s `TestBackend`
scenario suite driving `App::update(Msg::Reload(...))` directly with a
synthetic `ReloadSource`. There is no existing precedent in `cli_e2e.rs` for
spawn-and-wait-for-async-output testing, and adding one would need
timing-sensitive sleeps that risk flaky CI.

**Decision**: extract a pure function — check the file once, return the
rendered report — and unit-test it in `main.rs`'s existing `#[cfg(test)]`
module (same module that already tests `parse_report`/`strip_position`)
against fixtures for: valid deck, semantic errors, malformed JSON, missing
file. The `--watch` CLI path becomes a thin loop: poll `fingerprint()`, on
change call the pure function, print its output, sleep. One additional
`cli_e2e.rs` test spawns `fireside validate --watch` against a file, using
`assert_cmd`'s process handle to confirm the flag is accepted and the first
render happens (FR-003) within a short timeout, then kills the process —
covering the CLI wiring without asserting on reload timing.

## Resolved Technical Context

- **Language/Version**: Rust, workspace MSRV 1.88, 2024 edition (unchanged).
- **Primary Dependencies**: none added. Uses existing `clap`, `anyhow`,
  `serde_json` (already permitted for `fireside-cli`) plus `fireside_core`,
  `fireside_engine` (already dependencies).
- **Storage**: N/A — reads one file from disk per poll, no persistence.
- **Testing**: `cargo test --workspace`; new unit tests in
  `fireside-cli/src/main.rs`, one new `cli_e2e.rs` integration test.
- **Target Platform**: wherever `fireside` already runs (terminal, any OS
  the workspace targets) — no new platform-specific behavior.
- **Project Type**: CLI (single crate change: `fireside-cli`).
- **Performance Goals**: sub-second-to-a-few-seconds latency after save,
  matching `present`'s existing 250ms idle poll cadence (SC-001, Assumptions).
- **Constraints**: no new dependency; no `fireside-tui` involvement; default
  (non-watch) `validate` output byte-for-byte unchanged (FR-002, SC-004).
- **Scale/Scope**: one flag, one crate, no protocol change.
