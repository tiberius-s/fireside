# Contract: `fireside art` subcommand

Authoring-time convenience. Neither verb reads or writes a deck file
itself — both print ready-to-paste art to stdout. Getting the output
into a deck no longer requires a manual step, though one is still
available: **updated 2026-07-18** — `fireside new --banner` generates a
banner from the deck title directly into a scaffolded deck (see
`crates/fireside-cli/src/new.rs::add_title_banner`), and `fireside
import` promotes a ` ```ascii-art ` fence in Markdown source to a real
block (see `crates/fireside-cli/src/import.rs`). Hand-editing the
generated JSON, or quick-edit, remains available for anyone who'd rather
paste by hand.

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
- No trailing deck-JSON wrapping — output is the raw art text. A
  `--json` convenience flag that wraps the output as a ready-to-paste
  `{"kind":"ascii-art","art":"..."}` fragment was considered here and
  not built: `new --banner` and `import`'s ascii-art fence (added
  2026-07-18, see the top of this file) cover the two real workflows
  more directly than a JSON-fragment flag would have.
