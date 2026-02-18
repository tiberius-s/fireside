---
title: 'Deep Dive: fireside-cli'
description: 'Thin command-line entrypoint, command modules, and safe orchestration patterns.'
---

## Why This Crate Exists

`fireside-cli` is intentionally thin: parse command-line options, dispatch to
handlers, and let engine/TUI crates do real work.

This is an ideal beginner pattern in Rust because it avoids business logic in
`main.rs` and keeps testing boundaries obvious.

## Code Map

- `src/main.rs`: clap command model and top-level dispatch
- `src/commands/session.rs`: present loop bootstrap and terminal setup/teardown
- `src/commands/validate.rs`: validation command output
- `src/commands/project.rs`: `fireside.yml` parsing and entrypoint resolution
- `src/commands/scaffold.rs`: project/presentation scaffolding
- `src/commands/theme.rs`: iTerm2 import to theme TOML
- `src/commands/fonts.rs`: font discovery output

## Rust Patterns Used

### Subcommand enum with clap derive

Using `#[derive(Parser, Subcommand)]` gives a typed CLI model and reliable help
text without hand-written argument parsing.

### Small command modules

Each operation is in a dedicated file with a `run_*` function.
This keeps the binary easy to scan and minimizes merge conflicts.

### Context-rich error handling

`anyhow::Context` adds command-specific failure context while preserving causes.
Great for beginner-friendly debugging.

### RAII-like terminal lifecycle with explicit cleanup

`run_presentation` enters alternate screen and raw mode, then restores both.
This is essential for robust terminal apps.

## Rust Book References

- Command-line and binary structure (Chapter 12):
  <https://doc.rust-lang.org/book/ch12-00-an-io-project.html>
- Modules and crate organization (Chapter 7):
  <https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html>
- Error propagation and context (Chapter 9):
  <https://doc.rust-lang.org/book/ch09-00-error-handling.html>
- Workspaces and crate boundaries (Chapter 14):
  <https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html>

## Concepts To Know Before Editing

- Why `main.rs` should orchestrate, not implement
- How `Path`/`PathBuf` ergonomics work in command APIs
- Difference between recoverable command errors and process exit behavior
- CLI UX principles: explicit defaults, clear help, stable flags

## Gotchas To Watch

- Default help output path in `None` command arm duplicates clap capabilities
- Scaffold template schema URL and defaults may drift from local protocol model
- `run_editor` intentionally returns not-implemented; document this in CLI help text

## Improvement Playbook

### 1) Add integration tests for CLI behavior

Goal: ensure user-visible command contracts stay stable.

Steps:

1. Add tests with `assert_cmd` and fixture files.
2. Cover `validate`, `new`, and invalid path scenarios.
3. Assert exit codes and key output strings.
4. Add regression tests for project config parsing.

### 2) Strengthen command UX consistency

Goal: make errors and outputs uniform across subcommands.

Steps:

1. Standardize success/failure message formats.
2. Add a shared formatter for diagnostics.
3. Emit actionable next steps in common failure cases.
4. Keep examples in `--help` aligned with docs examples path.

### 3) Add explicit config precedence docs + behavior

Goal: avoid hidden surprises around theme and project settings.

Steps:

1. Define precedence matrix in docs and enforce in code.
2. Implement resolution helper used by all relevant commands.
3. Add tests for each precedence branch.
4. Print effective theme when verbose logging is enabled.
