# Quickstart: validating the ASCII art content block

## Prerequisites

- `cargo build --workspace`
- `cd protocol && npm install` (once) for `validate.mjs`/fixtures
- A small local PNG for the image-conversion scenario (any tiny image
  works, e.g. `crates/fireside-tui/src/render/snapshots/` has none
  reusable — generate one ad hoc if needed).

## Scenario 1 — banner generation (US1)

1. `cargo run -p fireside-cli -- art text "Fireside"`
2. Expect multi-line stylized banner text on stdout, exit code 0.
3. Paste the output into an `ascii-art` block's `art` field in a scratch
   deck (see `contracts/ascii-art-block.md` for the wire shape) and
   `cargo run -p fireside-cli -- present <file>`: expect the banner
   centered, sized to its own width, not stretched full-width.
4. Repeat with a phrase containing an unsupported character mixed with
   supported ones (e.g. `"Fireside 🔥"`); expect output for the
   recognized characters, not a hard failure (FR-013).

## Scenario 2 — image conversion (US2)

1. `cargo run -p fireside-cli -- art image /path/to/small.png`
2. Expect multi-line plain-text ASCII shading on stdout, exit code 0.
3. `cargo run -p fireside-cli -- art image /path/to/missing.png`
4. Expect a clear error on stderr, non-zero exit, no panic (FR-014).

## Scenario 3 — hand-typed art and reveal composition (US3)

1. Build a node with a hand-typed `ascii-art` block (not generator
   output) alongside other content, one of them `reveal: 1`.
2. Present: confirm the reveal-marked block is fully absent (no reserved
   space) until reveal step 1, then appears with every line at once, not
   progressively — verify via a `TestBackend` scenario test and a tmux
   smoke (constitution Principle VII, UI-change requirement).

## Scenario 4 — validator catches oversized/empty art (US4)

1. `node protocol/validate.mjs <fixture-with-90-col-ascii-art.json>`
2. Expect an `ascii-art-too-wide` warning naming the block's node.
3. `node protocol/validate.mjs <fixture-with-empty-ascii-art.json>`
4. Expect an `ascii-art-empty` warning naming the block's node.
5. Run the equivalent checks through `fireside-engine::validate` (new
   unit tests) and confirm the same rule ids fire — parity, per the
   fixture-corpus pattern already enforced in CI by B-2
   (`node protocol/run-fixtures.mjs`).

## Scenario 5 — compatibility break is real and clear (spec Edge Cases, FR-011)

1. Write a document containing `{"kind":"ascii-art","art":"x"}` before
   `protocol/main.tsp`/`fireside-core` are updated (or, after landing,
   temporarily check out `fireside-core` at the pre-0.1.3 commit).
2. `cargo run -p fireside-cli -- validate <file>` (or `present`).
3. Expect the whole document to be rejected with a clear "unknown
   variant `ascii-art`" message (verified directly during Phase 0
   research — see `research.md` §2), not a silent skip or partial render.

## Regression check

`cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D
warnings` MUST stay green/silent; `docs/examples/hello.json` (no
`ascii-art` blocks) MUST present identically before and after — diff a
tmux capture if in doubt; `node protocol/run-fixtures.mjs` and `node
protocol/validate.mjs ../docs/examples/hello.json` MUST stay clean (B-2's
CI gate already enforces both on every push).
