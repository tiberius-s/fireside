# Contract: `fireside-tui` authoring entry point

This is the internal Rust API contract between `fireside-tui` (presents,
never touches disk) and `fireside-cli` (owns all file I/O), extending the
existing `ReloadSource` contract with a symmetric write-back path.

## New public items (`crates/fireside-tui/src/lib.rs`)

```rust
/// A write-back sink: called with an edited graph when the presenter saves
/// a quick edit. The presenter itself never touches the filesystem; the
/// caller owns the I/O and reports back whether the save succeeded.
pub type WriteBackSink<'a> = &'a mut dyn FnMut(&Graph) -> Result<(), WriteBackError>;

/// Why a quick-edit save could not be applied.
#[derive(Debug, Clone)]
pub enum WriteBackError {
    /// No file backs this presentation (e.g. the built-in demo deck).
    Unavailable,
    /// The on-disk file changed since it was last loaded; the save was
    /// refused rather than risk silently discarding either version.
    Conflict,
    /// The write failed for a reason other than a conflict (permissions,
    /// disk full, etc.), carrying a human-readable message.
    Io(String),
}

/// Present a graph with live reload and quick-edit write-back.
///
/// # Errors
///
/// Returns [`TuiError::Engine`] for an unpresentable graph and
/// [`TuiError::Io`] for terminal failures.
pub fn present_authoring(
    graph: Graph,
    source: ReloadSource<'_>,
    sink: WriteBackSink<'_>,
) -> Result<(), TuiError>;
```

`present` and `present_watching` keep their existing signatures unchanged
(no breaking change to current callers) and are defined in terms of
`present_authoring`:

```rust
pub fn present(graph: Graph) -> Result<(), TuiError> {
    present_watching(graph, &mut || None)
}

pub fn present_watching(graph: Graph, source: ReloadSource<'_>) -> Result<(), TuiError> {
    present_authoring(graph, source, &mut |_| Err(WriteBackError::Unavailable))
}
```

## Caller contract (`crates/fireside-cli/src/main.rs`)

- `present(path: &Path)` (the `fireside <file>` verb) switches from calling
  `present_watching` to calling `present_authoring`, passing a sink closure
  built from the existing `Watcher`:

  ```rust
  let mut watcher = Watcher::new(path);
  fireside_tui::present_authoring(graph, &mut || watcher.poll(), &mut |graph| {
      watcher.write_back(graph)
  })
  ```

- `Watcher` (already `crates/fireside-cli/src/main.rs`) gains one new
  method:

  ```rust
  impl Watcher {
      /// Writes `graph` to the watched path, refusing if the file changed
      /// on disk since this watcher last observed it. Deliberately leaves
      /// the watcher's fingerprint stale on success, so the very next
      /// `poll()` sees this write as a change and reloads — exactly like
      /// any external editor's save, reusing `on_reload` with no new code
      /// path (see research.md §4 for why updating the fingerprint here
      /// was tried first and rejected: it suppressed the reload).
      fn write_back(&mut self, graph: &Graph) -> Result<(), WriteBackError> { .. }
  }
  ```

- `demo()` is unchanged — it keeps calling `fireside_tui::present(graph)`,
  which now resolves to a `WriteBackError::Unavailable` sink internally, so
  attempting to save a quick edit in `fireside demo` surfaces "no file to
  save to" rather than crashing or silently discarding the edit.

## Behavioral guarantees this contract exists to make testable

- `fireside-tui` never calls `std::fs::*` (crate boundary, constitution
  §III) — enforced by construction: the only filesystem-shaped operation
  (`write_back`) lives entirely in the sink closure, defined in
  `fireside-cli`.
- A save while the on-disk file is unchanged since load succeeds and is
  visible via the existing reload path (FR-006, FR-008).
- A save while the on-disk file changed underneath the presenter returns
  `WriteBackError::Conflict`, never silently overwriting (FR-013).
- A save with no backing file (`fireside demo`) returns
  `WriteBackError::Unavailable`, never panics (Edge Cases, spec.md).
