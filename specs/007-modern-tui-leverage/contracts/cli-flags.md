# Contract: CLI surface changes

`fireside-cli`'s `present` subcommand (and the bare `fireside <file>`
shorthand, which delegates to it) gains one new flag. No existing flag's
meaning changes.

## `fireside present <file> [--restart]`

| Flag        | Type | Default | Behavior                                                                 |
| ----------- | ---- | ------- | ------------------------------------------------------------------------- |
| `--restart` | bool | `false` | Skip the resume lookup for this run only; always start at the deck's normal entry node. Does not delete any existing resume record — the next run without `--restart` still resumes normally. |

No other subcommand (`validate`, `new`, `demo`, `import`) is affected.

## In-TUI surface

- Mouse clicks on the map screen and branch menu are additive; no existing
  key binding's meaning changes (constitution Principle II: footer remains
  the primary, always-visible contract).
- No new footer text is required for mouse (it is discoverable by trying
  it, per the source plan's "where it's discoverable" framing) but the
  `?`/`h` help screen gains one line noting click support, consistent with
  it already being the fuller reference than the footer.
