# Phase 0 Research: Markdown Import

## 1. Markdown parser and inline-content strategy

**Decision**: Use `pulldown-cmark` 0.13's `Parser::new_ext(source,
options).into_offset_iter()`, which yields `(Event, Range<usize>)` — every
event paired with its byte range in the original source. For paragraph and
heading text, **slice the original source** at that range instead of
reconstructing text from the event stream's inline events (`Strong`,
`Emphasis`, etc.).

**Rationale**: The spec requires inline Markdown to be "preserved as
written" (FR-009) — the protocol's `TextBlock`/`Heading` already accept
inline Markdown and the TUI renderer (`fireside-tui/src/render/markdown.rs`)
already knows how to render `**bold**`/`*italic*`/`` `code` `` at present
time. Reconstructing that syntax from a parsed AST (walking `Strong`/
`Emphasis`/`Code` events and re-emitting `**`/`*`/`` ` ``) is extra work
that only risks a lossy round-trip (e.g. authors who mix `_x_` and `*x*`
italics, which pulldown-cmark treats identically but a reconstructor might
normalize). Byte-range slicing sidesteps this entirely — the paragraph's
range in the source **is** its Markdown, verbatim, so "preserve as written"
holds by construction.

**Alternatives considered**: Walking the event tree and re-serializing to
Markdown (rejected — solves a problem slicing doesn't have, and risks
subtly changing an author's inline syntax); a different parser
(`comrak`, `markdown-rs`) — `pulldown-cmark` was chosen because it's the
parser `mdBook`/`rustdoc` already use in the wider Rust ecosystem, is
`unicode-width`-based (a dependency `fireside-tui` already carries), and
was verified to build clean under MSRV 1.88 (see ADR-006).

## 2. Node/section boundaries

**Decision**: Iterate the top-level event stream; on every
`Start(Tag::Heading { level: HeadingLevel::H2, .. })`, close the
in-progress node section (if any) and open a new one. Everything between
one `##` heading and the next (or end of file) belongs to that section.

**Rationale**: Matches FR-004 exactly and is a single linear pass over the
event stream — no lookahead or backtracking needed, since `pulldown-cmark`
already tokenizes heading level as part of the `Tag`.

## 3. Content-block conversion within a section

**Decision**: For each section, walk its events and dispatch by tag:

- `Heading { level: 3..=6, .. }` → `ContentBlock::Heading` (level cast to
  `u8`; text from inner `Text`/`Code` events concatenated, since a heading
  can contain inline code spans but slicing risks catching the `#` marker
  — headings are short enough that reconstructing from inner text is
  simpler and safe here, unlike paragraphs).
- `Paragraph` → `ContentBlock::Text` (body = source slice of the
  paragraph's range, trimmed).
- `CodeBlock(CodeBlockKind::Fenced(info))` → `ContentBlock::Code` unless
  `info.as_ref() == "branch"` (see §5); language = `info` (empty string
  becomes `None`); source = inner `Text` events concatenated (the code
  body, not sliced, since slicing would include the fence delimiters).
- `List(start)` → `ContentBlock::List`; `ordered = start.is_some()`. Each
  `Item` becomes one string: source-sliced and trimmed. Nesting is detected
  by tracking a list-depth counter — entering `List` while depth ≥ 1
  (already inside an `Item`) trips FR-012's diagnostic instead of emitting
  a block.
- `Image { dest_url, title, .. }` → `ContentBlock::Image`; `src =
  dest_url`, `alt` from inner `Text` events, `caption` from `title` when
  non-empty.
- `Event::Rule` → `ContentBlock::Divider`.

**Rationale**: This is a direct, mechanical mapping from `pulldown-cmark`'s
existing tag vocabulary to the protocol's existing `ContentBlock` variants
— no new intermediate representation needed beyond a small accumulator per
section.

## 4. Frontmatter

**Decision**: Detect frontmatter as a `---`-delimited block at byte offset
0 of the file (first line is exactly `---`, scan forward to the next line
that is exactly `---`). Parse the interior as flat `key: value` lines by
hand (split on the first `:`, trim both sides) — no YAML crate.

**Rationale**: Deck metadata (`title`/`author`/`date`/`description`/
`fireside-version`) is flat strings only; a general YAML parser is
unjustified dependency weight for five scalar fields (ADR-006 already
rejected pulling in a YAML crate for this reason). `pulldown-cmark` does
not interpret frontmatter itself — the block is skipped by finding its byte
range first and starting the Markdown parse after it, so the `---` closing
delimiter is never mistaken for a `Rule` in the body.

**Alternatives considered**: `pulldown-cmark-frontmatter` (exists on
crates.io) — still requires a YAML value parser for the interior, and for
five flat fields a ~15-line hand-parser is less risk than a second new
dependency.

## 5. Branch declaration syntax and parsing

**Decision**: A fenced block with info string exactly `branch` (the
reserved tag, per ADR-006 and spec Edge Cases) is parsed as its own
mini-grammar, line by line, from its raw text body (the same inner-`Text`
concatenation used for ordinary code blocks):

- The first line is the prompt **if** it does not start with `-` (a list
  marker); otherwise there is no prompt.
- Every subsequent non-blank line must match
  `- [<label>](#<target>)` optionally followed by `` `<key>` `` — parsed
  by hand (find `[`...`]`, then `(#`...`)`, then an optional trailing
  `` `...` ``) rather than a regex crate, since the grammar is fixed and
  small. A line that doesn't match this shape is a parse error naming the
  line (folds into FR-018/FR-019's "reject with a location" requirement,
  generalized to any malformed branch-fence line, not just unresolved
  targets).
- `target`s are resolved against the slug-to-node-id map built while
  walking `##` headings (§2) — this requires two passes: first collect all
  node ids (walk headings only), then a second full pass to build content
  and resolve branch targets, so a branch fence can reference a node that
  appears later in the document (forward references must work — a branch
  near the top choosing between two later sections is a completely normal
  document shape).
- A branch fence MUST be the last event in its section (FR-019): tracked
  by a `branch_seen` flag per section: any content-producing event after
  it is the "content after a branch declaration" error.

**Rationale**: Reuses the same source-slicing/event-walking machinery as
ordinary code blocks; the two-pass approach (ids first, then content) is
the simplest way to support forward references without a lazy/deferred
resolution scheme.

## 6. Node id slugification

**Decision**: Extract the existing slug logic already in
`crates/fireside-cli/src/main.rs::new_deck` (lowercase, map non-alphanumeric
to `-`, split on `-` and rejoin filtering empty segments) into a shared
`fn slugify(text: &str) -> String` used by both `new_deck`'s filename slug
and the importer's heading-to-node-id slug. On collision, append `-2`,
`-3`, ... (first collision gets `-2`, matching the common "second, third"
intuition rather than `-1`/`-0` ambiguity).

**Rationale**: `new_deck` already solves "turn arbitrary text into a safe
identifier" (FR-005 needs the identical transform) — reusing it removes
duplication rather than inventing a second slugifier with potentially
different edge-case behavior.

## 7. Validation before write

**Decision**: After building the `Graph` in memory, call the existing
`fireside_engine::validate` (already used by `validate_file`/`present`) and
refuse to write if any diagnostic is `Severity::Error`, printing the
diagnostics via the same `diagnostics_report` helper `validate`/`present`
already use.

**Rationale**: FR-021 asks for exactly the validation `validate` already
performs — reusing `diagnostics_report` keeps "what invalid looks like"
worded identically everywhere in the CLI, and means a v1-import-generated
deck is validated by the same code path a hand-written one is, with no
separate "import-only" validation rules to maintain.
