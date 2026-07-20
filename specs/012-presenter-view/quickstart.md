# Quickstart: Validating the Dual-Screen Presenter View

## Prerequisites

- `cargo build --workspace` (a release build matches the audit plan's own
  repro convention: `cargo build --release -p fireside-cli`, but debug is
  fine for this walkthrough).
- Two terminal windows/panes and a deck with speaker notes on at least one
  slide, a branch point, and a reveal sequence — `docs/examples/hello.json`
  already has traversal and branching; add a `speaker-notes` field and a
  `reveal` mark to a couple of blocks in a scratch copy if it doesn't cover
  all three (see data-model.md for the field shapes).

## 1. Start the presenter (User Story 1)

Terminal A:

```sh
cargo run -p fireside-cli -- /tmp/talk.fireside.json --fullscreen
```

Expect: the deck opens already in fullscreen view mode (the existing `f`
toggle's state, now set at launch — no manual key-press needed).

## 2. Start the follower (User Story 1)

Terminal B, same deck path:

```sh
cargo run -p fireside-cli -- notes /tmp/talk.fireside.json
```

Expect: within ~500ms it shows the current slide's title, its speaker
notes (or "No notes for this slide" if it has none — FR-012), the next
slide's title, reveal progress if the slide has any, and an elapsed timer
counting up. Confirm terminal A never displays any of the notes text
itself — it never has, but this feature must not change that (FR-002).

## 3. Navigate and reveal in the presenter (FR-003, SC-001)

In terminal A: advance (`Space`/`→`), go back (`←`), reveal a step if the
current slide has one. In terminal B: confirm each change is reflected
within ~500ms — title, notes, next-slide field, and (while on a reveal
slide) the `n/total revealed` counter all update together.

## 4. Reach a branch point (User Story 1, acceptance scenario 3)

In terminal A, advance to a node with a branch point. In terminal B:
confirm the "next" area switches from a single title to the list of
branch options (label + key), matching what terminal A's own branch menu
offers.

## 5. Reach the final slide (edge case, FR-013)

In terminal A, advance to a terminal node with no branch. In terminal B:
confirm it says plainly that this is the last slide, not an empty or
broken "next" field.

## 6. Kill the presenter (User Story 2, SC-002)

In terminal A, forcibly stop the process (`kill -9 <pid>` from a third
terminal, or close the terminal window without quitting cleanly). In
terminal B: confirm within ~2 seconds it switches to
`Presenter not running — start "fireside <deck>" in another window`.

## 7. Restart the presenter against the same follower (User Story 2)

Re-run terminal A's command from step 1 without restarting terminal B.
Expect: terminal B leaves the "not running" state and resumes tracking
within ~500ms of the new presenter's first tick, no restart needed on the
follower side.

## 8. Quit the presenter cleanly, then check the follower (User Story 2, acceptance scenario 3)

In terminal A, press `q` to quit normally. In terminal B: confirm it also
settles into the "not running" state (not a crash, not a frozen last
frame) within ~2 seconds — the session file is deleted on clean exit, so
this should in practice appear closer to immediate.

## 9. Live-edit the deck while both are open (User Story 3)

With both windows open and in sync, use quick-edit (`e` in the presenter,
on the current slide) to change its speaker notes text, and save
(`Ctrl+S`). In terminal B: confirm the notes text updates to the edited
version without restarting the follower (FR-006).

## 10. Non-tty guard (FR-010)

```sh
echo q | cargo run -p fireside-cli -- notes /tmp/talk.fireside.json
```

Expect: the same style one-line message the presenter itself gives for
piped stdio (P0-3's fix), not a raw panic/backtrace.

## Automated coverage

Steps 1–5 and 9 correspond to `fireside-tui/src/render/tests.rs`
TestBackend scenarios (follower states: notes, no-notes, branch, final
slide, live-edit reload) and `fireside-cli`'s `session.rs` unit tests
(read/write/staleness round-trips, mirroring `resume.rs`'s test style).
Steps 6–8 correspond to the tmux smoke extension in `scripts/smoke.sh`
(two panes, kill/restart/clean-quit assertions via `capture-pane`) — per
constitution Principle VII, TestBackend cannot observe cross-process
timing, so this step has no substitute. Step 10 corresponds to a new
`cli_e2e.rs` case mirroring `present_without_a_tty_gives_a_plain_message`.
Run:

```sh
cargo test --workspace
scripts/smoke.sh
```
