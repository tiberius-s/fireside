# Contract: `container-nesting-depth-exceeded` validator rule

A new ERROR-level rule fires when a node's `content` tree contains a
`Container` block nested more than 8 levels deep (see data-model.md for the
exact depth computation: `0` for a non-container leaf, `1 + max(child
depth)` for a `Container`). ERROR, not WARNING — unlike content-quality
rules (`malformed-link-url`, `reveal-masked-by-container`), an
over-nested document risks pathological recursion in both validators and
the renderer, so it is treated as a structural defect the same class as
`unique-node-ids` or `valid-traversal-target`, all of which block
presenting.

Implemented symmetrically in `fireside-engine::validation` and
`protocol/validate.mjs`, and covered by the shared fixture corpus
(`protocol/fixtures/{valid,invalid}/*.json` + `fixtures.expected.json`) with
one fixture at exactly the limit (accepted) and one fixture one level past
it (rejected), so Rust/Node parity on this rule is tested, not just
claimed — matching the precedent set by `empty-traversal`,
`reveal-masked-by-container`, and `malformed-link-url`.

## Spec text

`protocol/main.tsp`'s `ContainerBlock` doc comment currently reads:

> There is no protocol limit on nesting depth. Engines MAY impose practical
> limits.

This is engine-defined latitude already granted by the spec — no wire
format or schema change. `docs/src/content/docs/spec/appendix-engine-guidelines.md`
(or the nearest equivalent appendix covering engine-defined limits) gains a
sentence documenting the reference implementation's chosen limit (8) as an
example other engines may choose to match or diverge from, consistent with
how the image-clamp and history-cap ambiguities were resolved as
non-binding guidance in spec patch 0.1.1 (ADR-007).

## Diagnostic shape

Matches the existing `Diagnostic` shape used by every other rule (rule id,
severity, message, node id where the violation was found) — no new
diagnostic fields.
