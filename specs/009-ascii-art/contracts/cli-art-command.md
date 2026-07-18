# Contract: `fireside art` subcommand

Authoring-time convenience only. Neither verb reads or writes a deck
file — both print ready-to-paste art to stdout. Getting the output into a
deck remains a manual step (hand-edit, quick-edit, or a future `import`
extension), per spec 009's Assumptions.

## `fireside art text <PHRASE>`

Generates a stylized text banner from `PHRASE` via `figlet-rs`'s standard
font and prints it to stdout.

**Success**: multi-line banner text on stdout, exit code 0.

**Partial-recognition behavior (FR-013)**: if `PHRASE` contains characters
the font has no letterform for, alongside characters it does, the command
still produces output for every recognized character — one unsupported
character does not fail the whole phrase.

**Failure**: if `PHRASE` contains no character the font recognizes at
all, the command reports that no output could be produced (via
`anyhow::bail!` or equivalent, a clear one-line message to stderr) and
exits non-zero — it never emits a blank block.

## `fireside art image <PATH> [--width N]`

Converts the image at `PATH` to ASCII shading via `rascii_art` and prints
it to stdout. `--width` is optional; when absent, a default width that
fits the standard supported terminal size is used.

**Success**: multi-line plain-text ASCII art on stdout, exit code 0.

**Failure (FR-014)**: if `PATH` does not exist, is not readable, or is
not a decodable image format, the command reports a clear, actionable
error to stderr (e.g. "could not read `<path>`: <cause>", matching the
existing `with_context` style used throughout `fireside-cli`) and exits
non-zero — it never panics.

## Output guarantees (both verbs)

- Plain text only — no ANSI color/formatting escape codes, ever
  (`rascii_art`'s `colored()` render option is left off; `figlet-rs`
  has no color concept). Matches the wire contract's requirement that
  `ascii-art` block content stay plain text (constitution Principle IV).
- No trailing deck-JSON wrapping — output is the raw art text, so an
  author can decide for themselves whether to paste it as an
  `ascii-art` block's `art` value, a `CodeBlock`, or anywhere else. A
  `--json` convenience flag that wraps the output as a ready-to-paste
  `{"kind":"ascii-art","art":"..."}` fragment is a plausible follow-up,
  not required by spec 009's functional requirements — left for `tasks.md`
  to size and, if included, ship as an additive flag rather than a
  behavior change to the default output.
