# Contract: presenter session summary and resume flash

Covers `fireside-tui`'s public presenting entry points and their two CLI
callers (`present()`, `demo()` in `fireside-cli/src/main.rs`).

## `fireside-tui` public API change

```rust
pub struct PresentSummary {
    pub seen: usize,
    pub total: usize,
    pub elapsed: Duration,
}

pub fn present(graph: Graph) -> Result<PresentSummary, TuiError>;
pub fn present_watching(graph: Graph, source: ReloadSource<'_>) -> Result<PresentSummary, TuiError>;
pub fn present_authoring(
    graph: Graph,
    source: ReloadSource<'_>,
    sink: WriteBackSink<'_>,
    initial_node: Option<&str>,
    on_position_changed: PositionSink<'_>,
) -> Result<PresentSummary, TuiError>;
```

- Breaking change to all three function signatures (workspace-internal
  crate, not published — no semver contract with external consumers).
- `Ok(PresentSummary)` on any graceful stop (the `q` key or in-loop
  `Ctrl+C` — see `research.md` §3). `Err(TuiError)` unchanged: unpresentable
  graph, terminal I/O failure. No summary is available on the error path.
- `fireside-tui` performs no `println!`/`eprintln!` of the summary itself —
  every existing "TUI never touches stdout formatting outside its own
  frames" boundary is preserved. The two CLI call sites print it.

## CLI output contract

Both `present()` and `demo()` in `main.rs`, on receiving `Ok(summary)`,
print exactly one line to stdout after the terminal has been restored
(i.e. after the call returns — never interleaved with TUI frames):

```text
Presented {seen}/{total} slides in {mm}:{ss}.
```

- `{mm}:{ss}` — total elapsed minutes and seconds, zero-padded seconds
  (`12:05`, not `12:5`), no hours component (decks this long are out of
  scope; minutes accumulate past 59 rather than rolling to an hours field).
- No line is printed when the call returns `Err` — the existing error
  handling (`.context("the presenter hit a terminal error")`) is unchanged
  and takes over instead.

## Resume flash contract

- Fires at most once per session, on the very first frame drawn.
- Text: exactly `Resumed where you left off — --restart starts over`.
- `FlashKind::Info`, same expiry (`FLASH_DURATION`, 3000ms) as every other
  flash — no special-cased duration.
- Fires if and only if `present_authoring` was called with
  `initial_node: Some(id)` **and** `session.goto(id)` returned
  `Outcome::Moved`. Does not fire for `None` (no resume record / fresh
  deck), and does not fire when `goto` returns `Outcome::UnknownNode`
  (stale resume record pointing at a node the deck no longer has — falls
  back to the entry node silently, as it already does today).
- Orthogonal to the exit summary and to `--restart`: `--restart` is handled
  entirely by the caller (`main.rs`'s `present()`, via
  `resume::ResumeStore::resolve_initial_node`) before `initial_node` is ever
  computed — passing `None` for a restarted run, which this contract already
  treats as "no flash."
