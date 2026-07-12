# Phase 1 Data Model: Quick-Edit Modal

No protocol/wire-format entities are added or changed — `fireside-core`'s
`Graph`/`Node`/`ContentBlock` are reused exactly as they exist today. This
feature adds only in-process, non-persisted state inside `fireside-tui` and
`fireside-cli`.

## `BlockPath` (fireside-tui, new)

Addresses one heading/text block within the current node's content tree.

| Field | Type | Notes |
|---|---|---|
| indices | `Vec<usize>` | Path of `ContentBlock` indices from `Node::content` root through nested `Container::children`; e.g. `[0]`, `[2, 1]`. |

Computed by a depth-first walk of the current node's `content` each time the
modal opens; collects every `ContentBlock::Heading` / `ContentBlock::Text`
found, in document order, regardless of container nesting. Never persisted;
recomputed fresh each open, so a stale path can never be applied to a
different node's shape.

## `EditableField` (fireside-tui, new)

One entry in the modal — one editable block plus its in-progress buffer.

| Field | Type | Notes |
|---|---|---|
| path | `BlockPath` | Where this block lives in the current node. |
| kind | `enum { Heading(u8), Text }` | Heading carries its level for the label; Text does not. |
| buffer | `Vec<String>` | Multi-line editable text, initialized from the block's current `text`/`body`. |
| cursor | `(usize, usize)` | (row, column) within `buffer`, for rendering and key handling. |

## `Screen::Edit` (fireside-tui, extends existing `Screen` enum)

| Field | Type | Notes |
|---|---|---|
| fields | `Vec<EditableField>` | One per editable block found on the current node; empty means "nothing to quick-edit" (FR-011). |
| focused | `usize` | Index into `fields` — which block is being typed into. |

Reachable only from `Screen::Present`; on Esc/cancel returns to
`Screen::Present` with the app's `session` untouched (FR-005). On save,
`App` builds an edited `Graph` (clone of `session.graph()` with each
`fields[i].buffer` written back into the block at `fields[i].path` on the
current node) and stores it as a pending save for the event loop to hand to
the write-back sink (see `WriteBackSink` below); `App` does not write files
itself.

## `WriteBackSink` / `WriteBackError` (fireside-tui, new public API)

```rust
pub type WriteBackSink<'a> = &'a mut dyn FnMut(&Graph) -> Result<(), WriteBackError>;

pub enum WriteBackError {
    /// No file backs this presentation (e.g. `fireside demo`).
    Unavailable,
    /// The on-disk file changed since it was last loaded — save refused to
    /// avoid silently discarding either version (FR-013).
    Conflict,
    /// The write failed for some other reason (permissions, disk full, ...).
    Io(String),
}
```

Owned and implemented by `fireside-cli`; `fireside-tui` only calls it and
never touches the filesystem, preserving the crate boundary.

## `Msg` additions (fireside-tui, extends existing enum)

| Variant | Payload | Meaning |
|---|---|---|
| `SaveResult` | `Result<(), String>` | The write-back sink's outcome, fed back into `App::update` exactly like `Msg::Reload` today. |

Opening/closing the modal and editing keystrokes are handled as ordinary
`Msg::Terminal` key events dispatched through the existing `Screen`-based
`on_key` match, not new `Msg` variants — consistent with how `Screen::Map`
and `Screen::Help` are already handled.

## Relationships

- `Screen::Edit` is derived, at open time, from `Session::current()` (the
  existing engine-owned `Node`) — it does not change how `Session` or the
  engine work; `fireside-engine` is untouched by this feature.
- The edited `Graph` a save produces goes through the exact same
  `Msg::Reload` → `on_reload` path an external editor's save already goes
  through (`crates/fireside-tui/src/app.rs::on_reload`), once the CLI's
  sink writes it to disk and the existing `Watcher` notices the change —
  no new reload code path is introduced.
