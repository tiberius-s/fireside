---
title: 'App State Machine'
description: 'The App struct, AppMode FSM, Action intent model, update() dispatch loop, and mode transitions in fireside-tui.'
---

`app.rs` is the largest file in the `fireside-tui` crate at over 2,300 lines.
It owns three things: the complete application state (`App`), the finite state
machine that describes which mode the UI is in (`AppMode`), and the update
function that is the **sole mutation point** for the entire crate.

## `App` — the model

`App` is a plain Rust struct with no `Arc`, no `Mutex`, and no interior
mutability. Every piece of UI state lives in it:

```rust
pub struct App {
    // ── Core domain ──────────────────────────────────────────
    pub session:              PresentationSession,   // graph + traversal + command history
    pub mode:                 AppMode,

    // ── Presenter overlays ───────────────────────────────────
    pub show_help:            bool,
    pub show_speaker_notes:   bool,
    help_scroll_offset:       usize,
    active_transition:        Option<ActiveTransition>,
    show_progress_bar:        bool,
    show_elapsed_timer:       bool,

    // ── Editor state ─────────────────────────────────────────
    pub editor_selected_node: usize,
    pub editor_focus:         EditorPaneFocus,
    pub editor_target_path:   Option<PathBuf>,
    pub editor_text_input:    Option<String>,
    pub editor_status:        Option<String>,
    editor_inline_target:     Option<EditorInlineTarget>,
    pending_exit_action:      Option<PendingExitAction>,
    editor_picker:            Option<EditorPickerOverlay>,
    editor_search_input:      Option<String>,
    editor_search_query:      Option<String>,
    editor_index_jump_input:  Option<String>,
    editor_list_scroll_offset: usize,
    editor_graph_overlay:     bool,
    editor_graph_selected_node: usize,
    editor_graph_scroll_offset: usize,

    // ── Rendering and system ─────────────────────────────────
    pub theme:                Theme,
    pub start_time:           Instant,
    pub terminal_size:        (u16, u16),
    document_base_dir:        Option<PathBuf>,
    needs_redraw:             bool,
}
```

Private fields (no `pub`) are an intentional boundary: they may only be
changed through `update()`. Public fields are accessed read-only by `ui/`
compositor functions and by the CLI layer for initialization.

## `AppMode` — the finite state machine

```rust
pub enum AppMode {
    Presenting,
    Editing,
    GotoNode { buffer: String },
    Quitting,
}
```

The valid transitions are:

```text
        e         Esc / q
Presenting ──────► Editing ──────────────────► Quitting
    │  ▲               │                           ▲
  g │  │ Esc/Enter/Esc │ Esc                       │
    ▼  │               ▼                           │
 GotoNode          (any mode) ──── q / Ctrl-C ─────┘
```

State transitions are always explicit `match` arms inside `update()`. No
state transition happens in render functions or keybinding dispatch — those
are read-only.

`GotoNode { buffer }` is the only variant carrying data. The digits accumulated
by successive `GotoDigit(d)` actions are appended to `buffer`; `GotoConfirm`
parses `buffer` as a 1-based node number and calls `session.traversal.goto`.

## `Action` — the intent model

`Action` is defined in `event.rs` and is the protocol between the input
dispatch layer and `App::update`. There are approximately 45 variants organized
into semantic groups:

| Group             | Example variants                                                |
| ----------------- | --------------------------------------------------------------- |
| Navigation        | `NextNode`, `PrevNode`, `GoToNode(usize)`, `ChooseBranch(char)` |
| Mode transitions  | `EnterEditMode`, `ExitEditMode`, `EnterGotoMode`, `Quit`        |
| Go-to input       | `GotoDigit(usize)`, `GotoConfirm`, `GotoCancel`                 |
| Help / overlays   | `ToggleHelp`, `ToggleSpeakerNotes`                              |
| Editor navigation | `EditorSelectNextNode`, `EditorPageDown`, `EditorJumpTop`       |
| Editor mutations  | `EditorAddNode`, `EditorRemoveNode`, `EditorAppendTextBlock`    |
| Editor metadata   | `EditorOpenLayoutPicker`, `EditorCycleTransitionNext`           |
| Editor I/O        | `EditorSaveGraph`, `EditorUndo`, `EditorRedo`                   |
| System            | `Resize(u16, u16)`, `MouseClick { column, row }`, `Tick`        |

This separation of intent from mechanism is the core benefit of the TEA
pattern: keybinding tests check that a physical key produces the right
`Action`; `update()` tests verify that an `Action` produces the right state
change. Neither test requires a running event loop.

## Keybinding dispatch

`map_key_to_action(key: KeyEvent, mode: &AppMode) → Option<Action>` in
`config/keybindings.rs` is the only function that knows about physical keys.
It returns `None` for unbound keys (which `update()` ignores) and an `Action`
for bound keys. The function branches by `AppMode` first:

```rust
match mode {
    AppMode::GotoNode { .. } => return map_goto_mode_key(key),
    AppMode::Editing         => return map_edit_mode_key(key),
    AppMode::Presenting | AppMode::Quitting => {}   // fall through to shared bindings
}
```

`GotoNode` and `Editing` have completely separate keymaps defined in private
helper functions. Presenter-mode bindings are in the main `match key.code`
block. This structure means adding a new mode requires adding one `match` arm
and one private helper — the rest of the codebase is unaffected.

## `update()` — the sole mutation point

`App::update(&mut self, action: Action)` is a top-level `match` over `Action`.
The function is long by necessity — each non-trivial action requires reading
and writing several `App` fields — but it is structured to avoid buried
control flow. Three conventions keep it maintainable:

**No nested `if let` chains for mode guards.** Each match arm starts with an
explicit early-return if the action is only valid in a particular mode:

```rust
Action::EditorAddNode => {
    if self.mode != AppMode::Editing { return; }
    // ... mutation
}
```

**Transition animation computed at navigation time.** When `NextNode` succeeds
(`TraversalResult::Moved`), `update()` samples `graph.nodes[from].transition`
and constructs an `ActiveTransition`. The `Tick` action advances
`active_transition.frame` and clears the field when the animation completes.
This keeps animation state co-located with navigation state rather than in a
separate subsystem.

**Dirty flag on every mutation.** Any action that calls
`session.command_history.apply_command` also sets the `session.dirty` flag.
The save-confirmation dialog checks `session.dirty` before allowing
mode transitions that would discard unsaved changes.

## `ActiveTransition` — animation state

```rust
struct ActiveTransition {
    from_index:   usize,
    kind:         Transition,
    frame:        u8,
    total_frames: u8,
}

impl ActiveTransition {
    fn progress(self) -> f32 {
        if self.total_frames <= 1 { 1.0 }
        else { self.frame as f32 / (self.total_frames - 1) as f32 }
    }
}
```

`progress()` returns a normalized `f32` in `[0.0, 1.0]`. The presenter
renderer uses this value to compute blend ratios for visual transitions (fade,
slide, etc.). When `progress() == 1.0` the transition is complete and
`active_transition` is set to `None` by the next `Tick` action.

`total_frames` is derived from `Transition` variant at the time navigation
occurs. All frame counts are small integers; the `Tick` interval in the event
loop is ~16ms, giving approximately 60fps animation capability for a 16-frame
transition.

## Hot-reload

`update()` handles `Action::Tick` which also checks for file change timestamps
when in `AppMode::Presenting`. On detecting a change, it calls
`load_graph(path)` and replaces `self.session.graph`. The traversal state is
preserved by:

1. Saving the current node's ID before reload.
2. Calling `traversal.clamp_to_graph(new_len)` to handle structural changes.
3. Attempting `graph.node_by_id(saved_id)` and calling `traversal.goto(idx)`
   if the ID still exists.
4. Falling back to the clamped index if the ID was removed.

This sequence ensures the presenter stays at approximately the same position
even when the document is edited externally while presenting.

## Editor state fields

The editor mode state is kept in a flat set of `App` fields rather than a
nested `EditorState` struct. The rationale: `view()` needs to read editor
fields even when compositing the presenter frame (e.g., to apply the dirty
indicator in the status bar), so a nested struct would require field
forwarding. The naming convention `editor_*` provides the logical grouping
without structural nesting.

Notable editor fields:

- **`editor_text_input: Option<String>`** — the live buffer for inline text
  editing and node-ID search. `None` when no text input is active.
- **`editor_picker: Option<EditorPickerOverlay>`** — the active picker overlay
  (`Layout` or `Transition`). Stores the selected index for cyclic navigation.
- **`pending_exit_action: Option<PendingExitAction>`** — set when a mode
  transition that would discard unsaved changes is requested. The
  confirmation dialog reads this to know which action to take on confirmation.

## Testing strategy

`app.rs` has extensive inline tests covering:

- Mode transitions via action sequences
- Help overlay scroll behavior
- Graph overlay navigation and selection
- Hot-reload ID preservation and index clamping
- Editor breadcrumb status on mode entry

Tests construct a minimal `App` with a fixture graph via `App::new(session, Theme::default())`,
drive it with `app.update(action)` calls, and assert field values. No test
requires a running terminal or ratatui frame — all assertions are on model
state, which is the key advantage of the strict TEA model separation.
