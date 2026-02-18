---
title: 'Deep Dive: fireside-engine'
description: 'Business logic layer for loading, validation, traversal, and session state.'
---

## Why This Crate Exists

`fireside-engine` is the application logic layer. It sits between protocol data
(`fireside-core`) and presentation surfaces (`fireside-tui`, future web UI).

That architecture teaches an important Rust lesson: keep state machine logic
independent from I/O and rendering.

## Code Map

- `src/loader.rs`: read JSON and build runtime graph
- `src/validation.rs`: structural diagnostics (dangling refs, branch targets)
- `src/traversal.rs`: `TraversalEngine` implementing Next, Choose, Goto, Back
- `src/session.rs`: `PresentationSession` (graph + traversal + dirty flag)
- `src/commands.rs`: command model for future editor undo/redo
- `src/error.rs`: engine-level domain errors

## Rust Patterns Used

### Stateful domain object

`TraversalEngine` is a classic mutable state machine with explicit operations.
This pattern keeps rules centralized and testable.

### Layered error model

`EngineError` wraps `CoreError` and adds runtime concerns.
This is idiomatic composition over giant flat error enums.

### Diagnostic model

Validation returns `Vec<Diagnostic>` instead of failing fast.
Great for tooling because users can fix multiple issues per pass.

### Session as single source of truth

`PresentationSession` bundles graph data and traversal cursor.
This keeps UI integration straightforward and reduces shared-state bugs.

## Rust Book References

- Structs and method organization (Chapter 5):
  <https://doc.rust-lang.org/book/ch05-00-structs.html>
- Enums for state machines (Chapter 6):
  <https://doc.rust-lang.org/book/ch06-00-enums.html>
- Recoverable errors (Chapter 9):
  <https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html>
- Module organization (Chapter 7):
  <https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html>
- Testing behavior (Chapter 11):
  <https://doc.rust-lang.org/book/ch11-01-writing-tests.html>

## Concepts To Know Before Editing

- Command-query separation: mutation APIs vs lookup APIs
- Invariant ownership: what core validates vs what engine validates
- Fallible navigation APIs and boundary conditions
- Designing testable state transitions

## Gotchas To Watch

- `commands.rs` is currently scaffold-level; command application is not fully implemented
- `TraversalEngine::new(start)` assumes callers provide sensible start indices
- Validation currently reports errors and warnings, but warning policy is still minimal

## Improvement Playbook

### 1) Complete command application + undo/redo

Goal: move editor mutations fully into engine with reversible operations.

Steps:

1. Add `apply(command, &mut PresentationSession)` API.
2. Define inverse command generation for each command variant.
3. Implement `undo()` and `redo()` using `CommandHistory`.
4. Add property-like tests for command round-trips.

### 2) Introduce typed diagnostics

Goal: make diagnostics machine-readable and stable.

Steps:

1. Add `DiagnosticCode` enum (e.g. `DanglingNextRef`).
2. Include `code`, `message`, `node_id`, `severity`.
3. Keep human message formatting at CLI layer.
4. Snapshot test diagnostics for representative invalid graphs.

### 3) Harden traversal contracts

Goal: remove silent no-op style behavior and clarify semantics.

Steps:

1. Review every traversal method for explicit result meaning.
2. Separate user-facing boundary events from internal errors.
3. Add docs with state diagrams for each operation.
4. Add tests for branch loops and deep history behavior.
