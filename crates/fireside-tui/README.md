# fireside-tui

The ratatui presenter. One job: present a validated `Graph` in the terminal so
well that someone who has never seen Fireside can run a deck.

The experience contract:

- The footer always shows exactly the keys valid in the current state.
- Branch points render as menus (`↑↓` + Enter, number keys, author hotkeys).
- Terminal nodes announce themselves (`■ End of this path`).
- `m` opens a map — visited/current markers — that doubles as the goto picker.
- Every blocked action flashes guidance; nothing is a silent no-op.

Architecture: TEA. `App::update` in `src/app.rs` is the **sole** mutation
point; `src/render/` is pure drawing; every color lives in
`src/theme.rs::Tokens`. Content renders through a flat line flow
(`render/blocks.rs`), which makes scrolling, measuring, columns, and centering
simple.

Tested by an in-process scenario suite (`src/render/mod.rs` tests) that drives
real key events through `update` and asserts rendered screens via ratatui's
`TestBackend`.
