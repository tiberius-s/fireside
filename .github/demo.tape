# The README recording: a quick tour of `fireside demo`.
#
# Regenerate with:  cargo build --release && vhs .github/demo.tape
# (brew install vhs)

Output docs/src/assets/demo.gif

Set FontSize 15
Set Width 1000
Set Height 620
Set Padding 12
Set TypingSpeed 60ms

Hide
Type "PS1='$ '; clear; alias fireside=./target/release/fireside"
Enter
Show

Type "fireside demo"
Sleep 800ms
Enter
Sleep 2.5s

# Content blocks: markdown, lists, an image plate.
Space
Sleep 3s

# Highlighted code — then show the fullscreen toggle.
Space
Sleep 2.5s
Type "f"
Sleep 2s
Type "f"
Sleep 1s

# Layout: columns and centering.
Space
Sleep 2.5s

# The branch point: pick with arrows.
Space
Sleep 2s
Down
Sleep 1s
Up
Sleep 800ms

# The rail map: the deck's shape, forks and all.
Type "m"
Sleep 3.5s
Escape
Sleep 800ms

# Take the extras path via its shortcut key, and reveal its tips one at a
# time (see reveal.tape for a focused look at incremental reveal).
Type "o"
Sleep 1.5s
Space
Sleep 1.2s
Space
Sleep 1.2s
Space
Sleep 1.5s

# The elapsed timer.
Type "t"
Sleep 1.5s

# Finish.
Space
Sleep 3s

Type "q"
Sleep 500ms
