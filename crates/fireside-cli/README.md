# fireside-cli

The `fireside` binary. Four verbs, nothing else:

```text
fireside demo              see what a deck can do — no file needed
fireside <file>            present a deck (live-reloads on save)
fireside validate <file>   check a deck for problems, in plain language
fireside new <name>        create a starter deck you can present immediately
```

Validation always runs before presenting, so a broken deck fails loudly at
the prompt instead of during the show — parse failures point at the exact
line with a caret. While presenting, the deck file is watched: a good save
swaps in seamlessly (staying on the current slide), a broken save keeps the
working deck and explains itself in the footer.
