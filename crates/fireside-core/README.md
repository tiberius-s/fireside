# fireside-core

The protocol layer for the Fireside format. This crate owns every type that
appears in a Fireside JSON document — nothing more. It has no rendering code,
no I/O, and no validation logic beyond what serde enforces during
deserialization.

If you want to understand what Fireside _is_, start here.

## Design Philosophy

A central challenge in any portable format is keeping the serialization layer
thin and honest. `fireside-core` achieves this by owning **one explicit
boundary**: the difference between a _wire document_ (`GraphFile`) and a
_runtime graph_ (`Graph`).

```text
JSON on disk
    └── serde_json::from_str → GraphFile (wire repr)
                                   └── Graph::from_file → Graph (runtime repr)
```

`GraphFile` matches the JSON schema field-for-field. `Graph` adds the
`node_index` hash map for O(1) node lookup by ID — an optimization that would
be wrong to encode in the schema but is essential at runtime.

The rest of the crate is pure data. No `impl Trait`, no generics over
lifetimes, no async. Every type is `Debug + Clone + PartialEq + Serialize +
Deserialize` unless there's a good reason not to be.

## Module Map

```text
fireside-core/src/
├── lib.rs           — public re-exports
├── error.rs         — CoreError (thiserror)
└── model/
    ├── mod.rs       — module declarations
    ├── branch.rs    — BranchPoint, BranchOption
    ├── content.rs   — ContentBlock (8 variants), ListItem
    ├── graph.rs     — Graph (runtime), GraphFile (wire), GraphMeta, NodeDefaults
    ├── layout.rs    — Layout enum (12 variants)
    ├── node.rs      — Node, NodeId, helper methods
    ├── transition.rs — Transition enum (8 variants)
    └── traversal.rs — Traversal struct
```

## Key Types

### `Graph` and `GraphFile`

`GraphFile` is the exact shape of a `.json` document on disk. It maps directly
to the JSON Schema and uses `#[serde(rename_all = "kebab-case")]` to honour the
wire format.

`Graph` is what the engine actually uses at runtime. Its `from_file` associated
function converts a `GraphFile`, building a `node_index` for fast lookups and
applying document-level `NodeDefaults` to every node:

```rust
pub struct Graph {
    pub metadata: GraphMeta,
    pub nodes: Vec<Node>,
    // HashMap from NodeId → Vec index. Built by Graph::from_file.
    node_index: HashMap<NodeId, usize>,
}

impl Graph {
    /// Look up a node by ID in O(1) time.
    pub fn node_by_id(&self, id: &NodeId) -> Option<&Node> {
        self.node_index.get(id).map(|&i| &self.nodes[i])
    }

    /// Translate an ID to its position in `nodes`.
    pub fn index_of(&self, id: &NodeId) -> Option<usize> {
        self.node_index.get(id).copied()
    }
}
```

The separation ensures that the hash map is always consistent with the `nodes`
vec — there is no way to construct a `Graph` in an inconsistent state because
the only construction path goes through `from_file`.

### `Node`

A node is a graph vertex. It carries visual metadata (`layout`, `transition`),
optional speaker notes, traversal overrides, and the content blocks that make
up what the audience sees.

```rust
pub struct Node {
    pub id: Option<NodeId>,
    pub layout: Option<Layout>,
    pub transition: Option<Transition>,
    #[serde(rename = "speaker-notes")]
    pub speaker_notes: Option<String>,
    pub traversal: Option<Traversal>,
    pub content: Vec<ContentBlock>,
}
```

The three helper methods on `Node` are the contract the engine uses for
traversal decisions. They avoid spreading `Option`-chaining across the call
sites:

```rust
// Returns traversal.next if set (forces jump to another node).
pub fn next_override(&self) -> Option<&NodeId>;

// Returns traversal.branch_point if set (audience makes a choice).
pub fn branch_point(&self) -> Option<&BranchPoint>;

// Returns traversal.after if set (branch rejoin target).
pub fn after_target(&self) -> Option<&NodeId>;
```

`NodeId` is a type alias for `String`. The `minLength: 1` constraint from the
JSON Schema is enforced during validation (in `fireside-engine`), not here.

### `ContentBlock`

This is the discriminated union at the heart of the content model. Serde's
`tag` attribute makes `"kind"` the discriminator field, and `rename_all =
"kebab-case"` maps variant names to their wire values automatically:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ContentBlock {
    Heading { level: u8, text: String },
    Text { body: String },
    Code {
        language: Option<String>,
        source: String,
        #[serde(default, rename = "highlight-lines")]
        highlight_lines: Vec<u32>,
        #[serde(default, rename = "show-line-numbers")]
        show_line_numbers: bool,
    },
    List {
        #[serde(default)]
        ordered: bool,
        items: Vec<ListItem>,
    },
    Image { src: String, alt: Option<String>, caption: Option<String> },
    Divider,
    Container { layout: Option<Layout>, children: Vec<ContentBlock> },
    Extension {
        #[serde(rename = "type")]
        extension_type: String,
        fallback: Option<Box<ContentBlock>>,
        payload: Option<serde_json::Value>,
    },
}
```

Because the tag is embedded in the JSON object (`"kind": "heading"`, etc.),
there is no wrapper key in the output — a clean, human-readable wire format.

### `ListItem` — Custom Deserialization

`ListItem` illustrates a common Rust pattern: accepting two distinct JSON
shapes for the same field. A list item can be either a bare string or a
structured object with optional children:

```json
// Bare string form
"items": ["First point", "Second point"]

// Structured form
"items": [
  { "text": "First point", "children": ["sub-item a", "sub-item b"] }
]
```

The custom `Deserialize` impl tries the `&str` path first, then falls back to
the `{ text, children }` struct form. This keeps documents terse for simple
lists while allowing nesting where needed.

### `Traversal`, `BranchPoint`, `BranchOption`

`Traversal` is the per-node override record. All three fields are optional:

```rust
pub struct Traversal {
    /// Force the "next" operation to jump to this node instead of advancing.
    pub next: Option<NodeId>,
    /// After a branch sub-path finishes, rejoin here.
    pub after: Option<NodeId>,
    /// Present a choice overlay to the audience.
    #[serde(rename = "branch-point")]
    pub branch_point: Option<BranchPoint>,
}
```

`BranchPoint` holds a prompt and a list of `BranchOption`s. Each option has a
`key` (the character the audience presses) and a `target` node ID:

```rust
pub struct BranchOption {
    pub label: String,
    pub key: char,
    pub target: NodeId,
}
```

### `Layout` and `Transition`

Both are enums with `#[serde(rename_all = "kebab-case")]` plus explicit
`#[serde(rename = "...")]` on variants whose names would not map correctly by
convention alone (e.g., `SplitHorizontal` → `"split-horizontal"`).

`Layout` variants (12):

| Variant           | Wire value           | Description                          |
| ----------------- | -------------------- | ------------------------------------ |
| `Default`         | `"default"`          | Standard padding (the default)       |
| `Center`          | `"center"`           | Horizontally and vertically centered |
| `Top`             | `"top"`              | Top-aligned with standard padding    |
| `SplitHorizontal` | `"split-horizontal"` | Two equal columns                    |
| `SplitVertical`   | `"split-vertical"`   | Two rows                             |
| `Title`           | `"title"`            | Large centered title, subtitle below |
| `CodeFocus`       | `"code-focus"`       | Maximized code area, minimal chrome  |
| `Fullscreen`      | `"fullscreen"`       | Full screen, no chrome               |
| `AlignLeft`       | `"align-left"`       | Left-aligned content                 |
| `AlignRight`      | `"align-right"`      | Right-aligned content                |
| `Blank`           | `"blank"`            | No predefined layout                 |

`Transition` variants (8): `None`, `Fade`, `SlideLeft`, `SlideRight`, `Wipe`,
`Dissolve`, `Matrix`, `Typewriter`.

## Wire Format Rules

All JSON property names use **kebab-case**: `speaker-notes`, `branch-point`,
`highlight-lines`, `show-line-numbers`. Enum values also use kebab-case:
`"split-horizontal"`, `"slide-left"`, `"align-right"`.

The `"kind"` field is the discriminator for `ContentBlock`. Extension blocks
always carry `"kind": "extension"` plus a required `"type"` identifier.

## Error Handling

`CoreError` uses `thiserror` to give every failure mode a distinct type:

```rust
pub enum CoreError {
    FileRead { path: PathBuf, source: io::Error },
    InvalidJson(String),
    EmptyGraph,
    DuplicateNodeId(String),
}
```

Library code returns `Result<T, CoreError>`. Callers at the application
boundary can convert to `anyhow::Error` via `?` since `CoreError` implements
`std::error::Error`.

## Testing

```bash
cargo test -p fireside-core
```

Tests live next to the code they exercise. The `pretty_assertions` dev
dependency gives readable multi-line diffs when complex structs diverge.

A good test for a content block round-trip looks like:

```rust
#[test]
fn code_block_roundtrip() {
    let block = ContentBlock::Code {
        language: Some("rust".into()),
        source: "fn main() {}".into(),
        highlight_lines: vec![1],
        show_line_numbers: true,
    };
    let json = serde_json::to_string(&block).unwrap();
    let block2: ContentBlock = serde_json::from_str(&json).unwrap();
    assert_eq!(block, block2);
}
```

## Dependencies

| Crate        | Purpose                                                             |
| ------------ | ------------------------------------------------------------------- |
| `serde`      | Derive macros for `Serialize` / `Deserialize`                       |
| `serde_json` | JSON serialization and the untyped `Value` type used in `Extension` |
| `thiserror`  | Ergonomic error enum derivation                                     |
