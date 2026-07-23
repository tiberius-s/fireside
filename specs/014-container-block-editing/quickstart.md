# Quickstart: Validating Container Block Editing

Prerequisites: workspace builds (`cargo build --release -p fireside-cli`),
`tmux` available for the real-terminal smoke case.

## 1. Automated checks

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
```

Expect new/updated unit tests in `crates/fireside-tui/src/editor/{mod,hit,forms}.rs`
and a new `TestBackend` scenario in `crates/fireside-tui/src/render/mod.rs`
covering: selecting a container child via Tab, via click, opening its
form, editing it, reordering it, and deleting it — see
`contracts/nested-block-selection.md`'s Test contract section for the
exact required cases.

## 2. Manual walkthrough (the bundled demo deck's container slides)

```sh
fireside edit .github/demo-editing.fireside.json   # or any deck with
                                                     # a columns/box/stack
                                                     # slide, e.g. the
                                                     # "Welcome" slide
```

1. `]`/`[` to the "Welcome" (or "Layout"/"Extras"/"Finale") slide — a
   container-built slide.
2. `Tab` repeatedly: confirm the container's individual children (title,
   tagline, divider, closing text, etc.) each become selectable in turn,
   not just the container as a whole (User Story 1, Acceptance Scenario
   1–2).
3. Click directly on one child's rendered text: confirm that child (not
   the container) gets the selection glow.
4. `Enter` (or the child's `[ ✎ Edit ]` chip): confirm the child's own
   per-kind form opens (heading/text/etc.), not the container's layout
   form.
5. Open the container's own form (select the container itself, not a
   child, then edit); confirm the `ChildSummary` list rows are now
   selectable and each opens the matching child's form (User Story 1,
   Acceptance Scenario 3).
6. Edit a child's text, `[ Done ]`, `Ctrl+S` to save; `p` to preview via
   the embedded presenter; confirm the change is visible there too (same
   WYSIWYG guarantee spec 013's own quickstart/tape already verify for
   top-level blocks).
7. Drag a child to reorder it relative to its siblings; confirm the new
   order persists after save (User Story 2, Acceptance Scenario 1).
8. Select a child and delete it; confirm only that child disappears and
   its siblings are untouched (User Story 2, Acceptance Scenario 2).
9. Reduce a container to its last child, delete that child too; confirm
   the container becomes an empty container rather than being removed
   itself (User Story 2, Acceptance Scenario 3), and that Tab/click on it
   now behaves exactly as an empty container does today (User Story 1,
   Acceptance Scenario 4).
10. Select a container (empty, or with existing children) and add a new
    block "inside" it; confirm the new block becomes a child, not a new
    top-level sibling (User Story 3).

## 3. Real-terminal smoke case

Add a case to `scripts/smoke.sh` (tmux SGR mouse injection) covering:
click a container child, confirm the glow via `capture-pane`, drag it to
reorder, confirm the drop. Required per Constitution Principle VII for
any mouse-driven UI change — `TestBackend` alone cannot catch a real
click/drag ordering bug (per project convention: prior editor work found
outline-scroll and drag-auto-scroll bugs only under tmux).

## Success signal

All of the bundled demo deck's four container slides (`welcome`,
`layout`, `extras`, `finale`) are fully editable at the child level with
no JSON required — matching spec.md's SC-001.
