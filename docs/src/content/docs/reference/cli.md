---
title: 'CLI Reference'
description: 'Every fireside subcommand, its flags, and its exit codes.'
---

The `fireside` binary has five verbs. Running `fireside` with no arguments
prints this same summary:

```text
fireside demo              see what a deck can do
fireside <file>            present a deck
fireside validate <file>   check a deck for problems
fireside new               create a deck (asks a few questions)
fireside new <name>        create a starter deck instantly
fireside import <file.md>  compile a Markdown talk into a deck
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

Templates:

- **`linear`** — a straight-through talk, no branching.
- **`branching`** — a talk with one choice that rejoins (the default).
- **`workshop`** — an agenda branch point that jumps into a sequence of
  exercises, each flowing into the next.

Every generated node carries a `speaker-notes` hint describing what to
replace. `new` refuses to overwrite an existing file with the same name.

**Exit codes:** `0` on success; `1` if the name is empty after slugifying, or
the target file already exists.

## `fireside import <input.md> [output]`

Compiles a Markdown file into a deck: `##` headings become nodes in document
order, and a ` ```branch ` fence turns a section into a branch point. See
[Authoring a Deck in Markdown](/guides/authoring-markdown/) for the syntax.

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

## `fireside demo`

Presents the built-in showcase deck — no file needed. Useful for seeing every
content block kind and both view modes without writing any JSON first.

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
