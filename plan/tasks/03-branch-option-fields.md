# Task 03 — BranchOption: optional string key + description (D11)

**Depends on:** 02
**Crates:** fireside-core, fireside-engine, fireside-tui
**Phase:** 1

## Goal

Match `protocol/tsp-output/schemas/BranchOption.json`: `label` and `target` required; `key` an **optional string**; `description` an optional string.

## Background

`crates/fireside-core/src/model/branch.rs:29` declares `key: char` (required) and has no `description`. The spec example shows keys like `"a"`, `"1"` — strings. `BranchPoint` also carries an extra `id` field not in the spec; keep it for now (it is removed/decided in Task 19's ADR — do not remove it here).

## Steps

1. In `branch.rs`: change `key: char` → `key: Option<String>` (skip serializing if `None`); add `description: Option<String>` (skip if `None`).
2. In `crates/fireside-engine/src/traversal.rs`, `choose(key: char, ...)`: match options where `option.key.as_deref() == Some(key.to_string().as_str())` — or change the signature to `choose(&str)`; prefer `&str` and update callers in `fireside-tui` (`app/action_routing.rs` presenter branch keys).
3. In the TUI branch UI (`crates/fireside-tui/src/ui/branch.rs`), render the key hint only when `key` is `Some`; for options without a key, selection by arrow/enter must still work. Render `description` as dimmed text under the label using `DesignTokens` styles — no hardcoded colors.
4. Update tests that construct `BranchOption` (engine traversal tests, validation tests, TUI tests).

## Do NOT

- Remove `BranchPoint.id` (Task 19 decision).
- Add numeric auto-keys (engine extras need an ADR first).

## Acceptance

```bash
cargo test --workspace
cargo run -q -p fireside-cli -- validate docs/examples/hello.json   # exit 0
```
