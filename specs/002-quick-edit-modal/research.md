# Phase 0 Research: Quick-Edit Modal

## 1. Text-entry widget

**Decision**: Hand-roll a minimal multi-line text buffer (`Vec<String>` of
lines + cursor row/col) inside `fireside-tui`, rendered as a bordered popup
using existing `ratatui` primitives (`Paragraph`, `Block`). No new crate.

**Rationale**: The constitution's crate boundary table
(`.specify/memory/constitution.md`) permits `fireside-tui` exactly
`ratatui`, `crossterm`, `unicode-width`, `syntect`, `two-face`, `thiserror`
plus the workspace crates. A crate such as `tui-textarea` is not on that
list; adding it would need a deliberate constitution amendment, which ADR-005
did not ask for and the feature does not need — the editing surface is a
handful of short strings (heading/text bodies), not a code editor. Cursor
movement (arrows, Home/End, Enter for newline, Backspace/Delete) and
line-wrapped display are enough; this is well within what
`crossterm::event::KeyCode` + a `Vec<String>` buffer can do directly.

**Alternatives considered**: `tui-textarea` (rejected — new dependency,
crate-boundary amendment, more capability than a content-only editor needs);
single-line-only editing (rejected — text/heading bodies can be multi-line
prose per FR-003, and the spec explicitly calls for a text area).

## 2. Locating an editable block for write-back

**Decision**: Address each heading/text block by a `BlockPath` — the
sequence of `ContentBlock` indices from the node's `content` root down
through any nested `Container::children`, e.g. `[0]` for a top-level block or
`[2, 1]` for the second child of the third top-level block. Computed by
walking the current node's content tree when the modal opens; discarded when
the modal closes.

**Rationale**: `ContentBlock` (crates/fireside-core/src/model/mod.rs) has no
stable id — blocks are positional. A path of indices is exactly how the
renderer already walks containers (`render/blocks.rs`), so this reuses an
existing traversal shape rather than inventing an id scheme. It is also
inherently scoped to one node, matching FR-001's "current node" boundary —
no cross-node addressing is possible by construction.

**Alternatives considered**: Adding a `Uuid`/id field to `ContentBlock`
(rejected — touches the wire format, forbidden by ADR-005's "no new JSON
fields" boundary, and provides no benefit over a computed path for a
same-session edit).

## 3. Write-back plumbing (TUI has no file I/O)

**Decision**: Add a `WriteBackSink` callback type, symmetric to the existing
`ReloadSource`:

```rust
pub type WriteBackSink<'a> = &'a mut dyn FnMut(&Graph) -> Result<(), WriteBackError>;
```

`fireside-tui` gains `present_authoring(graph, source: ReloadSource, sink:
WriteBackSink) -> Result<(), TuiError>`. The existing `present` and
`present_watching` keep their current signatures and internally delegate to
`present_authoring` with a stub sink that always returns
`WriteBackError::Unavailable` (used by `fireside demo`, which has no backing
file to save to). `App` never touches the filesystem; it only ever produces
an edited `Graph` value.

Plumbing shape, mirroring the existing reload poll in `event_loop`
(`crates/fireside-tui/src/lib.rs`):

```rust
if let Some(graph) = app.take_pending_save() {
    let result = sink(&graph).map_err(|e| e.to_string());
    app.update(Msg::SaveResult(result));
}
```

`App::update` stays the sole state-mutation point; the actual disk write
happens in the closure the CLI supplies, exactly like `Watcher::poll` today.

**Rationale**: This is the literal shape ADR-005 specifies ("write-back via
a callback owned by fireside-cli, symmetric to the existing `ReloadSource`
callback"). It keeps `fireside-tui`'s crate-boundary guarantee (no direct
file I/O) intact with zero new exceptions to the boundary table.

**Alternatives considered**: Giving `fireside-tui` an `std::fs::write` call
directly (rejected — violates the boundary table explicitly, and ADR-005
already ruled this out).

## 4. Conflict detection (FR-013) and reformat-on-save

**Decision**: Reuse the existing `Watcher` fingerprint (`(SystemTime, u64)`
mtime+size pair in `crates/fireside-cli/src/main.rs`) as the concurrency
baseline. The sink closure, before writing:

1. Re-checks the on-disk fingerprint against the fingerprint the `Watcher`
   last saw (i.e. the fingerprint corresponding to the `Graph` currently
   loaded in the session) via `fingerprint(path)` — already implemented.
2. If unchanged: serialize the edited `Graph` with `Graph::to_json_pretty`
   (already implemented, canonical reformatting) and write it.
   **Deliberately does not update the `Watcher`'s stored fingerprint.**
   The first implementation attempt did update it immediately, on the
   theory that it would just avoid a redundant conflict check — but that
   made the very next `poll()` see "no change" and skip the reload
   entirely, leaving the presenter looking at the pre-save content despite
   the file being correctly updated on disk (caught by the manual
   real-terminal smoke test, `quickstart.md` scenario 1 — the on-screen
   slide did not update after `Ctrl+S`). Leaving the fingerprint stale lets
   the very next `poll()` treat the save exactly like any external editor's
   save: it updates the fingerprint itself, re-reads the file, and reloads
   — reusing `on_reload` with zero new code, which was the point of this
   design in the first place (§3, data-model.md "Relationships").
3. If changed: return `WriteBackError::Conflict`, which the TUI surfaces as
   a choice (retry-overwrite or cancel) per FR-013, rather than silently
   picking a side.

**Rationale**: The `Watcher` already exists purely to answer "has this file
changed since I last looked", which is precisely the question a
lost-update check needs — no new mechanism, no new dependency, and it keeps
the conflict-detection logic entirely inside `fireside-cli` (the only crate
that touches the filesystem), consistent with the crate boundary.

**Alternatives considered**: File locking (rejected — heavyweight, not
portable, not needed for a single-presenter tool); a hash of file contents
instead of mtime+size (rejected — `fingerprint` already exists and is
proven in the watch-mode tests; no reason to duplicate it with a different
mechanism for one feature).

## 5. Modal UI shape

**Decision**: A new `Screen::Edit` variant (alongside the existing
`Screen::Help`/`Screen::Map`) holding: the `BlockPath` + current buffer for
each editable block found on the node, an index of which block is focused,
and cursor position within that block's buffer. Rendered as a centered
bordered popup (reusing the existing popup-drawing style already used for
`draw_help`/`draw_notes` in `render/mod.rs`), listing each editable block
with a one-line label (block kind + position) and its text area.

**Rationale**: `Screen` is already the established pattern for "a whole
different key-handling mode layered over presenting" (`on_map_key` vs.
`on_present_key` in `app.rs`); adding `Edit` follows the same shape the
codebase already uses for `Help`/`Map`, keeping `App::update`'s dispatch
one flat `match self.screen`.

**Alternatives considered**: A one-block-at-a-time modal requiring re-open
per block (rejected — FR-004 requires applying all changed blocks on the
node at once; a single multi-field modal is simpler for the presenter and
for the single round-trip save).
