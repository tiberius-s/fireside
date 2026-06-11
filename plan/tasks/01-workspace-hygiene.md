# Task 01 — Workspace manifest hygiene

**Depends on:** none
**Crates:** workspace root
**Phase:** 0

## Goal

Remove the two invalid manifest/config keys that cargo warns about on every build.

## Background

- Root `Cargo.toml` has a `[workspace.dev-dependencies]` table — not a valid Cargo key (cargo prints `unused manifest key: workspace.dev-dependencies`). The `pretty_assertions = "1"` inside it is silently ignored.
- `.cargo/config.toml` has `build.pipelined-compilation` — also unused (cargo warns).

## Steps

1. Delete the `[workspace.dev-dependencies]` table from the root `Cargo.toml`.
2. Find which crates actually use `pretty_assertions` (`grep -r pretty_assertions crates/*/Cargo.toml crates/*/src crates/*/tests`). Add `pretty_assertions = "1"` to the `[dev-dependencies]` of exactly those crates (or to `[workspace.dependencies]` + per-crate `pretty_assertions.workspace = true` if more than one crate uses it).
3. Remove the `build.pipelined-compilation` key from `.cargo/config.toml`. If the file becomes empty, leave the rest of the file untouched — only remove that key.

## Do NOT

- Touch any other dependency versions.
- Reformat either file beyond the removed lines.

## Acceptance

```bash
cargo build --workspace 2>&1 | grep -c "unused"   # must print 0
cargo test -p fireside-core                        # passes
```
