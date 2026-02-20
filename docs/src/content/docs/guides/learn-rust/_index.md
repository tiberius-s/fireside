---
title: 'Learn Rust with Fireside'
description: 'A hands-on Rust series using real Fireside source code as the teaching scaffold.'
---

This series teaches Rust by reading and extending a production Rust workspace.
Each chapter anchors on one Fireside module and ends with a practical exercise.

## How to Use This Series

- Follow chapters in order; later chapters assume earlier concepts.
- Keep a local clone of the repository open while reading.
- Run verification commands exactly as written.
- Prefer small edits and tests after each section.

## Chapter Map

1. [Your First Data Model](./01-data-model/)
2. [Errors That Help](./02-errors/)
3. [Ownership, Borrowing, and Collections](./03-ownership/)
4. [Traits and Polymorphism](./04-traits/)
5. [When Derive Isn\'t Enough](./05-custom-serde/)
6. [State Machines](./06-state-machines/)
7. [Undo/Redo with the Command Pattern](./07-command-pattern/)
8. [The Elm Architecture in Rust](./08-tea-architecture/)

## Prerequisites

- Rust toolchain installed (`rustup`, `cargo`)
- Ability to run crate-scoped tests
- Basic comfort reading enums and pattern matching

## Expected Outcome

After chapter 8, you should be able to trace input from terminal events to
state updates, understand why Fireside structures data and errors this way, and
make safe changes without breaking protocol guarantees.
