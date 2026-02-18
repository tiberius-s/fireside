---
title: 'Deep Dive: fireside-tui'
description: 'TEA-style terminal UI with ratatui, crossterm events, and theme/config systems.'
---

## Why This Crate Exists

`fireside-tui` is the terminal frontend. It renders state from
`fireside-engine` and maps key input to actions.

From a learning perspective, this crate is a strong example of keeping the UI
thin while pushing domain behavior downward.

## Code Map

- `src/app.rs`: app model and update loop (`App`, `AppMode`, `update`, `view`)
- `src/event.rs`: `Action` intent enum
- `src/config/keybindings.rs`: key → action mapping
- `src/ui/presenter.rs`: frame composition (content + progress + help)
- `src/render/*`: content rendering pipeline
- `src/theme.rs`: theme model + parser + merge behavior
- `src/config/mod.rs`: theme resolution/loading

## Rust Patterns Used

### TEA / Elm Architecture in Rust

Flow is: input event → `Action` enum → `App::update` mutation → `App::view`.
This is one of the best patterns for avoiding ad-hoc mutable UI code.

### Intent mapping layer

`map_key_to_action` decouples physical keys from behavior, which makes
rebinding and testability much easier.

### Configuration overlay pattern

`ThemeFile::apply_to(&Theme::default())` merges partial config onto defaults.
This avoids giant required config files.

### Rendering as pure-ish function pipeline

UI functions consume state and return draw calls, with minimal hidden state.
That keeps rendering predictable.

## Rust Book References

- Enums and `match` for event/action handling (Chapter 6):
  <https://doc.rust-lang.org/book/ch06-02-match.html>
- Modules and visibility (Chapter 7):
  <https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html>
- Traits and derivations (`Debug`, `Clone`, etc.) (Appendix C):
  <https://doc.rust-lang.org/book/appendix-03-derivable-traits.html>
- Error propagation with `?` (Chapter 9):
  <https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html>
- Closures in event loops (Chapter 13):
  <https://doc.rust-lang.org/book/ch13-01-closures.html>

## Concepts To Know Before Editing

- Immediate mode rendering (ratatui draw-per-frame)
- State transitions in finite state machines (`AppMode`)
- Why key handling should produce intent, not mutate directly
- Theme parsing and graceful fallback behavior

## Gotchas To Watch

- `update()` currently ignores traversal errors (`let _ = ...`), so UI feedback is limited
- `run_editor` is still a placeholder in CLI; TUI editor path is not complete
- Theme fallback silently resets unknown colors to `Color::Reset`

## Improvement Playbook

### 1) Surface errors to users

Goal: stop swallowing navigation errors.

Steps:

1. Add a transient `status_message` field in `App`.
2. Capture `Err` from traversal calls and store user-friendly messages.
3. Render status in footer or overlay.
4. Add tests for branch-key mismatch and invalid goto.

### 2) Extract update handlers

Goal: keep `App::update` readable as features grow.

Steps:

1. Split actions into small methods (`handle_goto`, `handle_navigation`, etc.).
2. Keep a top-level match that dispatches only.
3. Unit-test each handler with focused state fixtures.
4. Document expected mode transitions.

### 3) Expand configuration model safely

Goal: make behavior configurable without hidden surprises.

Steps:

1. Wire `Settings` into event poll timeout and feature toggles.
2. Add config load path precedence (`CLI > project > defaults`).
3. Validate config values and report clear errors.
4. Add docs for each setting with examples.
