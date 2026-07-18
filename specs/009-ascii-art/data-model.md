# Phase 1 Data Model: ASCII art content block

## Wire model additions (`protocol/main.tsp`)

### `AsciiArtBlock` (new model)

```tsp
/** Pre-rendered ASCII/text art, generated at authoring time. */
model AsciiArtBlock {
  ...Revealable;
  kind: "ascii-art";

  /** The pre-rendered multi-line art content, as plain text. */
  art: string;

  /** Alternative text description, for anyone who can't see the art. */
  alt?: string;
}
```

Spreads the existing `Revealable` model (unchanged — no new shared field
needed; this feature reuses the 0.1.2 `reveal` mechanism as-is).

### `ContentBlock` union

Gains `AsciiArtBlock` as an eighth member:

```tsp
union ContentBlock {
  HeadingBlock,
  TextBlock,
  CodeBlock,
  ListBlock,
  ImageBlock,
  DividerBlock,
  ContainerBlock,
  AsciiArtBlock,
}
```

Doc comment's "Conforming engines MUST support all 7 block kinds" becomes
"...all 8 block kinds."

### `Versions` enum

Gains `v0_1_3: "0.1.3"`. Version doc banner updated to describe 0.1.3 as
adding the `ascii-art` block kind and stating explicitly — unlike every
prior version bump in this banner's history — that documents using it are
**not** safely readable by pre-0.1.3 engines (new enum member, not an
additive optional field; see `research.md` §2 and ADR-012).

## Rust model additions (`fireside-core::model`)

### `ContentBlock::AsciiArt` (new variant)

```rust
/// Pre-rendered ASCII/text art, generated at authoring time.
AsciiArt {
    /// The incremental-reveal step at which this block becomes
    /// visible. See [`ContentBlock::Heading::reveal`].
    #[serde(skip_serializing_if = "Option::is_none")]
    reveal: Option<u32>,
    /// The pre-rendered multi-line art content, as plain text.
    art: String,
    /// Alternative text description, for anyone who can't see the art.
    #[serde(skip_serializing_if = "Option::is_none")]
    alt: Option<String>,
},
```

`kind: "ascii-art"` is implicit via the existing `#[serde(tag = "kind",
rename_all = "kebab-case", rename_all_fields = "kebab-case")]` enum
attributes — no per-variant tag literal needed, matching every other arm.

### `ContentBlock::reveal()` and `::children()`

Both gain an `AsciiArt` match arm:

```rust
Self::Heading { reveal, .. }
| Self::Text { reveal, .. }
| Self::Code { reveal, .. }
| Self::List { reveal, .. }
| Self::Image { reveal, .. }
| Self::Divider { reveal }
| Self::AsciiArt { reveal, .. }   // new
| Self::Container { reveal, .. } => *reveal,
```

`children()` needs no new arm beyond the existing `_ => &[]` catch-all —
`AsciiArt` is a leaf, same as `Heading`/`Text`/etc.

### `proptest_support::arbitrary_leaf_block()`

Gains one more `prop_oneof!` arm generating `ContentBlock::AsciiArt` with
an arbitrary short string for `art` and an optional arbitrary string for
`alt`, following the exact shape of the existing `Text`/`Image` arms. This
keeps `graph_round_trips_through_json` and
`reveal_levels_are_sorted_deduped_and_positive` (spec 008's proptests)
covering the new variant automatically — no new property test needed,
the existing ones become exhaustive over 8 variants instead of 7 once
this arm exists.

## Validation additions (`fireside-engine::validation` / `protocol/validate.mjs`)

Two new rules, both `Severity::Warning` (content-quality class, same
severity as `reveal-masked-by-container`/`malformed-link-url` — never
blocks presenting):

### `ascii-art-too-wide`

Fires when an `AsciiArt` block's widest line exceeds 76 columns
(research.md §4). Width is measured as Unicode scalar-value count
(`chars().count()`/`[...line].length`), **not** true display width —
`fireside-engine` cannot depend on `unicode-width` per the crate boundary
table (Principle III), so this is a documented approximation, exact for
plain ASCII art (the common case) and only imprecise for wide/combining
Unicode characters. `blocks.rs`'s renderer, which *can* depend on
`unicode-width`, is unaffected — this approximation is validator-only.
Message names the node and the measured width vs. the limit, mirroring
`container-nesting-depth-exceeded`'s message shape.

### `ascii-art-empty`

Fires when an `AsciiArt` block's `art` string is empty or contains only
whitespace. Message names the node.

Both walk `node.content` recursively through `Container` children, same
traversal shape as `check_reveal_masked_by_container`/`walk_link_urls`.

## Rendering additions (`fireside-tui::render::blocks`)

### Shared helper (refactor of existing logic)

The box-width/centering computation currently inlined in `code()` behind
`is_ascii_art(language)` is extracted into a private helper, e.g.:

```rust
/// Box width for a sized-to-content, centered block: the label prefix
/// width, or the content's widest line plus the given left prefix,
/// whichever is larger, capped at the available width.
fn centered_box_width(label_prefix_width: usize, content: &[&str], prefix: usize, full_width: usize) -> usize
```

`code()`'s existing `is_ascii_art` branch and the new `ascii_art()` fn
both call it — no behavior change for existing language-less code blocks
(spec 005's 5 existing scenario/insta tests remain the regression oracle,
per this project's established practice of not touching a safety net
mid-refactor).

### `ascii_art(art: &str, alt: Option<&str>, width: u16, tokens: &Tokens) -> Vec<Line<'static>>`

Draws a bordered, centered box (`─ ascii-art ─...` header, same border
token as `code()`) sized to `art`'s widest line via the shared helper,
with plain (unstyled, non-syntax-highlighted) monospace lines inside —
`alt` is not rendered visibly (it is metadata for anyone who can't see the
art, not on-screen text); it exists in the model for future
accessibility/export tooling, consistent with `ImageBlock.alt`'s existing
purpose in this codebase.

### `render_block()` match

Gains one arm:

```rust
ContentBlock::AsciiArt { art, alt, .. } => ascii_art(art, alt.as_deref(), width, tokens),
```

## CLI additions (`fireside-cli`)

### `Command::Art` (new subcommand, nested)

```rust
/// Generate ASCII art to paste into a deck.
Art {
    #[command(subcommand)]
    mode: ArtMode,
},
```

```rust
#[derive(Debug, Subcommand)]
enum ArtMode {
    /// Turn a short phrase into a stylized text banner.
    Text {
        /// The phrase to render.
        phrase: String,
    },
    /// Convert a local image file into ASCII art.
    Image {
        /// Path to the image file.
        path: PathBuf,
        /// Output width in columns. Defaults to a size that fits the
        /// standard supported terminal width.
        #[arg(long)]
        width: Option<u32>,
    },
}
```

### `art.rs` (new sibling module, `new.rs`/`import.rs`/`report.rs` pattern)

```rust
/// Generate a stylized text banner from `phrase` via figlet-rs and print
/// it to stdout. Errors if the phrase contains no character the font
/// recognizes (FR-013).
pub(crate) fn art_text(phrase: &str) -> Result<()>

/// Convert the image at `path` to ASCII art via rascii_art and print it
/// to stdout. Errors with a clear, actionable message (FR-014) if the
/// path doesn't exist or isn't a readable image.
pub(crate) fn art_image(path: &Path, width: Option<u32>) -> Result<()>
```

Both are thin CLI-boundary functions (`anyhow::Result`, `Context` on every
fallible call), matching the existing style of `new::new_deck`/
`import::import`.

## Constitution amendment (`.specify/memory/constitution.md`)

Principle III table, `fireside-cli` row's "Permitted dependencies" column
gains `figlet-rs`, `rascii_art`. No other row changes. Version bump: MINOR
(1.1.0 → 1.2.0), same class of change as the prior `pulldown-cmark`
amendment (ADR-006).
