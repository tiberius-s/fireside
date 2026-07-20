---
title: 'Presenting a Deck'
description: 'Every key the TUI responds to, plus the map, speaker notes, fullscreen, and resume workflows.'
---

Everything you need to drive a Fireside presentation lives in the footer —
this page is the same information with room to explain the workflows around
it. Run `fireside <file>` (or `fireside demo` with no file) to try any of
this live.

## Moving through a deck

| Key                    | Effect                          |
| ----------------------- | -------------------------------- |
| `Space` / `→` / `Enter` / `n` / `PageDown` | Next slide (or reveal the next fragment — see below) |
| `←` / `Backspace` / `p` / `PageUp` | Previous slide |
| `↑` / `↓`               | Scroll long content, or move the selection at a branch point |

Every keypress gets visible feedback — a slide change, a reveal, a flash
message, or a selection move. Nothing is ever a silent no-op.

## Incremental reveal

If a slide's content uses staged reveal, the footer shows how many pieces
are still pending (`N/M revealed`) and `Space` advances one reveal step at a
time before moving to the next slide. Going back (`←`) always leaves reveal
and returns to the previous slide directly — reveal steps aren't
individually undoable.

![Revealing a slide's content one piece at a time](https://raw.githubusercontent.com/tiberius-s/fireside/main/.github/reveal.gif)

## Branch points

At a branch point the footer reads `↑↓ choose · Enter go`:

| Key                  | Effect                                     |
| --------------------- | -------------------------------------------- |
| `↑`/`k`, `↓`/`j`       | Move the selection among options            |
| `Enter`                | Choose the selected option                   |
| `1`–`9`                | Choose an option directly by its number      |
| a letter matching an option's declared key | Choose that option directly |
| click an option (mouse) | Choose it                                  |
| `Space`/`→`/`n`/`PageDown` | Flashes "This slide asks for a choice" — a branch point never has a fallback, so one of the choices above must be made |
| `←`/`Backspace`/`p`/`PageUp` | Back to the previous slide             |

## The map

Press `m` or `g` from anywhere to open the map — a list of every node with a
marker for where you are (`◉`), where you've been (`●`), where you haven't
(`○`), and terminal nodes (`■`).

| Key             | Effect                          |
| ---------------- | -------------------------------- |
| `↑`/`k`, `↓`/`j`  | Move the selection               |
| `Enter` / click a row | Jump straight to that node and return to presenting |
| `Esc` / `m` / `g` / `q` | Close the map without jumping |

The map is the fastest way to skip ahead, backtrack past several slides at
once, or recover if you've lost track of where a branch went.

![Toggling the elapsed timer and opening the map](https://raw.githubusercontent.com/tiberius-s/fireside/main/.github/timer-map.gif)

## Other keys while presenting

| Key | Effect                                                             |
| --- | -------------------------------------------------------------------- |
| `f` | Toggle fullscreen for the current slide                              |
| `s` | Toggle speaker notes (flashes a message if the slide has none)       |
| `t` | Toggle an elapsed-time timer in the footer                           |
| `e` | Open quick-edit for this slide's headings/text (see below)           |
| `?` / `h` | Open the help overlay — the same table as this page, any key closes it |
| `q` | Quit                                                                  |

## Quick-editing a slide

`e` opens a modal that edits the current node's heading and text blocks in
place — not a full editor: no adding or removing blocks, no restructuring.
It's for fixing a typo or rewording a line without leaving the terminal.

| Key      | Effect                                    |
| -------- | -------------------------------------------- |
| `Ctrl+S` | Save. The file is live-reloaded, same as an external edit — the deck updates in place, staying on the current slide. |
| `Esc`    | Cancel — discards the edit, changes nothing on disk |

If the deck file changed on disk since it was opened (someone else editing
it, or a `sync` from another tool), `Ctrl+S` reports a conflict and leaves
your edit in the modal instead of overwriting it silently — press `Ctrl+S`
again to overwrite deliberately, or `Esc` to abandon your edit and pick up
the external change instead.

![Quick-editing a slide's heading and saving in place](https://raw.githubusercontent.com/tiberius-s/fireside/main/.github/quick-edit.gif)

## Resuming after a crash or exit

Fireside remembers the last node you reached in each deck, keyed to that
deck's exact content — a `resume.json` in your platform's local state
directory, not part of the deck file itself. Relaunching
`fireside <file>` on the same deck reopens where you left off; reaching a
terminal (ending) node clears the saved position, since there's nothing
left to resume. Pass `--restart` to skip the saved position for one run
without discarding it.

## Fullscreen and speaker notes

Fullscreen (`f`) drops the map rail and widens the content area — useful for
a code sample or an ASCII diagram that needs the whole terminal. Speaker
notes (`s`) show the current node's `speaker-notes` field in a panel at the
bottom of the same window you're presenting from — meant for you, not the
audience, but only actually private while you're rehearsing solo or the
terminal isn't visible to anyone else; toggling with no notes present
flashes a message rather than showing an empty panel. If your terminal is
projected or mirrored, use [Presenting with two screens](#presenting-with-two-screens)
instead — that keeps notes off the shared display entirely.

## Presenting with two screens

For a talk with a projector or a second display, `s`'s single-window notes
panel isn't private — anything on the presenter's own screen is on the
shared one too. The two-screen setup is two separate terminal windows on
your laptop instead:

1. Drag the terminal running `fireside <file>` to the external display and
   make it fullscreen (OS-level, or start with `fireside <file> --fullscreen`
   to skip the manual `f` press).
2. On your laptop's own screen, open a second terminal and run
   `fireside notes <file>` on the same deck.

The second window shows the current slide's title and speaker notes, the
next slide's title (or the branch options, if you're at a choice), reveal
progress, and an elapsed timer — read-only, and never rendered on the
projected window. It updates within about half a second of anything you do
in the presenter, and switches to
`Presenter not running — start "fireside <deck>" in another window` within
about two seconds of the presenter stopping, for any reason — so you're
never looking at stale information without knowing it. Live edits (quick-edit
saves, or any external change to the deck file) show up in the notes window
the same way they show up on stage. Press `q` in the notes window to close
it; it has no other keys.
