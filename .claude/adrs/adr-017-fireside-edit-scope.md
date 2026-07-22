---
title: 'ADR-017: Scope extension — the authoring editor (`fireside edit`)'
status: 'accepted'
date: '2026-07-21'
deciders: ['@tiberius']
---

# ADR-017: Scope extension — the authoring editor (`fireside edit`)

## Status

Accepted.

## Context

ADR-004 fixed Fireside's product scope to `present`, `validate`, `new`;
ADR-014 already extended it once, to a fourth verb (`notes`), under
Constitution Principle II's explicit-user-request gate. The 2026-07-19
full-project UX audit (`.claude/plans/2026-07-19-fable-ux-audit.md`)
addendum A-3 identified the remaining authoring gap: building or
restructuring a deck still requires hand-editing JSON, which fails the
exact non-technical audience Principle II exists to protect — a presenter
who cannot edit JSON or think in graph structures currently cannot build a
deck at all without outside help.

The user explicitly scoped this the same day (2026-07-19) as a Tier-2
authoring editor, then revised it same-day to a stronger bar: not a text
editor bolted onto structural forms, but a **block editor** — Notion/
Gutenberg-style discrete blocks, **mouse-first with drag-and-drop** as the
primary interaction mode, keyboard-complete as the fallback. Full design
brief: `.claude/plans/2026-07-19-wysiwyg-editor-plan.md` (rev 3), formalized
through the Spec Kit pipeline as `specs/013-authoring-editor/`.

This is a fifth verb (`fireside edit <deck>`) — squarely Principle II's
gate — and, distinctly, an inversion of the presenter's own interaction
posture for one screen only, which the constitution's existing wording
doesn't anticipate: Principle II is framed around the presenter's
keyboard-taught, footer-driven experience, and this feature's own
foolproof bar requires the opposite default (mouse first, click-driven,
keyboard as the complete fallback) for the editor screen specifically.

## Decision

Extend Fireside's product scope with one new verb, `fireside edit <deck>`,
opening a full-screen authoring studio. Scope, spec, and task breakdown
recorded in `specs/013-authoring-editor/` (spec.md, plan.md, tasks.md).

Two things distinguish this scope addition from ADR-014's:

1. **WYSIWYG by construction.** The editing canvas is not a second
   rendering path — it reuses the presenter's own renderer (via the
   `SlideView` extraction, `specs/013-authoring-editor/research.md` §7), so
   there is nothing for the two screens to drift apart on.
2. **A deliberate, scoped inversion of interaction posture.** Every other
   Fireside screen is keyboard-first with the mouse as an additive
   convenience (the presenter's existing footer-teaches-its-keys
   convention). The editor inverts this: mouse-first (click, drag, hover
   cues) is the primary, discoverable path; every action remains
   keyboard-reachable, but the keyboard is the fallback layer, not the
   taught default. This inversion is **scoped to `fireside edit` only** —
   `present`, `notes`, and `validate` are completely unaffected, and a
   presenter who never opens the editor sees zero change in the tool's
   existing behavior or footer.

The inversion exists because the editor's target user is explicitly wider
than the presenter's: someone who has never used a terminal at all, per
the design brief's acceptance bar (the scripted mouse-only 10-minute test,
spec `SC-001`). Teaching that user keyboard shortcuts first, as the
presenter does for people already comfortable on stage with a deck, would
fail the exact audience this feature exists to serve.

## Consequences

### Positive

- Closes audit addendum A-3 — the last major authoring-path gap the
  2026-07-12 and 2026-07-19 audits identified.
- No crate-boundary or dependency change (Constitution III unaffected):
  `engine::authoring` uses only already-permitted deps; the editor's
  mouse-driven UI uses crossterm mouse capture the process already enables.
  See ADR-018 for the supporting `engine::authoring` module contract.
- The WYSIWYG-by-construction commitment means this feature cannot regress
  presenter rendering fidelity by design, not by discipline — verified as
  a property test (spec `SC-008`).

### Negative or Trade-offs

- Fireside's CLI surface grows from four verbs to five. Accepted, same
  reasoning ADR-014 already recorded: Principle II's gate is about
  *unrequested* surface creep, not a permanently frozen surface, and this
  is again an explicit, dated user request.
- The constitution's TEA-invariant wording (Principle IV) currently reads
  as `App`-specific; this feature adds a second, independent TEA struct
  (`EditorApp`). A PATCH amendment generalizing that wording is required —
  tracked in ADR-018, not this ADR, since it is a mechanical consequence of
  the `engine`/`tui` module design, not of the scope decision itself.
- The editor is a large, multi-week feature (waves E0–E4 in the design
  brief) — accepted as proportionate to closing the authoring gap
  completely rather than partially; each wave is independently releasable
  per the design brief's own gating discipline.

### Neutral / Follow-up

- No Constitution Principle I (protocol) change — `engine::authoring`
  operates entirely on the existing wire model; no new field, enum, or
  traversal behavior.
- Proceed with `specs/013-authoring-editor/tasks.md`'s Foundational phase
  (T005 onward) once ADR-018 and its bundled constitution amendment land.
