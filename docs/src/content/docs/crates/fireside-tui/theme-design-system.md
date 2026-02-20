---
title: 'Theme & Design System'
description: 'How Fireside TUI manages color, layout spacing, breakpoints, node templates, and iTerm2 color scheme import — from raw terminal colors to WCAG-checked design tokens.'
---

The design system in `fireside-tui` is a layered stack that translates an
abstract theme configuration into every color and layout decision made at
render time. The stack flows from user-facing configuration (`ThemeFile` /
`.itermcolors`) through semantic roles (`DesignTokens`) into responsive
layout helpers (`Breakpoint`, `Spacing`, `NodeTemplate`).

```text
User config / CLI flag
  └─► Theme resolution order (CLI > doc meta > user config > defaults)
        └─► Theme struct (concrete Color values + syntect theme name)
              └─► DesignTokens::from_theme()  (semantic roles)
                    ├─► renderer leaf functions (fg/bg per element)
                    └─► NodeTemplate::compute_areas()  (responsive layout)
                          └─► Breakpoint + Spacing constants
```

## `Theme` — the concrete color struct

`Theme` is the lowest-level runtime color carrier. It holds one `ratatui::style::Color`
field per UI role and one `syntax_theme: String` naming the syntect theme for
code blocks.

```rust
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub heading_h1: Color,
    pub heading_h2: Color,
    pub heading_h3: Color,
    pub code_background: Color,
    pub code_foreground: Color,
    pub code_border: Color,
    pub block_quote: Color,
    pub footer: Color,
    pub syntax_theme: String,
}
```

`Theme::default()` uses named ratatui colors (`Color::Cyan`, `Color::Reset`,
etc.) for portability across 16-color and 256-color terminals. The rich
One Dark RGB defaults live in `DesignTokens::default()` instead — they are
used when the full design token path is active.

### `ThemeFile` — the JSON overlay

`ThemeFile` is the serde target for user-authored JSON theme files. Every field
is `Option<String>` annotated with `#[serde(default)]`. String values are parsed
into `Color` via `parse_color()`:

- Named colors: `"red"`, `"darkgray"`, `"lightcyan"` (and snake_case variants)
- Hex: `"#ff0000"` (6-digit only)
- `"reset"` or `""` for `Color::Reset` (terminal default)

`ThemeFile::apply_to(base: &Theme) -> Theme` merges only the fields that are
`Some`, leaving unspecified fields at their baseline values. This means a
minimal theme file can override just `heading_h1` with only one line of JSON
and inherit everything else.

### Theme resolution order

`App::resolve_theme()` checks sources from highest to lowest priority and
returns the first non-trivial result:

1. `--theme <name|path>` CLI flag (resolved against `~/.config/fireside/themes/`)
2. Document `meta.theme` field in the `.json` graph file
3. `~/.config/fireside/config.json` `theme` key
4. `Theme::default()`

Hot-reload (triggered by `Action::ReloadTheme` or filesystem watch) re-runs
this resolution and then calls `DesignTokens::from_theme` on the new result.
The render immediately reflects the new palette on the next frame.

## `DesignTokens` — the semantic color layer

`DesignTokens` expands the 11 concrete fields of `Theme` into a richer set of
35+ semantic roles organized into four groups.

### Base palette (7 roles)

| Token        | Default (One Dark)   | Semantic use                             |
| ------------ | -------------------- | ---------------------------------------- |
| `background` | `Color::Reset`       | Terminal base background                 |
| `surface`    | `#282c34`            | Code blocks, panels, elevated cards      |
| `primary`    | `#61afef` (blue)     | Headings, active borders, selected items |
| `accent`     | `#c678dd` (purple)   | Links, interactive elements              |
| `muted`      | `#5c6370` (dim gray) | Borders, separators, dimmed text         |
| `error`      | `#e06c75` (red)      | Errors, warnings, destructive actions    |
| `success`    | `#98c379` (green)    | Confirmations, positive indicators       |

### On-colors (3 roles)

Text that appears on a colored background needs its own token to guarantee
readability. The on-color tokens are used wherever a colored surface needs
legible text overlaid on it.

| Token           | Use                                                             |
| --------------- | --------------------------------------------------------------- |
| `on_background` | Text on the base background                                     |
| `on_surface`    | Text inside code blocks and panels                              |
| `on_primary`    | Label text on primary-colored elements (e.g., active selection) |

### Typography tokens (8 roles)

| Token        | Use                                                   |
| ------------ | ----------------------------------------------------- |
| `heading_h1` | H1 color (`═` underrule)                              |
| `heading_h2` | H2 color (`─` underrule)                              |
| `heading_h3` | H3–H6 color                                           |
| `body`       | Body prose text                                       |
| `code_fg`    | Code base foreground (overridden per-span by syntect) |
| `code_bg`    | Code block background                                 |
| `quote`      | blockquote border and text                            |

### Chrome tokens (5 roles)

| Token             | Use                                   |
| ----------------- | ------------------------------------- |
| `footer`          | Progress bar and status line          |
| `border_active`   | Focused panel border                  |
| `border_inactive` | Unfocused panel border, divider rules |
| `toolbar_bg`      | Editor toolbar background             |
| `toolbar_fg`      | Editor toolbar text                   |

### `syntax_theme: String`

The name of the syntect theme to use for code blocks. This is passed through
from `Theme` and consumed by `render/code.rs` when calling `highlight_code`.
The two-face library provides an extended theme set; the default is
`"base16-ocean.dark"`.

### Round-trip path

`DesignTokens::to_theme()` and `DesignTokens::from_theme()` form a round-trip.
Some token fields (e.g., `accent`, `muted`, `on_primary`) have no direct
counterpart in the compact `Theme` struct and are filled in from
`DesignTokens::default()` during the `from_theme` direction. The test
`theme_roundtrip` in `tokens.rs` verifies that heading and code tokens survive
the round-trip without loss.

## Accessibility: WCAG contrast helpers

`tokens.rs` provides two public functions for contrast checking:

```rust
pub fn contrast_ratio(c1: Color, c2: Color) -> f64
pub fn meets_contrast_aa(fg: Color, bg: Color) -> bool  // threshold: 4.5:1
```

Both use the WCAG 2.1 relative luminance formula with correct gamma linearization:

$$L = 0.2126 \cdot R_{\text{lin}} + 0.7152 \cdot G_{\text{lin}} + 0.0722 \cdot B_{\text{lin}}$$

Non-RGB `Color` variants (named colors, `Reset`) return `1.0` from
`relative_luminance` because their actual pixel values are terminal-defined and
unknowable. The contrast test `default_tokens_body_on_background_contrast`
asserts that the One Dark defaults (`body` on `surface`) achieve at least AA.

Custom themes can validate their palettes with `cargo test` if they wire up a
similar assertion.

## `Breakpoint` — responsive layout

`Breakpoint::from_size(width, height)` maps a terminal `Rect` to one of three
responsive tiers:

| Breakpoint | Terminal dimensions         |
| ---------- | --------------------------- |
| `Compact`  | ≤ 80 columns or ≤ 24 rows   |
| `Standard` | 81–120 × 25–40              |
| `Wide`     | > 120 columns and > 40 rows |

The breakpoint is re-evaluated at every frame draw inside `App::view`. There is
no stored breakpoint state; it is derived live from the actual terminal `Rect`.

Two methods on `Breakpoint` drive responsive decisions:

| Method                | Compact | Standard | Wide    |
| --------------------- | ------- | -------- | ------- |
| `content_width_pct()` | 96%     | 85%      | 75%     |
| `h_padding()`         | 1 cell  | 2 cells  | 4 cells |

Tighter centering at wide widths prevents text from stretching across a large
monitor — a common problem for terminal presentations on 4K displays.

## `Spacing` — the scale

`Spacing` is a unit struct with only `const` values. All margins and padding
in the codebase must use these constants rather than literal integers:

| Constant      | Value   |
| ------------- | ------- |
| `Spacing::XS` | 1 cell  |
| `Spacing::SM` | 2 cells |
| `Spacing::MD` | 3 cells |
| `Spacing::LG` | 4 cells |
| `Spacing::XL` | 6 cells |

Using a scale rather than ad-hoc numbers keeps spacing harmonious and makes
global density changes (e.g., reducing all padding for compact mode) a
single-site edit.

## `NodeTemplate` — layout archetypes

`NodeTemplate` maps a protocol `Layout` enum variant to a named presentation
archetype. The mapping is many-to-one: multiple `Layout` values that share a
visual intent map to the same template.

| Template     | Maps from `Layout`                                                    |
| ------------ | --------------------------------------------------------------------- |
| `Title`      | `Layout::Title`                                                       |
| `TwoColumn`  | `Layout::SplitHorizontal`                                             |
| `CodeFocus`  | `Layout::CodeFocus`, `Layout::Fullscreen`                             |
| `Quote`      | `Layout::Center`                                                      |
| `BulletList` | `Default`, `Top`, `SplitVertical`, `AlignLeft`, `AlignRight`, `Blank` |

`TitleSubtitle`, `ImageCaption`, and `SpeakerNotes` are template-only archetypes
reachable via `NodeTemplate::from_name`, not directly from a `Layout` variant.
They are intended for use in the editor's template chooser.

### `TemplateAreas` — computed `Rect` output

`NodeTemplate::compute_areas(area, bp) -> TemplateAreas` returns:

```rust
pub struct TemplateAreas {
    pub main: Rect,
    pub secondary: Option<Rect>,  // right column, caption area, notes panel
    pub footer: Rect,
}
```

Every template calls `split_footer(area)` first to carve off a 1-row footer:

```rust
fn split_footer(area: Rect) -> (Rect, Rect) {
    Layout::default()
        .direction(Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area)
}
```

Area calculations per template:

| Template       | `main` area                             | `secondary` area   |
| -------------- | --------------------------------------- | ------------------ |
| `Title`        | `center_in(content, 70%, 50%)`          | None               |
| `TwoColumn`    | Left half with gutter                   | Right half         |
| `CodeFocus`    | Minimal horizontal padding, full height | None               |
| `Quote`        | Narrow center column                    | None               |
| `SpeakerNotes` | ~70% left (content)                     | ~30% right (notes) |

`center_in(area, width_pct, height_pct)` positions a rect using three-constraint
splits in both axes (margin | content | margin), which is the idiomatic ratatui
pattern for centering without absolute pixel math.

## iTerm2 color scheme import (`design/iterm2.rs`)

Fireside can import any of the thousands of color schemes from
[iterm2colorschemes.com](https://iterm2colorschemes.com/) directly into its
design token system.

### Parser

`.itermcolors` files are XML plists. Each color is a `<dict>` with three
float keys (`Red Component`, `Green Component`, `Blue Component`) in 0.0–1.0
range. The parser uses the `plist` crate to deserialize to a
`HashMap<String, plist::Value>`, then walks the expected keys.

Float components are converted to `u8` with `(value * 255.0).round() as u8`.

A **1 MiB file-size guard** is applied before parsing:

```rust
const MAX_ITERM2_FILE_SIZE_BYTES: u64 = 1_048_576;
```

Files larger than this return `Iterm2Error::FileTooLarge` immediately, avoiding
unbounded allocation on malformed or crafted inputs.

### Mapping

Each relevant iTerm2 key maps to one or more design tokens:

| iTerm2 key                    | Design tokens populated    |
| ----------------------------- | -------------------------- |
| `Background Color`            | `background`               |
| `Foreground Color`            | `on_background`, `body`    |
| `Bold Color`                  | `heading_h1`, `primary`    |
| `Selection Color`             | `surface`, `code_bg`       |
| `Cursor Color`                | `border_active`            |
| `Ansi 1 Color` (red)          | `error`                    |
| `Ansi 2 Color` (green)        | `success`, `heading_h2`    |
| `Ansi 3 Color` (yellow)       | `heading_h3`               |
| `Ansi 4 Color` (blue)         | `accent`                   |
| `Ansi 8 Color` (bright black) | `muted`, `border_inactive` |

Tokens not covered by any iTerm2 mapping fall back to `DesignTokens::default()`
values, ensuring the resulting token set is always complete and usable.

### CLI integration

```sh
fireside theme import path/to/MyScheme.itermcolors
fireside theme import path/to/MyScheme.itermcolors --name my-scheme
```

The import command parses the plist, materializes the full `DesignTokens` via
the mapping, converts to `ThemeFile` format (JSON), and writes it to
`~/.config/fireside/themes/<name>.json`. From that point the theme is available
by name to all theme resolution paths.
