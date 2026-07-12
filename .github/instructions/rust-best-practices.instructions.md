---
name: 'Rust Best Practices'
description: 'Guidance for planning and implementing Rust work in the Fireside repository with maintainable, modular, and well-bounded code.'
applyTo: 'crates/**/*.rs'
---

# Rust Best Practices for Fireside

Use these rules whenever the task involves Rust code, crate design, or implementation planning.

## Canonical rules

The canonical rules live in the project constitution at
`/.specify/memory/constitution.md` — load and enforce them. They cover the MSRV,
the crate boundary table, the mandatory idioms (no `unwrap()`/`expect()` in library code,
TEA invariant, index rebuild, kebab-case serde), and the error handling stratification.
`/AGENTS.md` is the short operational pointer to the same rules.

## Core expectations

- Prefer small, focused functions and clear module boundaries.
- Do not move logic across crate boundaries just to make a quick fix.
- Prefer readability, explicit types, and predictable control flow over clever abstractions.

## Planning rules

- Before recommending a crate or API, verify it with Context7 and confirm it fits the Fireside MSRV and crate boundary rules.
- When planning a Rust change, call out:
  - the crate(s) affected,
  - the boundary impact,
  - the test coverage needed,
  - any refactor risk or maintainability trade-off.
- Prefer incremental changes that are easy to verify with tests.

## Maintainability checklist

For any Rust implementation plan or code review, verify that the proposal:

- keeps responsibilities modular,
- avoids duplication,
- uses existing patterns in the repo,
- includes tests for the changed behavior,
- and is small enough for a token-efficient model to implement safely.
