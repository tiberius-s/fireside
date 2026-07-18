---
title: 'CLI Reference'
description: 'Every fireside subcommand, its flags, and its exit codes.'
---

The `fireside` binary has six verbs. Running `fireside` with no arguments
prints this same summary:

```text
fireside demo              see what a deck can do
fireside <file>            present a deck
fireside validate <file>   check a deck for problems
fireside new               create a deck (asks a few questions)
fireside new <name>        create a starter deck instantly
fireside import <file.md>  compile a Markdown talk into a deck
fireside art text <phrase> generate a text banner to paste in
```

`fireside <file>` is shorthand for `fireside present <file>` — the `present`
verb name is optional.

## `fireside present <file>`

Validates and presents a deck in the terminal. Presenting always validates
first: if the deck has any error-severity diagnostic, `present` refuses to
start, prints the diagnostics, and exits `1` rather than opening the TUI on a
broken deck.

While presenting, the deck file is watched. A save that still validates
swaps in seamlessly and keeps the current slide; a save that doesn't parse or
fails validation keeps the last-good deck on screen and explains what's wrong
in the footer.

| Flag        | Effect                                                          |
| ----------- | ---------------------------------------------------------------- |
| `--restart` | Ignore any saved resume position for this deck and start at the entry node. |

Without `--restart`, `present` resumes from the last node reached in a
previous session for this exact deck content (see
[Presenting a Deck](/guides/presenting/#resuming-after-a-crash-or-exit)).
Reaching a terminal node clears the saved position.

**Exit codes:** `0` on a clean exit from the TUI; `1` if the deck fails to
parse, fails validation, or the presenter hits a terminal error.

## `fireside validate <file>`

Checks a deck and reports every diagnostic in plain language — no TUI. Parse
failures point at the exact line and column with a caret; validation
diagnostics are grouped by severity (`✗` error, `⚠` warning, `ℹ` info) with a
one-line summary.

| Flag      | Effect                                                                 |
| --------- | ----------------------------------------------------------------------- |
| `--watch` | Re-check the file on every save and re-print the report. Runs until interrupted (Ctrl+C). |

This is the authoring loop: an editor on one side, `fireside validate --watch`
on the other, errors appearing as you save.

![fireside validate --watch catching a broken branch target, then a fix](https://raw.githubusercontent.com/tiberius-s/fireside/main/.github/validate-watch.gif)

**Exit codes:** `0` if the deck has no error-severity diagnostics (warnings
and info are fine); `1` otherwise. `--watch` never exits on its own — only on
interruption.

## `fireside new [name]`

Scaffolds a starter deck. With no name, asks three questions interactively
(title, template, author); with a name, creates the deck immediately using
defaults.

| Argument/Flag  | Effect                                                                |
| -------------- | ------------------------------------------------------------------------ |
| `name`         | Deck title, slugified into the output filename `<slug>.fireside.json`. Omit to be prompted. |
| `--template`   | `linear`, `branching` (default), or `workshop` — see below.          |
| `--author`     | Author name embedded in the deck.                                    |
| `--banner`     | Adds an ASCII title banner (a FIGlet rendering of the deck's title) as an `ascii-art` block on the first slide. Skipped, with a note, if the title renders too wide to fit the card — deck creation still succeeds. |

Templates:

- **`linear`** — a straight-through talk, no branching.
- **`branching`** — a talk with one choice that rejoins (the default).
- **`workshop`** — an agenda branch point that jumps into a sequence of
  exercises, each flowing into the next.

Every generated node carries a `speaker-notes` hint describing what to
replace. Without a name, `new` also asks whether to add the title banner.
`new` refuses to overwrite an existing file with the same name.

**Exit codes:** `0` on success; `1` if the name is empty after slugifying, or
the target file already exists.

## `fireside import <input.md> [output]`

Compiles a Markdown file into a deck: `##` headings become nodes in document
order, a ` ```branch ` fence turns a section into a branch point, and a
` ```ascii-art ` fence becomes a real `ascii-art` block — paste the output
of `art text`/`art image` straight into one, no hand-editing the generated
JSON required. See [Authoring a Deck in Markdown](/guides/authoring-markdown/)
for the syntax.

| Argument | Effect                                                                          |
| -------- | -------------------------------------------------------------------------------- |
| `input`  | Path to the Markdown source.                                                     |
| `output` | Path for the generated deck. Defaults to `input` with its extension replaced by `.fireside.json`. |

`import` runs the generated deck through the same Layer-2 validation as
`validate` before writing anything — an import that would produce an invalid
deck fails instead of writing a broken file. `import` refuses to overwrite an
existing output file.

**Exit codes:** `0` on success; `1` if the source can't be parsed into a deck
(no `##` headings, a nested list, an unresolved branch link, a malformed
branch line) or the generated deck fails validation.

## `fireside art text <phrase>`

Turns `phrase` into a large stylized text banner (a FIGlet-style rendering)
and prints it to stdout — no external tool or website needed. This is an
authoring-time convenience: it doesn't read or write a deck file itself, but
two things in the CLI consume its output for you: `fireside new --banner`
generates one from the deck's title automatically, and `fireside import`
promotes any ` ```ascii-art ` fence in your Markdown source to a real block
— paste this command's output there instead of hand-editing JSON. You can
still paste it into an `ascii-art` block's `art` field by hand (see
[§2 Data Model, AsciiArtBlock](/spec/data-model/#asciiartblock)) if you'd
rather.

Characters the built-in font has no letterform for are skipped, not fatal —
`fireside art text "Hi 🔥"` still produces output for `Hi`. Only a phrase
with *no* recognized character fails.

**Exit codes:** `0` on success; `1` if no character in `phrase` is
recognized.

![Generating a stylized text banner with fireside art text](https://raw.githubusercontent.com/tiberius-s/fireside/main/.github/art-text.gif)

## `fireside art image <path> [--width N]`

Converts the image at `path` to ASCII shading and prints it to stdout — same
authoring-time convenience as `art text`, and just as file-free: nothing is
written to disk.

| Argument/Flag | Effect                                                               |
| -------------- | --------------------------------------------------------------------- |
| `path`         | Path to a local image file.                                          |
| `--width`      | Output width in columns. Defaults to 76 — the same width the `ascii-art-too-wide` validator warns past, so default output already fits the presentation card. |

**Exit codes:** `0` on success; `1` if `path` doesn't exist or isn't a
readable image — reported with a clear message, never a panic.

The source photo below ("People sitting around a camp fire" by Hynek Janáč,
[CC0 1.0](https://commons.wikimedia.org/wiki/File:People_sitting_around_a_camp_fire.jpg),
Wikimedia Commons) is what the recording converts — shown here so you can
compare input and output directly, not just take the GIF's word for it:

![The source photo used in the recording below](https://raw.githubusercontent.com/tiberius-s/fireside/main/.github/demo-art.png)

![Converting a local image into ASCII shading with fireside art image](https://raw.githubusercontent.com/tiberius-s/fireside/main/.github/art-image.gif)

## `fireside demo`

Presents the built-in showcase deck — no file needed. Useful for seeing every
content block kind, including `ascii-art` on the welcome slide, and both view
modes without writing any JSON first.

**Exit codes:** `0` on a clean exit; `1` on a terminal presenter error.

## Common conventions across verbs

- A file argument that is a directory or fails to read produces a message
  naming the path, not a raw I/O error.
- Malformed JSON always reports as `<path>:<line>:<column>` with the
  offending line and a caret, never a raw serde one-liner.
- Every write (`new`, `import`, and quick-edit save while presenting) refuses
  to silently clobber unrelated existing content — `new`/`import` refuse to
  overwrite an existing output file outright, and quick-edit save refuses a
  write if the file changed on disk since it was last read.
