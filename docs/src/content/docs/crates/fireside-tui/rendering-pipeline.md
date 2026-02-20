---
title: 'Rendering Pipeline'
description: 'How fireside-tui converts ContentBlock values into styled ratatui Lines — syntax highlighting, image rendering, container layout, and the path security model.'
---

The rendering pipeline in `fireside-tui` is the sequence of transformations
that converts a `Vec<ContentBlock>` (the protocol model) into a flat
`Vec<Line<'_>>` (ratatui primitives ready for drawing). It is implemented
across three modules: `render/markdown.rs`, `render/code.rs`, and
`render/layout.rs`.

## Pipeline overview

```text
App::view()
  └─► ui/presenter.rs: render_presenter(app, frame)
        └─► render/layout.rs: compute_areas(frame.size(), layout)
              └─► render/markdown.rs: render_node_content(blocks, theme, width)
                    ├─► render_block()  ×N  (one per ContentBlock)
                    │     ├─► render_heading()
                    │     ├─► render_text()       ← textwrap
                    │     ├─► render_code()        ← render/code.rs
                    │     ├─► render_list()
                    │     ├─► render_image_placeholder()  ← image crate
                    │     ├─► render_divider()
                    │     ├─► render_container()   ← render/layout.rs
                    │     └─► render_extension()
                    └─► Vec<Line<'_>>  →  ratatui Paragraph widget
```

All rendering functions accept a **`width: u16`** parameter derived from the
computed `Rect` for the content area. Width is used by `textwrap`, line-rule
generation, and image scaling; it must be passed down rather than read from
terminal state to keep rendering pure and testable without a live terminal.

## `DesignTokens` as the color interface

No rendering function accepts a `Theme` directly at the leaf level. Instead,
`render_node_content` converts `&Theme` to `DesignTokens` once at the top and
threads `&DesignTokens` through all recursive calls:

```rust
pub fn render_node_content<'a>(
    blocks: &'a [ContentBlock],
    theme: &Theme,
    width: u16,
) -> Vec<Line<'a>> {
    let tokens = DesignTokens::from_theme(theme);
    render_node_content_with_tokens(blocks, &tokens, width, None)
}
```

This conversion is cheap (all fields are `Color` values, which are `Copy`) and
means every leaf renderer sees semantic role names (`tokens.heading_h1`,
`tokens.code_bg`, `tokens.muted`) rather than raw color fields. Theme changes
require only updating `DesignTokens::from_theme`; no leaf renderer needs to
change.

## Block separation

`render_node_content_with_tokens` inserts one blank `Line::default()` between
every pair of adjacent blocks:

```rust
for (i, block) in blocks.iter().enumerate() {
    if i > 0 { lines.push(Line::default()); }
    lines.extend(render_block_with_tokens(block, tokens, width, base_dir));
}
```

This produces vertical rhythm at the block level without requiring individual
renderers to pad their own output.

## Block renderers

### Heading

Headings use `BOLD` modifier and level-specific color from `DesignTokens`.
H1 and H2 receive a decorative underline rule using Unicode box-drawing
characters (`═` for H1, `─` for H2). The rule width is derived from `width`
minus the heading prefix indent:

```rust
let rule_width = width.saturating_sub(prefix.len() as u16).max(10) as usize;
lines.push(Line::from(Span::styled(
    dash.to_string().repeat(rule_width),
    Style::default().fg(tokens.border_inactive),
)));
```

### Text

Body text is wrapped using `textwrap::wrap(text, width as usize)` before being
styled. `textwrap` handles Unicode correctly and respects word boundaries.
Wrapping is computed at render time from the current `width`; no pre-computed
wrap state is cached.

### Code

Code rendering has two paths depending on whether `highlight_lines` or
`show_line_numbers` are set:

**Plain syntax-highlighted path** (no line directives): delegates to
`highlight_code(source, lang, syntax_theme)` which uses syntect. If syntect
recognizes the language, it returns styled `Vec<Line<'_>>` with per-span RGB
colors. The result is wrapped in `add_code_chrome` which adds a top border
and a language label badge.

**Manual line-by-line path** (with line directives): iterates `source.lines()`
and constructs spans manually. Highlighted lines receive `BOLD` and a `▎`
gutter marker in `tokens.success` color. Line numbers are rendered in `tokens.muted`
with a `│` separator.

The two paths are mutually exclusive because syntect works on the full source
string and returns `Line` values per-line, which cannot be merged with
per-line metadata without an additional pass.

### Syntax highlighting (`render/code.rs`)

`highlight_code` is the only function in this module. It uses two `LazyLock`
statics initialized at first call:

```rust
static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(two_face::syntax::extra_newlines);
static THEME_SET:  LazyLock<ThemeSet>  = LazyLock::new(|| two_face::theme::extra().into());
```

`two_face` provides an extended syntax and theme registry beyond syntect's
defaults. `extra_newlines` adds a variant that handles lines without trailing
newlines correctly (relevant for the last line of a code block).

The fallback chain:

```rust
let syntax = SYNTAX_SET
    .find_syntax_by_token(language)          // try language token (e.g. "rs")
    .or_else(|| SYNTAX_SET.find_syntax_by_extension(language))?;  // try extension

let theme = THEME_SET.themes.get(theme_name)
    .or_else(|| THEME_SET.themes.values().next())?;  // any theme as last resort
```

Both `?` operators propagate `None`, causing `highlight_code` to return
`None`. The caller (`render_code`) treats `None` as "no syntax available" and
falls through to the plain code rendering path.

RGB colors from syntect's `Style` are mapped to ratatui `Color::Rgb(r, g, b)`:

```rust
let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
Span::styled(text.to_string(), Style::default().fg(fg))
```

Background per-span is not set; the block background comes from the
`code_background` theme token applied at the Paragraph widget level by the
presenter.

### List

Lists render recursively with a `depth` parameter controlling indentation.
Top-level ordered items use `1.`, `2.`, … prefixes; unordered items use `•`.
Nested items indent by 3 spaces per level. Items call `render_text` internally,
so list item text is wrapped at available width minus the prefix and indent.

### Image

Images call `local_image_path` to resolve the `src` to an absolute path, then
attempt to open and decode the file using `image::ImageReader`. On success,
a PPM-format pixel render is produced (sixel or block character rendering for
terminals that support it). On any failure, a styled placeholder is shown:

```text
[ image: alt text (path/to/image.png) ]
```

Failures are logged with `tracing::warn!` rather than returned as errors,
because a broken image should degrade gracefully rather than crash the
presentation.

#### Path security model

`local_image_path(src, base_dir)` enforces that image paths cannot escape the
document's base directory:

```rust
// Step 1: reject absolute paths outside the base dir immediately
// Step 2: canonicalize and verify the result starts with base_dir
if !resolved.starts_with(base_dir) {
    return Err(RenderError::PathTraversal(src.to_string()));
}
```

Paths containing `..` components are rejected before canonicalization. After
canonicalization, the resolved path is verified to remain within `base_dir`.
This prevents a malicious or accidentally crafted document from reading
arbitrary files via the image renderer.

Tests cover the three rejection cases explicitly:

- An absolute path outside `base_dir`
- A relative path with `..` traversal
- A path that resolves within `base_dir` (expected to succeed)

### Divider

A single `Line` containing `─` repeated to `width`. Uses `tokens.border_inactive`.

### Container

`render_container(layout, children, tokens, width, base_dir)` dispatches on
the `layout` string from the wire format. The primary layout handled is
`"split-horizontal"`, which divides `width` equally between two halves and
renders odd and even children respectively:

```rust
let half_width = width / 2;
// left column: children[0], children[2], …
// right column: children[1], children[3], …
```

The resulting two `Vec<Line<'_>>` are interleaved line-by-line (zip-padded to
the longer column) with a `│` separator in the center. This is the mechanism
that implements `Layout::SplitHorizontal` for content-level two-column layouts.

Unknown layout strings fall through to rendering children sequentially, which
provides a reasonable degraded experience for layouts not yet implemented.

### Extension

`render_extension` checks the `extension_type` for known built-in extensions:

- `"fireside.mermaid"` — extracts the `"diagram"` field from `payload`, wraps
  it in a fenced code block for preview display, and truncates payloads larger
  than 2KB with a warning message.

For any unrecognized `extension_type`, the `fallback` block is rendered if
present, otherwise a placeholder `[ extension: <type> ]` line is emitted.

## Layout computation (`render/layout.rs`)

`compute_areas(area: Rect, layout: Layout) → NodeAreas` is called by the
presenter before any content rendering. It splits the frame into:

```text
┌───────────────────────────┐
│  content area             │  ← Constraint::Min(1)
│  (layout-specific padding)│
└───────────────────────────┘
│  footer (1 row)           │  ← Constraint::Length(1)
└───────────────────────────┘
```

The `apply_layout` function then applies padding or centering to the content
area based on the `Layout` variant and the current `Breakpoint`:

| Layout                    | Content area treatment                                           |
| ------------------------- | ---------------------------------------------------------------- |
| `Default`, `Top`          | Responsive horizontal and vertical padding                       |
| `Center`                  | Horizontal centering at `content_width_pct()` of available width |
| `Title`                   | Centered at narrower width than `Center`                         |
| `CodeFocus`, `Fullscreen` | Minimal horizontal padding, no vertical padding                  |
| `Blank`                   | No padding — full area returned                                  |
| `SplitHorizontal`, etc.   | Standard padding, column splitting handled by `render_container` |

`Breakpoint` (Compact / Standard / Wide) is derived from terminal dimensions
at render time. This means layout automatically adapts as the user resizes the
terminal without any state tracking beyond the live `Rect`.
