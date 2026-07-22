# Contract: `engine::authoring` operations

New module `crates/fireside-engine/src/authoring.rs`. Every operation has
this shape:

```text
fn apply(graph: &Graph, op: &Op) -> Result<Graph, AuthoringError>
```

Pure, no I/O, no `App`/TUI dependency (Constitution III: `fireside-engine`
forbids ratatui/crossterm/clap/anyhow). `EditorApp::update` is the only
caller; `EditorApp::history` pushes the resulting clone.

## Universal guarantees (every `Op`, spec FR-023 / SC-007)

1. **No dangling reference.** If the result of applying `op` would leave any
   `next`, branch-option `target`, or the graph's entry-node reference
   pointing at a nonexistent id, `apply` returns `Err` instead of that
   `Graph` — never a graph with a dangling reference, even transiently.
2. **No duplicate id.** `apply` never returns a `Graph` with two nodes
   sharing an `id`.
3. **No `next`+`branch_point` conflict.** `apply` never returns a `Graph`
   where any node's `traversal` sets both.
4. **No gapped reveal numbering.** `apply` never returns a `Graph` where any
   node's distinct positive `reveal` values (per `Node::reveal_levels()`)
   skip a step.
5. **Atomicity.** `apply` either returns the fully-transformed `Graph` or an
   `Err` and the *caller's* graph is untouched (ordinary Rust `Result`
   semantics — no partial mutation, since `Graph` is never mutated in
   place).

## Operation reference

| `Op` variant | Preconditions (else `Err`) | Postcondition |
| --- | --- | --- |
| `AddSlide { after: NodeId, title: String }` | `after` exists | New node inserted with a slug id (unique, per `data-model.md`'s algorithm), wired as `after`'s `next` if `after` had none, else left unreachable-until-wired |
| `DeleteSlide { id: NodeId }` | `id` exists; `id` is not the entry node | Node removed; every `next`/target reference to `id` rewritten to `id`'s own `next` target (or cleared to an ending if `id` had none) — "heals wiring," spec US3 scenario 3 |
| `DuplicateSlide { id: NodeId }` | `id` exists | New node with a fresh slug id, content cloned, `traversal` cleared (duplicate starts unreachable, author wires it) |
| `RetitleSlide { id: NodeId, title: String }` | `id` exists | Node's `title` set; if the slug derived from `title` differs from `id`, the id changes and every reference to the old id (every `next`, every branch `target`, the entry-node position) is rewritten in the same op — proptest-covered: no rename sequence can dangle a reference |
| `ReorderSlide { id: NodeId, before: Option<NodeId> }` | `id` and `id`'s predecessor(s) are all in one unbranched linear run as `before` | Node array order updates to match; wiring (`next` chain) updates to match the new order |
| — attempted across a branch boundary | — | `Err(CrossesBranchBoundary)` — no partial reorder |
| `SetNext { id: NodeId, target: NodeId }` | both exist; `id` is not currently a branch point | `id`'s `traversal` becomes `Rules { next: Some(target) }` |
| `ClearNext { id: NodeId }` | `id` exists | `id`'s `traversal` becomes `None` (ending) |
| `TurnIntoChoice { id: NodeId, prompt: Option<String>, first_answer: (String, NodeId) }` | `id` exists; `first_answer.1` exists | `id`'s `traversal` becomes `Rules { branch_point: Some(..) }` with one option; any prior `next` is discarded |
| `TurnBackIntoSlide { id: NodeId }` | `id` is a branch point | `id`'s `traversal` becomes `Rules { next: Some(first_option.target) }` — keeps the first answer's target, per spec |
| `AddAnswer { id: NodeId, label: String, key: Option<String>, target: NodeId }` | `id` is a branch point; `target` exists; `key` (if set) is not a reserved presenter key | New `BranchOption` appended |
| `RemoveAnswer { id: NodeId, index: usize }` | `id` is a branch point with >1 option, `index` valid | Option removed (removing the last option is rejected — `TurnBackIntoSlide` is the path to zero-branch) |
| `RetargetAnswer { id: NodeId, index: usize, target: NodeId }` | `id` is a branch point, `index` valid, `target` exists | Option's `target` updated |
| `AddBlock { node: NodeId, path: BlockPath, kind: BlockKind, at: usize }` | `node` exists; `path` resolves (root or into an existing `Container`) | New block with kind-appropriate placeholder content inserted at `at` |
| `DeleteBlock { node: NodeId, path: BlockPath }` | block exists | Block removed |
| `EditBlock { node: NodeId, path: BlockPath, content: BlockContent }` | block exists, `content`'s shape matches the existing block's kind | Block's fields replaced |
| `MoveBlock { node: NodeId, path: BlockPath, to: usize }` | block exists; `to` is a valid index within the same parent (siblings only — no cross-slide, no cross-container move) | Block reordered among siblings |
| `SetRevealStep { node: NodeId, path: BlockPath, step: Option<u32> }` | block exists | Block's `reveal` set; every distinct positive value across the node's content is renumbered to stay consecutive from 1 (per `Node::reveal_levels()`'s existing ordinal semantics) |

`BlockPath` addresses a block by its position within a node's (possibly
nested, via `Container`) content tree — an in-memory index path, never
serialized, never shown to the user (spec FR-024).

## Outline ordering (not an `Op` — a pure query)

```text
fn outline_order(graph: &Graph) -> Vec<OutlineRow>
```

Deterministic: depth-first from `graph.entry()`, `next` before branch
options in declared order, first-visit wins (cycles terminate), then every
node never visited that way, appended after a divider, in `graph.nodes`
declaration order. See `research.md` §8 and `data-model.md` for why this is
implemented fresh here rather than shared with `render/map.rs`.

## Error surface

`AuthoringError` (thiserror) — one variant per precondition failure above,
each carrying enough context (ids involved) for the caller to render a
plain-language toast (spec FR-024: never the variant name itself, never a
raw id in chrome — the toast wraps the id in the slide's title where one is
being named, e.g. "Features is one of Pick-a-path's answers — …").
