# Quickstart: validating incremental reveal

## Prerequisites

- `cargo build --workspace`
- `cd protocol && npm install` (once) for `validate.mjs`/fixtures

## Scenario 1 — bullets reveal one at a time (US1)

1. Write a small deck to `/tmp/reveal-demo.json`:
   ```json
   {
     "nodes": [{
       "id": "bullets",
       "content": [
         { "kind": "heading", "level": 2, "text": "Roadmap" },
         { "kind": "list", "items": ["Always visible intro line"] },
         { "kind": "text", "body": "First reveal", "reveal": 1 },
         { "kind": "text", "body": "Second reveal", "reveal": 2 },
         { "kind": "text", "body": "Also second reveal (grouped)", "reveal": 2 }
       ]
     }]
   }
   ```
2. `cargo run -p fireside-cli -- present /tmp/reveal-demo.json`
3. Expect: only "Always visible intro line" shown, plus a footer badge
   reading something like "0/2 revealed".
4. Press Space: "First reveal" appears; badge reads "1/2 revealed".
5. Press Space again: both "Second reveal" lines appear together (shared
   step); badge disappears (nothing left to reveal); this node is
   terminal (no traversal), so pressing Space once more reports end of
   path — confirms reveal fully precedes the terminal check (FR-008).

## Scenario 2 — gap in numbering never wastes a keypress (FR-004)

Same as Scenario 1 but change `"reveal": 2` to `"reveal": 5` on both
lines. Expect identical behavior — two presenter-visible steps, no dead
third press — proving steps are ordinal over distinct values, not raw
magnitudes.

## Scenario 3 — reveal blocks a branch choice (US1 edge case, FR-007)

1. Add a `traversal.branch-point` to the node above with two options.
2. While reveal is pending (before all steps shown), press `1`/`2`/Enter
   (the branch-selection keys): expect the branch is NOT selected —
   instead the keypress continues revealing (same as Space).
3. Once all reveal steps are shown, the same keys now select a branch
   option normally.

## Scenario 4 — columns compose correctly with reveal (US2)

1. Build a node with a `container { layout: "columns" }` of two children,
   one plain, one `"reveal": 1`.
2. Render at first entry: expect the plain column alone, using the full
   available width (not half-width with an empty second slot).
3. Reveal step 1: expect both columns now side by side.
4. Verify at 80×24 via a `TestBackend` scenario test and, since this
   changes real keypress-driven rendering, via a tmux smoke walk.

## Scenario 5 — validator catches a masked reveal (US3, FR-012)

1. `node protocol/validate.mjs <fixture-with-container-reveal-2-child-reveal-1.json>`
2. Expect a `reveal-masked-by-container` warning naming the child block.
3. Run the equivalent check through `fireside-engine::validate` (via
   `cargo test -p fireside-engine` covering the new unit test) and confirm
   the same rule id fires — parity, per the existing fixture-corpus
   pattern from `specs/004-spec-patch-0-1-1/`.

## Regression check

`cargo test --workspace` and `cargo clippy --workspace --all-targets` MUST
stay green/silent; `docs/examples/hello.json` (no reveal marks) MUST
present identically before and after — diff a tmux capture if in doubt.
