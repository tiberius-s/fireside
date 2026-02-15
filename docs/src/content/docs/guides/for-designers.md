---
title: 'For Designers'
description: 'Designing Fireside experiences with semantic tokens, terminal-aware intent mapping, and accessible layout patterns.'
---

Fireside design is intent-first: define semantic tokens and let each engine map
those tokens to platform capabilities.

## Design in Fireside

Fireside does not mandate CSS, specific fonts, or a required theme file format.
Instead, documents and engines share semantic expectations:

- readable structure
- consistent emphasis
- predictable contrast
- graceful adaptation across terminals

## Token Essentials

Use these token intents as your base palette vocabulary:

| Token               | Intent                            |
| ------------------- | --------------------------------- |
| `surface-primary`   | Primary background canvas         |
| `surface-secondary` | Secondary panel surfaces          |
| `text-primary`      | Main readable text                |
| `text-secondary`    | Supporting labels and subtitles   |
| `text-muted`        | De-emphasized metadata            |
| `accent-primary`    | Focus, active, and key highlights |
| `accent-secondary`  | Secondary emphasis                |
| `border-default`    | Borders and separators            |

## iTerm2 Mapping Workflow

If your team starts from iTerm2 colors, map slots by intent:

1. `Background Color` → `surface-primary`
2. `Foreground Color` → `text-primary`
3. `Selection Color` → `surface-secondary`
4. `Cursor Color` → `accent-primary`

Then test legibility at 16-color and true-color terminal settings.

## Layout Patterns

The protocol includes layout hints you can design against:

- `default`, `center`, `fullscreen`
- `split-horizontal`, `split-vertical`
- `align-left`, `align-right`
- `focus-code`, `agenda`, `compare`
- `image-left`, `image-right`

Design recommendation: define token states for each layout family rather than
hard-coding page-specific colors.

## Accessibility Checklist

- Maintain AA-level contrast intent for primary text.
- Use non-color affordances for branch selection and focus.
- Validate in low-color environments.
- Ensure dense code views remain readable in `focus-code` layouts.

## Theme Source Flexibility

Engine implementations may load tokens from JSON, YAML, CLI flags, or
platform-native settings. As a designer, focus on semantic token intent and
contrast behavior, not storage syntax.
