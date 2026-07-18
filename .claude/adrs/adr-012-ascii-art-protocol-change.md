---
title: 'ADR-012: ASCII art as a new `ascii-art` ContentBlock kind (protocol 0.1.3)'
status: 'accepted'
date: '2026-07-18'
deciders: ['@tiberius']
---

# ADR-012: ASCII art as a new `ascii-art` ContentBlock kind (protocol 0.1.3)

## Status

Accepted

## Context

The 2026-07 strategic improvement plan (Stream C, `.claude/plans/2026-07-17-research-and-improvement-plan.md`)
names ASCII art as a Wave 2 feature, with the user's decision already
locked before this ADR: "additive protocol change (spec bump → 0.1.3,
ADR, symmetric Rust+Node validator rules) — follows the reveal precedent
(ADR-009)." Today, ASCII art is only expressible by (ab)using a `CodeBlock`
with no `language`, or `"text"`/`"ascii"` — spec 005 gave that path
engine-side centering/clipping, but it remains, structurally, a source-code
listing wearing a costume: no `alt` text field, no semantic distinction
from a real code sample, and a renderer that must guess intent from the
absence of a language tag.

C-1 of the strategic plan set the architectural direction before this ADR
was written: all art generation happens at **authoring time** (via new
`fireside-cli`-only commands, see spec `009-ascii-art`), never at render
time. A render-time `big-text` heading attribute (rendered via
`tui-big-text`) was explicitly considered and rejected in the plan itself,
because it would add a runtime dependency to `fireside-tui` — the exact
crate ADR-008 fought to keep dependency-light — couple rendering to
`ratatui` version lockstep for a widget, and make the same document render
differently across engines depending on which one bundled a big-text
renderer. A pre-rendered `art: string` field sidesteps all three: the
value is already what every engine, present and future, needs to display,
using rendering machinery (spec 005's centered/clipped monospace path)
that already exists and needs no new dependency.

ADR-011 (`.claude/adrs/adr-011-ascii-art-crate-msrv-spike.md`) separately
resolved which crates generate that pre-rendered text at authoring time
(`figlet-rs`, `rascii_art`, both `fireside-cli`-only) — this ADR is scoped
to the wire-format decision only.

## Decision

Add `AsciiArtBlock` as an eighth `ContentBlock` union member (protocol
0.1.3): `kind: "ascii-art"`, `art: string` (required, pre-rendered
multi-line plain text), `alt?: string`, plus the standard `Revealable`
spread (`reveal?: int32`) every other block kind already has. No new
shared/base model beyond the existing `Revealable` spread — `AsciiArtBlock`
follows the exact same shape convention as every other block.

**This is a new enum member, not an additive optional field** — the
distinction matters enough to state explicitly, because every prior
protocol change this project has shipped (0.1.1's link syntax, 0.1.2's
`reveal` field) was safely ignorable by an engine that predates it. A new
tagged-union variant is not: `fireside-core`'s existing closed-enum
`#[serde(tag = "kind")]` design (unchanged by this feature) means a
pre-0.1.3 engine parsing a document containing an `ascii-art` block fails
the *entire document* with a `serde` "unknown variant" error — verified
directly against the current implementation (`specs/009-ascii-art/research.md`
§2) rather than assumed. **This ADR accepts that compatibility break as
the correct trade-off**, for the same reason a permissive/silent-drop
degrade was rejected in the linked spec's research: a live presentation
silently missing content it was authored to show is worse than a deck
that refuses to open before the presenter ever goes on stage — read at
authoring/validate time, well before the audience is in the room.

Rendering reuses spec 005's centered/sized-to-content box treatment via a
small refactor (the box-width math already inside `blocks.rs::code()`'s
`is_ascii_art` branch is extracted into a shared helper) rather than
routing `ascii-art` blocks through `code()` itself, since `ascii-art` has
no `language`/`highlight-lines`/`show-line-numbers` concept to thread
through. Zero new `fireside-tui` dependency — the design's headline
property, matching ADR-008's dependency-discipline precedent.

Validation gains two new symmetric (Rust + Node) `WARNING`-severity
rules, `ascii-art-too-wide` and `ascii-art-empty`, following the exact
pattern `reveal-masked-by-container` (ADR-009) and
`container-nesting-depth-exceeded` (ADR-010) already established: same
rule-name-parity discipline, same fixture-corpus proof, now enforced in
CI by B-2 (`node protocol/run-fixtures.mjs`) rather than local-only as it
was when those two rules shipped.

## Consequences

### Positive

- `ascii-art` gets first-class semantics (`alt` text, no false
  "language" implication, an unambiguous renderer dispatch) instead of
  overloading `CodeBlock` — the authoring intent is explicit in the wire
  format, not inferred from an absent field.
- Zero new dependency in `fireside-core`, `fireside-engine`, or
  `fireside-tui` — the entire dependency cost of this feature (`figlet-rs`,
  `rascii_art`) lands in `fireside-cli` only, keeping the crate boundary
  table's tightest rows (core, engine, tui) exactly as tight as ADR-008
  left them.
- The compatibility break is honestly named and produces a clear,
  actionable error rather than a silent corruption — consistent with this
  project's error-handling philosophy (constitution Principle V,
  "presenter-friendly, not parser-friendly" diagnostics) even though this
  particular failure mode (whole-document parse rejection) already existed
  for every other kind of malformed document.

### Negative or Trade-offs

- This is the first genuinely breaking protocol change Fireside has
  shipped. A deck authored with `ascii-art` cannot be opened by any
  engine — including this project's own — built before 0.1.3. Authors who
  need broad backward compatibility with older Fireside installs (or
  hypothetical third-party engines) should avoid the feature until they
  can guarantee their audience's engine version, same caveat any breaking
  format change in any versioned format carries.
- `docs/examples/hello.json`, the canonical example (Principle I), does
  not gain an `ascii-art` block as part of this change — it must keep
  validating on *every* protocol version per the constitution's own
  wording, and doing otherwise would itself become a compatibility trap
  for exactly the reason this ADR just accepted for opt-in content. Any
  future canonical-example update showcasing `ascii-art` would need its
  own explicit decision.

### Neutral / Follow-up

- If a future feature ever needs the same trade-off (a genuinely new
  block kind, not an additive field), this ADR is the precedent to cite
  rather than re-deriving the closed-enum compatibility analysis from
  scratch.
- `fireside-cli`'s two new dependencies (`figlet-rs`, `rascii_art`) are
  tracked in the constitution's Principle III amendment accompanying this
  feature, not duplicated here.

### Follow-up (2026-07-18, second pass)

Two corrections after user review of the shipped feature:

1. **The `fireside demo` showcase deck was wrongly excluded.** Task T046
   (spec 009) cited this ADR's hello.json reasoning to justify keeping
   `ascii-art` out of `crates/fireside-cli/assets/demo.fireside.json` too
   — but that reasoning is about `docs/examples/hello.json` specifically
   (the protocol's canonical cross-version example) and never applied to
   the CLI's own bundled demo deck, which ships in lockstep with the
   engine and has no cross-version constraint. Net effect: the feature
   was invisible in the one place a non-technical presenter would see it
   working, and the generation commands (`art text`/`art image`) were a
   disconnected utility rather than a presentation feature. Fixed: the
   `welcome` node's heading is now a FIGlet `ascii-art` banner;
   `demo_deck_shows_every_block_kind` asserts 8 kinds.
2. **`docs/examples/hello.json` itself is no longer treated as a frozen
   baseline.** The "does not gain an ascii-art block" trade-off recorded
   above is superseded by user decision: "canonical" means the example
   showcases everything the protocol currently offers, not a compat
   snapshot pinned to an old version — the constitution's actual text
   ("MUST parse, validate, and present correctly after every change")
   doesn't mandate the frozen reading this ADR (and ADR-007/ADR-009
   before it) added. `hello.json` now declares `"fireside-version":
   "0.1.3"` and includes an `ascii-art` block in its `intro` node; the
   stale "7 content block types" list item was corrected to "8". See the
   constitution's Principle I amendment (1.2.0 → 1.2.1) for the durable
   wording change, and ADR-007/ADR-009 for their own short follow-up
   notes. **Not done in this pass**: retroactively adding reveal marks
   (ADR-009) to `hello.json` — flagged, not bundled in, since it's a
   separate decision from the ascii-art fix this pass addressed.

Also added in this pass, closing the "authoring workflow" gap the
original Consequences section left as a manual step: `fireside new
--banner` generates a title banner directly into a scaffolded deck, and
`fireside import` promotes a ` ```ascii-art ` fence to a real block
instead of a generic code block. Both are `fireside-cli`-only, no
protocol change.
