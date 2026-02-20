# fireside-engine

The business logic layer for Fireside. This crate bridges the pure protocol
types in `fireside-core` and the rendering layer in `fireside-tui`. It owns
four concerns: loading graphs from disk, validating their integrity, navigating
between nodes, and mutating graphs in editor mode.

## Design Philosophy

The engine follows a clean boundary rule: it knows about *what Fireside does*,
but nothing about *how it looks*. There are no `ratatui` or `crossterm` imports
here. This separation means the engine is testable in isolation and could, in
principle, power a web renderer or a headless validator just as well as a TUI.

Error handling is stratified deliberately:

- **Library functions** return `Result<T, EngineError>` — typed errors that
  callers can match on and handle programmatically.
- **Application-boundary functions** (the loader) return `anyhow::Result<T>`
  — rich context chains that are shown to the user as error messages.

```text
┌─────────────────────────────────────────────────────┐
│                  fireside-engine                     │
│                                                      │
│  loader.rs   → load_graph / save_graph               │
│  validation.rs → validate_graph → Vec<Diagnostic>    │
│  traversal.rs → TraversalEngine (state machine)      │
│  commands.rs → Command + CommandHistory (undo/redo)  │
│  session.rs  → PresentationSession (runtime root)    │
└─────────────────────────────────────────────────────┘
```

## Module Map

```text
fireside-engine/src/
├── lib.rs         — public re-exports
├── error.rs       — EngineError (thiserror)
├── loader.rs      — load_graph, load_graph_from_str, save_graph
├── validation.rs  — validate_graph → Vec<Diagnostic>
├── traversal.rs   — TraversalEngine (Next, Choose, Goto, Back)
├── commands.rs    — Command enum, CommandHistory (undo/redo)
└── session.rs     — PresentationSession
```

## Key Types

### `PresentationSession`

The session is the runtime root that the TUI layer holds. It combines the graph,
the traversal state machine, a dirty flag, and the command history for
undo/redo:

```rust
pub struct PresentationSession {
    pub graph: Graph,
    pub traversal: TraversalEngine,
    pub dirty: bool,
    pub command_history: CommandHistory,
}
```

All of these fields are `pub`. The session is a simple value carrier — it does
not enforce invariants through access control. Instead, the engine functions
(`TraversalEngine::next`, `Command::apply`, etc.) maintain correctness by
always keeping these fields in sync.

Creating a session is straightforward:

```rust
let graph = load_graph(Path::new("talk.json"))?;
let session = PresentationSession::new(graph, 0 /* start at first node */);
```

### `TraversalEngine`

The traversal engine is a small state machine. It owns the current node index
and a history stack for the `Back` operation:

```rust
pub struct TraversalEngine {
    current: usize,
    history: Vec<usize>,
}
```

Both fields are private. The only way to change position is through the four
traversal operations, which enforce the navigation contract:

```rust
// Respects traversal.next override → traversal.after rejoin → sequential.
pub fn next(&mut self, graph: &Graph) -> TraversalResult;

// Pops history, or falls back to sequential backward.
pub fn back(&mut self) -> TraversalResult;

// Jumps directly to a node index; errors if out of bounds.
pub fn goto(&mut self, index: usize, graph: &Graph)
    -> Result<TraversalResult, EngineError>;

// Resolves a branch option key → node ID → index; errors if not found.
pub fn choose(&mut self, key: char, graph: &Graph)
    -> Result<TraversalResult, EngineError>;
```

`TraversalResult` communicates what happened without requiring the caller to
diff state before and after:

```rust
pub enum TraversalResult {
    Moved { from: usize, to: usize },
    AtBoundary,   // already at first/last node
}
```

#### The `next` Priority Chain

When `next` is called, the engine checks three conditions in order:

1. **`traversal.next` override** — the node author forced the next destination.
2. **`traversal.after` rejoin target** — the branch sub-path has ended; return
   to the main flow.
3. **Sequential advance** — move to `current + 1`.

This layering means that a branch looks like any other set of nodes at the
encoding level. The `after` field on the last node of a branch sub-path is all
that is needed to rejoin the main sequence.

### `Command` and `CommandHistory`

The editor exposes graph mutations as command objects, making undo/redo a
first-class concern:

```rust
pub enum Command {
    UpdateNodeContent { node_id: NodeId, content: Vec<ContentBlock> },
    AddNode { node_id: NodeId, after_index: Option<usize> },
    RestoreNode { node: Node, index: usize },
    RemoveNode { node_id: NodeId },
    SetTraversalNext { node_id: NodeId, target: NodeId },
    ClearTraversalNext { node_id: NodeId },
}
```

Every command has a computable inverse. `CommandHistory` records the applied
command alongside its pre-computed inverse, stored as a `HistoryEntry`:

```rust
struct HistoryEntry {
    command: Command,
    inverse: Command,
}

pub struct CommandHistory {
    applied: Vec<HistoryEntry>,
    undone: Vec<HistoryEntry>,
}
```

Undo pops from `applied`, runs `inverse`, and pushes to `undone`. Redo pops
from `undone`, runs `command`, and pushes back to `applied`. There is no
complex diff logic — the inverse is always a concrete `Command` variant that
exactly reverses the original.

### `validate_graph`

Graph validation returns diagnostics rather than a single error, so tools can
report all issues at once:

```rust
pub struct Diagnostic {
    pub severity: Severity,   // Error | Warning
    pub message: String,
    pub node_id: Option<String>,
}
```

The validator checks:

- The graph contains at least one node.
- All `traversal.next`, `traversal.after`, and `BranchOption.target` references
  point to existing node IDs.
- Branch points have at least one option.

An empty `Vec<Diagnostic>` means the graph is structurally valid.

### Loader

The loader provides two entry points, both returning `anyhow::Result` so error
messages carry full context for the user:

```rust
// Load from a file path.
pub fn load_graph(path: &Path) -> Result<Graph>;

// Load from an in-memory string (useful for tests).
pub fn load_graph_from_str(source: &str) -> Result<Graph>;

// Serialize a Graph back to a JSON file.
pub fn save_graph(path: &Path, graph: &Graph) -> Result<()>;
```

`save_graph` reconstructs a `GraphFile` from the runtime `Graph` before
serializing, ensuring the output is always a clean wire-format document.

## Error Handling

```rust
pub enum EngineError {
    InvalidTraversal(String),
    CommandError(String),
    UnknownBranchKey(char),
}
```

These are the typed errors for programmatic handling. The loader uses `anyhow`
for I/O and JSON errors, enriching them with `with_context` call-site
annotations.

## Testing

```bash
cargo test -p fireside-engine
```

The engine is purely functional given a `Graph`, which makes unit tests
straightforward — construct a `Graph` in code, run an operation, assert on the
result. The `load_graph_from_str` function is particularly useful in tests
because it skips file I/O entirely:

```rust
#[test]
fn next_respects_override() {
    let json = r#"{
        "nodes": [
            {
                "id": "intro",
                "traversal": { "next": "conclusion" },
                "content": []
            },
            { "id": "skipped", "content": [] },
            { "id": "conclusion", "content": [] }
        ]
    }"#;
    let graph = load_graph_from_str(json).unwrap();
    let mut engine = TraversalEngine::new(0);
    let result = engine.next(&graph);
    assert_eq!(result, TraversalResult::Moved { from: 0, to: 2 });
}
```

## Dependencies

| Crate | Purpose |
| --- | --- |
| `fireside-core` | Protocol types (`Graph`, `Node`, `ContentBlock`, etc.) |
| `serde_json` | JSON deserialization in the loader |
| `thiserror` | `EngineError` derivation |
| `anyhow` | Rich error context in the loader |
| `tracing` | Structured log emission for diagnostic events |
