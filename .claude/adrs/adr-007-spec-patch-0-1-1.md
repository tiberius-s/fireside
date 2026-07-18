---
title: 'ADR-007: Protocol spec patch 0.1.1 — resolve audit ambiguities'
status: 'accepted'
date: '2026-07-12'
deciders: ['@tiberius']
---

# ADR-007: Protocol spec patch 0.1.1 — resolve audit ambiguities

## Status

Accepted

## Context

The 2026-07-12 strategic improvement plan (`.claude/plans/2026-07-12-strategic-improvement-plan.md`
§1) catalogued seven places where `protocol/main.tsp` and its docs
(`docs/src/content/docs/spec/`) leave engine behavior unstated: branch-option
key uniqueness, the empty `Traversal` object (`{}`), the `choose()` contract,
ViewMode toggle persistence, whether `ListBlock.items` allows inline
Markdown, image width/height overflow, and unbounded history growth. The plan
flagged these as a Week 1 "spec patch 0.1.1" item, paired with a shared
conformance fixture corpus (originally scoped as P2, pulled into Week 1
because the corpus exists specifically to pin down the rules this patch
touches).

Reading the actual code — not just the audit's original guesses — showed
that six of the seven ambiguities are **already resolved in behavior**; the
reference implementation picked a consistent answer for each one, it just
was never written down (or, in one case, was written down wrong):

1. **Branch-option key uniqueness** is already an ERROR-level rule
   (`unique-branch-keys`) in both `crates/fireside-engine/src/validation.rs`
   and `protocol/validate.mjs`. But `docs/src/content/docs/spec/validation.md`
   §4 lists it under "Recommended Checks," which implies warning severity —
   a doc/impl mismatch, not an unresolved question.
2. **Empty `Traversal` object `{}`** — `Node::is_terminal()` in
   `crates/fireside-core/src/model/mod.rs` already treats `{}` as terminal,
   identical to an absent `traversal` field. Neither validator flags it,
   though, so a leftover empty object (a plausible authoring slip) is
   currently indistinguishable in diagnostics from a deliberate terminal
   node. This is the one ambiguity that needs new code, not just new prose.
3. **`choose()` contract** — `main.tsp`'s documentary interface reads
   `choose(option: BranchOption): void`, which looks like it accepts an
   arbitrary option. The Rust reference implementation
   (`fireside_engine::Session::choose(&mut self, option: usize)`) sidesteps
   the whole class of bug by taking an index into the *current* node's
   branch-point options — forging an option belonging to another node isn't
   representable. Spec-text gap only.
4. **ViewMode toggle persistence** — `crates/fireside-tui/src/app.rs`'s
   `view_override: Option<ViewMode>` is a single App-level field set by the
   `f` key handler and never cleared on navigation. The reference behavior
   already is "persists across node transitions until toggled again"; the
   spec just doesn't say so.
5. **`ListBlock.items` and Markdown** — `blocks.rs::list()` already renders
   items through the same `markdown::wrap_styled` path as `TextBlock.body`.
   Spec-text gap only.
6. **Image width/height overflow** — `blocks.rs::image()` doesn't read
   `width`/`height` at all today; it self-sizes a placeholder box to the alt
   text, because real image rendering is still the deferred P1
   `ratatui-image` work. The audit's "engines MUST clamp" recommendation is
   the right rule to write down, but it has to be stated honestly as
   forward guidance, not as something the current placeholder renderer
   already does.
7. **Unbounded history growth** — `crates/fireside-engine/src/session.rs`'s
   `history: Vec<NodeId>` has no cap. A non-issue in practice; worth one
   sentence in Appendix B so a third-party engine knows capping is allowed.

Doing nothing leaves the spec silent on behavior the reference engine has
already committed to, which is exactly the drift Principle I ("Spec Is the
Source of Truth") exists to prevent — a third-party engine reading only
`main.tsp` today would have to guess at six settled behaviors and would
find one validator rule undocumented at its true severity.

## Decision

We will ship protocol version **0.1.1**: add `v0_1_1: "0.1.1"` to the
`Versions` enum in `main.tsp` and update its version banner, then resolve
all seven ambiguities as spec-text and documentation changes, plus exactly
one new validator rule pair. Concretely:

- New WARNING rule `empty-traversal` added symmetrically to
  `fireside-engine/src/validation.rs` and `protocol/validate.mjs` — the only
  behavior change in this patch. Fires when a node's `traversal` is present
  as an object but sets neither `next` nor `branch-point`; the engine still
  treats it as terminal (unchanged), the rule only adds a diagnostic.
- `validation.md` §4: move `unique-branch-keys` from "Recommended Checks" to
  "Required Checks," matching its actual Error severity; add
  `empty-traversal` under "Recommended Checks."
- `traversal.md` "Operation: Choose": one added sentence requiring
  implementations to validate the selected option belongs to the current
  node's branch point, citing the index-based approach as the recommended
  pattern.
- `appendix-engine-guidelines.md` (Appendix B): add ViewMode persistence
  behavior, the image clamp-to-content-area rule (marked as forward
  guidance, not current placeholder behavior), and the history-cap
  allowance.
- `main.tsp` `ListBlock` doc comment + `appendix-content-blocks.md`
  (Appendix C): state that `items` entries MAY contain inline Markdown.
- New shared conformance fixture corpus at `protocol/fixtures/{valid,invalid}/*.json`,
  consumed by both `validate.mjs` and a new Rust test in
  `fireside-engine`'s validation suite, asserting the two validators fire
  identical rule-id sets per fixture — turning the claimed Rust/Node parity
  into a tested invariant and covering the new `empty-traversal` rule.
- Regenerate and commit `tsp-output/` per the constitution's Operational
  Constraints.

This is not a wire-format-breaking change. No field, block kind, or
required property changes shape — only one new optional enum value for
`fireside-version`, prose, a severity-classification fix, and one new
WARNING rule. All 0.1.x-additive per Principle I.

## Consequences

### Positive

- Six of seven ambiguities close with zero behavior risk — they document
  what the reference implementation already does, so there is nothing to
  regress.
- `unique-branch-keys`'s doc fix removes a real spec/impl inconsistency a
  third-party implementer could have tripped on.
- `empty-traversal` gives presenters a diagnostic for a plausible authoring
  slip (`"traversal": {}`) that was previously silent.
- The fixture corpus makes "Rust and Node validators agree" a tested fact
  instead of a claim resting on matching rule-name strings, and seeds a
  reusable conformance suite for any future third-party engine.

### Negative or Trade-offs

- `docs/examples/hello.json`, the canonical example, is NOT being bumped to
  `"fireside-version": "0.1.1"` — it stays valid as a 0.1.0 document under
  0.1.1 engines, but this means the canonical example doesn't exercise the
  new enum value. Acceptable: nothing in this patch requires 0.1.1 documents
  to exist, only permits them.
- The image-clamp rule in Appendix B describes intended behavior for
  capability the reference renderer doesn't have yet (real image sizing is
  still P1/`ratatui-image`). Anyone reading Appendix B without also reading
  the placeholder-renderer caveat could believe clamping already happens.
- Adding a validator rule to two independently-maintained files
  (`validation.rs`, `validate.mjs`) means every future rule change carries a
  two-file tax; the fixture corpus mitigates but doesn't eliminate the risk
  of the two drifting.

### Neutral / Follow-up

- The plan's separate docs-audit item ("sidebar skips §5" in
  `astro.config.mjs`) is explicitly out of scope for this ADR/patch.
- P1 (`ratatui-image`, incremental reveal) and P2 (mouse, synchronized
  output, resume) items are unaffected and remain unstarted.
- When real image rendering lands (P1), `blocks.rs::image()` will need to
  actually implement the clamp rule this ADR only documents — tracked as
  future work under the P1 image stage, not here.

### Follow-up (2026-07-18)

The "hello.json NOT bumped" decision above is superseded: the user
clarified that `docs/examples/hello.json` should grow to showcase every
protocol feature as it ships, not stay pinned to a compat baseline —
"canonical" means comprehensive, not frozen. `hello.json` now declares
`"fireside-version": "0.1.3"` (as of the ascii-art feature). This ADR's
0.1.1-specific reasoning ("nothing requires 0.1.1 documents to exist,
only permits them") is left as historical record of the original
(now-reversed) policy; see ADR-012's own follow-up note for the fuller
rationale and the corresponding constitution amendment.
