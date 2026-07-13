# Phase 1 Data Model: Incremental reveal

## Wire model additions (`protocol/main.tsp`)

### `Revealable` (new shared spread model)

```tsp
model Revealable {
  /**
   * The incremental-reveal step at which this block becomes visible.
   * Absent or 0 means the block is visible as soon as the node is
   * entered. See TraversalOps.next() for how engines consume reveal
   * steps. Engines that do not implement reveal MUST ignore this field
   * and render the block immediately — a safe degrade to "everything
   * visible."
   */
  @minValue(0)
  reveal?: int32;
}
```

Spread (`...Revealable;`) into all seven block models: `HeadingBlock`,
`TextBlock`, `CodeBlock`, `ListBlock`, `ImageBlock`, `DividerBlock`,
`ContainerBlock`. No per-variant duplication of the field or its doc
comment.

### `Versions` enum

Gains `v0_1_2: "0.1.2"`. Version doc banner updated to "0.1.2 (0.1.1
documents remain valid; 0.1.2 adds the optional `reveal` field and one new
validator diagnostic)".

### `TraversalOps.next()` (documentary interface)

Doc comment gains the reveal-precedes-everything algorithm step, inserted
before the existing branch-point check (see `contracts/next-operation.md`).

## Rust model additions (`fireside-core::model`)

### `ContentBlock`

Every variant gains:

```rust
/// The incremental-reveal step at which this block becomes visible.
/// `None` and `Some(0)` are equivalent: visible immediately.
#[serde(skip_serializing_if = "Option::is_none")]
reveal: Option<u32>,
```

New accessor:

```rust
impl ContentBlock {
    /// This block's own reveal marker, if any. `None` means "always
    /// visible" — equivalent to `Some(0)` for step-computation purposes.
    #[must_use]
    pub fn reveal(&self) -> Option<u32> { /* match over all 7 variants */ }
}
```

### `Node`

New method:

```rust
impl Node {
    /// The distinct positive `reveal` values used anywhere in this
    /// node's content, recursively through `Container::children`, sorted
    /// ascending. An empty result means the node uses no reveal marks —
    /// `next()` never pauses for reveal on such a node.
    #[must_use]
    pub fn reveal_levels(&self) -> Vec<u32> { /* walk content, dedup, sort */ }
}
```

## Engine model additions (`fireside-engine::session`)

### `Outcome`

New variant:

```rust
/// A reveal step was consumed: more of the current node's content
/// became visible. The current node did NOT change.
Revealed,
```

### `Session`

New private field `reveal_level: u32` (starts at `0`, reset to `0` in
`move_to` and in `back`).

New public methods:

```rust
impl Session {
    /// The reveal threshold currently reached at the current node. Blocks
    /// with `reveal().unwrap_or(0) <= this` are visible.
    #[must_use]
    pub fn reveal_level(&self) -> u32;

    /// Whether the current node has reveal steps not yet reached.
    #[must_use]
    pub fn has_pending_reveal(&self) -> bool;

    /// (revealed, total) distinct reveal steps for the current node.
    /// `None` when the node uses no reveal marks at all.
    #[must_use]
    pub fn reveal_progress(&self) -> Option<(usize, usize)>;
}
```

`next()` rewritten: consult `current().reveal_levels()`; if any value
exceeds `reveal_level`, set `reveal_level` to the smallest such value and
return `Outcome::Revealed`. Otherwise, fall through to the existing
branch-point/next-target logic unchanged.

## Validation additions

### Rust (`fireside-engine::validation`)

New rule `reveal-masked-by-container` (WARNING): for every `ContainerBlock`
whose own `reveal` is `Some(n)`, walk its `children` (recursively, stopping
recursion at nested containers with their own higher-or-equal reveal —
each container is checked against its own immediate parent's threshold,
not a global maximum) and warn on any child whose own `reveal` is `Some(m)`
with `m < n`.

### Node (`protocol/validate.mjs`)

`checkRevealMaskedByContainer(graph)` — same rule, same message tone,
mirroring the Rust implementation exactly (rule-id parity, per the
project's dual-validator convention).

## Rendering additions (`fireside-tui::render`)

`blocks::render_blocks(blocks: &[ContentBlock], width: u16, tokens:
&Tokens, reveal_level: u32) -> Vec<Line<'static>>` — filters `blocks` to
those with `reveal().unwrap_or(0) <= reveal_level` before iterating; for
`ContentBlock::Container`, the same filter applies to `children` before
layout (so `columns()` divides width by the *visible* child count only),
and `reveal_level` is threaded through recursively.

`mod.rs::node_lines` passes `app.session().reveal_level()` through to
`render_blocks`.

## App/TUI state additions (`fireside-tui::app`)

`App::apply()` gains an `Outcome::Revealed` arm: clears flash, resets
scroll (so newly revealed content is in view), does NOT touch
`fade_started` or `branch_selected` (no real navigation occurred).

`App::on_present_key`'s `at_branch` gate becomes:

```rust
let at_branch = self.session.branch_point().is_some()
    && !self.session.has_pending_reveal();
```

`draw_footer` gains a reveal-progress badge, shown only when
`session.reveal_progress()` returns `Some((revealed, total))` with
`revealed < total`.
