# Project Brief

Fireside is a portable format for branching presentations and lessons.
The protocol baseline is `0.1.0` and TypeSpec is the source-of-truth model.

## Core Requirements

1. JSON wire format with kebab-case core properties.
2. Directed-graph traversal with `Next`, `Choose`, `Goto`, `Back`.
3. Seven mandatory core block kinds:
   `heading`, `text`, `code`, `list`, `image`, `divider`, `container`.
4. Extensibility via explicit typed extensions:
   `kind: "extension"` + required `type`.
5. JSON Schema 2020-12 output generated from TypeSpec.
6. Docs organized as six normative chapters plus three appendices.

## Documentation Source Layout

- Normative: `docs/src/content/docs/spec/`
- Schema reference: `docs/src/content/docs/schemas/`
- Quick reference: `docs/src/content/docs/reference/`

## Current Priority

Keep TypeSpec, generated schemas, and documentation terminology aligned.
