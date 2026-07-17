---
title: 'ADR-010: Container nesting depth limit'
status: 'accepted'
date: '2026-07-17'
deciders: ['@tiberius']
---

# ADR-010: Container nesting depth limit

## Status

Accepted

## Context

The strategic plan's P2 "Protocol & workflow hardening" item
(`.claude/plans/2026-07-12-strategic-improvement-plan.md`) lists "deep
container nesting" as a robustness fixture to add: "spec says engines MAY
impose limits — pick one" (spec 008,
`specs/008-protocol-workflow-hardening/`). `protocol/main.tsp`'s
`ContainerBlock` doc comment already grants this latitude explicitly:

> There is no protocol limit on nesting depth. Engines MAY impose practical
> limits.

This is engine-defined behavior the spec anticipates, not a wire-format gap
— no schema change is needed to add a limit, only a decision on the number
and a validator rule enforcing it.

Read the actual reference implementation and every existing document before
picking a number: `docs/examples/hello.json` (the canonical example) nests
`Container` blocks at most 1 level deep. No fixture, template
(`crates/fireside-cli/src/main.rs`'s `linear_template`/`branching_template`/
`workshop_template`), or test anywhere in the repository exceeds 2 levels.
Recursive functions that walk the content tree exist in three places:
`fireside-engine::validation`'s `walk_reveal_masking`/`walk_link_urls`,
and `fireside-tui::render::blocks`'s `container()`/`render_blocks()` — all
three recurse without an explicit depth bound today, which is fine for
realistic decks but leaves them open to unbounded recursion on a
pathological or adversarial input (e.g. a machine-generated or malicious
deck with thousands of nested containers).

Considered:

- **No limit** — rejected; leaves the three recursive functions above
  unbounded against exactly the kind of input this hardening pass exists to
  guard against.
- **A limit close to observed usage (e.g. 3)** — rejected as
  presenter-hostile. A `columns`-inside-a-centered-container-inside-a-stack
  deck is already a plausible 3-level design; the limit should have real
  headroom above what's been seen, not hug it.
- **A generous limit (8, chosen)** — comfortably an order of magnitude
  above any realistic authored deck (hello.json's 1, the deepest template's
  2), while still giving the three recursive functions a hard, testable
  ceiling.

## Decision

The reference implementation imposes a maximum `Container` nesting depth of
**8**: a node is invalid if any content block's depth (0 for a non-container
leaf; `1 + max(child depth)` for a `Container`) exceeds 8.

New diagnostic `container-nesting-depth-exceeded`, **Error** severity —
unlike the content-quality warnings this codebase has added before
(`malformed-link-url`, `reveal-masked-by-container`), an over-nested
document risks pathological recursion in the validators and the renderer,
so it is treated as a structural defect in the same class as
`unique-node-ids`/`valid-traversal-target`, both of which already block
presenting. Implemented symmetrically in `fireside-engine::validation`
(`check_container_nesting_depth`) and `protocol/validate.mjs`, and covered
by the shared fixture corpus
(`protocol/fixtures/valid/nesting-depth-at-limit.json` at exactly 8,
`protocol/fixtures/invalid/nesting-depth-exceeds-limit.json` at 9) so
Rust/Node parity on this rule is tested, not just claimed — the same
discipline every prior validator-rule addition in this repo has used since
ADR-007's fixture corpus.

`protocol/main.tsp`'s `ContainerBlock` doc comment gains a sentence noting
the reference implementation's chosen example limit (8) as informational
guidance for other engines, not a protocol requirement — no schema or
`tsp-output/` regeneration needed, since no field changes.

## Consequences

### Positive

- Closes a real (if currently theoretical) unbounded-recursion gap in three
  existing recursive functions, with a limit that has zero practical impact
  on any authored deck seen so far.
- No wire-format change, no protocol version bump, no crate-boundary
  change — pure validator-rule + fixture + doc-comment work.
- The limit is enforced identically by both validators and proven so by the
  fixture corpus, consistent with every other structural rule in this
  codebase.

### Negative or Trade-offs

- 8 is a judgment call, not a value derived from a hard technical
  constraint (e.g. actual stack-depth measurement). If a future legitimate
  authoring pattern needs deeper nesting, this is a one-line change plus a
  fixture update, not a wire-format migration — cheap to revisit.
- Third-party engines are not required to match this exact number (the
  spec only grants latitude, doesn't mandate a value); a document that
  passes the reference validator's depth-8 rule is not guaranteed to pass
  every third-party engine's own chosen limit. This is inherent to the
  spec's "Engines MAY impose practical limits" wording, not something this
  ADR could resolve differently without a spec change.

### Neutral / Follow-up

- No `tsp-output/` regeneration (no schema change).
- `docs/src/content/docs/spec/appendix-engine-guidelines.md` (or the
  nearest equivalent engine-latitude appendix) gets the same one-sentence
  note as `main.tsp`, keeping the docs site and the spec source in sync per
  existing practice.
