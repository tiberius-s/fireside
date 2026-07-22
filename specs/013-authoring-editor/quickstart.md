# Quickstart: Validating the Authoring Editor

Validation scenarios proving the feature works end-to-end, layered per
Constitution VII's test discipline map (full detail in each wave's own
tasks; this is the runnable proof, not the implementation).

## Prerequisites

```sh
cd /Users/tiberius/Development/rust/fireside
cargo build --workspace
```

## Layer 1 — `engine::authoring` (pure, fastest feedback)

```sh
cargo test -p fireside-engine authoring::
```

Expect: unit tests per `Op` variant in `contracts/authoring-ops.md`, plus
the two proptests (`data-model.md`'s id/slug rename never dangles a
reference; SC-007's four unrepresentable-by-construction invariants hold
over arbitrary op sequences). Reuses the existing `arbitrary_*` proptest
helpers already in `fireside-engine/src/session.rs` as a model.

## Layer 2 — `hit()` and `EditorApp::update` (TestBackend, fast)

```sh
cargo test -p fireside-tui editor::
```

Expect: table-driven `hit()` tests per `contracts/hit-testing.md`;
`TestBackend` scenario tests per editor screen state, driving both real
`KeyEvent`s and synthetic `MouseEvent`s (press/move/release sequences for
every drag path — block reorder, outline reorder) through
`EditorApp::update`; `insta` snapshots for layouts;
`contains()`-style assertions for behavior contracts (spec's acceptance
bar items 2–5).

## Layer 3 — vocabulary gate (snapshot grep, spec FR-024)

```sh
cargo test -p fireside-tui render_suite_vocabulary_gate
```

Expect: every editor `insta` snapshot fixture fails the run if it matches
`\b(node|nodes|graph|traversal|kind|id)\b`, a raw `ContentBlock` kind
string (`ascii-art`, `container`, `divider`, …), or a `"`-quoted JSON key —
except the preview-fidelity fixture (exempt: it renders presenter output
only, which is already covered by the presenter's own existing gate-free
snapshots).

## Layer 4 — preview fidelity (spec SC-008, property test)

```sh
cargo test -p fireside-tui preview_fidelity
```

Expect: for every fixture deck, the editor canvas's at-rest render buffer
at a given `(width, height)` is byte-identical to `present`'s own render
buffer for the same slide at the same size — proves `SlideView`
(`research.md` §7) is a structural guarantee, not a discipline.

## Layer 5 — CLI e2e

```sh
cargo test -p fireside-cli --test cli_e2e edit
```

Expect: `fireside edit` argument parsing; non-tty refusal
(`contracts/cli-edit-command.md` precondition 1); unparseable-deck refusal
with the "Fix the file first" line (precondition 2); `.md` import hint
(precondition 3); create-if-missing flow (precondition 4).

## Layer 6 — real terminal (tmux smoke, per wave)

```sh
scripts/smoke.sh
```

Add editor scenarios alongside the existing presenter/quick-edit/dual-screen
ones (`scripts/smoke.sh`'s existing five scenarios are the model): open,
select a block, edit it via its form, save, confirm the file changed; a
block drag-reorder via injected SGR mouse sequences (`ESC [<0;x;yM` /
`m`, proven as a technique in wave E1); a forced-kill mid-edit followed by
reopen, confirming the draft-restore prompt appears.

**Flagship smoke** (spec SC-001/SC-002, wired in wave E3): the two scripted
10-minute walkthroughs — build a 5-slide deck with one branch and one
multi-step reveal, present it, save it — once mouse-only (SGR-injected),
once keyboard-only.

## Layer 7 — full verification

```sh
scripts/verify.sh
```

Mirrors every CI job (fmt, clippy, tests, tmux smoke, MSRV 1.88 check,
protocol schema drift, docs build). Per the design brief's wave
discipline: this must pass, plus the wave's tmux smoke must have run in a
real terminal, plus `graphify update .`, before a wave's Progress Log line
is ticked.

## Manual exploratory pass (before calling any wave done)

1. `cargo build --release -p fireside-cli && ./target/release/fireside edit docs/examples/hello.json`
2. Click a text block → confirm selection border + chip row appear, no more
   than ~7 controls visible before selecting anything (SC-003).
3. Edit its wording, confirm, `Ctrl+Z` → confirm exact prior wording
   returns.
4. Resize the terminal below 80×24 → confirm the single centered guard
   message, no overlapping panes.
5. `[ ▶ Present ]` → step through with `Space`/arrow keys → `q` → confirm
   you're back in the editor at the same selection.
6. Grep the whole session's on-screen text for `node`, `graph`, `kind`,
   `"id"` — should find none (spec FR-024, manually re-confirming Layer 3's
   automated gate).
