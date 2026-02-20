---
title: 'fireside-core'
description: 'Protocol types, serde wire format, and data model invariants for the Fireside format.'
---

`fireside-core` is the **protocol boundary** of the Fireside workspace. It owns
the complete data model, the JSON wire format, and the runtime graph
representation. No other crate is allowed to own these concerns; every crate
above it consumes its types through re-exports.

## Crate responsibilities

| Owns                                                    | Explicitly excluded                     |
| ------------------------------------------------------- | --------------------------------------- |
| All protocol types (`Graph`, `Node`, `ContentBlock`, …) | I/O and file reading                    |
| `serde` derive configuration and wire-format names      | Validation logic                        |
| Runtime index construction (`Graph::from_file`)         | Any `ratatui` or `crossterm` dependency |
| `CoreError` variants for model-level failures           | Application-level error handling        |

## Module map

```text
fireside-core/src/
├── lib.rs                  re-exports; public API surface
├── error.rs                CoreError (thiserror)
└── model/
    ├── mod.rs
    ├── content.rs          ContentBlock enum + ListItem custom deserializer
    ├── graph.rs            GraphFile (serde), Graph (runtime), GraphMeta, NodeDefaults
    ├── layout.rs           Layout enum (12 variants)
    ├── node.rs             Node, NodeId type alias, traversal accessor helpers
    ├── transition.rs       Transition enum (8 variants)
    └── traversal.rs        Traversal, BranchPoint, BranchOption
```

## Wire format design

The Fireside protocol mandates **kebab-case JSON**. Every struct and enum in
this crate uses `#[serde(rename_all = "kebab-case")]` globally, with explicit
`#[serde(rename = "...")]` overrides on the handful of identifiers that do not
map cleanly:

```rust
// extension_type cannot map automatically because "type" is a Rust keyword
#[serde(rename = "type")]
pub extension_type: String,
```

Rust field names are idiomatic snake_case while wire names are kebab-case.
The table below lists fields where this difference matters most:

| Rust field          | JSON wire name        |
| ------------------- | --------------------- |
| `speaker_notes`     | `"speaker-notes"`     |
| `branch_point`      | `"branch-point"`      |
| `highlight_lines`   | `"highlight-lines"`   |
| `show_line_numbers` | `"show-line-numbers"` |
| `extension_type`    | `"type"`              |

## The `GraphFile` / `Graph` split

Two separate types represent the same document at different stages of the
pipeline:

**`GraphFile`** is the direct `serde` target. Its field layout mirrors the JSON
schema one-to-one. It is only instantiated during deserialization and is
immediately consumed by `Graph::from_file`.

**`Graph`** is the runtime representation. It adds `node_index: HashMap<NodeId,
usize>` for O(1) ID lookup and applies `NodeDefaults` to every node that omits
`layout` or `transition`. The index is built once and never partially updated;
any structural mutation must call `Graph::rebuild_index()`.

```rust
pub struct Graph {
    pub metadata:   GraphMeta,
    pub nodes:      Vec<Node>,
    pub node_index: HashMap<NodeId, usize>,   // built once in from_file()
}
```

`rebuild_index` is the contract: after any `nodes.push`, `nodes.remove`, or
`nodes.swap`, every index position may be invalidated. The engine's command
system calls `rebuild_index` after applying a mutation command; forgetting to
do so produces stale `node_by_id` results without any compile-time warning.

## `ContentBlock` — the discriminated union

The eight content block types are a single Rust enum with an **internally tagged
`"kind"` discriminator**:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ContentBlock {
    Heading { level: u8, text: String },
    Text    { body: String },
    Code    { language: Option<String>, source: String,
               highlight_lines: Vec<u32>, show_line_numbers: bool },
    List    { ordered: bool, items: Vec<ListItem> },
    Image   { src: String, alt: String, caption: Option<String> },
    Divider,
    Container { layout: Option<String>, children: Vec<ContentBlock> },
    Extension { extension_type: String, fallback: Option<Box<ContentBlock>>,
                #[serde(flatten)] payload: serde_json::Value },
}
```

The internal tag approach (as opposed to `#[serde(untagged)]` or adjacently
tagged) was chosen because it produces the most readable wire JSON — the
discriminator appears alongside peer fields as `"kind": "heading"` — and
because it gives serde better error messages when a required field is missing.

### Unit variant without fields: `Divider`

`Divider` carries no data. In JSON it serializes as `{"kind": "divider"}`.
Internally tagged unit variants in serde produce exactly this output without
any manual implementation.

### Boxed recursive type: `Extension.fallback`

`fallback: Option<Box<ContentBlock>>` is `Box`ed because `ContentBlock` would
otherwise be infinitely sized on the stack (a type containing itself
transitively). The `Box` indirection breaks the recursive size dependency
while preserving full type safety.

### Flattened arbitrary payload: `Extension.payload`

`#[serde(flatten)]` on `payload: serde_json::Value` absorbs all JSON fields
not claimed by `extension_type` and `fallback` into an opaque `Value`.
This means an extension author can place any JSON key at the top level of the
extension object without defining a Rust struct.

## `ListItem` — custom `Deserialize`

The wire format permits list items as either a bare string or a structured
object:

```json
// bare string form
{ "kind": "list", "items": ["Alpha", "Beta"] }

// object form
{ "kind": "list", "items": [{ "text": "Alpha", "children": [...] }] }
```

`ListItem` only derives `Serialize` (always outputs the object form) but
implements `Deserialize` by hand with a `Visitor` that branches on whether the
incoming value is a JSON string or a JSON map:

```rust
impl<'de> Deserialize<'de> for ListItem {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = ListItem;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a string or list-item object")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<ListItem, E> {
                Ok(ListItem { text: v.into(), children: vec![] })
            }
            fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<ListItem, A::Error> {
                // standard struct deserialization via MapAccessDeserializer
            }
        }
        d.deserialize_any(Visitor)
    }
}
```

This is one of the more advanced serde patterns in the codebase. The key
insight is calling `deserialize_any` rather than `deserialize_map` or
`deserialize_str`: it lets serde forward whichever token type the input
provides, and the `Visitor` dispatches accordingly.

## Error model

`CoreError` is a `thiserror`-derived enum covering exactly the failure modes
that can arise from the model layer:

```rust
pub enum CoreError {
    FileRead    { path: PathBuf, #[source] source: std::io::Error },
    InvalidJson(String),
    EmptyGraph,
    DuplicateNodeId(String),
}
```

`FileRead` carries the path and the underlying `io::Error` as a `#[source]`
chain, which means `anyhow` callers in the engine layer get the full context
automatically. `InvalidJson`, `EmptyGraph`, and `DuplicateNodeId` are
terminal; they indicate a document that cannot be used.

`CoreError` deliberately excludes validation errors (dangling references,
unreachable nodes). Those live in `fireside-engine::validation::Diagnostic`
because they require graph-level reasoning that depends on the full node index.

## `NodeId` — type alias tradeoff

```rust
pub type NodeId = String;
```

`NodeId` is a type alias, not a newtype. This keeps serde and `HashMap` usage
simple at the cost of accidentally mixing a node ID with any other `String`.
The alias documents intent without enforcing it at compile time. A future
newtype (`pub struct NodeId(String)`) would add a smart constructor for
validation but would require `AsRef<str>` and `Borrow<str>` impls to work
cleanly with `HashMap::get`.

## `NodeDefaults` and cascading layout

`GraphFile` carries an optional `defaults: Option<NodeDefaults>` that sets
`layout` and `transition` for all nodes that do not override them. `Graph::from_file`
applies these during construction:

```rust
let nodes: Vec<Node> = file.nodes.into_iter().map(|mut n| {
    if n.layout.is_none()     { n.layout     = default_layout; }
    if n.transition.is_none() { n.transition = default_transition; }
    n
}).collect();
```

The override check uses `is_none()` on both fields rather than a single
boolean flag. This means a node can override only one dimension (e.g., set a
custom `transition` but inherit the document `layout`) without any additional
protocol machinery.

## Testing strategy

`fireside-core` has two test layers:

1. **Inline unit tests** (`model/content.rs`, `model/graph.rs`) — cover
   round-trip serialization, the custom `ListItem` visitor, duplicate-ID
   rejection, and defaults inheritance.

2. **Integration tests** (`tests/content_roundtrip.rs`) — exercise every
   `ContentBlock` variant for serde round-trip fidelity (serialize → deserialize
   → compare). These are the ground-truth contract for wire format stability.

Round-trip tests use `serde_json::to_string` → `serde_json::from_str` with
`assert_eq!` (via `pretty_assertions` for human-readable diff output).
A deliberate regression pattern: any serde attribute change that silently
drops a field will be caught immediately by these tests.
