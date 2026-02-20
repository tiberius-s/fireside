---
title: 'Chapter 8: The Elm Architecture in Rust'
description: 'Separate events, state updates, and rendering with a TEA-style loop in terminal UI.'
---

## Learning Objectives

- Trace TEA flow from input event to rendered frame.
- Separate intent mapping from state mutation.
- Keep rendering functions stateless and predictable.
- Understand why TEA improves testability in TUI apps.

## Concept Introduction

The Elm Architecture (TEA) is a simple but powerful loop: input produces an
intent, intent updates state, and state renders view. Rust and TEA pair well
because enum intents, exhaustive matching, and strict mutability rules keep the
loop honest. Fireside applies this in a terminal context through crossterm and
ratatui.

In practice, TEA prevents a common UI pitfall: mutating state in many event
handlers spread across render code. Fireside avoids this by mapping keys to
`Action` first, then routing all mutation through `App::update`. Rendering code
reads state and paints frames, but does not decide domain transitions. This
separation keeps bugs local and easy to test.

Intent mapping is especially useful for keybindings. Physical keys differ across
modes, while behavior stays stable. `map_key_to_action` translates key events to
semantic actions (`NextNode`, `EditorUndo`, `GotoConfirm`). The update loop then
operates only on semantic actions. That decoupling makes rebinding possible
without rewriting mutation logic.

TEA also scales with complexity. As modes grow (presenting, goto, editing,
overlays), the state machine remains explicit in `AppMode` and action matches.
You can still unit test update behavior by constructing app state and applying
one action at a time, without launching a terminal. That is a major productivity
advantage over tightly coupled event-driven UI code.

Finally, redraw policy becomes manageable. Fireside uses a `needs_redraw` flag,
set during updates and consumed by the CLI session loop. This keeps rendering
efficient while preserving correctness: state updates request repaint, idle loops
do not redraw unnecessarily.

## Fireside Walkthrough

Source anchors: `crates/fireside-tui/src/event.rs`,
`crates/fireside-tui/src/config/keybindings.rs`, and
`crates/fireside-tui/src/app.rs`.

```rust
// key event -> action
pub fn map_key_to_action(key: KeyEvent, mode: &AppMode) -> Option<Action>

// single mutation hub
pub fn update(&mut self, action: Action) {
    self.needs_redraw = true;
    match action {
        Action::NextNode => { /* ... */ }
        // ...
    }
}
```

Why this design:

- One mutation hub reduces hidden side effects.
- Stateless renderers are easier to reason about.
- Mode-aware key mapping keeps logic modular.

## Exercise

Add one new action end-to-end (for example, toggling a compact footer mode):

1. add enum variant in `event.rs`
2. bind a key in `keybindings.rs`
3. implement behavior in `App::update`
4. render visible state change in presenter UI

## Verification

Run:

```bash
cargo test -p fireside-tui
```

## What would break if…

If rendering functions started mutating session state directly, behavior would
depend on draw order and frame frequency. Bugs would become timing-sensitive,
input handling would duplicate logic, and tests would require full UI execution
instead of small update-level assertions.

## Key Takeaways

TEA is a discipline that pays off as features accumulate. In Rust, enums plus
centralized update logic create a robust architecture for interactive tools.
Fireside demonstrates how intent mapping, explicit modes, and pure rendering can
keep a terminal app maintainable even as editor and graph workflows expand.
