# Plan: Fireside Improvement Initiative

All 7 areas from the analysis, scoped as one cohesive plan with phased execution. Protocol changes are additive (optional fields only, staying within 0.1.x). Functional tests span all layers. The tutorial gets a full chapter outline.

The plan is organised into 6 phases, ordered by dependency. Phases 1–2 can partially overlap. Phase 3 depends on Phase 1 (protocol changes propagate to tests). Phases 4–6 are independent of each other and can run in any order.

---

## Phase 1 — Protocol Enhancements (additive 0.1.x)

All fields are `Option` — no existing document breaks. TypeSpec first, then cascade.

**Steps**

1. Edit `models/main.tsp`: add three optional fields to the `Node` model: `title?: string`, `tags?: string[]`, `duration?: string` (ISO 8601 duration or free-form). All get `@doc` annotations.

2. Edit `models/main.tsp`: add `fireside-version?: Versions` to the `Graph` model. The `Versions` enum schema already exists at `models/tsp-output/schemas/Versions.json` — use it as the type. This gives engines a machine-readable protocol version to check.

3. Edit `models/main.tsp`: add `extensions?: ExtensionDeclaration[]` to the `Graph` model. Define a new `ExtensionDeclaration` model with `type: string` and `required?: boolean`. This lets documents declare which extension blocks they depend on.

4. Run `cd models && npm run build` to regenerate all 18+ JSON Schema files in `models/tsp-output/schemas/`.

5. Cascade to `fireside-core`: add `title`, `tags`, `duration` fields to the `Node` struct in `crates/fireside-core/src/model/node.rs`. Add `fireside_version` and `extensions` to `GraphFile` and `GraphMeta` in `crates/fireside-core/src/model/graph.rs`. All fields `Option` or `Vec` with `#[serde(default)]`.

6. Update `docs/src/content/docs/spec/data-model.md` and `docs/src/content/docs/schemas/node.md` to document the new fields. Update `docs/src/content/docs/schemas/graph.md` for `fireside-version` and `extensions`.

7. Update `docs/examples/hello.json` to include at least one node with `title` and a `fireside-version` on the graph root.

**Verification**

- `cd models && npm run build` — clean
- `cargo build && cargo test --workspace` — green
- `cd docs && npm run build` — clean

---

## Phase 2 — Build Speed & Dependency Cleanup

Low-risk, high-impact changes that make everything else faster.

**Steps**

1. Remove `serde_yaml` and `toml` from `[workspace.dependencies]` in the root `Cargo.toml` (confirmed unused by all four crates).

2. Create `.cargo/config.toml` with linker optimisation for macOS:

   ```toml
   [target.aarch64-apple-darwin]
   rustflags = ["-C", "link-arg=-ld_prime"]
   ```

3. Add dev-profile dependency optimisation to root `Cargo.toml`:

   ```toml
   [profile.dev.package."*"]
   opt-level = 2
   ```

4. Evaluate `syntect` feature flag swap: change `default-fancy` to `default-syntaxes` + `regex-fancy` (pure Rust, no C FFI). Test that syntax highlighting still works for Rust, Python, JSON, and Markdown code blocks. If quality is acceptable, commit the swap. If not, keep `default-fancy` and document the trade-off.

5. Add `cargo-nextest` to the CI/dev workflow. Document in root README under **Build & Test**:
   ```bash
   cargo nextest run --workspace  # parallel test runner
   ```

**Verification**

- `cargo build` — confirm `serde_yaml`/`toml` no longer compile
- `cargo test --workspace` — green
- Compare `time cargo build` before and after the linker + profile changes on a clean build

---

## Phase 3 — Reference Implementation Fixes & Optimisations

**Steps**

1. **Fix `node_index` staleness bug.** Add a `pub fn rebuild_index(&mut self)` method to `Graph` in `crates/fireside-core/src/model/graph.rs`. It clears and reconstructs `node_index` from `self.nodes`. Call `rebuild_index` at the end of every `apply_command` variant that adds, removes, or reorders nodes in `crates/fireside-engine/src/commands.rs` (`AddNode`, `RestoreNode`, `RemoveNode`).

2. **Cache `syntect` assets with `LazyLock`.** In `crates/fireside-tui/src/render/code.rs`, replace the per-call `two_face::syntax::extra_newlines()` and `two_face::theme::extra().into()` with `static SYNTAX_SET: LazyLock<SyntaxSet>` and `static THEME_SET: LazyLock<ThemeSet>`. The `highlight_code`, `available_languages`, and `available_themes` functions all reference the statics instead of constructing fresh sets.

3. **Add `needs_redraw` flag.** Add a `needs_redraw: bool` field to `App` in `crates/fireside-tui/src/app.rs`. Set it to `true` at the top of `update()`. In the event loop in `crates/fireside-cli/src/commands/session.rs`, only call `terminal.draw()` when `app.needs_redraw` is true, and clear the flag after drawing. Also set it on `Resize` events.

4. **Cap traversal history.** Add `const MAX_HISTORY: usize = 256;` to `TraversalEngine` in `crates/fireside-engine/src/traversal.rs`. Convert `history: Vec<usize>` to `history: VecDeque<usize>`. After every `self.history.push_back(from)`, check `if self.history.len() > MAX_HISTORY { self.history.pop_front(); }`.

**Verification**

- `cargo test --workspace` — green, specifically the new `rebuild_index` tests (added in Phase 5)
- `cargo clippy --workspace -- -D warnings` — clean
- Manual: open a large graph, add/remove nodes in editor, confirm undo/redo still works and `node_by_id` returns correct results

---

## Phase 4 — Security Hardening

**Steps**

1. **Sanitize image paths.** In `local_image_path` in `crates/fireside-tui/src/render/markdown.rs`:
   - After resolving the path (whether relative or `file://`), call `.canonicalize()`.
   - If a `base_dir` is set, verify that the canonicalized path starts with `base_dir.canonicalize()`. If it doesn't, return `None`.
   - Reject paths containing `..` components _before_ canonicalization as a defence-in-depth measure.
   - Log a `tracing::warn!` when a path is rejected so users can diagnose loading failures.

2. **Add plist file size check.** In the iTerm2 import path in `crates/fireside-tui/src/design/iterm2.rs`, check the file size before parsing. Reject files > 1 MB with a clear error message — legitimate `.itermcolors` files are typically 2–5 KB.

3. **Document extension payload safety.** In `docs/src/content/docs/spec/extensibility.md`, add a normative `Security Considerations` section stating: extension payloads are data, not executable code. Engines MUST NOT evaluate, compile, or execute payload values. Payloads SHOULD be validated against a schema specific to the extension type before rendering.

4. **Add `EngineError::PathTraversal` variant** to `crates/fireside-engine/src/error.rs` to give the image sanitization a proper typed error for callers to handle.

**Verification**

- Write a unit test for `local_image_path` with inputs: `"../../../etc/passwd"`, `"/etc/passwd"` (absolute, no base_dir), `"valid-image.png"` (relative, with base_dir). Assert the first two return `None` when a base_dir is set.
- `cargo test --workspace` — green
- `cd docs && npm run build` — clean (spec page updated)

---

## Phase 5 — Functional Tests

**Steps**

1. **`fireside-core` round-trip tests.** Create `crates/fireside-core/tests/content_roundtrip.rs`. One `#[test]` per `ContentBlock` variant: construct, serialize to JSON string, deserialize back, assert equality. Include edge cases: `ListItem` bare-string form, `Extension` with nested `fallback`, `Container` with children.

2. **`fireside-engine` fixture tests.** Create `crates/fireside-engine/tests/fixtures/` with JSON files:
   - `valid_linear.json` — 5 sequential nodes, no traversal overrides
   - `valid_branching.json` — branching with `BranchPoint`, `after` rejoin
   - `invalid_dangling_ref.json` — `traversal.next` points to non-existent ID
   - `invalid_empty.json` — no nodes
   - `invalid_duplicate_id.json` — two nodes share an ID

   Create `crates/fireside-engine/tests/validation_fixtures.rs`: load each fixture, run `validate_graph`, and assert the expected diagnostics (count, severity, message substring).

3. **`fireside-engine` command history invariant test.** Create `crates/fireside-engine/tests/command_history.rs`:
   - Load `valid_linear.json`, snapshot `graph.nodes.clone()`.
   - Apply a sequence: `AddNode`, `UpdateNodeContent`, `RemoveNode`.
   - Undo all three.
   - Assert `graph.nodes == snapshot`.
   - This also serves as a regression test for the `rebuild_index` fix.

4. **`fireside-cli` end-to-end tests.** Add `assert_cmd` and `predicates` as dev-dependencies to `crates/fireside-cli/Cargo.toml`. Create `crates/fireside-cli/tests/cli_e2e.rs`:
   - `validate_hello_exits_zero` — runs `fireside validate ../../docs/examples/hello.json`, asserts exit 0 and stdout contains `"is valid"`.
   - `validate_missing_file_exits_nonzero` — runs `fireside validate nonexistent.json`, asserts non-zero exit.
   - `new_scaffolds_file` — runs `fireside new test-talk --dir <tmpdir>`, asserts the file exists and is valid JSON.
   - `new_project_scaffolds_directory` — runs `fireside new test-course --project --dir <tmpdir>`, asserts directory structure with `fireside.json`.

5. **`fireside-tui` render smoke extension.** Extend `crates/fireside-tui/tests/hello_smoke.rs`: after constructing `App`, call `render_node_content_with_base` for each node and assert the returned `Vec<Line>` is non-empty. This catches regressions where a content block variant produces no output.

**Verification**

- `cargo test --workspace` — all new tests pass
- `cargo nextest run --workspace` — same, faster

---

## Phase 6 — Documentation & Tutorial

**Steps**

1. **Theme authoring guide.** Create `docs/src/content/docs/guides/theme-authoring.md`. Cover: JSON theme file structure, all `Theme` fields with default values, supported color formats (named, hex, `reset`), the `syntax_theme` field and available syntect theme names, iTerm2 import via `fireside import-theme`, and theme resolution order (CLI flag > document metadata > user config > default).

2. **Extension authoring guide.** Create `docs/src/content/docs/guides/extension-authoring.md`. Cover: the `"kind": "extension"` wire format, required `type` field naming convention (reverse-domain), the `fallback` contract, the `payload` shape, how engines should discover and render extensions, and the new `extensions` declaration array on `Graph` (from Phase 1).

3. **Keybinding reference.** Create `docs/src/content/docs/reference/keybindings.md`. Two sections (Presentation, Editor) with full tables matching the keybindings defined in `crates/fireside-tui/src/config/keybindings.rs`. Add a third section for Go-To mode. Mark `keybindings.rs` as the canonical source.

4. **Migration guide placeholder.** Create `docs/src/content/docs/spec/migration.md` with a stub explaining that 0.1.x is the initial protocol baseline; all changes within 0.1.x are additive. Include a section header for "0.1.0 → future" to be filled when breaking changes are proposed.

5. **Tutorial series: "Learn Rust with Fireside"** — full outline.

   Create `docs/src/content/docs/guides/learn-rust/` as a directory with an index page and 8 chapter files:

   | Chapter | File                     | Title                                 | Key Rust concept                              | Source anchor                               |
   | ------- | ------------------------ | ------------------------------------- | --------------------------------------------- | ------------------------------------------- |
   | 0       | `_index.md`              | Series overview                       | —                                             | —                                           |
   | 1       | `01-data-model.md`       | Your First Data Model                 | `struct`, `enum`, `#[derive]`, `Option<T>`    | `model/content.rs` — `ContentBlock`         |
   | 2       | `02-errors.md`           | Errors That Help                      | `Result<T,E>`, `?`, `thiserror`               | `error.rs` in core and engine               |
   | 3       | `03-ownership.md`        | Ownership, Borrowing, and Collections | `Vec`, `HashMap`, `&` vs `&mut`, `clone` cost | `model/graph.rs` — `Graph::from_file`       |
   | 4       | `04-traits.md`           | Traits and Polymorphism               | `trait`, `impl Trait`, `Display`, `From`      | `CoreError: Display + Error`                |
   | 5       | `05-custom-serde.md`     | When Derive Isn't Enough              | Manual `Deserialize`, visitor pattern         | `model/content.rs` — `ListItem`             |
   | 6       | `06-state-machines.md`   | State Machines                        | Encapsulation, `enum` as state, `#[must_use]` | `traversal.rs` — `TraversalEngine`          |
   | 7       | `07-command-pattern.md`  | Undo/Redo with the Command Pattern    | `Clone` trade-offs, pre-computed inverses     | `commands.rs` — `Command`, `CommandHistory` |
   | 8       | `08-tea-architecture.md` | The Elm Architecture in Rust          | Separating state from view, testability       | `app.rs` + `event.rs` — TEA loop            |

   **Per-chapter structure (required for each chapter):**
   - **Learning objectives** — 3–4 bullet points stating what the reader will be able to do.
   - **Concept introduction** — explain the Rust concept in isolation (400–600 words).
   - **Fireside walkthrough** — examine the real Fireside source with annotated code snippets and "why did they do it this way?" callouts.
   - **Exercise** — a small coding task the reader applies to their clone of the repo.
   - **Verification** — a `cargo test` command or assertion they can run to check their work.
   - **What would break if…** — a deliberately wrong approach, showing the compiler error and explaining it.
   - **Key takeaways** — 3–5 sentence summary.

   **Chapter 1 detail (representative):**
   - Objectives: define a struct, use derive macros, understand `Option` vs required fields, serialize with serde.
   - Walkthrough: open `ContentBlock`, explain `#[serde(tag = "kind", rename_all = "kebab-case")]`, show how `Heading { level: u8, text: String }` maps to `{"kind":"heading","level":1,"text":"Hello"}`.
   - Exercise: add an `Aside` variant to `ContentBlock` with a `body: String` field. Write a round-trip test.
   - Verification: `cargo test -p fireside-core content_roundtrip`.
   - What would break: remove `Serialize` from the derive list → show the compiler error about `ser::Serialize` not implemented.

   **Chapter 5 detail (hardest chapter — prerequisites: chapters 1–4):**
   - Objectives: understand when derive macros are insufficient, implement `Deserialize` manually, use the visitor pattern.
   - Walkthrough: show how `ListItem` accepts both `"hello"` and `{"text":"hello","children":[]}` — explain why this can't be done with `#[serde(untagged)]` alone (ambiguity, error quality).
   - Exercise: add an `InlineStyle` type that accepts both `"bold"` (bare string) and `{"style":"bold","color":"red"}`.
   - Verification: `cargo test -p fireside-core inline_style_roundtrip`.
   - What would break: try `#[serde(untagged)]` — show the panics or incorrect fallback behavior.

6. **Update Starlight sidebar config.** In `docs/astro.config.mjs`, add the new guide pages, the learn-rust directory, the keybinding reference page, and the migration guide to their respective sidebar sections.

**Verification**

- `cd docs && npm run build` — clean, new pages generate at expected URLs
- All internal links resolve (no 404s in build output)
- Total page count increases by ~14 (4 new guides + migration page + 9 tutorial pages + keybinding ref)

---

## Execution Order & Dependencies

```text
Phase 2 (Build speed)  ←── no dependencies, do first for faster iteration
    │
    ├── Phase 1 (Protocol)  ←── TypeSpec → schema → core → docs cascade
    │       │
    │       └── Phase 3 (Optimisations)  ←── rebuild_index fix depends on
    │               │                        core changes settling
    │               └── Phase 5 (Tests)  ←── tests validate all fixes
    │
    ├── Phase 4 (Security)  ←── independent, can overlap with 1/3
    │
    └── Phase 6 (Docs & Tutorial)  ←── depends on Phase 1 for protocol
                                       docs; tutorial is independent
```

## Task IDs

| Task ID | Phase | Title                                                                |
| ------- | ----- | -------------------------------------------------------------------- |
| TASK007 | 2     | Build speed and dependency cleanup                                   |
| TASK008 | 1     | Protocol enhancements (Node fields, version, extensions declaration) |
| TASK009 | 3     | Reference implementation optimisations                               |
| TASK010 | 4     | Security hardening                                                   |
| TASK011 | 5     | Functional test suite                                                |
| TASK012 | 6     | Documentation gaps and tutorial series                               |

## Key Decisions

- Protocol changes are additive (0.1.x) — no breaking changes, all new fields optional
- `BranchOption.description` already exists in TypeSpec — no action needed in Phase 1
- `syntect` feature swap is evaluate-then-commit — if highlighting quality degrades, keep `default-fancy`
- Tutorial series gets full chapter outlines with exercises, not just a stub
- `VecDeque` preferred over `Vec::remove(0)` for history capping (O(1) vs O(n))
- Tests use `assert_cmd` for CLI e2e — the standard Rust approach for binary testing
