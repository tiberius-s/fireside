# Task 20 — Spec fixes S1–S4 + schema regeneration

**Depends on:** none (parallel-safe; coordinate with 16 if it has landed)
**Crates:** none (protocol/ and docs/ only)
**Phase:** 5

## Goal

Close the four gaps inside the spec/tooling itself.

## Steps

1. **S1** — `protocol/main.tsp:204`: change `ContainerBlock.layout?: string` to an enum (`stack | columns | center`, default `stack`). Mirror the wording already in the doc comment.
2. **S2** — `ImageBlock.width` doc comment says "columns or percentage" for an `int32`. Pick: width/height are **terminal cells (columns/rows)**; percentages are out of scope for 0.1.0. Update `main.tsp` doc comments and `docs/src/content/docs/spec/appendix-content-blocks.md` to match.
3. **S3** — `protocol/validate.mjs`: missing node `id` currently surfaces as `Duplicate node ID "undefined"`. Ensure the schema layer (Layer 1) runs first and reports `missing required property 'id'` with the instance path; the unique-id check should skip nodes lacking ids.
4. **S4** — the canonical example triggers `[dead-end-branch]` on its own terminal node. Either downgrade that lint to Info when the dead end is the documented terminal pattern, or add a lint suppression note; update the lint message to say terminal nodes are legitimate ("only back() can exit" stays accurate).
5. Regenerate schemas: `cd protocol && npm run build`; commit `tsp-output/` (models.yml enforces this).
6. If S1 makes any existing document invalid (a container with an unlisted layout string), fix the document — the schema is right.

## Do NOT

- Touch Rust code (if S1 tightens the schema, the Rust render fallback from Task 17 already treats unknown layouts as `stack`).
- Bump the protocol version (these are clarifications, not breaking changes).

## Acceptance

```bash
cd protocol && npm run build && git diff --stat tsp-output/
node validate.mjs ../docs/examples/hello.json    # 0 errors; dead-end no longer a Warning (or message updated)
cd ../docs && npm run check
```
