# Focused recording: the elapsed-time timer and the rail map.
#
# Regenerate with:  cargo build --release && vhs .github/timer-map.tape
# (brew install vhs)

Output docs/src/assets/timer-map.gif

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

# The elapsed timer.
Type "t"
Sleep 2s
Space
Sleep 2s

# The map: every slide, marked for progress, closed without jumping.
Type "m"
Sleep 2s
Down@250ms 2
Sleep 800ms
Escape
Sleep 1s

Type "q"
Sleep 500ms
