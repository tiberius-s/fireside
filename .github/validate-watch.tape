# Focused recording: `fireside validate --watch` catching a bad edit and
# then a fix, live.
#
# Regenerate with:  cargo build --release && vhs .github/validate-watch.tape
# (brew install vhs)

Output .github/validate-watch.gif

Set FontSize 15
Set Width 900
Set Height 560
Set Padding 12
Set TypingSpeed 60ms

Hide
Type "PS1='$ '; clear; alias fireside=./target/release/fireside; cp crates/fireside-cli/assets/demo.fireside.json .github/demo-watch.fireside.json"
Enter
Show

Type "fireside validate --watch .github/demo-watch.fireside.json &"
Sleep 500ms
Enter
Sleep 2s

# Break a branch target — the watcher re-checks on save and reports it.
Type "bash .github/demo-watch-toggle.sh"
Sleep 500ms
Enter
Sleep 2.5s

# Fix it back — the watcher clears the error on the next save.
Type "bash .github/demo-watch-toggle.sh"
Sleep 500ms
Enter
Sleep 2.5s

Hide
Type "kill %1; rm -f .github/demo-watch.fireside.json"
Enter
Show
Sleep 500ms
