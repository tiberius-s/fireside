#!/usr/bin/env bash
# CH-2: automated real-terminal smoke test, driving the release binary
# through a detached tmux session — the exact loop the 2026-07-19 UX audit
# ran by hand. Project rule (see memory): TestBackend snapshot tests can't
# catch reload/ordering/timing bugs the way a real terminal can, so this is
# the check that would have caught them.
#
# Covers, per CH-2's four scenarios: a demo walk, quick-edit save (P2-6:
# the "Saved" flash must survive the deck's own self-triggered reload), an
# externally-broken save refusing to reload, and resume-on-relaunch. Spec
# 012 (W4-DS-5) adds a fifth, two-pane scenario: a presenter and a
# `fireside notes` follower tracking each other live, and the follower
# going stale on both a kill and a clean quit — the cross-process timing
# TestBackend cannot observe.
#
# Usage: scripts/smoke.sh

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

if ! command -v tmux >/dev/null 2>&1; then
  echo "error: tmux is not installed (apt-get install tmux / brew install tmux)" >&2
  exit 1
fi

echo "==> cargo build --release -p fireside-cli"
cargo build --release -p fireside-cli
BIN="$(pwd)/target/release/fireside"

WORKDIR="$(mktemp -d)"
export XDG_STATE_HOME="$WORKDIR/state"
mkdir -p "$XDG_STATE_HOME"
SESSION="fireside-smoke-$$"

pass=0
fail=0

cleanup() {
  tmux kill-session -t "$SESSION" >/dev/null 2>&1 || true
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

# Ctrl+S is XOFF under a terminal's default flow-control settings, which
# would freeze the pane instead of reaching the app — disable it before
# fireside grabs raw mode.
start() {
  tmux kill-session -t "$SESSION" >/dev/null 2>&1 || true
  tmux new-session -d -s "$SESSION" -x 100 -y 30 "stty -ixon 2>/dev/null; exec $1"
  sleep 0.5
}

keys() {
  tmux send-keys -t "$SESSION" "$@"
}

pane() {
  tmux capture-pane -t "$SESSION" -p
}

wait_for() {
  local needle="$1" tries=25
  for _ in $(seq 1 "$tries"); do
    if pane | grep -qF "$needle"; then
      return 0
    fi
    sleep 0.2
  done
  return 1
}

assert_contains() {
  local label="$1" needle="$2"
  if wait_for "$needle"; then
    printf '  \033[1;32m\xe2\x9c\x93\033[0m %s\n' "$label"
    pass=$((pass + 1))
  else
    printf '  \033[1;31m\xe2\x9c\x97\033[0m %s \xe2\x80\x94 expected to see: %s\n' "$label" "$needle"
    echo "    --- pane contents ---"
    pane | sed 's/^/    | /'
    fail=$((fail + 1))
  fi
}

# Waits briefly, then fails if `needle` is still on screen — used to prove
# a mode switch (e.g. entering the embedded presenter) actually replaced
# the editor's chrome, not just left stale text sitting under it.
assert_not_contains() {
  local label="$1" needle="$2"
  sleep 0.4
  if pane | grep -qF "$needle"; then
    printf '  \033[1;31m\xe2\x9c\x97\033[0m %s \xe2\x80\x94 still showing: %s\n' "$label" "$needle"
    echo "    --- pane contents ---"
    pane | sed 's/^/    | /'
    fail=$((fail + 1))
  else
    printf '  \033[1;32m\xe2\x9c\x93\033[0m %s\n' "$label"
    pass=$((pass + 1))
  fi
}

# Injects a synthetic SGR mouse click (press + release, left button, mode
# 1006) directly into the pty's input stream at 1-based terminal
# coordinates (col, row) — crossterm's event parser reads these exactly as
# it would a real mouse event, so this exercises the same code path a
# physical click does without depending on the terminal emulator's own
# mouse-reporting support (spec 013, T026: the first use of this technique
# in this project's smoke suite).
mouse_click() {
  local col="$1" row="$2"
  tmux send-keys -t "$SESSION" -l -- $'\x1b[<0;'"${col}"';'"${row}"'M'
  tmux send-keys -t "$SESSION" -l -- $'\x1b[<0;'"${col}"';'"${row}"'m'
}

# Injects a synthetic SGR mouse *drag*: press at (col1,row1), a motion
# event with the left button still held at (col2,row2) — button code 32
# added to the press code per the SGR protocol's motion-report convention,
# always terminated with 'M' regardless of press/release — then release
# at (col2,row2). Exercises the same crossterm Down/Drag/Up event path a
# physical block drag would (spec 013, T048).
mouse_drag() {
  local col1="$1" row1="$2" col2="$3" row2="$4"
  tmux send-keys -t "$SESSION" -l -- $'\x1b[<0;'"${col1}"';'"${row1}"'M'
  sleep 0.1
  tmux send-keys -t "$SESSION" -l -- $'\x1b[<32;'"${col2}"';'"${row2}"'M'
  sleep 0.1
  tmux send-keys -t "$SESSION" -l -- $'\x1b[<0;'"${col2}"';'"${row2}"'m'
}

# ─── Two-pane helpers (spec 012: presenter + `fireside notes` follower) ──
# `$SESSION` gets a second pane via split-window; pane ids (`%N`) are
# stable handles independent of tmux's on-screen pane numbering.
start_dual() {
  tmux kill-session -t "$SESSION" >/dev/null 2>&1 || true
  tmux new-session -d -s "$SESSION" -x 200 -y 30 "stty -ixon 2>/dev/null; exec $1"
  tmux split-window -h -t "$SESSION" "stty -ixon 2>/dev/null; exec $2"
  sleep 0.5
  PRESENTER_PANE="$(tmux list-panes -t "$SESSION" -F '#{pane_id}' | sed -n '1p')"
  FOLLOWER_PANE="$(tmux list-panes -t "$SESSION" -F '#{pane_id}' | sed -n '2p')"
}

pane_of() {
  tmux capture-pane -t "$1" -p
}

wait_for_pane() {
  local pane_id="$1" needle="$2" tries=25
  for _ in $(seq 1 "$tries"); do
    if pane_of "$pane_id" | grep -qF "$needle"; then
      return 0
    fi
    sleep 0.2
  done
  return 1
}

assert_pane_contains() {
  local label="$1" pane_id="$2" needle="$3"
  if wait_for_pane "$pane_id" "$needle"; then
    printf '  \033[1;32m\xe2\x9c\x93\033[0m %s\n' "$label"
    pass=$((pass + 1))
  else
    printf '  \033[1;31m\xe2\x9c\x97\033[0m %s \xe2\x80\x94 expected to see: %s\n' "$label" "$needle"
    echo "    --- pane contents ---"
    pane_of "$pane_id" | sed 's/^/    | /'
    fail=$((fail + 1))
  fi
}

# ─── Scenario 1: demo walk ──────────────────────────────────────────────
echo
echo "=== demo walk ==="
start "$BIN demo"
assert_contains "demo shows the title slide" "Branching presentations, in your terminal."
keys " "
assert_contains "space advances to the next slide" "Everything is a block"
keys "q"
sleep 0.4
if ! tmux list-panes -t "$SESSION" >/dev/null 2>&1; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m q quits and the terminal is restored\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m q did not end the session\n'
  fail=$((fail + 1))
fi

# ─── Scenario 2: quick-edit save, and P2-6's Saved flash ───────────────
echo
echo "=== quick-edit save (P2-6: Saved survives the self-reload) ==="
(cd "$WORKDIR" && "$BIN" new "Smoke Talk" >/dev/null)
DECK="$WORKDIR/smoke-talk.fireside.json"
start "$BIN $DECK"
assert_contains "deck presents" "Smoke Talk"
keys "e"
assert_contains "quick-edit modal opens" "Quick edit"
keys "X"
keys "C-s"
assert_contains "save confirms" "Saved"
sleep 0.6
assert_contains "Saved is still shown after the deck's own reload" "Saved"

# ─── Scenario 3: broken save refuses to reload ──────────────────────────
echo
echo "=== broken external save is refused, not swallowed ==="
assert_contains "edited title is on screen" "XSmoke Talk"
printf 'not valid json' >"$DECK"
assert_contains "the reload guard refuses the broken file" "Reload failed"
assert_contains "the old, working slide is still on screen" "XSmoke Talk"
keys "q"
sleep 0.3

# ─── Scenario 4: resume-on-relaunch (P1-1) ──────────────────────────────
echo
echo "=== resume: quit mid-deck, relaunch, land on the same slide ==="
(cd "$WORKDIR" && "$BIN" new "Resume Talk" >/dev/null)
RDECK="$WORKDIR/resume-talk.fireside.json"
start "$BIN $RDECK"
assert_contains "deck presents" "Resume Talk"
keys " "
assert_contains "advanced to the branch point" "Decks can branch"
keys "q"
sleep 0.4
start "$BIN $RDECK"
assert_contains "relaunch resumes where it left off" "Resumed where you left off"
assert_contains "still on the branch point" "Decks can branch"
keys "q"
sleep 0.3

# ─── Scenario 5: dual-screen presenter view (spec 012) ──────────────────
echo
echo "=== dual-screen: presenter + fireside notes follower (spec 012) ==="
(cd "$WORKDIR" && "$BIN" new "Dual Screen Talk" >/dev/null)
DUALDECK="$WORKDIR/dual-screen-talk.fireside.json"
start_dual "$BIN $DUALDECK" "$BIN notes $DUALDECK"
assert_pane_contains "presenter shows the deck" "$PRESENTER_PANE" "Dual Screen Talk"
assert_pane_contains "follower tracks the title slide's notes" "$FOLLOWER_PANE" \
  "This is your title slide"

tmux send-keys -t "$PRESENTER_PANE" " "
assert_pane_contains "follower follows the presenter to the branch point" \
  "$FOLLOWER_PANE" "Pick a path"
assert_pane_contains "follower shows the branch options, not a single next title" \
  "$FOLLOWER_PANE" "Show me content blocks"

PRESENTER_PID="$(tmux list-panes -a -F '#{pane_id} #{pane_pid}' | awk -v p="$PRESENTER_PANE" '$1==p{print $2}')"
kill -9 "$PRESENTER_PID"
assert_pane_contains "follower goes stale within ~2s of a kill -9" \
  "$FOLLOWER_PANE" "Presenter not running"

tmux split-window -h -t "$SESSION" "stty -ixon 2>/dev/null; exec $BIN $DUALDECK"
sleep 0.5
PRESENTER_PANE="$(tmux list-panes -t "$SESSION" -F '#{pane_id}' | sed -n '2p')"
assert_pane_contains "follower reconnects to a relaunched presenter" \
  "$FOLLOWER_PANE" "Pick a path"

tmux send-keys -t "$PRESENTER_PANE" "q"
assert_pane_contains "follower goes stale on a clean quit too" \
  "$FOLLOWER_PANE" "Presenter not running"

tmux send-keys -t "$FOLLOWER_PANE" "q"
sleep 0.3
if ! tmux list-panes -t "$SESSION" >/dev/null 2>&1; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m follower q quits and the terminal is restored\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m follower q did not end the session\n'
  fail=$((fail + 1))
fi

# ─── Scenario 6: fireside edit — read-only studio + mouse clicks (spec 013) ──
echo
echo "=== fireside edit: read-only studio, SGR mouse clicks, present-and-return ==="
(cd "$WORKDIR" && "$BIN" new "Smoke Edit Talk" >/dev/null)
EDITDECK="$WORKDIR/smoke-edit-talk.fireside.json"
start "$BIN edit $EDITDECK"
assert_contains "studio opens showing the deck title" "Smoke Edit Talk"
assert_contains "outline lists the first two slides" "Welcome"
assert_contains "status line says the deck is ready" "ready to present"
assert_contains "hint line teaches click-to-select" "Click a slide or block to select"

# Outline row 2 ("Pick a path") sits at 0-indexed (col 2, row 2) given the
# 100x30 studio's fixed layout (toolbar row 0, outline starting row 1) —
# SGR coordinates are 1-based, so (3, 3).
mouse_click 3 3
assert_contains "clicking the outline (mouse) selects Pick a path" "A choice"

# A click inside the canvas card, well clear of any block — proves mouse
# events reach the canvas's hit-testing path without upsetting the studio.
mouse_click 41 9
assert_contains "canvas click leaves the studio in a healthy state" "ready to present"

# The Present chip sits in the toolbar's fixed right-aligned chip row —
# see hit::toolbar_chip_rects; at 100 columns it lands around column 61-74
# (0-indexed), so 1-based (66, 1) is well inside it.
mouse_click 66 1
assert_not_contains "presenting (via mouse) hides the editor's hint line" \
  "Click a slide or block to select"
keys "q"
assert_contains "q returns from the embedded presenter to the editor" \
  "Click a slide or block to select"

keys "q"
sleep 0.4
if ! tmux list-panes -t "$SESSION" >/dev/null 2>&1; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m editor q quits and the terminal is restored\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m editor q did not end the session\n'
  fail=$((fail + 1))
fi

# ─── Scenario 7: fireside edit — US1 block editing, mouse then keyboard (spec 013) ──
echo
echo "=== fireside edit: select -> edit -> save via the mouse, then again via the keyboard only ==="
US1DECK="$WORKDIR/smoke-us1-talk.fireside.json"
cat >"$US1DECK" <<'JSON'
{
  "fireside-version": "0.1.0",
  "title": "Smoke US1 Talk",
  "nodes": [
    {
      "id": "intro",
      "title": "Welcome",
      "content": [
        {"kind": "heading", "level": 1, "text": "Hello there"},
        {"kind": "text", "body": "Original wording"}
      ]
    }
  ]
}
JSON
start "$BIN edit $US1DECK"
assert_contains "studio opens on the fixture deck" "Smoke US1 Talk"

# The text block ("Original wording") renders at row 14 of the fixed
# 100x30 studio layout for this exact fixture (spec 013 US3's "Goes to"
# wiring strip takes one row off the bottom of the canvas); SGR
# coordinates are 1-based. Clicking it selects it — the hint line's
# [ Edit ] chip follows.
mouse_click 35 14
assert_contains "clicking the text block selects it (mouse)" "Edit ]"

# The hint-line [ Edit ] chip sits at the studio's fixed bottom row.
mouse_click 3 30
assert_contains "clicking [ Edit ] opens the block's form" "Edit text"

# Typing is inherently a keyboard action even on the mouse-driven path —
# FR-005 asks for a form with explicit confirm/cancel, not mouse text
# entry. "Done"'s cell sits at a fixed offset inside the always-identical
# single-field Text form.
keys "X"
mouse_click 18 17
assert_not_contains "[ Done ] commits and closes the form" "Edit text"
assert_contains "the canvas shows the edited wording immediately" "XOriginal wording"

# The toolbar's [ Save ] chip sits at a fixed offset for this 100-column,
# untitled-dot-free deck title.
mouse_click 82 1
assert_contains "[ Save ] writes the file and flashes Saved" "Saved"
if grep -qF "XOriginal wording" "$US1DECK"; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m the saved file reflects the mouse-driven edit\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m the saved file was not updated\n'
  fail=$((fail + 1))
fi

# Repeat content-only, keyboard-only (spec 013 US1 acceptance scenario 5):
# Tab selects the next block (no mouse at all), Enter opens its form,
# Ctrl+S commits, a second Ctrl+S saves.
keys "Tab"
keys "Tab"
keys "Enter"
assert_contains "Tab, Tab, Enter opens the text block's form without the mouse" "Edit text"
keys "Y"
tmux send-keys -t "$SESSION" C-s
assert_not_contains "Ctrl+S commits and closes the form" "Edit text"
tmux send-keys -t "$SESSION" C-s
assert_contains "a second Ctrl+S saves" "Saved"
if grep -qF "YXOriginal wording" "$US1DECK"; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m the saved file reflects the keyboard-only edit\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m the keyboard-only edit was not saved\n'
  fail=$((fail + 1))
fi

keys "q"
sleep 0.4
if ! tmux list-panes -t "$SESSION" >/dev/null 2>&1; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m editor q quits and the terminal is restored\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m editor q did not end the session\n'
  fail=$((fail + 1))
fi

# ─── Scenario 8: fireside edit — US2 drag-reorder blocks (spec 013, T048) ──
echo
echo "=== fireside edit: drag-reorder two blocks via injected SGR mouse sequences ==="
US2DECK="$WORKDIR/smoke-us2-talk.fireside.json"
cat >"$US2DECK" <<'JSON'
{
  "fireside-version": "0.1.0",
  "title": "Smoke US2 Talk",
  "nodes": [
    {
      "id": "intro",
      "title": "Welcome",
      "content": [
        {"kind": "heading", "level": 1, "text": "Hello there"},
        {"kind": "text", "body": "Original wording"}
      ]
    }
  ]
}
JSON
start "$BIN edit $US2DECK"
assert_contains "studio opens on the fixture deck" "Smoke US2 Talk"

# Same fixture shape as scenario 7's US1DECK, so the same 100x30 layout
# applies: the level-1 heading (2 rendered lines: text + rule) spans rows
# 11-12, the text block sits at row 14 (scenario 7 already confirmed a
# click there selects it). Pressing anywhere on the heading and dragging
# past the text block's row drops it after the text block (spec FR-009:
# drag from anywhere on the block, not just a handle).
mouse_drag 35 12 35 14
sleep 0.3
mouse_click 82 1
assert_contains "[ Save ] writes the reordered deck" "Saved"

text_offset="$(grep -bo "Original wording" "$US2DECK" | head -1 | cut -d: -f1)"
heading_offset="$(grep -bo "Hello there" "$US2DECK" | head -1 | cut -d: -f1)"
if [[ -n "$text_offset" && -n "$heading_offset" && "$text_offset" -lt "$heading_offset" ]]; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m the saved file reflects the drag-reordered block order\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m the saved file did not reflect the new block order\n'
  fail=$((fail + 1))
fi

keys "q"
sleep 0.4
if ! tmux list-panes -t "$SESSION" >/dev/null 2>&1; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m editor q quits and the terminal is restored\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m editor q did not end the session\n'
  fail=$((fail + 1))
fi

# ─── Scenario 9: fireside edit — US3 flagship walkthrough, mouse-only (spec 013, T057) ──
echo
echo "=== fireside edit: build a slide, a choice, and a reveal step — mouse-only ==="
US3DECK="$WORKDIR/smoke-us3-talk.fireside.json"
cat >"$US3DECK" <<'JSON'
{
  "fireside-version": "0.1.0",
  "title": "Smoke US3 Talk",
  "nodes": [
    {
      "id": "intro",
      "title": "Welcome",
      "content": [
        {"kind": "heading", "level": 1, "text": "Hello there"},
        {"kind": "text", "body": "Original wording"}
      ]
    },
    {"id": "middle", "title": "Middle", "content": [{"kind": "text", "body": "middle content"}]},
    {"id": "end", "title": "End", "content": [{"kind": "text", "body": "end content"}]}
  ]
}
JSON
start "$BIN edit $US3DECK"
assert_contains "studio opens on the fixture deck" "Smoke US3 Talk"

# Select "Welcome" (outline row 1 — always the entry slide's row,
# regardless of any later structural edits) and turn it into a choice
# pointing at two of the deck's other slides, chosen by name from a
# picker — never a typed id anywhere in this flow.
mouse_click 5 2
assert_contains "selecting a slide shows its structural chips" "Turn into a choice"
mouse_click 30 30
assert_contains "the choice prompt opens" "Turn into a choice"
tmux send-keys -t "$SESSION" -l -- "To Middle"
mouse_click 15 19
assert_contains "the slide picker lists every slide by title" "Choose a slide"
mouse_click 14 14
assert_contains "the branch now names its first answer" "Branches to: Middle"

mouse_click 30 30
assert_contains "+ Add answer opens a second answer's prompt" "Add an answer"
tmux send-keys -t "$SESSION" -l -- "To End"
mouse_click 15 19
mouse_click 14 15
assert_contains "both named answers are wired" "Branches to: Middle, End"

# The toolbar's [ + Slide ] chip sits at a fixed offset regardless of
# outline state — add a slide via the toolbar rather than the outline's
# own "+ new slide" row so this step never depends on whether a
# "not linked yet" divider happens to be showing.
mouse_click 52 1
assert_contains "[ + Slide ] opens the title prompt" "New slide"
tmux send-keys -t "$SESSION" -l -- "Recap"
tmux send-keys -t "$SESSION" C-s
assert_contains "the new slide is added and selected" "Recap"

# Reselect "Welcome" (outline row 1, still deterministic) before touching
# its blocks — adding "Recap" moved the studio's selection to it.
mouse_click 5 2
# The text block sits at row 13 of the canvas for this exact fixture,
# stable regardless of the outline's own row count.
mouse_click 35 13
assert_contains "selecting the text block shows its Reveal chip" "Reveal: none"
mouse_click 32 30
assert_contains "the Reveal chip cycles to step 1" "Reveal: 1"

mouse_click 80 1
assert_contains "[ Save ] writes the deck" "Saved"
if grep -qF '"reveal": 1' "$US3DECK" && grep -qF '"branch-point"' "$US3DECK"; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m the saved file has the branch and the reveal step\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m the saved file is missing the branch or the reveal step\n'
  fail=$((fail + 1))
fi

mouse_click 65 1
assert_contains "presenting shows the reveal gate before the branch" "0/1 revealed"
keys " "
assert_contains "revealing shows the wording and both named answers" "Original wording"
assert_contains "the branch's answers render by name" "To Middle"
keys "q"
assert_contains "q returns to the editor (the block stays selected across present-and-return)" "ready to present"

keys "q"
sleep 0.4
if ! tmux list-panes -t "$SESSION" >/dev/null 2>&1; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m editor q quits and the terminal is restored\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m editor q did not end the session\n'
  fail=$((fail + 1))
fi

# ─── Scenario 10: fireside edit — US3 flagship walkthrough, keyboard-only (spec 013, T057) ──
echo
echo "=== fireside edit: build a slide, a choice, and a reveal step — keyboard-only ==="
US3KBDECK="$WORKDIR/smoke-us3-kb-talk.fireside.json"
cat >"$US3KBDECK" <<'JSON'
{
  "fireside-version": "0.1.0",
  "title": "Smoke US3 KB Talk",
  "nodes": [
    {
      "id": "intro",
      "title": "Welcome",
      "content": [
        {"kind": "heading", "level": 1, "text": "Hello there"},
        {"kind": "text", "body": "Original wording"}
      ]
    },
    {"id": "middle", "title": "Middle", "content": [{"kind": "text", "body": "middle content"}]},
    {"id": "end", "title": "End", "content": [{"kind": "text", "body": "end content"}]}
  ]
}
JSON
start "$BIN edit $US3KBDECK"
assert_contains "studio opens on the fixture deck" "Smoke US3 KB Talk"

# `]` from no selection lands on the outline's first (entry) row,
# deterministically, regardless of prior structural edits.
keys "]"
keys "c"
assert_contains "c opens the choice prompt without the mouse" "Turn into a choice"
tmux send-keys -t "$SESSION" -l -- "To Middle"
tmux send-keys -t "$SESSION" C-s
assert_contains "Ctrl+S on a choice prompt opens the slide picker" "Choose a slide"
# Picker rows list every slide in the deck's declaration order — with
# nothing else added yet, digit 2 is unambiguously "Middle".
keys "2"
assert_contains "digit 2 picks the second listed slide" "Branches to: Middle"

keys "a"
assert_contains "a opens the add-answer prompt without the mouse" "Add an answer"
tmux send-keys -t "$SESSION" -l -- "To End"
tmux send-keys -t "$SESSION" C-s
keys "3"
assert_contains "both named answers are wired, entirely via the keyboard" "Branches to: Middle, End"

keys "n"
assert_contains "n opens the new-slide prompt without the mouse" "New slide"
tmux send-keys -t "$SESSION" -l -- "Recap"
tmux send-keys -t "$SESSION" C-s
assert_contains "the new slide is added and selected" "Recap"

# Reselect "Welcome" — adding "Recap" moved the studio's selection to it,
# and `]` from nothing is deterministically the entry slide again.
keys "Escape"
keys "]"
keys "Tab"
keys "Tab"
keys "r"
assert_contains "r cycles the selected block's reveal step" "Reveal: 1"

tmux send-keys -t "$SESSION" C-s
assert_contains "Ctrl+S saves" "Saved"
if grep -qF '"reveal": 1' "$US3KBDECK" && grep -qF '"branch-point"' "$US3KBDECK"; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m the saved file has the branch and the reveal step\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m the saved file is missing the branch or the reveal step\n'
  fail=$((fail + 1))
fi

keys "p"
assert_contains "presenting shows the reveal gate before the branch" "0/1 revealed"
keys " "
assert_contains "revealing shows the wording and both named answers" "Original wording"
assert_contains "the branch's answers render by name" "To Middle"
keys "q"
assert_contains "q returns to the editor (the block stays selected across present-and-return)" "ready to present"

keys "q"
sleep 0.4
if ! tmux list-panes -t "$SESSION" >/dev/null 2>&1; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m editor q quits and the terminal is restored\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m editor q did not end the session\n'
  fail=$((fail + 1))
fi

# ─── Scenario 11: fireside edit — US4 crash-safety (spec 013, T065) ───────
echo
echo "=== fireside edit: force-kill mid-edit recovers a draft; quit-with-unsaved-changes asks first ==="
US4DECK="$WORKDIR/smoke-us4-talk.fireside.json"
cat >"$US4DECK" <<'JSON'
{
  "fireside-version": "0.1.0",
  "title": "Smoke US4 Talk",
  "nodes": [
    {
      "id": "intro",
      "title": "Welcome",
      "content": [
        {"kind": "heading", "level": 1, "text": "Hello there"},
        {"kind": "text", "body": "Original wording"}
      ]
    }
  ]
}
JSON
start "$BIN edit $US4DECK"
assert_contains "studio opens on the fixture deck" "Smoke US4 Talk"

# Same fixture shape as scenarios 7/8, so the same fixed 100x30 layout
# applies: the text block sits at row 14, its [ Edit ] chip at the hint
# line (row 30), and this single-field form's [ Done ] cell at (18, 17).
mouse_click 35 14
mouse_click 3 30
assert_contains "the block's form opens" "Edit text"
keys "Z"
mouse_click 18 17
assert_not_contains "[ Done ] commits and closes the form" "Edit text"
assert_contains "the unsaved edit shows on the canvas" "ZOriginal wording"

# Force-kill the process mid-edit, before any save — only the draft
# sidecar's autosave (spec 013 US4, T060) can have preserved this.
KILL_PID="$(tmux list-panes -t "$SESSION" -F '#{pane_pid}' | head -1)"
kill -9 "$KILL_PID" >/dev/null 2>&1 || true
sleep 0.5

start "$BIN edit $US4DECK"
assert_contains "reopening offers to restore the crashed session's draft" \
  "Recovered unsaved changes"
assert_contains "the draft's timestamp is shown in plain language" \
  "Draft last touched: just now"
keys "r"
assert_contains "restoring the draft (keyboard) brings back the unsaved edit" \
  "ZOriginal wording"

mouse_click 82 1
assert_contains "saving the restored draft writes it to disk" "Saved"
if grep -qF "ZOriginal wording" "$US4DECK"; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m the recovered edit reached the saved file\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m the recovered edit never reached the saved file\n'
  fail=$((fail + 1))
fi

# Now the quit-with-unsaved-changes prompt (FR-019): one more edit, then
# both of its non-save outcomes, keyboard-only.
mouse_click 35 14
mouse_click 3 30
keys "Y"
mouse_click 18 17
assert_not_contains "a second edit's form closes" "Edit text"
keys "q"
assert_contains "q with unsaved changes asks first, instead of quitting" \
  "unsaved changes"
keys "k"
assert_not_contains "Keep editing dismisses the prompt" "unsaved changes"
assert_contains "the unsaved edit is still there after Keep editing" \
  "YZOriginal wording"

keys "q"
keys "d"
sleep 0.4
if ! tmux list-panes -t "$SESSION" >/dev/null 2>&1; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m Discard quits without saving\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m Discard did not end the session\n'
  fail=$((fail + 1))
fi
if grep -qF "YZOriginal wording" "$US4DECK"; then
  printf '  \033[1;31m\xe2\x9c\x97\033[0m Discard must not have saved the second edit, but it did\n'
  fail=$((fail + 1))
else
  printf '  \033[1;32m\xe2\x9c\x93\033[0m the discarded edit never reached the saved file\n'
  pass=$((pass + 1))
fi
if grep -qF "ZOriginal wording" "$US4DECK"; then
  printf '  \033[1;32m\xe2\x9c\x93\033[0m the earlier, actually-saved edit is still intact\n'
  pass=$((pass + 1))
else
  printf '  \033[1;31m\xe2\x9c\x97\033[0m Discard corrupted the previously saved content\n'
  fail=$((fail + 1))
fi

echo
echo "----------------------------------------"
if [[ "$fail" -eq 0 ]]; then
  printf '\033[1;32mAll %d smoke checks passed.\033[0m\n' "$pass"
  exit 0
else
  printf '\033[1;31m%d passed, %d failed.\033[0m\n' "$pass" "$fail"
  exit 1
fi
