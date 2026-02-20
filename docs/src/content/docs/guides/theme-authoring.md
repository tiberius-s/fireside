---
title: 'Theme Authoring'
description: 'Create, import, and apply Fireside themes with predictable fallback behavior.'
---

Fireside themes let you control presentation color, code styling, and visual
contrast while keeping behavior deterministic across terminals.

## Theme File Shape

A theme file is JSON. Fields are optional; missing fields fall back to
`Theme::default()`.

```json
{
  "background": "#282a36",
  "foreground": "#f8f8f2",
  "heading_h1": "#bd93f9",
  "heading_h2": "#50fa7b",
  "heading_h3": "#f1fa8c",
  "code_background": "#44475a",
  "code_foreground": "#f8f8f2",
  "code_border": "#6272a4",
  "block_quote": "#6272a4",
  "footer": "#6272a4",
  "syntax_theme": "base16-ocean.dark"
}
```

## Fields and Defaults

The JSON parser maps these fields onto `Theme` values used in
`crates/fireside-tui/src/theme.rs`.

| Field             | Purpose                            | Default behavior    |
| ----------------- | ---------------------------------- | ------------------- |
| `background`      | Main canvas background             | `Color::Reset`      |
| `foreground`      | General text color                 | `Color::Reset`      |
| `heading_h1`      | Level-1 heading color              | `Color::Cyan`       |
| `heading_h2`      | Level-2 heading color              | `Color::Green`      |
| `heading_h3`      | Level-3 heading color              | `Color::Yellow`     |
| `code_background` | Code block background              | `Color::Black`      |
| `code_foreground` | Code block foreground              | `Color::White`      |
| `code_border`     | Code border/divider accents        | `Color::DarkGray`   |
| `block_quote`     | Quoted text tone                   | `Color::Blue`       |
| `footer`          | Footer/progress text               | `Color::DarkGray`   |
| `syntax_theme`    | Syntect theme for highlighted code | `base16-ocean.dark` |

## Supported Color Formats

Theme color fields accept:

- named colors recognized by Fireside (`red`, `dark-blue`, `cyan`, etc.)
- hex colors in `#RRGGBB`
- `reset` to delegate to terminal defaults

Examples:

```json
{
  "background": "reset",
  "foreground": "white",
  "heading_h1": "#7aa2f7"
}
```

If a color token is invalid, Fireside safely falls back to `Color::Reset`.

## Syntax Highlighting Theme (`syntax_theme`)

`syntax_theme` controls syntect code highlighting and is independent from your
UI palette fields.

Use one of the names exposed by the highlighter theme set, for example:

- `base16-ocean.dark`
- `base16-eighties.dark`
- `InspiredGitHub`
- `Solarized (dark)`

If `syntax_theme` is unknown, Fireside falls back to `base16-ocean.dark`.

## Import from iTerm2

You can convert `.itermcolors` directly into Fireside JSON:

```bash
fireside import-theme ~/Downloads/Dracula.itermcolors
fireside import-theme ~/Downloads/Dracula.itermcolors --name dracula
```

Imported themes are saved to `~/.config/fireside/themes/<name>.json`.

## Theme Resolution Order

When presenting, Fireside resolves theme precedence in this order:

1. CLI `--theme` flag
2. Document metadata theme field
3. User config (`~/.config/fireside/config.json`)
4. `Theme::default()`

This means a CLI flag always overrides document and user defaults.

## Practical Workflow

1. Start from a known base (`default` or imported iTerm2 theme).
2. Tune heading and body contrast first.
3. Tune code colors (`code_*` + `syntax_theme`) second.
4. Validate in a real session: `cargo run -- present docs/examples/hello.json --theme <name-or-path>`.
