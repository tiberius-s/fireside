---
title: 'fireside-engine'
description: 'Document loading, graph validation, traversal state machine, and command-based editor session for Fireside.'
---

`fireside-engine` is the **business logic layer** of the Fireside stack. It
owns everything that happens to a graph after it has been parsed from JSON and
before any pixels are drawn: loading, validation, traversal, undo/redo-capable
mutations, and the session object that ties all of these together. It has no
knowledge of terminal rendering or command-line flags.

## Crate responsibilities

| Owns                                                       | Explicitly excluded                         |
| ---------------------------------------------------------- | ------------------------------------------- |
| JSON loading and `GraphFile → Graph` construction          | Protocol type definitions (`fireside-core`) |
| Structural graph validation and diagnostic emission        | Ratatui/crossterm dependencies              |
| `TraversalEngine` state machine (Next, Choose, Goto, Back) | Direct file I/O beyond loading              |
| `Command` model with full undo/redo via `CommandHistory`   | CLI argument parsing                        |
| `PresentationSession` — unified graph + traversal handle   | UI rendering                                |

## Module map

```text
fireside-engine/src/
├── lib.rs          public re-exports; crate entry point
├── error.rs        EngineError (thiserror)
├── loader.rs       load_graph(), save_graph() — file I/O + GraphFile → Graph
├── validation.rs   validate_graph() → Vec<Diagnostic>; Severity enum
├── traversal.rs    TraversalEngine; TraversalResult enum
├── session.rs      PresentationSession
└── commands.rs     Command enum; CommandHistory; apply_command()
```

## Loading pipeline

`load_graph(path)` is the single entry point from outside the crate:

```text
Path
 └─► fs::read_to_string          → CoreError::FileRead on I/O failure
      └─► serde_json::from_str    → CoreError::InvalidJson on parse failure
           └─► GraphFile           (wire-format serde target)
                └─► Graph::from_file → CoreError::EmptyGraph | DuplicateNodeId
                     └─► validate_graph()  → Vec<Diagnostic> (non-fatal)
```

`save_graph` performs the reverse: `Graph → GraphFile → serde_json::to_string_pretty
→ fs::write`. Keeping load and save symmetric in one module ensures wire-format
fidelity — a graph that survives a save/reload cycle will produce identical
`Graph` state.

### Why load wraps `CoreError` into `EngineError`

`EngineError::Load` wraps a `CoreError`. This is idiomatic layered error
composition: the engine's caller (the CLI or TUI) catches `EngineError` and
handles it uniformly. If the caller needs to distinguish a bad JSON parse from
a duplicate node ID, it can match through the `EngineError::Load(CoreError::…)`
chain.

## Validation

`validate_graph` performs **structural integrity checks** that require the
full node index built by `Graph::from_file`. It returns `Vec<Diagnostic>` —
never panics, never returns an `Err`. An empty vec means the graph is valid.

Three severity classes are checked:

**Dangling `traversal.next` references** — a node names a next-hop ID that
does not exist in `node_index`. This is an `Error` because traversal would
call `graph.index_of(id)` and receive `None`, producing an `AtBoundary` result
even mid-deck.

**Dangling `traversal.after` references** — same failure mode as `next` but
affects the rejoin target after a branch sequence.

**Dangling `branch_point.options[*].target`** — a branch option's destination
is unknown. This is also an `Error`; a `ChooseBranch` call would have nowhere
valid to go.

```rust
pub struct Diagnostic {
    pub severity: Severity,   // Error | Warning
    pub message:  String,
    pub node_id:  Option<String>,   // present when the issue is node-specific
}
```

The diagnostic model is intentionally message-string based right now. The
`node_id` field allows the CLI validation command to group issues by node and
the TUI to highlight specific nodes; a future `DiagnosticCode` enum would make
diagnostics machine-readable without changing the public type shape.

## Traversal state machine

`TraversalEngine` is a two-field struct:

```rust
pub struct TraversalEngine {
    current: usize,              // current node index (0-based)
    history: VecDeque<usize>,    // navigation history for Back
}

const MAX_HISTORY: usize = 256;
```

### The four operations

**`next(&mut self, graph: &Graph) → TraversalResult`**

Priority chain:

1. If `current_node.traversal.next` is set and resolves to a valid index →
   push history, jump to that node.
2. Else if `current_node.traversal.after` is set and resolves → push history,
   jump to the rejoin target.
3. Else if `current + 1 < graph.len()` → push history, advance sequentially.
4. Else → `TraversalResult::AtBoundary` (no mutation, no history push).

The `after` check in `next` is how branch sequences rejoin the main flow:
a branch option typically points into a sub-sequence, and the last node in
that sub-sequence carries `traversal.after` pointing back to the main deck.

**`choose(key: char, &mut self, graph: &Graph) → Result<TraversalResult, EngineError>`**

Searches `current_node.traversal.branch_point.options` for an option whose
`key` matches. On success, pushes history and jumps to `option.target`. Returns
`EngineError::NoSuchBranchOption` if no option matches `key`, giving the caller
actionable feedback to display in the UI.

**`goto(index: usize, &mut self, graph: &Graph) → Result<TraversalResult, EngineError>`**

Bounds-checks `index < graph.len()`, then pushes history and sets `current`.
Returns `EngineError::NodeIndexOutOfBounds` on failure.

**`back(&mut self) → TraversalResult`**

Pops `history`. If the history is empty, falls back to `current - 1` if
`current > 0`, otherwise returns `AtBoundary`. The 256-entry cap on the
`VecDeque` prevents unbounded memory growth during long sessions; old entries
are dropped from the front with `pop_front` when the limit is reached.

### `TraversalResult`

```rust
pub enum TraversalResult {
    Moved { from: usize, to: usize },
    AtBoundary,
}
```

`Moved` carries both the source and destination index. The TUI's transition
animation system uses the `from` index to determine direction and the type of
visual effect to play. `AtBoundary` is not an error — the presenter treats it
as a no-op, which is the correct UX for pressing Next on the last slide.

### History and `clamp_to_graph`

When a graph is reloaded (hot-reload or after an edit), node indices may have
shifted. `clamp_to_graph(graph_len)` adjusts `current` and prunes any history
entries that now fall out of bounds:

```rust
pub fn clamp_to_graph(&mut self, graph_len: usize) {
    if self.current >= graph_len { self.current = graph_len - 1; }
    self.history.retain(|idx| *idx < graph_len);
}
```

The `App` layer in `fireside-tui` calls this after every successful reload,
then attempts to restore the last-known node by ID rather than by index.

## Command model and undo/redo

`commands.rs` implements a classic **command pattern** with inverses:

```rust
pub enum Command {
    UpdateNodeContent { node_id: NodeId, content: Vec<ContentBlock> },
    AddNode           { node_id: NodeId, after_index: Option<usize> },
    RestoreNode       { node: Node, index: usize },
    RemoveNode        { node_id: NodeId },
    SetTraversalNext  { node_id: NodeId, target: NodeId },
    ClearTraversalNext { node_id: NodeId },
}
```

Each `Command` variant has a natural inverse: `AddNode` ↔ `RemoveNode`,
`UpdateNodeContent` ↔ `UpdateNodeContent` (previous content), `RemoveNode` ↔
`RestoreNode` (full node snapshot). `apply_command` computes the inverse
_before_ applying the mutation, stores the `(command, inverse)` pair in
`CommandHistory.applied`, and returns `Ok(())`.

```rust
struct HistoryEntry { command: Command, inverse: Command }

pub struct CommandHistory {
    applied: Vec<HistoryEntry>,   // undo stack
    undone:  Vec<HistoryEntry>,   // redo stack
}
```

**Undo**: pop from `applied`, apply `entry.inverse` to the graph, push
`(inverse, original command)` to `undone`.

**Redo**: pop from `undone`, apply `entry.command` to the graph, push back to
`applied`.

Any new mutation (non-undo/redo command) clears `undone` — the standard
linear-history undo model.

### `RestoreNode` as the snapshot command

`RemoveNode { node_id }` produces `RestoreNode { node: full_node_clone, index }` as
its inverse. The full `Node` clone is taken immediately before removal so that
any content, layout, transition, speaker notes, or traversal overrides are
faithfully restored on undo. This is more memory-intensive than a diff-based
approach but simpler and correct.

## `PresentationSession`

`PresentationSession` is the engine's primary facade:

```rust
pub struct PresentationSession {
    pub graph:        Graph,
    pub traversal:    TraversalEngine,
    pub command_history: CommandHistory,
    dirty:            bool,
}
```

It bundles the graph, the traversal cursor, and the command history into a
single object so the TUI can pass exactly one handle to every function that
needs to work with the presentation state. The `dirty` flag tracks unsaved
mutations and drives the save-confirmation dialog in the editor.

## `EngineError`

```rust
pub enum EngineError {
    Load(CoreError),
    Save(String),
    CommandError(String),
    NodeIndexOutOfBounds { index: usize, len: usize },
    NoSuchBranchOption   { key: char },
    NodeNotFound(String),
}
```

`EngineError` is the boundary type for all fallible engine operations. The
TUI layer catches it and converts it to a user-facing status message; the CLI
layer propagates it through `anyhow::Context` for terminal error printing.

## Testing strategy

`fireside-engine` has three test layers:

1. **Inline unit tests** in each module — `traversal.rs` tests every operation
   including boundary conditions, override priorities, and back-with-empty-history.
   `validation.rs` tests dangling-reference detection. `loader.rs` tests
   save/reload round-trip fidelity.

2. **`tests/command_history.rs`** — end-to-end test of add → update → remove →
   undo chain asserting original state is fully restored.

3. **`tests/validation_fixtures.rs`** — loads JSON fixture files from
   `tests/fixtures/` (valid and invalid graphs) and asserts diagnostic counts
   and severities. Fixtures make regressions obvious: a change that starts
   emitting a new diagnostic on a valid graph will fail the test immediately.
