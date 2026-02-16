---
name: 'typespec-build'
description: 'Automated workflow for compiling TypeSpec to JSON Schema and updating documentation'
---

# TypeSpec Build Pipeline Skill

When changes are detected in the `models/` directory (especially `main.tsp`), follow this automated pipeline:

## Step 1: Compile TypeSpec

```bash
cd models && npm run build
```

This generates JSON Schema 2020-12 files in `models/tsp-output/schemas/`.

Expected output: 18 schema files:

- Graph.json, Node.json, NodeId.json, NodeDefaults.json
- ContentBlock.json, HeadingBlock.json, TextBlock.json, CodeBlock.json
- ListBlock.json, ImageBlock.json, DividerBlock.json, ContainerBlock.json
- ExtensionBlock.json, Traversal.json, BranchPoint.json, BranchOption.json
- Layout.json, Transition.json

## Step 2: Verify Schema Output

Check that compilation succeeded with no errors. Verify the expected number of schema files was generated.

## Step 3: Update Documentation Pages

After schema changes, these doc pages may need updates:

### Schema Reference Pages

- `docs/src/content/docs/schemas/graph.md` — Graph, NodeDefaults, Layout, Transition
- `docs/src/content/docs/schemas/node.md` — Node, Traversal, BranchPoint, BranchOption
- `docs/src/content/docs/schemas/content-blocks.md` — ContentBlock union, all 7 core blocks, ExtensionBlock

### Spec Pages (if type definitions changed)

- `docs/src/content/docs/spec/data-model.md` — Type hierarchy, property tables
- `docs/src/content/docs/spec/validation.md` — Validation constraints
- `docs/src/content/docs/spec/serialization.md` — Schema file listing

### Quick Reference

- `docs/src/content/docs/reference/data-model-quick-reference.md` — Type hierarchy summary

## Step 4: Validate Documentation Build

```bash
cd docs && npm run build
```

Ensure the docs build cleanly with no errors.

## Step 5: Update Example Files

If schema changes affect the wire format, update example files in `docs/examples/` to match.

## Key Conventions

- **Wire format:** kebab-case property names (e.g., `speaker-notes`, `branch-point`)
- **Enum values:** kebab-case (e.g., `split-horizontal`, `slide-left`)
- **Discriminator:** `kind` field on ContentBlock union
- **Extension blocks:** `kind: "extension"` + required `type`; `fallback` recommended
- **Namespace:** `Fireside`
- **Version:** Read from TypeSpec model header comment
