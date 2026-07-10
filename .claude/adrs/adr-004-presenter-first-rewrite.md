---
title: 'ADR-004: Presenter-first rewrite against protocol 0.1.0'
status: 'accepted'
date: '2026-06-11'
deciders: ['@tiberius']
---

# ADR-004: Presenter-first rewrite against protocol 0.1.0

## Status

Accepted. Supersedes ADR-003.

## Context

A protocol audit on 2026-06-11 judged spec 0.1.0 sufficient, but the
implementation carried years of accumulated surface: a node editor with
undo/redo, project scaffolding, font listing, theme import, a welcome
screen, and the non-protocol extras catalogued in ADR-003. The presenting
experience — the one thing a non-technical user touches — was not good
enough, and incremental fixes (the 24-task plan in `plan/`) were patching
around structure rather than fixing it.

The alternative to a rewrite was continuing that plan. It was rejected: it
preserved the editor-era architecture the spec had already left behind, and
spread effort across surface that presenters never see.

## Decision

We rewrote the four crates from scratch against the spec, presenter-first:

- `fireside-core` is the protocol model exactly — no `traversal.after`, no
  node-level `Layout`, no extension blocks, two transitions, three container
  layouts, string-only list items. Unknown JSON fields are ignored on read;
  absent fields stay absent on write.
- `fireside-engine` is the §3 state machine (`Session`) plus §4 Layer-2
  validation. Every operation returns an `Outcome` so the UI can give
  feedback for every keypress.
- `fireside-tui` presents only. The footer always shows exactly the valid
  keys; branches render as menus; terminal nodes announce themselves; a map
  doubles as the goto picker; one polished theme over terminal-native
  colors. Driven by an in-process `TestBackend` scenario suite.
- `fireside-cli` has three verbs: `fireside <file>` (present), `validate`,
  and `new`. Validation always runs before presenting.

The cut features (editor, projects, themes, fonts, extras) are deletions,
not deprecations. Any of them may return later through its own decision,
specified first if it touches the wire format.

## Consequences

### Positive

- The implementation and the spec agree completely; conformance claims are
  testable and honest.
- A non-technical presenter can run a deck knowing only what the footer
  teaches.
- The workspace shrank to code the presenting path actually uses, with the
  scenario suite (65 tests) guarding behavior rather than structure.

### Negative or Trade-offs

- Editing decks means editing JSON by hand until an editor returns.
- Documents using removed extras (extra transitions, nested list items,
  extension blocks) lose those affordances; transitions degrade per the
  spec's fallback, the rest are ignored by serde or rejected by the schema.

### Neutral / Follow-up

- ADR-003's "Engine Extensions" appendix now documents that the engine
  implements no extensions.
- Syntax highlighting, the `fade` transition effect, and terminal image
  rendering are deliberate polish follow-ups, not regressions to fix
  ad hoc.
