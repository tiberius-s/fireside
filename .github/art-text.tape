# Focused recording: turning a phrase into a stylized banner with
# `fireside art text`.
#
# Regenerate with:  cargo build --release && vhs .github/art-text.tape
# (brew install vhs)

Output .github/art-text.gif

Set FontSize 15
Set Width 900
Set Height 420
Set Padding 12
Set TypingSpeed 60ms

Hide
Type "PS1='$ '; clear; alias fireside=./target/release/fireside"
Enter
Show

Type `fireside art text "Fireside"`
Sleep 800ms
Enter
Sleep 3s
