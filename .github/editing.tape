# Focused recording: fireside edit's full-screen studio — selecting a
# slide and a block, editing it through its form, previewing in-process
# via [ Present ], and saving — all without touching JSON. Mouse actions
# aren't recordable through VHS (no mouse-injection command as of 0.11),
# so this walks the same flow keyboard-only, exercising the
# keyboard-complete half of the editor's mouse-first/keyboard-complete
# posture (ADR-017).
#
# Regenerate with:  cargo build --release && vhs .github/editing.tape
# (brew install vhs)

Output docs/src/assets/editing.gif

Set FontSize 15
Set Width 900
Set Height 560
Set Padding 12
Set TypingSpeed 60ms

Hide
Type "PS1='$ '; clear; alias fireside=./target/release/fireside; cp crates/fireside-cli/assets/demo.fireside.json .github/demo-editing.fireside.json"
Enter
Show

Type "fireside edit .github/demo-editing.fireside.json"
Sleep 800ms
Enter
Sleep 1.5s

# `]` selects the next slide in outline order — first press lands on the
# entry slide itself, second moves to "Content blocks".
Type "]"
Sleep 500ms
Type "]"
Sleep 1s

# Tab selects the slide's first block (its heading); Enter opens the
# block's edit form.
Tab
Sleep 500ms
Enter
Sleep 1s

# Tweak the heading text in place, then Ctrl+S commits the form.
Right@60ms 8
Sleep 300ms
Type "!"
Sleep 500ms
Ctrl+S
Sleep 1s

# `p` tries the deck in-process, from the slide just edited — proof the
# canvas and the presenter are always in sync (WYSIWYG by construction).
Type "p"
Sleep 2s
Type "q"
Sleep 1s

# Save the deck file, then quit.
Ctrl+S
Sleep 1s
Type "q"
Sleep 500ms

Hide
Type "rm -f .github/demo-editing.fireside.json"
Enter
Show
