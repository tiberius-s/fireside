---
title: 'Appendix A — Design System'
description: 'Non-normative token guidance, terminal adaptation, and iTerm2 intent mapping for Fireside engines.'
---

:::note
This appendix is non-normative. It provides implementation guidance for
consistent rendering across terminal environments.
:::

## Token Model

Fireside design guidance is token-first: renderers map semantic tokens to
terminal styles, rather than hard-coding colors into components.

### Core Color Tokens

| Token               | Intent                                   |
| ------------------- | ---------------------------------------- |
| `surface-primary`   | Base presentation background.            |
| `surface-secondary` | Secondary panels and callout containers. |
| `text-primary`      | Primary readable content.                |
| `text-secondary`    | Supporting labels and metadata.          |
| `text-muted`        | De-emphasized helper text.               |
| `accent-primary`    | Active focus and key highlights.         |
| `accent-secondary`  | Secondary emphasis and links.            |
| `border-default`    | Default panel and separator borders.     |

### Spacing Tokens

Use cell-based spacing in terminal renderers.

| Token     | Suggested Cells |
| --------- | --------------- |
| `space-1` | 1               |
| `space-2` | 2               |
| `space-3` | 3               |
| `space-4` | 4               |
| `space-6` | 6               |
| `space-8` | 8               |

## iTerm2 Intent Mapping

Engines that import iTerm2 schemes SHOULD map intent, not just index.

| iTerm2 Slot                 | Fireside Token      | Render Intent                      |
| --------------------------- | ------------------- | ---------------------------------- |
| `Background Color`          | `surface-primary`   | Global canvas background.          |
| `Foreground Color`          | `text-primary`      | Main text color.                   |
| `Selection Color`           | `surface-secondary` | Focused panel background.          |
| `Cursor Color`              | `accent-primary`    | Active pointer/focus affordance.   |
| `Ansi 8` / bright neutral   | `text-muted`        | Secondary metadata and hints.      |
| `Ansi 4` / link-like accent | `accent-secondary`  | Link or secondary callout accents. |

When a token cannot be mapped directly, engines SHOULD use nearest-contrast
fallbacks that preserve readability before visual fidelity.

## Theme Source Guidance

The protocol does not mandate a theme file format or folder structure.
Engines MAY accept JSON, CLI flags, or platform-native settings,
as long as token intent remains consistent.

## Accessibility Guidance

- Target at least WCAG AA contrast intent where terminal capabilities allow.
- Never rely on color alone for branch selection state.
- Provide redundant cues (`▸`, bold, underline, border changes).
- Support monochrome fallback for low-color terminals.

## Renderer Contract

A renderer SHOULD separate concerns:

1. Token resolution (`theme`/defaults/runtime context).
2. Layout resolution (from `layout` value).
3. Component drawing (block-by-block render).

Keeping these stages separate makes adaptation to different terminal palettes
and accessibility modes predictable.
