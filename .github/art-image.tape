# Focused recording: converting a local image into ASCII shading with
# `fireside art image`.
#
# Regenerate with:  cargo build --release && vhs .github/art-image.tape
# (brew install vhs)

Output docs/src/assets/art-image.gif

Set FontSize 15
Set Width 900
Set Height 560
Set Padding 12
Set TypingSpeed 60ms

Hide
Type "PS1='$ '; clear; alias fireside=./target/release/fireside"
Enter
Show

Type "fireside art image docs/src/assets/demo-art.png --width 50 --charset block"
Sleep 800ms
Enter
Sleep 3s
