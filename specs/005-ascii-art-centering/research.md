# Research: ASCII art centering and clipping

## Decision: classify by exact-match on `language`, not a fuzzy heuristic

**Decision**: A code block is ASCII art when `language` is `None`, or
exactly `Some("text")`, or exactly `Some("ascii")` — case-sensitive, no
trimming, no synonym list.

**Rationale**: `blocks.rs::code()` already treats language as an exact,
case-sensitive string elsewhere (passed straight to
`syntax::highlight(language, ...)`, which does its own exact matching
against known Syntect language names). A fuzzy classifier (case-insensitive,
trimming, `"plaintext"`/`"plain"` synonyms) would need its own design
discussion and testing surface disproportionate to this feature's scope,
and would risk misclassifying a real language the author intended
(unlikely today, but the exact-match rule keeps the boundary crisp for
whenever a `"typescript"` vs `"ts"`-style ambiguity comes up elsewhere).

**Alternatives considered**: Case-insensitive matching (rejected — no
existing precedent elsewhere in the renderer, adds surface for no
requested benefit). A synonym list (`"plain"`, `"plaintext"`, `"ansi"`)
(rejected — not requested, expands scope past "the plain-text/ASCII-art
case" the strategic plan named).

## Decision: box sizing computed from content, not measured after render

**Decision**: Compute `box_width` (the natural content width including row
prefix) via `source.lines().map(UnicodeWidthStr::width).max()` plus the
existing prefix-width calculation, BEFORE building any `Line`s — not by
building the full-width box first and then measuring the longest rendered
line.

**Rationale**: The top rule's `fill` (dash-repeat) calculation and each
content row's `avail` (clip width) calculation both need `box_width` as an
input, not an output — computing it upfront avoids a two-pass render or
retrofitting padding onto already-built lines in a way that could disagree
with how gutters/highlighting spans were sized.

**Alternatives considered**: Render at full width first, measure, then
reflow at the measured width (rejected — doubles the render work for no
benefit, and risks the reflowed pass producing different clip decisions
than the first pass since `avail` would change between passes).

## Decision: centering pad applied as a uniform line-prefix, after box construction

**Decision**: Once the box's lines are built at `box_width`, if
`box_width < full_width`, prepend an identical `Span::raw(" ".repeat(pad))`
to every line (top rule, every content row, bottom rule), where
`pad = (full_width - box_width) / 2`.

**Rationale**: This is exactly the pattern `image()` already uses (`lead`
in `blocks.rs::image`) and matches how `center()` centers a whole rendered
unit — reusing an established idiom rather than introducing a new one.
Applying the pad uniformly (same amount, every line) is what makes the
existing `centered_code_keeps_its_internal_alignment` container-level test
keep passing: every line shifts by the same amount, so relative alignment
between rows is untouched regardless of whether the shift happens inside
`code()` itself or from an enclosing `center()` container.

**Alternatives considered**: Right-pad instead of / in addition to
left-pad (rejected — not needed; `Line`s don't need trailing whitespace
since ratatui doesn't require uniform line width, and no existing block
in `blocks.rs` right-pads for this reason).

## Decision: no minimum-width floor beyond the label's own width

**Decision**: `box_width` is `(prefix_width + content_max).max(label_prefix.width()).min(full_width)` —
guarantees the box is at least wide enough to show its own top-rule label
(`"─ text ─..."`), with no additional arbitrary minimum.

**Rationale**: Handles the empty-content edge case (FR: "sizes to a sane
minimum... rather than collapsing to nothing or panicking") using a value
that's already meaningful (the label) rather than inventing a new magic
number.

**Alternatives considered**: A fixed minimum like 16 columns (rejected —
arbitrary, and the label-width floor already prevents collapse without
inventing an unexplained constant).
