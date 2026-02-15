---
title: 'Appendix B — Engine Guidelines'
description: 'Non-normative implementation guidance for TEA architecture, traversal invariants, and layout-template rendering.'
---

:::note
This appendix is non-normative. It documents proven engine patterns from the
reference implementation.
:::

## Core Runtime Guarantees

Recommended guarantees for robust engines:

1. Graph data is immutable after load and validation.
2. State mutation happens only in one update path.
3. History is maintained as a strict LIFO stack.
4. Rendering is deterministic for equivalent state.

## TEA-Oriented Flow

```text
Input Event -> Action -> Update(State) -> Render(View)
```

This flow keeps behavior testable and avoids hidden side effects.

## Layout Templates and ratatui Mapping

These mappings are practical defaults, not protocol requirements.

| Fireside `layout`  | ratatui Strategy                                         |
| ------------------ | -------------------------------------------------------- |
| `default`          | Vertical stack with standard margins.                    |
| `center`           | Single centered container with horizontal/vertical flex. |
| `split-horizontal` | Two-column `Layout::horizontal` split.                   |
| `split-vertical`   | Two-row `Layout::vertical` split.                        |
| `fullscreen`       | Single full-frame region, minimal chrome.                |
| `align-left`       | Primary content constrained left, right gutter empty.    |
| `align-right`      | Primary content constrained right, left gutter empty.    |
| `focus-code`       | Header + code panel + optional output/status panel.      |
| `agenda`           | Title row + list rail + detail pane.                     |
| `compare`          | Symmetric two-column compare with shared header.         |
| `image-left`       | Left media pane + right narrative pane.                  |
| `image-right`      | Left narrative pane + right media pane.                  |

## Container Rendering Guidance

For `container` blocks:

- Treat `children` as a local composition tree.
- Resolve container `layout` first, then render children in slots.
- Preserve child order unless the selected layout explicitly reflows.

## Extension Rendering Guidance

For `extension` blocks:

- Dispatch on `type` when supported.
- Render `fallback` when unsupported.
- Never discard unsupported extension content silently.

## Input and Error Strategy

- Map key events to semantic actions before state updates.
- Keep presenter-facing failures recoverable where possible.
- Favor placeholders over crashes for content-level issues.
