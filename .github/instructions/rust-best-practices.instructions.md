---
name: 'Rust Best Practices'
description: 'Guidance for planning and implementing Rust work in the Fireside repository with maintainable, modular, and well-bounded code.'
applyTo: 'crates/**/*.rs'
---

# Rust Best Practices for Fireside

Use these rules whenever the task involves Rust code, crate design, or implementation planning.

## Core expectations

- Prefer small, focused functions and clear module boundaries.
- Keep business logic in `fireside-engine`, protocol types in `fireside-core`, and UI/rendering in `fireside-tui`.
- Do not move logic across crate boundaries just to make a quick fix.
- Prefer readability, explicit types, and predictable control flow over clever abstractions.

## Implementation rules

- Do not use `unwrap()` or `expect()` in library code.
- Return typed `Result`/`Option` values instead of panicking.
- Use `thiserror` for typed library errors and `anyhow` only at CLI/application boundaries.
- Preserve the existing TEA invariant in `fireside-tui`: mutation happens only in `App::update`.
- Rebuild or update graph indexes when structural graph mutations occur.
- Use existing naming and serde conventions, especially kebab-case wire format.

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
