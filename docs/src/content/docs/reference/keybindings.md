---
title: 'Keybindings'
description: 'Canonical keyboard mappings for presentation, editor, and go-to modes.'
---

Canonical source: `crates/fireside-tui/src/config/keybindings.rs`.

## Presentation Mode

| Key                        | Action               | Notes                                             |
| -------------------------- | -------------------- | ------------------------------------------------- |
| `→`, `Space`, `Enter`, `l` | Next node            | Sequential next unless traversal override applies |
| `←`, `h`                   | Previous node        | Back navigation                                   |
| `g`                        | Enter go-to mode     | Opens numeric node prompt                         |
| `?`                        | Toggle help overlay  | Context-aware help dialog                         |
| `s`                        | Toggle speaker notes | Presenter-only notes panel                        |
| `e`                        | Enter editor mode    | Switches from presenter to editor                 |
| `a`..`f`                   | Choose branch option | Active at branch points                           |
| `q`, `Esc`, `Ctrl-c`       | Quit                 | Exit app                                          |

## Editor Mode

| Key           | Action                    | Notes                         |
| ------------- | ------------------------- | ----------------------------- |
| `j`, `↓`      | Select next node          | Editor node list navigation   |
| `k`, `↑`      | Select previous node      | Editor node list navigation   |
| `PageDown`    | Page down                 | Move by viewport page         |
| `PageUp`      | Page up                   | Move by viewport page         |
| `Home`        | Jump top                  | Select first node             |
| `End`         | Jump bottom               | Select last node              |
| `/`           | Start node-id search      | Search input mode             |
| `[`           | Previous search hit       | Step backward through matches |
| `]`           | Next search hit           | Step forward through matches  |
| `g`           | Start index jump          | Jump by numeric node index    |
| `Tab`         | Toggle focus              | Swap pane focus               |
| `i`           | Start inline edit         | Edit selected node text       |
| `o`           | Start notes edit          | Edit speaker notes            |
| `l`           | Open layout picker        | Choose node layout            |
| `L`           | Cycle layout previous     | Backward layout cycle         |
| `t`           | Open transition picker    | Choose transition             |
| `T`           | Cycle transition previous | Backward transition cycle     |
| `a`           | Append text block         | Adds quick text block         |
| `n`           | Add node                  | Insert node                   |
| `d`           | Remove node               | Delete selected node          |
| `v`           | Toggle graph view overlay | Editor graph map              |
| `w`, `Ctrl-s` | Save graph                | Write graph to target path    |
| `u`           | Undo                      | Undo latest editor command    |
| `r`           | Redo                      | Redo latest undone command    |
| `Esc`         | Exit editor mode          | Return to presenter mode      |
| `?`           | Toggle help overlay       | Context-aware help dialog     |
| `q`, `Ctrl-c` | Quit                      | Exit app                      |

## Go-To Mode

| Key      | Action        | Notes                     |
| -------- | ------------- | ------------------------- |
| `0`..`9` | Enter digit   | Builds target node number |
| `Enter`  | Confirm go-to | Jumps to entered node     |
| `Esc`    | Cancel go-to  | Returns to previous mode  |
