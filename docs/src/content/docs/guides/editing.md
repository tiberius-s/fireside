---
title: 'Editing a Deck'
description: 'Build and restructure a deck with the mouse — every key the full-screen editor responds to, forms, drafts, and the save/undo model.'
---

`fireside edit <file>` opens a full-screen, mouse-first authoring studio.
Every slide is a stack of clickable blocks — heading, text, code, list,
image, divider, container, ascii-art — never raw JSON or graph
vocabulary. The canvas renders through the exact same code path
`fireside <file>` presents with, so what you see while editing is always
what an audience would see. Every mouse action also has a keyboard
equivalent (`?` lists them all), so the editor is just as usable without a
mouse.

```bash
fireside edit my-talk.fireside.json
```

If the file doesn't exist yet (and doesn't end in `.md`/`.markdown`),
`edit` offers to create it using the same templates as `fireside new`. A
file that fails to parse is refused — fix it first with
`fireside validate <file>`.

![Selecting a slide and a block, editing it through its form, trying the change with Present, and saving](../../../assets/editing.gif)

## The studio's layout

| Region | Shows |
| ------- | ------ |
| Toolbar (top) | The deck's title (click to rename), a dirty dot (`●`) when there are unsaved changes, and the `[ + Slide ]` `[ ▶ Present ]` `[ Save ]` `[ ↶ Undo ]` `[ ? ]` chips. |
| Outline (left) | Every slide in presentation order, a marker for choices (`⑂`) and endings (`■`), and a divider before any slide not yet reachable from the start. |
| Canvas (center) | The selected slide, rendered exactly as the presenter would show it. |
| Status line | `✓ ready to present` or `✗ won't present yet: N problems` — click it to jump straight to the slide a problem is about. |
| Hint line | The selected block or slide's actions, or a rotating first-run tip when nothing is selected. |

## Selecting and editing

Click a slide in the outline or a block on the canvas to select it — or
use `[`/`]` to move between slides and `Tab`/`Shift+Tab` to move between
a slide's blocks without a mouse. A selected block shows `[ ✎ Edit ]`,
`[ + Add below ]`, `[ Reveal ]`, and `[ Delete ]`; `Enter` opens the
selected block's form directly. Each block kind gets its own form —
text fields for headings/text, a language picker plus source for code, one
list item per line, path/description for pictures (with a
`[ Convert to text art ]` shortcut), a paste area plus
`[ Generate from a phrase… ]` for text art, and a layout picker for
columns/box/stack containers. `Ctrl+S` (or `[ Done ]`) commits a form;
`Esc` (or `[ Cancel ]`) discards it.

## Adding, deleting, and reordering blocks

The gap between any two blocks (and the top of an empty slide) is an
insertion point — click it, or a block's `[ + Add below ]` chip, to open
an add-block palette of all eight kinds; picking one inserts a placeholder
and opens its form immediately. `[ Delete ]` removes a block with a
non-blocking "Deleted — Undo" toast. Press and drag any block to reorder
it within its slide — a dimmed ghost and an insertion line track where it
will land, the canvas auto-scrolls near its edges, and `Esc` cancels the
drag and returns the block to where it was.

## Restructuring the deck

`[ + Slide ]` (or the outline's `+ new slide` row, or `n`) asks for a
title and adds a new slide. A selected slide's hint-line chips offer
`[ Duplicate ]`, `[ Delete ]`, and `[ Turn into a choice ]` (`c`) —
turning a slide into a choice adds a prompt and answer rows, each wired to
another slide through the same picker the "Goes to" strip's `[ change ]`
chip (or `g`) uses for an ordinary slide's next slide. `[ Reveal ]` (or
`r`) cycles a block's incremental-reveal step, with a live `[ ▷ preview ]`
to check what stages in when. Drag a slide within the outline to reorder
it; dragging one that's only reachable through a branch answer is refused
with an explanation and a link straight to the branch to fix it there
instead. Click the toolbar's title, or a slide's `[ Notes ]` chip, to
rename the deck or edit a slide's speaker notes.

## Trying it, saving, and undo

`[ ▶ Present ]` (or `p`) runs the real presenter in place, starting from
the selected slide — press `q` to come straight back to the editor.
`Ctrl+S` (or `[ Save ]`) writes the deck file; `[ ↶ Undo ]`/`u` and `U`
step backward and forward through every change this session, up to 100
steps.

## Crash safety

The editor autosaves your in-progress work to a separate draft file as you
go — not the deck file itself. If it didn't get a chance to save cleanly
last time, reopening the same deck offers `[ Restore draft ] [ Open saved
file ]` with both timestamps shown, so a crash or a force-quit never loses
work. Quitting with unsaved changes prompts `[ Save ] [ Discard ] [ Keep
editing ]` rather than exiting silently.

## Every key

| Key | Effect |
| --- | ------- |
| click / `Tab` | Select a slide or block |
| `[` / `]` | Select the previous / next slide |
| `Enter` | Edit the selected block |
| `n` | New slide · `c` turn into/back a choice |
| `a` | Add an answer · `g` change where a slide goes |
| `r` | Cycle the selected block's reveal step |
| `1`–`9`, `n`, `e` | In a picker: pick a row, a new slide, or an ending |
| `Ctrl+S` | Save · `u`/`U` undo/redo |
| `p` | Present from the selected slide |
| `↑`/`↓`, wheel | Scroll the canvas or the outline |
| `Esc` | Deselect |
| `q` | Quit |
| `?` | This screen |

## Where to go next

| If you want to...                                    | Go to                                     |
| ------------------------------------------------------- | ---------------------------------------------- |
| Present the deck you just built                         | [Presenting a deck](/guides/presenting/)      |
| Start from a Markdown outline instead                    | [Authoring a Deck in Markdown](/guides/authoring-markdown/) |
| Look up a `fireside edit` flag or exit code               | [CLI Reference](/reference/cli/)              |
