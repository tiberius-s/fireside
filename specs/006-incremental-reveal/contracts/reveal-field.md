# Contract: the `reveal` ContentBlock field

## Wire shape

Every content block kind may carry an optional, non-negative integer:

```json
{ "kind": "text", "body": "...", "reveal": 1 }
```

- Absent, or `"reveal": 0` → the block is visible the instant the node is
  entered. These two forms are semantically identical; authors may use
  either.
- `"reveal": N` for `N >= 1` → the block is hidden until the presenter has
  reached reveal step `N` at the current node (see
  `next-operation.md` for exactly how steps are numbered).
- Negative values are rejected at the schema layer (`@minValue(0)`); this
  is a parse-time/validate-time failure, not an engine-runtime concern.

## Step derivation (per node, recomputed fresh on every entry)

1. Walk the node's `content` array recursively (including into every
   `ContainerBlock.children`, at any depth).
2. Collect every block's own `reveal` value where present and `>= 1`
   (values of `0` or absent are excluded — they are always visible,
   not a "step").
3. Deduplicate and sort ascending. This sequence is the node's reveal
   steps, in presentation order. A node with an empty sequence has no
   reveal behavior at all — `next()` is completely unaffected by this
   feature.

## Visibility rule

A block (at any depth) is visible at reveal threshold `T` (an engine's
current `reveal_level`, starting at `0`) exactly when
`block.reveal.unwrap_or(0) <= T`. This is evaluated independently at every
depth — a container becomes visible/invisible by this same rule applied to
its own `reveal` value; a child's own value is evaluated the same way
regardless of whether its ancestor container is itself hidden. (Note:
`reveal-masked-by-container`, a separate validator warning, flags the
common authoring mistake where a child's value is lower than its
container's — such a child is well-defined but can never actually appear
earlier than its container, since the container being hidden hides
everything inside it regardless of the children's own values.)

## Compatibility

An engine that does not implement this feature at all (a 0.1.0/0.1.1
engine, or any third-party engine that has not adopted 0.1.2) reads
`reveal` as an unrecognized field, per every existing content block's
"unknown properties are ignored on read" rule, and renders the block
immediately — every block, on first entry, exactly as if `reveal` had
never been specified. This is the intended, safe forward-compatible
degrade: a reveal-authored deck is fully presentable (all content
eventually visible, in authored order — just not incrementally) on any
engine, old or new.
