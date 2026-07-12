# Quickstart: Validating the Quick-Edit Modal

## Prerequisites

- `cargo build --workspace`
- A deck file to present, e.g. the `linear`/`branching` template from
  `fireside new`, or `crates/fireside-cli/assets/demo.fireside.json` (note:
  the built-in `fireside demo` command is expected to refuse saves — see
  Scenario 4).

## Scenario 1 — Fix a typo and see it live (User Story 1, P1)

```sh
fireside new my-talk --template branching   # or hand-write a small deck
fireside my-talk.fireside.json
```

1. On the first slide, open the quick-edit modal (key TBD by
   `tasks.md`/implementation — document the chosen key in the CLI help and
   the in-app `?` overlay when implemented).
2. Confirm the modal shows the slide's heading and text block(s), pre-filled
   with current content.
3. Edit the heading text, save.
4. **Expected**: the modal closes, the slide immediately shows the new
   heading, and `cat my-talk.fireside.json` shows the updated text on disk.

## Scenario 2 — Cancel leaves everything untouched (User Story 1)

1. Repeat steps 1–2 above.
2. Edit the text, then cancel instead of saving.
3. **Expected**: the on-screen slide is unchanged, and the file on disk is
   byte-for-byte unchanged (`git diff` shows nothing, or `sha256sum` matches
   before/after).

## Scenario 3 — Other nodes are untouched by a save (User Story 2, P2)

1. Use a deck with at least one branch (e.g. the `branching` template).
2. Note the full content of a node other than the one you'll edit
   (`cat` the file, or `fireside validate` before/after).
3. Quick-edit and save a change on one node.
4. **Expected**: every other node's content and every traversal/branch
   structure is semantically unchanged (`fireside validate` reports the
   same structure; a JSON diff shows changes scoped to the edited node's
   edited block(s), plus expected key-order/whitespace reformatting
   elsewhere per ADR-005).

## Scenario 4 — No file, no save (Edge Case)

```sh
fireside demo
```

1. Open the quick-edit modal on any slide, edit some text, try to save.
2. **Expected**: a clear message that there is no file to save to (not a
   crash, not a silent no-op that discards the edit without explanation).

## Scenario 5 — Nothing to quick-edit (User Story 3, P3)

1. Build or find a node whose content is only non-text blocks (e.g. a
   single `code` or `image` block, no heading/text).
2. Present that node and try to open the quick-edit modal.
3. **Expected**: a clear "nothing to quick-edit here" message; no blank or
   non-functional modal appears.

## Scenario 6 — Concurrent edit conflict (Edge Case, FR-013)

1. Present a deck file.
2. Open the quick-edit modal, but before saving, edit and save the same file
   externally (in another terminal/editor) with different content.
3. Now save from the quick-edit modal.
4. **Expected**: the presenter is warned of the conflict and offered a
   choice (overwrite or abandon), not a silent loss of either version.

## Automated coverage (see `tasks.md` for concrete test files)

- `fireside-tui` scenario suite (`render/mod.rs` `TestBackend` tests):
  open modal → edit → save → assert `Msg::SaveResult`/pending-save wiring;
  open modal on a no-editable-content node → assert the "nothing to
  quick-edit" message; cancel → assert no pending save is produced.
- `fireside-cli` e2e (`tests/cli_e2e.rs`): `Watcher::write_back` unit-level
  behavior isn't reachable from e2e directly (it needs a live TUI session),
  so cover it with a focused unit test in `main.rs` exercising
  `Watcher::write_back` against a temp file for the success, conflict, and
  I/O-failure paths.
