# Focused recording: quick-editing a slide's heading and a list item
# without leaving the terminal — list items are just another editable
# field, no separate add/remove UI needed.
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

Type "fireside .github/demo-quick-edit.fireside.json --restart"
Sleep 800ms
Enter
Sleep 1.5s

# The "Content blocks" slide has a heading, a list, and text — all
# editable.
Space
Sleep 1s
Type "e"
Sleep 1s

# Tweak the heading in place.
Right@60ms 8
Sleep 300ms
Type "!"
Sleep 500ms

# Move down past the text field into the list, and edit a bullet.
Down
Down
Sleep 500ms
Right@60ms 8
Sleep 300ms
Type "!"
Sleep 800ms

Ctrl+S
Sleep 1.5s

Type "q"
Sleep 500ms

Hide
Type "rm -f .github/demo-quick-edit.fireside.json"
Enter
Show
