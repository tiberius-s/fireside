# Contract: editor `hit()` region-testing

New `crates/fireside-tui/src/editor/hit.rs`, generalizing the existing
pattern in `crates/fireside-tui/src/render/hits.rs:26,51`
(`branch_option_hit`, `map_row_hit`) — see `research.md` §1 for why this
shape, not a stateful hover registry.

```text
fn hit(app: &EditorApp, area: Rect, col: u16, row: u16) -> Option<Target>
```

Pure: recomputes the same layout the last frame drew (from
`app.terminal_size` and `app.working_graph`/`app.selection`/`app.open_form`
— never from anything the renderer produced), then resolves `(col, row)`
against it. No render-to-update back-channel — the same TEA-purity
guarantee `render/hits.rs`'s existing functions already provide.

## `Target` enum (one variant per interactive region)

| Variant | Region | Triggered actions |
| --- | --- | --- |
| `ToolbarChip(ToolbarAction)` | One of the 5 toolbar chips (add slide, present, save, undo, help) | Click → run the chip's action |
| `OutlineRow(NodeId)` | A slide row in the outline | Click → select that slide; press+drag → outline reorder drag |
| `OutlineNewSlide` | The permanent `＋ new slide` outline row | Click → `AddSlide` |
| `Block(NodeId, BlockPath)` | A block's rendered extent on the canvas | Click → select; press+drag from anywhere on it (`research.md` §5) → block reorder drag; double-click → open its form |
| `BlockChip(NodeId, BlockPath, BlockAction)` | One of a selected block's contextual chips (`✎ Edit`, `＋ Add below`, `↑`, `↓`, `Reveal ▾`, `Delete`) | Click → run the action |
| `InsertionSlot(NodeId, BlockPath, usize)` | The gap between two blocks (hover-revealed `── ＋ add a block here ──`, always click-reachable) | Click → open the add-block palette targeting this position; drag-over → drop indicator during a block drag |
| `GoesToChip(NodeId)` | The `[ change ]` chip on the "Goes to" strip | Click → open the slide picker |
| `FormChip(FormChipKind)` | `[ Done ]` / `[ Cancel ]` / palette card / picker row inside the currently open form | Click → form-specific action |
| `StatusBanner` | The status line, when it shows a diagnostic | Click → jump selection to the offending slide/block |

`None` when `(col, row)` misses every region — same "click elsewhere
deselects" semantics the design brief specifies.

## Priority order (top-most drawn wins)

Toolbar chips > open-form chips (a form, when open, captures the area it
occupies) > block/insertion-slot regions on the canvas > outline rows >
status banner > `None`. Exactly one `Target` (or `None`) per call — no
overlap resolution beyond this fixed priority, matching how
`render/hits.rs`'s existing functions resolve overlapping candidates today
(most-specific-drawn-last wins).

## Drag resolution (not a separate function — `Target` sequencing)

A drag is: a `press` event resolving to `Block(..)` or `OutlineRow(..)`,
followed by `move` events each re-resolving to an `InsertionSlot`/
`OutlineRow` (the current drop candidate, stored in `EditorApp::drag`), and
a `release` event committing the move at the last-resolved slot via
`MoveBlock`/`ReorderSlide`. `hit()` itself is stateless per call — the drag
state machine lives in `EditorApp`, not in `hit()`.

## Test contract

Table-driven unit tests: `(EditorApp fixture, area, col, row) -> expected Target`,
covering at minimum: each toolbar chip's exact cell, a block's full
rendered extent (not just its top-left cell), an insertion slot's narrow
band, an outline row, a coordinate outside every region (`None`), and the
priority order when regions would otherwise overlap (an open form drawn
over the canvas).
