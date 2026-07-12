---
title: 'ADR-005: Quick-edit modal scope (Stage C authoring path)'
status: 'accepted'
date: '2026-07-12'
deciders: ['@tiberius']
---

# ADR-005: Quick-edit modal scope (Stage C authoring path)

## Status

Accepted.

## Context

ADR-004 (2026-06-11, presenter-first rewrite) deleted the old node editor —
undo/redo, structural editing, the works — as a deliberate cut, not a
regression: "the editor may return as its own milestone after presenting is
excellent." The strategic plan
(`.claude/plans/2026-07-12-strategic-improvement-plan.md`, P0 Stage C) picks
this milestone up, but ADR-004 also set a rule that must be honored before
any editor code is written: anything touching the wire format needs a spec
change first. This ADR exists to draw the line precisely — what "editor
returns" means for Stage C, and what it permanently excludes — so that
implementation cannot creep back toward the deleted editor's scope.

Two constraints from the constitution (`.specify/memory/constitution.md`,
crate boundary table) bear directly on the design:

- `fireside-tui` is forbidden direct file I/O. The M3 author loop already
  established the pattern for this: the CLI's `Watcher` (in
  `fireside-cli/src/main.rs`) does file I/O and polls mtime+size; the TUI's
  `present_watching` takes a `ReloadSource` callback and does no I/O itself.
  A quick-edit modal must follow the same shape — the TUI owns the modal's
  UI and produces an edited value, but the actual disk write happens through
  a callback supplied by `fireside-cli`, symmetric with how reload already
  works.
- Content edits (block text) do not change the wire format — no new JSON
  shape, no new protocol field — so no `main.tsp` change and no spec-kit
  feature spec is required for this ADR's scope. If Stage C's design later
  wants to let the modal add a node or rewire a `traversal`, that is a
  structural edit and is explicitly out of scope here (see Decision).

## Decision

We will add a quick-edit modal to `fireside-tui` scoped to **content-only
edits of the current node's existing `text` and `heading` blocks**. The user
opens the modal on the current node, edits the block's string content
in-place, and on save the TUI emits the edited `Graph` (or a minimal patch)
through a write-back callback owned by `fireside-cli`, which serializes and
writes the file. The CLI's existing file watcher then reloads the file
exactly as it does for any other external edit — no special-case reload path
for self-writes.

The following are explicitly and permanently out of scope for this ADR (a
future ADR is required before any of these are built, not a task-level
decision):

- **Structural edits**: adding, removing, or reordering nodes; editing
  `traversal` (`next`, `branch-point`, or the empty/absent forms); adding or
  removing branch options.
- **Undo/redo** of any kind. A mistaken edit is corrected by editing again or
  reloading from disk (live-reload already gives a safety net — a broken or
  unwanted save can be discarded before the next intentional save).
- **Multi-node batch edits** — the modal only ever operates on the
  currently-displayed node.
- **Non-text block editing** — `image`, `code`, `divider`, `columns`/`stack`
  container structure, and `list` blocks are read-only in the modal. (List
  and code content editing may be a future ADR; they raise their own
  questions — code needs a language-aware editor, lists need item-level
  add/remove which is a structural edit.)
- **Any project/theme/font scaffolding** ADR-004 already deleted. This ADR
  does not revisit that decision.

On the write-back format question flagged in the strategic plan: `serde_json`
does not preserve the original key order or formatting of a hand-authored
file. We accept this. Saving through the quick-edit modal MAY reformat the
whole file to Fireside's canonical `serde_json` pretty-print output, not just
the edited node. This is documented behavior, not a bug — a presenter using
the modal at all has opted into letting the tool own formatting for that
file, the same way `cargo fmt` owns formatting once you run it.

## Consequences

### Positive

- Closes the most-felt gap in the authoring path (hand-editing JSON for
  every typo) without reopening any of the surface ADR-004 cut.
- Reuses the M3 live-reload round-trip exactly as built — no new
  architecture, no new crate-boundary exception beyond the existing
  CLI-owns-I/O pattern.
- The exclusion list gives future Stage C/D work (or a future editor ADR) a
  clear "you are now outside ADR-005" boundary, preventing scope creep back
  toward the deleted editor.

### Negative or Trade-offs

- A presenter who wants to add a node, branch, or slide still has to hand-edit
  JSON — the modal narrows the gap but does not close it. Stage D (Markdown
  import) or a future structural-editor ADR is still needed for that.
- Saving reformats the whole file. A presenter who hand-crafted specific
  whitespace/ordering for readability or diff-friendliness loses that on
  first modal save.
- `code`/`list` content is read-only in the modal, which may feel
  inconsistent to a user who doesn't understand the distinction between
  content and structural edits.

### Neutral / Follow-up

- Implementation must add a write-back callback to `present_watching`
  (or a sibling entry point) in `fireside-cli`, symmetric to the existing
  `ReloadSource` callback — this is an engine-boundary detail to settle in
  the tasks/plan for Stage C, not in this ADR.
- If Stage C proves the round-trip solid and presenters ask for structural
  edits next, that is Stage D territory (Markdown import) or a new ADR —
  not an amendment to this one.
