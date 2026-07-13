# Data Model: ASCII art centering and clipping

No new types, no wire-format entities. This feature is a rendering
computation over the existing `ContentBlock::Code` variant
(`fireside-core`'s `language: Option<String>`, `source: String`,
`highlight_lines: Option<Vec<u32>>`, `show_line_numbers: Option<bool>` —
unchanged).

## Internal (non-public) concepts introduced in `blocks.rs`

| Concept | Description |
| --- | --- |
| ASCII-art classification | A boolean computed from `language`: true when `None`, `Some("text")`, or `Some("ascii")`; false otherwise. Not stored anywhere — computed fresh each render, matching the pure-render contract of `render_block`. |
| `box_width` | The natural or full-width sizing target for the code box's top rule, content rows, and bottom rule, computed once per `code()` call. For ASCII art: `(prefix_width + content_max).max(label_width).min(full_width)`. For everything else: `full_width` (unchanged from today). |
| centering pad | `(full_width - box_width) / 2`, applied as a uniform leading `Span::raw` on every line in the box when `box_width < full_width`. Zero (no-op) whenever classification is false or the content fills/exceeds `full_width`. |

No relationships beyond this — the computation is entirely local to one
function call, consistent with the module's existing design ("every block
renders to a flat `Vec<Line>` flow at a given width").
