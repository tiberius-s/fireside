# Plan: Cargo Workspace Reorganization with Core, Engine, TUI, and CLI Crates

**TL;DR** — Restructure the current monolithic crate into a Cargo virtual workspace with four crates under `crates/`:

- `fireside-core`: protocol model, wire-format serialization/deserialization, and protocol-level invariants
- `fireside-engine`: document lifecycle, validation, traversal state machine, and graph mutation commands (CRUD + undo/redo)
- `fireside-tui`: Ratatui rendering and interaction (present mode + edit mode)
- `fireside-cli`: thin command-line entrypoint that launches/dispatches to app modes

This plan also renames Rust vocabulary to protocol terms (Slide→Node, SlideDeck→Graph, Navigation→Traversal), aligns JSON wire format (`kebab-case` + `kind` discriminator), and assumes the protocol model lives under `models/`.

---

## Phase 1: Workspace Scaffolding

1. **Convert root `Cargo.toml` to a virtual workspace manifest.**
   - Remove root `[package]` sections
   - Add:
     - `[workspace]` with `resolver = "3"`
     - `members = ["crates/*"]`
     - optional `default-members = ["crates/fireside-cli"]`
     - `[workspace.package]` shared metadata (`version`, `edition`, `license`, `repository`, `rust-version`)
     - `[workspace.dependencies]` for shared versions (`serde`, `serde_json`, `thiserror`, `anyhow`, `tracing`, `clap`, etc.)
     - `[workspace.lints]` for consistent lint policy

2. **Create crate manifests.**
   - `crates/fireside-core/Cargo.toml` (lib)
   - `crates/fireside-engine/Cargo.toml` (lib, depends on `fireside-core`)
   - `crates/fireside-tui/Cargo.toml` (lib, depends on `fireside-core` + `fireside-engine` + Ratatui stack)
   - `crates/fireside-cli/Cargo.toml` (bin, depends on `fireside-tui`, and optionally direct access to engine for non-TUI commands)

3. **Keep `models/` as-is.**
   - The `typespec/` → `models/` rename is already complete
   - Only update docs paths if there are unavoidable build references from workspace moves

---

## Phase 2: `fireside-core` Responsibilities (Protocol Surface)

4. **Create core protocol model modules.**
   - `src/model/graph.rs`: `Graph`, `GraphMeta`, `NodeDefaults`
   - `src/model/node.rs`: `Node`, `NodeId`
   - `src/model/content.rs`: `ContentBlock` enum with `kind` discriminator
   - `src/model/branch.rs`: `BranchPoint`, `BranchOption`
   - `src/model/layout.rs`: protocol `Layout` variants
   - `src/model/transition.rs`: protocol `Transition` variants
   - `src/model/traversal.rs`: per-node `Traversal` overrides

5. **Align serialization format to protocol.**
   - Use `#[serde(rename_all = "kebab-case")]` where applicable
   - Use `#[serde(tag = "kind", rename_all = "kebab-case")]` for content blocks
   - Ensure docs/examples fixtures round-trip with protocol-compatible JSON

6. **Keep core crate UI-agnostic.**
   - No Ratatui dependencies in `fireside-core`
   - Keep theming/UI color model out of core

7. **Core error surface.**
   - `src/error.rs` for format/model-level errors only (deserialization, malformed fields, unsupported variants)

8. **Core unit tests.**
   - Serde round-trip tests
   - Backward-compat/parsing tests (if needed)
   - Type-level invariant tests (ID constraints, required fields)

---

## Phase 3: `fireside-engine` Responsibilities (State + Rules)

9. **Move lifecycle and validation into engine.**
   - `src/loader.rs`: `load_graph(path)`, `load_graph_from_str(json)` returning core models
   - `src/validation.rs`: graph integrity validation (dangling refs, duplicate IDs, missing start, etc.)

10. **Traversal state machine in engine.**

- `src/traversal.rs`: `TraversalEngine` implementing `Next`, `Choose`, `Goto`, `Back`
- Maintain navigation history stack
- Return typed traversal errors for invalid operations

11. **Mutable session model for editor support.**

- `src/session.rs`: `PresentationSession` containing current graph + traversal state + dirty flag
- Session is the single source of truth used by TUI modes

12. **Graph mutation command API (for edit mode).**

- `src/commands.rs`: command types like:
  - `UpdateNodeContent`
  - `AddNode`, `RemoveNode`
  - `AddBranchOption`, `UpdateBranchOption`, `RemoveBranchOption`
  - `SetTraversalNext`, `ClearTraversalNext`
- `apply(command)` validates and mutates session atomically

13. **Undo/redo in engine (not in TUI).**

- Maintain command history and inverse ops in engine
- Expose `undo()` and `redo()` APIs so all frontends get this behavior

14. **Engine integration tests.**

- Traversal legality tests
- Validation behavior tests
- Mutation + undo/redo tests
- Session consistency tests after mixed present/edit commands

---

## Phase 4: `fireside-tui` Responsibilities (Rendering + UX)

15. **Move Ratatui UI/render modules here.**

- `src/app.rs`, `src/event.rs`, `src/render/*`, `src/ui/*`, `src/config/*`, `src/design/*`
- Use `fireside-engine::PresentationSession` for all data reads/writes

16. **Two primary app modes.**

- `AppMode::Present`: traversal-focused UI
- `AppMode::Edit`: editor-focused UI that mutates current session graph via engine commands

17. **Editor UX scope (MVP in TUI).**

- Node list panel + active node content panel
- Inline field editing (title/text/code/list items)
- Branch/traversal editing controls
- Save/discard prompt for dirty session
- Keyboard-driven mode switch and command palette/help hints

18. **No business rules in TUI.**

- TUI dispatches commands to engine
- TUI renders engine state and errors
- Validation and traversal legality remain engine-owned

19. **TUI integration tests.**

- Simulated input sequences for present/edit switching
- Error display for invalid edits
- Ensure editor commands update rendered node state

---

## Phase 5: `fireside-cli` Responsibilities (Thin Binary)

20. **Create CLI crate as thin entrypoint.**

- Command parsing and dispatch only
- Keep business logic out of CLI crate

21. **Initial CLI commands.**

- `present <file>` → launches TUI in present mode
- `edit <file>` → launches TUI in edit mode
- `validate <file>` → run engine validation and print report

22. **CLI wiring.**

- Keep `main.rs` minimal (parse args, call into `fireside-tui` / `fireside-engine`)
- Optional non-interactive commands can call engine directly

---

## Phase 6: Migration Plan from Current Source Tree

23. **Migrate current `src/model/*` into `fireside-core` first.**

- Rename legacy types to protocol names during move
- Keep compatibility shims only if needed short-term

24. **Move current loader/validation/traversal logic into `fireside-engine`.**

- Refactor `App` to depend on engine API instead of local traversal state

25. **Move existing Ratatui rendering code into `fireside-tui`.**

- Keep theme and style types in TUI crate

26. **Split current CLI and runtime loop into `fireside-cli` + `fireside-tui`.**

- CLI crate owns `clap` interface
- TUI crate owns runtime loop and drawing

27. **Remove obsolete root `src/` once migration is complete.**

---

## Phase 7: Documentation and Examples

28. **Update `README.md` with workspace structure and commands.**

- `cargo build --workspace`
- `cargo test --workspace`
- `cargo run -p fireside-cli -- present docs/examples/hello.json`
- `cargo run -p fireside-cli -- edit docs/examples/hello.json`
- `cargo run -p fireside-cli -- validate docs/examples/hello.json`

29. **Update docs pages for architecture and contribution flow.**

- Clarify crate responsibilities
- Document where new features should go
- Document editor-mode architecture and command path (TUI → Engine)

30. **Update examples to protocol-aligned wire format.**

- `kebab-case` keys
- `kind` discriminator
- Include branching example that exercises traversal + editing

---

## Phase 8: Verification and Quality Gates

31. **Workspace verification.**

- `cargo check --workspace`
- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`

32. **Runtime smoke checks.**

- Present mode traversal (`Next`, `Choose`, `Goto`, `Back`)
- Edit mode mutation + save/discard
- Validation command output for valid/invalid documents

33. **Regression checks.**

- Ensure old examples either migrate or fail with clear errors
- Confirm docs build remains green (`docs` project)

---

## Final Responsibility Split (Authoritative)

- **`fireside-core`**: protocol types + serde/wire-format + minimal type invariants
- **`fireside-engine`**: validation, traversal rules, lifecycle loading, mutable session, command application, undo/redo
- **`fireside-tui`**: rendering, input mapping, app modes (`present`/`edit`), UX flow
- **`fireside-cli`**: command parsing and bootstrapping only

This keeps domain logic reusable for future frontends (web/editor/api), and keeps TUI/CLI thin and replaceable.

