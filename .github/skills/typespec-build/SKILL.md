---
name: 'typespec-build'
description: 'Automated workflow for compiling TypeSpec to JSON Schema and updating documentation'
---

## TypeSpec build — workflow and checklist

This skill captures the _workflow_ for making, verifying, and publishing TypeSpec-driven schema changes. It avoids hard-coded schema filenames — treat the generated schema directory as the canonical artifact and verify effects rather than exact file lists.

### When to run

- Any change to files under `models/` (especially `main.tsp`) that alters the TypeSpec model or metadata.

### Primary steps

1. Compile TypeSpec
   - Run: `cd models && npm run build`
   - Artifact: updated JSON Schema files written to `models/tsp-output/schemas/`.
   - Verify the command exits successfully and the output folder timestamps/contents changed for the types you modified.

2. Smoke-check generated schemas
   - Confirm the generated schemas reflect your model changes (open the relevant schema files or diff them).
   - Run any repository schema-consumer tests (docs pages, example validators, or unit tests that depend on the schema).
   - If you have automated schema validation (AJV or similar), run it against representative examples.

3. Update documentation & examples
   - Update schema reference pages and spec pages under `docs/src/content/docs/` that describe any changed types, constraints, or examples.
   - Update example payloads in `docs/examples/` if the wire format changed.

4. Validate documentation build
   - Run: `cd docs && npm run build`
   - Ensure the docs site builds cleanly and the changed pages render as expected.

5. Run repository checks
   - Run unit/integration tests and any format/lint checks that may exercise generated artifacts.
   - Optionally smoke-run consumers that use the schema (CLI examples, sample apps).

6. Commit and open PR
   - Include both the TypeSpec source changes and the regenerated schemas in the _same_ commit (or same PR) so reviewers can see the generated delta.
   - Recommended commit message format: `typespec: <short change summary> — regenerate schemas`

### PR reviewer checklist ✅

- [ ] Model change (TypeSpec) and regenerated schemas are present together.
- [ ] Documentation and examples updated where needed.
- [ ] Docs build passes locally / CI.
- [ ] Any schema-consumer tests updated or added.
- [ ] No breaking changes to the public wire format unless intentional and documented.

### Troubleshooting tips

- TypeSpec compile errors: run `cd models && npm run build` and fix the compiler output (it reports the exact model/location).
- Missing/incorrect output: confirm `models/tsp-output/schemas/` is writable and your node dependencies are installed (`npm ci` in `models/`).
- Unexpected runtime errors in consumers: re-run consumer tests and inspect schema diffs to identify breaking changes.

### Automation suggestions

- Add a CI job that regenerates schemas and fails if generated outputs differ from the committed files.
- Add a small integration test that validates at least one example JSON against the generated schema set.

### Key conventions (reminder)

- Wire format uses kebab-case property names and enum values.
- Use `kind` as the ContentBlock discriminator.
- Extension blocks: `kind: "extension"` with a required `type` and sensible `fallback` where appropriate.
- Treat the TypeSpec model as the source of truth; generated schemas are derived artifacts.

---

Short and focused — this document captures the workflow and checks to follow when editing TypeSpec and regenerating schemas.
