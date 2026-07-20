# Focused recording: quick-editing a slide's heading without leaving the
# terminal.
#
# Regenerate with:  cargo build --release && vhs .github/quick-edit.tape
# (brew install vhs)

Output docs/src/assets/quick-edit.gif

Set FontSize 15
Set Width 900
Set Height 560
Set Padding 12
Set TypingSpeed 60ms

Hide
Type "PS1='$ '; clear; alias fireside=./target/release/fireside; cp crates/fireside-cli/assets/demo.fireside.json .github/demo-quick-edit.fireside.json"
Enter
Show

Type "fireside .github/demo-quick-edit.fireside.json"
Sleep 800ms
Enter
Sleep 1.5s

# Open quick-edit and add an exclamation point to the heading.
Type "e"
Sleep 1s
Right@60ms 8
Sleep 300ms
Type "!"
Sleep 500ms
Ctrl+S
Sleep 1.5s

Type "q"
Sleep 500ms

Hide
Type "rm -f .github/demo-quick-edit.fireside.json"
Enter
Show
