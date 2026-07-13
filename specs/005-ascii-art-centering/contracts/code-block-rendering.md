# Contract: `code()` rendering behavior

`fn code(language: Option<&str>, source: &str, highlight: &[u32], line_numbers: bool, width: u16, tokens: &Tokens) -> Vec<Line<'static>>`

Signature is UNCHANGED — this is a behavior contract, not an API change.

## Classification

```text
is_ascii_art = language is None
            or language == Some("text")
            or language == Some("ascii")
```

## Sizing contract

| Classification | Box width | Position |
| --- | --- | --- |
| ASCII art | `min(full_width, max(prefix_width + content_max_line_width, label_width))` | Centered via uniform leading pad on every line |
| Not ASCII art | `full_width` (unchanged from today) | Left-aligned, no pad (unchanged from today) |

Where:
- `full_width` is the `width` parameter as given.
- `prefix_width` is the existing row-prefix width: `num_width + 4` when
  `line_numbers` is true, else `2` (matches current behavior exactly, no
  change to this calculation).
- `content_max_line_width` is the maximum Unicode display width across
  `source.lines()`.
- `label_width` is the display width of `"─ {label} "` where
  `label = language.unwrap_or("code")`.

## Invariants (MUST hold for every input, including empty source)

1. Every returned `Line` in the box has the same total leading pad (0 when
   not ASCII art or when the box is full/over width).
2. The top rule and bottom rule always have equal width to each other and
   to `box_width` before padding.
3. No line's *content region* (post-prefix, post-pad) exceeds
   `box_width - prefix_width` columns; overflow is cut and marked with the
   existing ellipsis via `clip`/`clip_spans` — no new truncation logic.
4. For non-ASCII-art blocks, output is byte-for-byte/span-for-span
   identical to the current implementation for the same inputs (this is
   the regression contract — verified by the fact that no existing test
   changes).
5. Never panics for any `(language, source, highlight, line_numbers,
   width)` combination, including `width == 0` (already short-circuited
   by `render_block`'s existing `if width == 0 { return Vec::new(); }`
   guard before `code()` is ever called) and empty `source`.

## Composition with `container { layout: "center" }`

`center()` calls `render_block(child, inner_width, tokens)` then pads the
resulting flow by `(outer_width - unit_width) / 2` where `unit_width` is
the max rendered line width. Since this feature's pad is applied
uniformly to every line inside `code()`, `unit_width` for an ASCII-art
child equals `inner_width` (the box still reports as filling its given
width once its own internal pad is included) — `center()`'s additional
pad then centers that already-self-centered unit within the outer width.
The compose is additive, not conflicting: total effective indentation is
the sum of `code()`'s internal pad and `center()`'s external pad, and
because both are computed from `(available - content)/2`-style formulas,
the art ends up positioned correctly relative to the full outer width.
