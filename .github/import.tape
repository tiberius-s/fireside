# Focused recording: compiling a Markdown talk into a deck with
# `fireside import`, then presenting the result.
#
# Regenerate with:  cargo build --release && vhs .github/import.tape
# (brew install vhs)

Output .github/import.gif

Set FontSize 15
Set Width 900
Set Height 560
Set Padding 12
Set TypingSpeed 60ms

Hide
Type "PS1='$ '; clear; alias fireside=./target/release/fireside; rm -f .github/demo-import.fireside.json"
Enter
Show

Type "fireside import .github/demo-import.md"
Sleep 1s
Enter
Sleep 2.5s

Type "fireside .github/demo-import.fireside.json"
Sleep 800ms
Enter
Sleep 2s
Space
Sleep 2s

Type "q"
Sleep 500ms

Hide
Type "rm -f .github/demo-import.fireside.json"
Enter
Show
