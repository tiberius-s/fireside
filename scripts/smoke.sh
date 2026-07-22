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

echo
echo "----------------------------------------"
if [[ "$fail" -eq 0 ]]; then
  printf '\033[1;32mAll %d smoke checks passed.\033[0m\n' "$pass"
  exit 0
else
  printf '\033[1;31m%d passed, %d failed.\033[0m\n' "$pass" "$fail"
  exit 1
fi
