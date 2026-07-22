---
title: 'ADR-018: `engine::authoring` module charter, TEA-wording amendment, and new theme tokens'
status: 'accepted'
date: '2026-07-21'
deciders: ['@tiberius']
---

# ADR-018: `engine::authoring` module charter, TEA-wording amendment, and new theme tokens

## Status

Accepted. Depends on ADR-017 (the scope extension this module exists to
support).

## Context

`fireside edit` (ADR-017) needs a way to mutate a `Graph` that upholds the
same invariants `fireside-engine::validation` already checks for —
never producing a dangling reference, a duplicated id, a `next`+
`branch_point` conflict, or a gapped reveal sequence — but as
*construction*, not *detection*: the editor's whole foolproof premise
(spec `013-authoring-editor` FR-023) is that these states must be
unrepresentable, not merely flagged after the fact.

Three design questions needed settling before any code: how mutation is
represented and undone, how slide identity is generated and kept
consistent under rename, and — a knock-on consequence of adding a second
independent TEA state machine to `fireside-tui` — whether the
constitution's TEA-invariant wording still holds as written.

## Decision

### 1. `engine::authoring` module (`fireside-engine/src/authoring.rs`)

A pure transform layer: `fn apply(graph: &Graph, op: &Op) -> Result<Graph, AuthoringError>`.
Full operation set and per-op contract: `specs/013-authoring-editor/contracts/authoring-ops.md`.
Two decisions worth recording here specifically:

- **Undo representation: full `Graph` clones, not op inversion.** Every
  committed op pushes a clone of the resulting `Graph` (plus the current
  selection) onto a capped history stack (100 entries). Considered and
  rejected: inverting each `Op` into an "undo op." Rejected because decks
  in this feature's scope (spec `SC-009`: up to 500 slides) clone cheaply —
  well under the 100ms interaction budget — and because snapshot-based undo
  means the proptests (`specs/013-authoring-editor/tasks.md` T011/T012)
  only have to prove the forward transforms correct, never a second,
  independently-maintained inverse of each one. Simplicity and test-surface
  reduction both point the same way.
- **Id/slug algorithm.** New or retitled slides derive their id by
  lowercasing the title, collapsing non-alphanumeric runs to a single `-`,
  trimming, falling back to `"slide"` on an empty result, and deduping
  against every existing id with `-2`, `-3`, … suffixes. `RetitleSlide` is
  one atomic op that rewrites the id *and* every reference to it (`next`
  edges, branch-option targets, the entry-node position) together — a
  proptest asserts no rename sequence can ever leave a dangling reference.
  Considered and rejected: stable, author-invisible UUIDs instead of
  slugs. Rejected because slugs stay human-legible in the deck's own JSON
  (still hand-readable/diffable outside the editor, a property worth
  keeping even though the editor itself never shows ids per the
  vocabulary rule), while a UUID would only ever help this feature and
  would actively hurt every other way of touching the file.

### 2. TEA-invariant wording (Constitution Principle IV, PATCH amendment)

The editor introduces `EditorApp`, a second, independent Elm-Architecture
state machine in `fireside-tui`, alongside the presenter's existing `App`.
Principle IV currently reads:

> TEA invariant: `App::update` in `fireside-tui` is the ONLY function that
> mutates `App` state; rendering is pure.

Read literally, this already doesn't forbid `EditorApp::update` existing
(it constrains `App`, not the crate), but it also doesn't *say* the
invariant generalizes — a future reader could plausibly treat `App`'s
wording as the whole rule and let a second struct grow multiple mutators
by omission, not by a considered exception. Rather than rely on that being
obviously implied, this ADR amends the wording explicitly, PATCH-level
(1.3.0 → 1.3.1: a clarification of existing guidance, not a new principle
or a redefinition — same class of change as prior PATCH-level constitution
edits):

> TEA invariant: each TUI application struct has exactly one `update`
> function, which is its sole mutator; rendering is pure.

This covers `App::update` (unchanged behavior) and `EditorApp::update`
(new) identically, and would cover any future third TEA struct the same
way without requiring another amendment.

### 3. New `theme.rs::Tokens` entries

Four additive tokens — `affordance`, `selection`, `drop-target`, `ghost` —
for the editor's mouse-affordance styling (hover outline, selection
border, drop-position indicator, drag ghost). These follow the existing
rule (Principle IV: no raw `Style` construction in render code) without
changing it; recorded here only because the constitution's Principle IV
bullet enumerating the styling rule is touched by the same PATCH edit as
the TEA wording above.

## Consequences

### Positive

- The undo/id decisions mean `AuthoringError`'s surface is exactly the set
  of precondition failures in `contracts/authoring-ops.md` — no
  op-inversion bookkeeping, no separate UUID-vs-slug reference table to
  keep consistent.
- The TEA wording amendment closes a real ambiguity before a second
  struct existed to expose it, rather than after a violation shipped.
- Both decisions are provable by proptest (id-rename-never-dangles;
  the four unrepresentable-by-construction invariants hold over arbitrary
  op sequences) — this ADR's claims are testable, not just asserted.

### Negative or Trade-offs

- Full-clone undo means `EditorApp::history` holds up to 100 full `Graph`
  values in memory at once. Accepted: at the 500-slide scale this feature
  targets, this is a small, bounded, one-time memory cost, far cheaper
  than the complexity of a correct op-inversion implementation would be.
- Slug ids can collide in *meaning* (two different slides both titled
  "Q&A" get `q-a` and `q-a-2`) even though they never collide in *value*.
  Accepted as a pre-existing property of every hand-authored Fireside deck
  today (the protocol's `NodeId` has always been a bare string) — this
  feature does not need to, and does not, change that.

### Neutral / Follow-up

- No Constitution Principle III (crate boundary) or allowlist amendment —
  `authoring.rs` uses only `fireside-core` + `thiserror`, both already
  permitted for `fireside-engine`.
- The constitution PATCH amendment itself (Principle IV wording + Sync
  Impact Report entry) is applied as its own task
  (`specs/013-authoring-editor/tasks.md` T003), immediately following this
  ADR, before any `EditorApp` code is written.
- Proceed with `specs/013-authoring-editor/tasks.md`'s Foundational phase.
