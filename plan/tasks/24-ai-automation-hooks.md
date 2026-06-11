# Task 24 — AI workflow: automate graphify + permissions

**Depends on:** none (parallel-safe)
**Crates:** none (.claude config only)
**Phase:** any (findings A7, A8)

## Goal

Stop relying on model memory for graph freshness, and give every contributor's Claude session the low-risk permission allowlist.

## Steps

1. **Graph freshness (A7)** — root `CLAUDE.md` asks the model to remember to run `graphify update .` after edits; hooks are reliable, memory is not. Add to `.claude/settings.json` a `PostToolUse` hook matching `Edit|Write` that runs `graphify update .` in the background when the edited file is a source file and `graphify-out/graph.json` exists. Pattern it on the existing `PreToolUse` hooks in the same file (they already do JSON parsing of `tool_input` via python3 — reuse that style). Debounce is unnecessary (`graphify update` is incremental, AST-only, no API cost), but redirect output to `/dev/null` and `&` it so edits never block.
2. **Permissions (A8)** — promote the stable read-only/low-risk entries from `.claude/settings.local.json` into the checked-in `.claude/settings.json` under `permissions.allow`:
   - `Bash(cargo test *)`, `Bash(cargo run *)`, `Bash(cargo build *)`, `Bash(cargo clippy *)`, `Bash(cargo fmt *)`
   - `Bash(node validate.mjs *)`, `Bash(graphify query *)`, `Bash(graphify explain *)`, `Bash(graphify path *)`
   - `mcp__context7__resolve-library-id`, `mcp__context7__query-docs`
   Leave session-specific entries (tmp paths, `rm`) in local settings. Do not allowlist anything destructive or network-mutating.
3. Verify the hook JSON is valid (`python3 -m json.tool .claude/settings.json`) and that a test edit triggers the update (check `graphify-out/` mtime).

## Do NOT

- Hook on every Bash call (only Edit/Write).
- Allowlist `cargo publish`, `git push`, `npm install`, or anything that mutates remote state.

## Acceptance

```bash
python3 -m json.tool .claude/settings.json > /dev/null && echo OK
# In a fresh Claude session: edit a comment in any .rs file; confirm graphify-out/manifest.json mtime updates.
```
