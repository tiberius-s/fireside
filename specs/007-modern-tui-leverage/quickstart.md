# Quickstart: validating Modern TUI Leverage

Prerequisites: `cargo build --workspace`, a terminal that supports mouse
reporting + OSC 8 + synchronized output for the full checks (iTerm2, WezTerm,
kitty, or a modern tmux ≥ 3.2 all qualify); a plain terminal for the
graceful-degradation checks.

## 1. Mouse — map and branch menu

```sh
cargo run -p fireside-cli -- present docs/examples/hello.json
```
1. Press `m` to open the map. Click a row other than the highlighted one →
   presenter jumps there (FR-001).
2. Navigate to a branch-point slide (or `docs/examples/branching.fireside.json`
   if present). Click an option → same effect as pressing its key (FR-002).
3. Confirm every existing keyboard control still works untouched (FR-003).

## 2. Resume

```sh
cargo run -p fireside-cli -- present docs/examples/hello.json
# navigate a few slides in, then kill the process (Ctrl+\ / kill -9), not `q`
cargo run -p fireside-cli -- present docs/examples/hello.json
# expect: reopens on the same slide
```
Then reach the deck's natural end via `q`-after-finish or navigating to a
terminal node and confirm a subsequent relaunch starts at slide 1 again
(FR-002 edge case). Confirm `--restart` forces slide 1 regardless (FR-007).
Confirm `fireside demo` never persists or reads a resume position (FR-009).

## 3. Synchronized output

```sh
cargo run -p fireside-cli -- present docs/examples/hello.json
```
Transition rapidly between slides (hold `Space`/arrow keys) in a
synchronized-output-capable terminal; confirm no visible tearing across ~50
transitions (SC-003). Repeat in a terminal without the capability (or
`TERM=dumb`-adjacent) and confirm presenting is unaffected.

## 4. OSC 8 hyperlinks

1. Author a small deck with a text block containing `[Fireside repo](https://example.invalid/fireside)`.
2. `fireside validate` it — confirm no warning for a well-formed URL, and a
   warning for a deliberately malformed one (e.g. `[bad](not a url)`).
3. Present it in a capable terminal — confirm the label is distinctly
   styled and cmd/ctrl-clickable to the right destination.
4. Present the same deck in an incapable terminal — confirm the label still
   reads as plain text with no visible escape codes.

## Real-terminal (tmux) smoke — required per Constitution Principle VII

Per the project's established practice ([[feedback-tmux-smoke-catches-timing-bugs]]),
every item above additionally needs a detached-tmux pass, not just
`TestBackend` scenario tests:

- Mouse clicks can be injected as raw SGR mouse escape sequences via
  `tmux send-keys -H <hex bytes>` (press: `1b 5b 3c 30 3b <col> 3b <row> 4d`),
  simulating a real click without a physical mouse.
- `tmux capture-pane -e` preserves escape sequences in the captured output,
  so the OSC 8 open/close bytes can be grepped for directly to confirm they
  were actually emitted (not just that the label text is present).
- Resume: kill the tmux pane's process mid-deck (not `q`), start a fresh
  pane against the same file, capture-pane, confirm the same slide's content
  is showing.
