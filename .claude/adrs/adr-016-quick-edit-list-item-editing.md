---
title: 'ADR-016: Quick-edit modal gains list item text editing'
status: 'accepted'
date: '2026-07-20'
deciders: ['@tiberius']
supersedes: 'ADR-005 (partially — the `list` line of its exclusion list only)'
---

# ADR-016: Quick-edit modal gains list item text editing

## Status

Accepted. Amends ADR-005's exclusion list; does not reopen the rest of that
ADR.

## Context

ADR-005 scoped the quick-edit modal to content-only edits of `heading` and
`text` blocks, and explicitly excluded `list` blocks as read-only:

> lists need item-level add/remove which is a structural edit... List and
> code content editing may be a future ADR.

In practice this meant a slide built from a heading, a bulleted list, and a
trailing text block — a completely ordinary slide shape, and the one the
bundled demo deck itself uses — had a hole in the middle: the presenter
could fix a typo in the heading or the closing sentence, but not in any of
the bullets between them, with no explanation in the UI for why. Reported
directly against the shipped modal (screenshot: `hello.json`'s "features"
node, list untouchable while heading/text above and below it work fine).

Revisiting the "structural edit" framing: `EditableField` already models an
edit target as a `Vec<String>` buffer plus a `(row, col)` cursor, with
`Enter` splitting the current row in two and `Backspace` at column 0 merging
a row into the previous one (`crates/fireside-tui/src/app.rs`,
`EditableField::newline`/`backspace`). A list's `items: Vec<String>` is
already exactly that shape — one buffer row per item. Treating a list field
this way means "add a bullet" is just pressing Enter mid-item, and "remove
a bullet" is just Backspace-merging it away, with no new editing
mechanics and no dedicated add/remove UI. What ADR-005 called a structural
edit requiring new machinery turned out to be the *same* content edit the
modal already knew how to do, once the block's items are the buffer.

## Decision

The quick-edit modal now includes one `EditableField` per `list` block on
the current node, labeled "List" or "Ordered list" depending on the
block's `ordered` flag. Its buffer is the list's `items` directly (not
joined into a single string the way heading/text bodies are); on save,
`items` is replaced wholesale with the buffer's rows
(`crates/fireside-tui/src/app.rs`: `collect_editable`'s new `List` arm,
`save_edit`'s new `List` arm; `crates/fireside-tui/src/render/overlays.rs`:
the label match). A list with zero items gets no field, matching how a
block with no editable text never gets one.

This still does not touch:

- **Item order independent of edits** — there is no "move item up/down";
  reordering happens by editing item text in place, same as ADR-005 already
  allowed for reordering sentences within a paragraph.
- **`code` block content** — still out of scope per ADR-005 (language-aware
  editing is a distinct problem).
- **`image`/`divider`/`container` structure**, and **node/traversal
  structural edits** (adding/removing/rewiring nodes or branch options) —
  still explicitly out of scope, unchanged from ADR-005.

## Consequences

### Positive

- Closes a hole a presenter would hit on essentially any real slide with a
  bullet list — no separate "why can't I edit this" explanation needed.
- Zero new editing primitives: `EditableField`'s existing multi-row buffer,
  cursor, and key handling are reused as-is. The only new code is wiring
  (`collect_editable`, `save_edit`, the modal's label), not new mechanics.
- Mouse click-to-position (added alongside word-wrap in the same modal)
  works on list fields for free, for the same reason.

### Negative or Trade-offs

- ADR-005's framing — "item add/remove is structural, needs its own ADR" —
  turned out to be wrong once the buffer model was reused rather than a
  literal insert/delete-item UI being built. Worth remembering when scoping
  future exclusions: check whether the existing edit primitive already
  covers the "structural" case before ruling it out.
- No explicit UI affordance teaches "Enter adds a bullet" — a presenter has
  to discover it, same as they already have to discover it for paragraph
  line breaks in a text field. Not a new gap, just an extended one.

### Neutral / Follow-up

- `code` block editing remains genuinely deferred — it needs a
  language-aware editor, which is a different shape of problem than reusing
  the row buffer.
- Node/traversal structural edits (ADR-005's other exclusions) are
  unaffected and still require a future ADR before any code is written
  against them.
