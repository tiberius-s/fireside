---
title: 'Quickstart'
description: 'Install Fireside, run the demo, and present your first deck in a few minutes.'
---

This page is the fastest path from nothing installed to a deck on screen. It
assumes no protocol knowledge — if you want to understand *why* Fireside is
shaped the way it is, [Mental models](/spec/mental-models/) is the next stop
after this.

## Requirements

The presenter renders with 24-bit RGB colors and has no 256-color fallback,
so you'll need a truecolor terminal (most modern terminal emulators; set
`COLORTERM=truecolor` if colors look off). It also expects a monospace font
with Unicode box-drawing support, and is most comfortable at ~80 columns by
24 rows or larger — narrower windows still work, but content wraps tighter.

Fireside draws into whatever font size your terminal is already using — it
has no way to set this itself, since font size is the terminal emulator's
setting, not the app's. If you're presenting to an audience (a projector, a
screen share), bump your terminal's font size up *before* you launch
Fireside, the same way you would before opening any other presentation
tool.

## Install

```bash
git clone https://github.com/tiberius-s/fireside.git
cd fireside
cargo install --path crates/fireside-cli
```

Requires Rust 1.88+ (MSRV).

## See what a deck can do

```bash
fireside demo
```

Press `Space` to move forward, `↑`/`↓` at a branch point to choose, and `?`
any time — the presenter teaches its own keys. `q` quits.

## Make your own

```bash
fireside new my-first-deck
fireside my-first-deck.fireside.json
```

`new` scaffolds a small starter deck; the second command presents it.

## Live-edit while presenting

Decks live-reload while you present: edit the JSON in another window, save,
and the slide on screen updates in place. You can also press `e` during a
presentation to quick-edit the current slide's heading, text, and list
items without leaving the terminal — see [Presenting a deck](/guides/presenting/#quick-editing-a-slide).

## Write in Markdown instead of JSON

Most talks start as an outline, not hand-written JSON:

```bash
fireside import talk.md
fireside talk.fireside.json
```

`import` compiles a Markdown file into a deck — each `##` heading becomes a
node, and a small fence syntax declares branch points. See
[Authoring a Deck in Markdown](/guides/authoring-markdown/) for the full
syntax.

## Edit a deck without touching JSON

```bash
fireside edit my-first-deck.fireside.json
```

Opens a full-screen, mouse-first editor: click a slide or a block to
select it, drag blocks to reorder them, and the canvas renders through
the exact same code path the presenter uses, so it's always WYSIWYG.
Click `[ ▶ Present ]` any time to try the deck without leaving the
editor. See [Editing a deck](/guides/editing/) for the full walkthrough.

## Where to go next

| If you want to...                                        | Go to                                            |
| ---------------------------------------------------------- | --------------------------------------------------- |
| Build or restructure a deck with the mouse, no JSON         | [Editing a deck](/guides/editing/)                   |
| Learn every key the presenter responds to                  | [Presenting a deck](/guides/presenting/)             |
| Present on a projector with notes on your own laptop screen | [Presenting with two screens](/guides/presenting/#presenting-with-two-screens) |
| Write a talk in Markdown instead of JSON                    | [Authoring a Deck in Markdown](/guides/authoring-markdown/) |
| Hand-write a deck's JSON and see how branching works        | [Your First Fireside Graph](/guides/getting-started/) |
| Look up a `fireside` flag or exit code                      | [CLI Reference](/reference/cli/)                     |
| Understand the graph model conceptually                     | [Mental models](/spec/mental-models/)                |
