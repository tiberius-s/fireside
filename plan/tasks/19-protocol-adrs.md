# Task 19 — ADRs for protocol decisions

**Depends on:** none (write before Tasks 06 and 17 land, ideally first)
**Crates:** none (docs only)
**Phase:** 4 (decision record; unblocks 06/17 socially, not technically)

## Goal

Record the three protocol-affecting decisions as Architecture Decision Records using the project's ADR skill (`.github/skills/adr/SKILL.md`), so implementing tasks execute against an authoritative decision instead of re-deriving it.

## ADRs to write

1. **ADR-0001 — Remove `traversal.after`.** Decision: branch rejoin is expressed via explicit `next` on branch endpoints (traversal.md "Branch return wiring"); `after` is deleted from the Rust model. Alternatives: spec it (rejected — redundant with explicit edges, adds a second rejoin mechanism). Consequence: Task 06.
2. **ADR-0002 — Retire node-level `Layout` in favor of `view-mode` + container layouts.** Decision: spec 0.1.0 model wins; legacy `Layout` values get a one-place rendering translation (Task 17) and the enum is removed in a future major. Lists the 12→2 mapping table. Consequence: Tasks 07/17.
3. **ADR-0003 — Non-normative engine extras.** Decision: `extension` blocks, `ExtensionDeclaration`, graph `theme`/`font`/`tags`, the 6 extra transitions, nested list items, and `BranchPoint.id` are **engine features, not protocol** — they remain implemented, are excluded from conformance claims, and get a short "Engine extensions" appendix page in `docs/src/content/docs/spec/` clearly marked non-normative. Alternative: delete them (rejected — working code, real value) or spec them (rejected for 0.1.0 — scope).

## Steps

1. Invoke the ADR skill per its SKILL.md (Nygard format) and write the three records to wherever the skill specifies (e.g. `docs/adr/` — follow the skill's convention; create the directory if this is the first ADR).
2. Write the "Engine extensions (non-normative)" appendix page referenced by ADR-0003.
3. Link each ADR from the relevant task files' PRs when they land.

## Do NOT

- Change any code or schemas.
- Decide things not listed above.

## Acceptance

- Three ADR files exist, each with Status/Context/Decision/Consequences.
- `cd docs && npm run check` passes (the new appendix builds).
