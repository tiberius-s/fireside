# Focused recording: incremental reveal on the demo deck's "extras" slide.
#
# Regenerate with:  cargo build --release && vhs .github/reveal.tape
# (brew install vhs)

Output .github/reveal.gif

Set FontSize 15
Set Width 900
Set Height 560
Set Padding 12
Set TypingSpeed 60ms

Hide
Type "PS1='$ '; clear; alias fireside=./target/release/fireside"
Enter
Show

Type "fireside demo"
Sleep 800ms
Enter
Sleep 1.5s

# Jump straight to the extras slide via the map instead of walking there.
Type "m"
Sleep 800ms
Down@250ms 5
Sleep 500ms
Enter
Sleep 1s

# Reveal the three tips one Space at a time.
Space
Sleep 1.5s
Space
Sleep 1.5s
Space
Sleep 2s

Type "q"
Sleep 500ms
