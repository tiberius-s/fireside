# fireside-cli

The `fireside` binary. Three verbs, nothing else:

```text
fireside <file>            present a deck
fireside validate <file>   check a deck for problems, in plain language
fireside new <name>        create a starter deck you can present immediately
```

Validation always runs before presenting, so a broken deck fails loudly at
the prompt instead of during the show.
