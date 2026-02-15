# Tech Context

## Core Stack

- Rust 2024 edition for the reference implementation.
- TypeSpec for protocol modeling.
- JSON Schema 2020-12 as machine-readable validation contract.
- Astro + Starlight for documentation.

## Build/Validation Commands

- TypeSpec compile: `cd typespec && npm run build`
- Docs build: `cd docs && npm run build`
- Rust build: `cargo build`
- Rust tests: `cargo test`
- Rust lint gate: `cargo clippy -- -D warnings`

## Protocol Technical Constraints

- JSON wire format uses kebab-case core properties.
- Content block discriminator is `kind`.
- Extensions use explicit typed form (`kind: "extension"`, `type`).
