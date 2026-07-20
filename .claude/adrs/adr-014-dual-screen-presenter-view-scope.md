---
title: 'ADR-014: Scope extension — dual-screen presenter view'
status: 'accepted'
date: '2026-07-20'
deciders: ['@tiberius']
---

# ADR-014: Scope extension — dual-screen presenter view

## Status

Accepted.

## Context

ADR-004 fixed Fireside's product scope to three verbs — `present`
(`fireside <file>`), `validate`, `new` — and Constitution Principle II
states scope additions "are rejected unless the user explicitly asks for
them." The 2026-07-19 full-project UX audit
(`.claude/plans/2026-07-19-fable-ux-audit.md`) surfaced, as addendum A-2,
that speaker notes are visible to the audience whenever the presenter's
terminal is on or mirrored to the shared display (addendum A-1) — the
notes panel (`s` key) has exactly one window, so "meant for you" is not
actually true whenever the terminal is projected. The proven fix in every
comparable terminal-deck tool (presenterm's speaker-notes mode) is two
processes: the deck fullscreened on the external display, and a second,
read-only follower window on the presenter's own laptop.

This is additional product surface — a new verb (`fireside notes <deck>`)
and a new launch flag (`--fullscreen`) — squarely the kind of change
Principle II's gate exists to catch. The user explicitly asked for it on
2026-07-19, promoting addendum A-2 from a "worth doing" note to scoped work
("Wave 4 — scoped feature: dual-screen presenter view" in the same audit
plan), which is exactly the explicit request Principle II requires before
scope grows. This ADR is the record of that request, per the plan's own
"Constitution flags" section, which calls out that this gate needs an ADR
rather than a second ask.

## Decision

Extend Fireside's product scope with one new verb, `fireside notes <deck>`,
and one new launch flag, `--fullscreen` (on `present` and the shorthand
`fireside <file>` form). Scope, spec, and task breakdown recorded in
`specs/012-presenter-view/` (spec.md, plan.md, tasks.md), spec-kit feature
candidate `012-presenter-view`.

The new verb is deliberately narrow and stays inside Principle II's spirit
rather than fighting it:

- It is **read-only** — it never writes to the deck file, the resume store,
  or any other presentation artifact (Constitution Principle IV's "no
  silent no-op" concern doesn't apply; there is nothing for it to mutate).
- It reuses the presenter's existing footer-teaches-its-keys convention —
  `q` to quit is the entire interactive surface.
- It adds no new dependency and no crate-boundary change (see ADR-015 for
  the supporting session-state file contract, which is the only new
  cross-process plumbing this feature needs).

`--fullscreen` is a one-line addition: it sets the existing `f`-key view
toggle's state at launch instead of requiring a manual keypress once the
projector window is up. It is not a new capability, only a new way to reach
one that already shipped.

## Consequences

### Positive

- Closes addendum A-1 (speaker notes visible to the audience) without
  weakening the audience-facing window's behavior in any way — the fix is
  entirely a new, separate, opt-in window.
- Matches the architecture that already existed for this: `PositionSink`
  already fires on every navigation change, and `ReloadSource`/
  `WriteBackSink`'s closure-injection pattern extends cleanly to a new pair
  of sources rather than requiring new infrastructure.
- Keeps the presenter's own on-stage surface (`present`) completely
  unchanged — a presenter who never runs `fireside notes` sees zero
  difference in the tool's existing behavior or footer.

### Negative or Trade-offs

- Fireside's CLI surface grows from three verbs to four. Accepted:
  Principle II's gate is about *unrequested* surface creep, not about
  surface staying frozen forever — the same principle that rejected
  earlier fresh ideas without a user ask (`.claude/adrs` precedent) is
  satisfied here by an explicit ask.
- A presenter must now understand *two* commands to get the full dual-
  screen experience, rather than one. Mitigated in the docs update
  (`specs/012-presenter-view/tasks.md` T032–T034): the existing
  single-screen notes panel (`s`) is explicitly reframed as the
  rehearsal/solo path, `fireside notes` as the on-stage path, so a
  presenter who only ever rehearses alone never needs to learn the second
  command at all.

### Neutral / Follow-up

- No Constitution Principle III (crate boundary) or Principle I (protocol)
  amendment is needed by this ADR alone — those are addressed, if at all,
  by ADR-015 (the session-state file, which is host-local cache, not a
  protocol or crate-boundary change).
- Proceed with `specs/012-presenter-view/tasks.md`'s Foundational phase.
