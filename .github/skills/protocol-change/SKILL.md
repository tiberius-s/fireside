---
name: protocol-change
description: 'Execute a complete, end-to-end additive protocol change for the Fireside format — from TypeSpec model edit through Rust structs, documentation, examples, and verification gates.'
---

# Protocol Change Skill

Execute a complete, end-to-end additive protocol change for the Fireside format — from
TypeSpec model edit through Rust structs, documentation, examples, and verification gates.

## When to Use

Invoke this skill for **any** change to the Fireside wire format:

- Adding a new `ContentBlock` variant
- Adding an optional field to an existing type
- Adding a new enum variant to `Layout`, `Transition`, `Traversal`, or `BranchOption`
- Adding a new top-level graph metadata field
- Adding a new `NodeDefaults` field

**Do not use for:**

- Removing or renaming existing fields (breaking change — requires protocol version bump and a full deprecation process)
- Changing the discriminator key or serde tagging strategy (architectural change — write an ADR first)
- Pure Rust implementation changes with no wire-format impact (use `refactor` skill instead)

---

## Protocol Stability Rule

All `0.1.x` changes **must** be additive:

- New fields on structs must be `Option<T>` or carry `#[serde(default)]`.
- New enum variants must not change the serialized form of existing variants.
- Round-trip tests must pass with existing `.json` files (no breaking deserialization).

If the proposed change is not additive, stop and consult the user before proceeding.

---

## The Five-Phase Cascade

```
Phase 1: TypeSpec model
    ↓
Phase 2: JSON Schema (generated — do not edit manually)
    ↓
Phase 3: Rust structs in fireside-core
    ↓
Phase 4: Documentation (spec + schema reference + example)
    ↓
Phase 5: Verification gates (all must pass before committing)
```

---

## Phase 1 — TypeSpec Model

File: `models/main.tsp`

1. Open `models/main.tsp` and locate the relevant model (`ContentBlock`, `Node`,
   `Graph`, `Traversal`, etc.).
2. Apply the change following existing TypeSpec patterns in the file.
3. New optional fields use `?: Type` syntax.
4. New enum members should be added at the **end** of the enum.
5. Add a `@doc` comment on any new field or variant.

**Do not touch** any file in `models/tsp-output/schemas/` — these are generated.

---

## Phase 2 — JSON Schema Compilation

```bash
cd models && npm run build
```

This regenerates all 18+ JSON Schema files under `tsp-output/schemas/`. Verify:

- The target schema file changed (inspect the diff — new field should appear).
- No other schema files changed unexpectedly.
- The generated schema uses kebab-case property names.

If compilation fails, fix the TypeSpec source and re-run before proceeding.

---

## Phase 3 — Rust Struct Update (`fireside-core`)

Crate: `crates/fireside-core/src/model/`

Module map for common targets:

| Change type                                                 | Rust file       |
| ----------------------------------------------------------- | --------------- |
| New `ContentBlock` variant                                  | `content.rs`    |
| New field on `Node`                                         | `node.rs`       |
| New field on `Graph`/`GraphFile`/`GraphMeta`/`NodeDefaults` | `graph.rs`      |
| New `Layout` variant                                        | `layout.rs`     |
| New `Transition` variant                                    | `transition.rs` |
| New `BranchOption`/`BranchPoint` field                      | `branch.rs`     |
| New `Traversal` field                                       | `traversal.rs`  |

### Serde conventions to follow

```rust
// Struct fields — use kebab-case rename
#[serde(rename_all = "kebab-case")]
pub struct MyType {
    pub my_field: Option<String>,        // optional field — deserializes from null/absent
}

// ContentBlock discriminated union
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ContentBlock {
    MyVariant(MyVariantBlock),  // kind: "my-variant" in JSON
}

// Enum variants
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    SplitHorizontal,   // "split-horizontal" in JSON
}
```

- Add `#[must_use]` to any new public function.
- Add `///` doc comments to every public item.
- No `unwrap()` or `expect()` in library code.
- If a new `ContentBlock` variant is added: also add its struct to `content.rs` and a
  `From` impl if appropriate.

### `Graph` index rebuild rule

If the change affects the node structure, call `Graph::rebuild_index()` after any
mutation to keep `node_index: HashMap<NodeId, usize>` consistent.

---

## Phase 4 — Documentation

### 4a — Spec updates

Location: `docs/src/content/docs/spec/`

| What changed                | Spec chapter to update                                             |
| --------------------------- | ------------------------------------------------------------------ |
| New `ContentBlock` variant  | `§2-data-model.md` (type table) and `appendix-c.md` (full catalog) |
| New `Node` or `Graph` field | `§2-data-model.md`                                                 |
| New traversal behavior      | `§3-traversal.md`                                                  |
| New validation rule         | `§4-validation.md`                                                 |
| New extension type          | `§5-extensibility.md`                                              |
| Wire format change          | `§6-serialization.md`                                              |

For each change, update:

- The **type vocabulary table** entry (if any)
- The **JSON example** within the spec page (use kebab-case, include `kind` discriminator)
- Any **normative statements** that reference the affected type

### 4b — Schema reference

Location: `docs/src/content/docs/schemas/`

If a new type was added, create `new-type.md` following the pattern of existing schema
reference files (frontmatter, summary, field table, JSON example, cross-links).

If an existing type gained a field, update the field table.

### 4c — Example file

File: `docs/examples/hello.json`

If the new field/variant can be demonstrated without breaking the existing example,
add it. The example must remain a valid graph that the engine can load and present.

---

## Phase 5 — Verification Gates

Run all five gates in order. **All must pass before committing.**

```bash
# Gate 1: Cargo format
cargo fmt --check

# Gate 2: Clippy (zero warnings)
cargo clippy --workspace -- -D warnings

# Gate 3: Tests (nextest preferred)
cargo nextest run --workspace
# fallback: cargo test --workspace

# Gate 4: Docs (no broken intra-doc links)
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Gate 5: Docs site build (no broken Astro/Starlight links)
cd docs && npm run build
```

If any gate fails: fix the failure before reporting completion. Do not skip gates.

---

## Checklist

Use this as a final review before marking the task done:

- [ ] `models/main.tsp` updated with `@doc` comments on new items
- [ ] `cd models && npm run build` succeeded cleanly
- [ ] Expected schema file(s) in `tsp-output/schemas/` updated (no unexpected changes)
- [ ] Rust struct(s) in `fireside-core/src/model/` updated with correct serde attributes
- [ ] New fields are `Option<T>` or have `#[serde(default)]`
- [ ] All public items have `///` doc comments
- [ ] No `unwrap()` or `expect()` added
- [ ] Round-trip test added to `crates/fireside-core/tests/` (serialize → deserialize → assert)
- [ ] Relevant spec page(s) updated
- [ ] Schema reference page updated (or created)
- [ ] `docs/examples/hello.json` updated if applicable
- [ ] All 5 verification gates pass

---

## Common Pitfalls

| Pitfall                                              | Fix                                                                             |
| ---------------------------------------------------- | ------------------------------------------------------------------------------- |
| Forgetting to recompile TypeSpec before editing Rust | Always run Phase 2 before Phase 3                                               |
| Adding a required field without a `default`          | Breaks all existing `.json` files — make it `Option` or add `#[serde(default)]` |
| camelCase field name in Rust                         | Apply `#[serde(rename_all = "kebab-case")]` to the struct                       |
| Editing generated schema files directly              | Run `npm run build` — your edits will be overwritten                            |
| Not rebuilding the graph index                       | After structural node mutations, call `Graph::rebuild_index()`                  |
| Forgetting `kind` in a JSON example                  | Required on every `ContentBlock` in docs and examples                           |
