# System Patterns

## Protocol Source of Truth

- Type definitions live in `typespec/main.tsp`.
- JSON Schema 2020-12 output is generated to `typespec/tsp-output/schemas/`.

## Content Model Pattern

`ContentBlock` is a discriminated union keyed by `kind`.

Core kinds:

- heading
- text
- code
- list
- image
- divider
- container

Extension shape:

- `kind: "extension"`
- `type: string`
- optional `fallback`
- extension-specific payload fields

## Docs Architecture Pattern

- `spec/`: normative chapters and appendices.
- `schemas/`: generated-schema-oriented references.
- `reference/`: concise vocabulary and quick-reference pages.
- Sidebar uses explicit manual order for normative chapters.

## Engine Pattern (Reference Implementation)

TEA-style loop:

`Event -> Action -> App::update -> Render`

State mutation belongs in update logic; rendering remains stateless.
