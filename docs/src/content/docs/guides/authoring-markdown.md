---
title: 'Authoring a Deck in Markdown'
description: 'Write a talk in plain Markdown and compile it to a Fireside deck with fireside import.'
---

Fireside decks are protocol JSON, but you don't have to write JSON by hand.
`fireside import talk.md` compiles a Markdown file into a deck: each `##`
heading becomes a node, in document order, and a small fence syntax
declares branch points. This is the fastest way from a talk outline to a
presentable deck.

## The shape

```mermaid
graph LR
  A[talk.md] -->|fireside import| B[talk.fireside.json]
  B -->|fireside validate| C[checked]
  B -->|fireside talk.fireside.json| D[presented]
```

Every `##` heading starts a new node. Its slugified heading text becomes the
node's id (`## Core Features` → `core-features`), and its heading level-2
text becomes the node's title. Everything between one `##` and the next
becomes that node's content.

## A linear talk

```markdown
---
title: My Talk
author: Ada Lovelace
---

## Welcome

Thanks for coming. Here's what we'll cover.

- Point one
- Point two

## The Code

​```rust
fn main() {
    println!("hello");
}
​```

## Thanks

Questions?
```

Optional YAML-ish frontmatter (`title`, `author`, `date`, `description`,
`fireside-version`) sets deck metadata. Without frontmatter, a leading `#`
(H1) heading before the first `##` is used as the deck title instead. Any
other content between that title and the first `##` isn't included in the
deck — `import` prints a warning rather than silently dropping it.

**Migrating a `#`-per-slide file** (the presenterm/patat convention): if
your document has two or more `#` headings and no `##` at all, `import`
treats each `#` as a slide instead of erroring — no frontmatter or `##`
required. A single `#` with no `##` anywhere is still an error (it reads as
an intended title, not a slide), with a message telling you which of the
two you meant.

Nodes wire together in document order automatically: `welcome` → `the-code`
→ `thanks`, with `thanks` terminal since nothing follows it. Run it:

```sh
fireside import talk.md
fireside validate talk.fireside.json
fireside talk.fireside.json
```

![Compiling a Markdown talk with fireside import, then presenting it](../../../assets/import.gif)

## What each Markdown element becomes

| Markdown                          | Content block                          |
| ---------------------------------- | ---------------------------------------- |
| `##` heading                       | Starts a new node; its text is the node's `title` |
| paragraph text                     | `text` block, inline `**bold**`/`_italic_`/`` `code` `` preserved |
| `- item` / `1. item`               | `list` block (`ordered: true` for numbered lists) |
| `- [ ] item` / `- [x] item`        | `list` item prefixed `☐`/`☑` — the checkbox state is baked into the text, not a separate field |
| fenced code block                  | `code` block, language tag preserved (` ```ascii-art ` fences become an `ascii-art` block instead — see below) |
| a paragraph containing only one image | `image` block (`alt`/title captured) |
| `---` horizontal rule              | `divider` block                        |
| table (`| a | b |`)                | `code` block: a monospace, column-aligned grid with a rule under the header — there's no `table` block kind in the protocol. Cell formatting (`**bold**`, etc.) is stripped to plain text; `import` notes this on stderr |
| `> quote`                          | flattens to a plain `text` block — the quote styling itself isn't preserved |
| `~~strikethrough~~`                | the `~~` markers are dropped, text kept (the renderer has no strike-through support); noted on stderr |
| footnote reference (`[^1]`) and definition (`[^1]: ...`) | both dropped entirely — footnotes aren't supported yet; each drop is noted on stderr with a line number |

**Nested lists are rejected, not flattened.** `import` fails with the line
number of the nested item rather than silently losing structure — flatten
the list, or hand-edit the generated JSON afterward.

**Every table/footnote/strikethrough conversion prints a note.** None of
these fail the import — they're accepted with a plain-language stderr note
naming the line and what changed, the same voice as the nested-list
rejection, so nothing is silently lossy.

## Branch points

A ` ```branch ` fence is the last thing in a section and turns that node
into a branch point instead of a linear step:

```markdown
## Choose your path

​```branch
What would you like to see?
- [Explore the features](#core-features) `f`
- [Watch a demo](#code-demo) `d`
​```

## Core Features

Some features.

## Code Demo

​```rust
fn demo() {}
​```
```

The first line of the fence is the prompt. Each following line is
`- [label](#target-slug)` with an optional `` `key` `` — the backtick-quoted
single character a presenter can press to choose that option directly.
`#target-slug` must match another section's heading slug in the same
document; an unresolved target fails the import with the line number and
the slug it couldn't find. Content after a `branch` fence within the same
section is also rejected — the fence must be the section's last element.

## ASCII art

A fence tagged ` ```ascii-art ` imports as a real `ascii-art` block, not a
`code` block — the natural way to get generated art into a Markdown-authored
deck without hand-editing the compiled JSON. This matters because a plain
` ```` ` fenced image reference doesn't render as a photo: a Markdown `image`
element compiles to an `image` block, and the reference presenter renders
that as a labeled placeholder frame, not real pixels (real image rendering is
out of scope for 0.1.0 — see
[Appendix C, Engine Extensions](/spec/appendix-engine-extensions/)). An
`ascii-art` block is the only way to get a photo or a title banner to
actually show up on screen.

A text banner, end to end:

```markdown
## Welcome

​```ascii-art
 _____ ___ ____  _____ ____ ___ ____  _____
|  ___|_ _|  _ \| ____/ ___|_ _|  _ \| ____|
| |_   | || |_) |  _| \___ \| || | | |  _|
|  _|  | ||  _ <| |___ ___) | || |_| | |___
|_|   |___|_| \_\_____|____/___|____/|_____|
​```
```

Generate the fence contents with `fireside art text "<phrase>"` and paste the
output straight in — see
[CLI Reference](/reference/cli/#fireside-art-text-phrase). Alternatively,
`fireside new --banner` skips Markdown entirely and generates a title
banner directly into a scaffolded deck.

A photo, end to end: convert a source image to ASCII shading, paste the
output into an ` ```ascii-art ` fence the same way, then import and present:

```sh
fireside art image sunset.png > /tmp/sunset.txt
```

```markdown
## The View From Here

​```ascii-art
<paste the contents of /tmp/sunset.txt here>
​```
```

```sh
fireside import talk.md
fireside talk.fireside.json
```

The slide renders the converted photo as text art, centered in the content
area — see [`fireside art image`](/reference/cli/#fireside-art-image-path---width-n---charset-name---invert---no-normalize)
for the input/output comparison, contrast-stretch behavior, and flags like
`--charset` and `--invert`.

## What v1 import doesn't carry over

Import is deliberately a compiler for the common case, not a full protocol
surface. It doesn't produce columns/multi-column containers, per-slide
`view-mode` or `transition` overrides, speaker notes, or **incremental
reveal** — `fireside import` prints a reminder of this after every
successful run. There's no Markdown marker syntax for reveal today, so a
deck that needs it has to be hand-edited (add `"reveal": N` to the blocks
that should appear progressively — see
[the `reveal` field](/spec/data-model/#the-reveal-field-all-kinds)), or with
quick-edit (`e` while presenting) for headings, text, and list items.

## Validation is not optional

`import` runs the generated deck through the same Layer-2 validation as
`fireside validate` before writing anything. If the compiled deck would fail
validation — for instance, two branch options that both need the same key —
`import` reports it and writes nothing, rather than handing you a broken
file to debug after the fact.

## Re-running import

`import` refuses to overwrite an existing output file, so a second
`fireside import talk.md` after you've hand-edited `talk.fireside.json`
fails rather than clobbering your edits — pick a different name, or delete
the old file first if you intend to regenerate from scratch.
