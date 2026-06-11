# Task 18 — TUI: traversal UX polish + golden tests

**Depends on:** 17
**Crates:** fireside-tui
**Phase:** 4

## Goal

Make the new explicit-edge traversal model *legible* in the terminal: the presenter should always know why they can or cannot move, and every such state is locked in by a golden test.

## UX requirements (all styling via `DesignTokens`; reuse existing chrome/flash patterns)

1. **Branch point**: the options list (`ui/branch.rs`) renders with the focused option highlighted, key hints (`[a]`) in the accent style, and `description` (Task 03) as a dimmed second line. The footer shows `a–c choose · ↑↓ focus · Enter select`.
2. **Blocked next** (Task 09's `Blocked`): flash "Choose an option to continue" — and visually pulse/highlight the options block once (a single restyle on the next frame is enough; no animation system).
3. **Terminal node**: footer swaps the `→ next` hint for `End of path · Backspace back · g goto`. No flash spam on repeated presses.
4. **History/breadcrumb** (`ui/breadcrumb.rs`): shows the actual visited path (NodeId history from Task 10), truncated from the left with `…` when too wide; current node in accent style.
5. **Footer consistency audit**: every mode (presenter, branch, editor, goto overlay, help) lists only the keys currently valid, same ordering convention (movement · action · meta), same separator. One helper builds footer spans — if hints are currently assembled ad hoc per screen, consolidate into one function in `ui/chrome.rs`.

## Golden tests

Restore/extend `tests/harness_golden.rs` using the existing harness:

- `hello_branch_choose_golden`, `hello_full_path_golden_ids` (must pass — they exercise hello.json end-to-end);
- new: blocked-next flash frame; terminal-node footer frame; columns layout frame for `layout-demo`.

## Do NOT

- Add mouse-only affordances (keyboard-first).
- Introduce new overlay types — reuse flash + footer.

## Acceptance

```bash
cargo test -p fireside-tui
# Manual: full hello.json walkthrough — intro → features → choose → each branch → thanks,
# confirming every state communicates its valid actions in the footer.
```
