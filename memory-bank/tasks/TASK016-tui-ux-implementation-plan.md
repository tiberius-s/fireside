# TASK016 — TUI UX Implementation Plan

**Status:** Completed
**Added:** 2026-02-20
**Updated:** 2026-02-20

---

## Original Request

Research the Penpot designs, reference excellent TUIs, and produce a complete
implementation plan for a coding agent. Identify gaps in the design before code
lands. Prioritise discoverability so that a non-technical user can navigate
intuitively.

---

## Research Summary

### Penpot designs inspected

| Board                               | Key insight                                                                                                                                                       |
| ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Library 01 — Design System          | Sound token set; One Dark palette with verified WCAG AA ratios                                                                                                    |
| Library 02 — Presenter Layout       | Two-column (content + image) + footer hints; forms the baseline                                                                                                   |
| Library 03 — Editor Layout          | Form-based node detail panel — too sparse, keybinding list consumes left-pane space                                                                               |
| Library 04 — Mode Badges            | Mode badge, undo/redo, status messages all designed; none fully realised in code                                                                                  |
| Library 05 — Branch Overlay         | Dim backdrop + centred dialog; key badges per option — mostly implemented                                                                                         |
| Exploration 02 — Presenter Mode     | Single-column default layout; footer hints `[←] next [←] back [g] goto [?] help`                                                                                  |
| Exploration 03 — Presenter + Branch | Branch overlay centred, key chips styled; current code matches design closely                                                                                     |
| Exploration 04 — Presenter + Help   | **Right-panel help** (55% width) — slide in from right, left 45% dimmed but visible; section tabs at footer `[1] Nav [2] Branch [3] Display [4] General`          |
| Exploration 05 — Editor Mode        | **Structured sections** (METADATA / CONTENT BLOCKS / TRAVERSAL / SPEAKER NOTES) instead of sparse form; flat node list; context-specific footer hints per section |
| Exploration 06 — Editor + Graph     | ASCII-art node graph; branch boxes rendered In gold; minimap top-right; edge colour coding                                                                        |
| Exploration 07 — UX Improvements    | **9 formal proposals** (see below)                                                                                                                                |

### 9 UX Improvement Proposals (from Exploration 07)

1. **Persistent Mode Indicator** — coloured badge always visible in presenter top-right
2. **Progress Bar Upgrade** — next-node preview lightly; branch nodes shown with ⎇ gold segment
3. **Branch Overlay Button Affordance** — options rendered as focusable rows with key chips; first option highlighted; separator lines between rows
4. **Metadata Selectors (Editor)** — replace `◄ Default ►` text with a visual chip row showing adjacent values
5. **Help Overlay Context-Preserving** — right-side panel (55%), left 45% dimmed; section tabs at bottom; no full-screen takeover
6. **Graph Edge Colour Coding** — blue=next, gold=branch, green=after, red=goto
7. **GotoNode Visual Feedback** — mini autocomplete list above footer showing matching node IDs and titles
8. **Undo/Redo Visual State** — chips `[Z undo]` `[Y redo]` dimmed + strikethrough when unavailable
9. **Compact Breakpoint ≤80 cols** — node list collapses to overlay (toggle `n`); footer icon-only hints

### Code audit findings

| File               | Current state                                            | Gap vs. design                                                          |
| ------------------ | -------------------------------------------------------- | ----------------------------------------------------------------------- |
| `ui/presenter.rs`  | Renders node content, goto badge top-right, help, branch | No persistent PRESENTING badge; goto has no node preview                |
| `ui/progress.rs`   | Segment dots, hints, position string, elapsed timer      | Hints hard-coded `[k]/[j]`; no branch-segment colour; no next-node peek |
| `ui/help.rs`       | `centered_popup` modal, all keybindings in one list      | Full-screen, context-destroying; no section tabs                        |
| `ui/branch.rs`     | Dim + centred dialog with option rows                    | First option not highlighted; no row separators; no focused row state   |
| `ui/editor.rs`     | 30/70 split, form detail pane, lower tools pane          | Doesn't follow Exploration 05 structured sections design                |
| `ui/graph.rs`      | List + minimap popup (74×76%)                            | Monochrome edges; not ASCII-art tree; no edge colour coding             |
| `app.rs`           | TEA loop, all AppMode transitions                        | Goto mode has no autocomplete state; no flash-message timer             |
| `render/blocks.rs` | Renders `ContentBlock` variants to ratatui `Line`s       | Previously misnamed `markdown.rs` — now correctly named                 |

### Reference TUIs studied

| TUI         | Key principle adopted                                                                                          |
| ----------- | -------------------------------------------------------------------------------------------------------------- |
| **lazygit** | Context-sensitive footer shows only the _currently useful_ bindings. Spatial memory: pane layout never shifts. |
| **helix**   | Mode indicator always visible in status line, never ambiguous. Key chord display shows partial completions.    |
| **gitui**   | Colour-coded diff sections. Popup prompts are narrow, inline – not full-screen.                                |
| **broot**   | Real-time input drives visible output (autocomplete/filter as-you-type).                                       |
| **ranger**  | Column navigation with persistent preview pane. Esc always cancels and returns to previous context.            |

### Non-tech-savvy user analysis

A first-time user arriving at a Fireside presentation needs to answer these questions at a glance:

1. **Where am I?** — Node position and total count must be permanent and prominent.
2. **What can I do right now?** — The footer must show the 2–3 most important actions for the current context, not 8.
3. **Is this a decision moment?** — Branch nodes must look and feel different before the overlay appears.
4. **Am I safe?** — Destructive actions (quit, delete node) must ask for confirmation with a clear undo path.
5. **Did my action work?** — Status messages need to be visible, timed, and colour-coded.

---

## Design Gaps — Fix Before Coding

These are places where the design is incomplete or ambiguous; a coding agent
must NOT begin Phase 1–8 implementation until these are resolved in the Penpot
file **or** the spec below is accepted.

### GAP-1 — Help panel exact width at narrow terminals

**Problem:** The Exploration 04 design shows "55% width" for the right-panel
help, but at 80 cols that is only 44 chars — not enough to render two-column
`key / description` rows legibly.

**Resolution (accepted):**

- Panel width = `max(45, min(60, area.width * 55 / 100))` columns.
- At ≤80 cols the panel goes full-screen (matching the current modal behaviour)
  as the presentation content behind it has too little value at that width.
- Section tabs at footer are replaced by a scrollable single-column list at
  ≤80 cols.

### GAP-2 — GotoNode autocomplete list height budget

**Problem:** Unknown how many rows are available above the footer before overlapping
presenter content.

**Resolution (accepted):**

- Autocomplete occupies a fixed 6-row floating strip anchored to the bottom, above the progress bar footer.
- Shows up to 5 matches: `  idx │ node-id                │ heading-preview`
- Active (first) match has `border_active` highlight row.
- At narrow terminals (height ≤ 24) reduce to 3 rows.

### GAP-3 — Branch overlay focused row state

**Problem:** Exploration 03 shows option rows but the Penpot design does not
specify what "focused" looks like — required for initial highlight of the first
option so keyboard users get instant feedback.

**Resolution (accepted):**

- First option row bg = `surface` + left accent bar `border_active` 2-char wide.
- Unfocused rows bg = transparent.
- Row separators are a single `─` line in `border_inactive`.
- Arrow keys `↑/↓` move focus; letter key selects directly.

### GAP-4 — Editor structured sections: inline edit affordance

**Problem:** Exploration 05 shows structured sections but it is unclear which
fields are directly editable inline vs. require a picker overlay.

**Resolution (accepted):**
| Section | Editable how |
|---------|-------------|
| METADATA · Layout | Chip row, `←/→` to cycle, or `l` / `L` |
| METADATA · Transition | Chip row, `←/→` to cycle, or `t` / `T` |
| CONTENT BLOCKS | Press `i` on a block to enter inline edit; `a`/`A` to insert after/before |
| TRAVERSAL | Read-only display; jump to referenced node with `Enter` |
| SPEAKER NOTES | Press `o` to open inline edit textarea |

### GAP-5 — Compact breakpoint ≤80 cols in editor mode

**Problem:** What triggers the breakpoint? Viewport width at render time or a persisted setting?

**Resolution (accepted):**

- Determined at render time from `Breakpoint::from_size(area.width, area.height)`.
- `Breakpoint::Compact` (≤80 cols) maps to:
  - Node list hidden by default; toggle with `n`.
  - When shown, node list renders as a 100% width overlay for 30% of height.
  - Footer hints collapse to icon glyphs: `← → g ? e q`.

---

## Implementation Phases

Phases execute in order. Each phase is a self-contained Git commit or small PR.
The coding agent should run `cargo nextest run --workspace` + `cargo clippy --workspace -- -D warnings` before marking any phase done.

---

### Phase 1 — Mode Identity & Always-On Chrome

_Priority: highest — directly unblocks non-tech-savvy users._

#### P1.1 — Persistent PRESENTING badge

**File:** `crates/fireside-tui/src/ui/chrome.rs` (new file)

Create `render_mode_badge(frame, area, mode, theme)`:

- Renders a 1-row × variable-width badge anchored to `area.x + area.width - badge_width`, `area.y`.
- PRESENTING → blue (`heading_h1`) border + bg fill + bold text.
- EDITING → purple (`accent`) border + bg fill.
- GotoNode → gold (`heading_h3`) border (existing `render_goto_badge` handles the buffer display).
- Add `#[must_use]` badge-width calculation helper: `mode_badge_width(mode: &AppMode) -> u16`.

**File:** `crates/fireside-tui/src/ui/presenter.rs`

Call `render_mode_badge` after all other layers, passing `AppMode::Presenting`.
Remove the old duplicate "PRESENT" label from the progress bar right side.

#### P1.2 — Branch-segment colour in progress bar

**File:** `crates/fireside-tui/src/ui/progress.rs`

For each segment `i`, check whether _any_ node mapped to that segment bucket has a `branch_point()`. If yes, colour that segment with `theme.heading_h3` (gold) instead of `theme.border_inactive`.
Current node's segment stays `theme.border_active` (blue), overriding the gold.

```rust
// Pseudocode
let seg_style = if i == active_seg {
    Style::default().fg(theme.border_active)  // current node
} else if any_node_in_bucket_is_branch(i, total, segment_count, session) {
    Style::default().fg(theme.heading_h3)     // branch node bucket
} else {
    Style::default().fg(theme.border_inactive)
};
```

Add helper: `fn any_node_in_bucket_is_branch(seg: usize, total: usize, count: usize, session: &PresentationSession) -> bool`

#### P1.3 — Smarter progress bar footer hints

**File:** `crates/fireside-tui/src/ui/progress.rs`

At ≥120 cols: show `[ ← ] prev  ·  next [ → ]  ·  [?] help  ·  [e] edit`.
At 80–119 cols: show `[ ← ] prev  [ → ] next  [?] help`.
At ≤80 cols (`Breakpoint::Compact`): show `← → ? e` (icon only, no brackets).
The right-side hint changes to `⎇ BRANCH` (gold) when the current node has a branch point, replacing `next [ → ]`.

#### P1.4 — Status flash system

**File:** `crates/fireside-tui/src/app.rs`

Add to `App`:

```rust
flash_message: Option<(String, FlashKind, Instant)>,
```

```rust
pub enum FlashKind { Info, Success, Warning, Error }
```

`App::set_flash(msg, kind)` stores message + `Instant::now()`.
In `view()`, if `flash_message` is set and age < 3 s, render a 1-row strip above the progress bar using the appropriate colour.
Call `self.session.is_dirty()` check → auto-set a `FlashKind::Warning` flash on unsaved changes older than 30 s.

---

### Phase 2 — Context-Preserving Help Panel

#### P2.1 — Right-side panel help overlay

**File:** `crates/fireside-tui/src/ui/help.rs`

Add function `help_panel_rect(area: Rect) -> (Rect, Rect)` returning `(dim_area, panel_area)`:

```rust
let panel_width = (area.width * 55 / 100).max(45).min(60);
// If area.width <= 80: fall through to existing centered_popup
let panel = Rect { x: area.width - panel_width, y: area.y, width: panel_width, height: area.height };
let dim = Rect { x: area.x, y: area.y, width: area.width - panel_width, height: area.height };
```

Replace `centered_popup` call in `render_help_overlay` with:

- Render a semi-transparent `toolbar_bg` dim block over `dim_area`.
- Render a `Clear` + surface block over `panel_area`.
- Content tabs at footer: `[1] Nav  [2] Branch  [3] Display  [4] Editor`.

#### P2.2 — Section-tab navigation in help

**File:** `crates/fireside-tui/src/ui/help.rs`

Add `HelpSection { Nav=0, Branch, Display, Editor }` (4-variant enum).
`HelpNavigation` gains `active_section: HelpSection`.
Number keys `1`–`4` while `show_help` is true jump to that section's first row.
The footer shows the tabs as chips, active tab in `heading_h1`.

**File:** `crates/fireside-tui/src/app.rs`

Handle `KeyCode::Char('1'..='4')` when `show_help && matches!(mode, Presenting)` → set `help_scroll_offset` to the section start row.

---

### Phase 3 — GotoNode Autocomplete

#### P3.1 — Autocomplete state in AppMode

**File:** `crates/fireside-tui/src/app.rs` — `AppMode::GotoNode`

The buffer is already a `String`; no struct change needed.
Add helper: `fn goto_matches<'a>(buffer: &str, session: &'a PresentationSession) -> Vec<(usize, &'a str)>`
Returns `(node_index, node_id)` pairs where `node_id` starts with `buffer`, capped at 5.

#### P3.2 — Autocomplete strip rendering

**File:** `crates/fireside-tui/src/ui/presenter.rs`

Add `render_goto_autocomplete(frame, area, buffer, session, theme)`:

- Computes `goto_matches`.
- Draws a floating strip of up to `min(5, matches.len())` rows.
- Strip anchored to `y = area.height - progress_bar_height - rows`, full-width.
- Row format: ` 1 │ intro       │ # Getting Started …`
- Active row (index 0) has `bg = surface`, `fg = heading_h1`.
- Column widths: idx=4, id=16, heading=rest.

**File:** `crates/fireside-tui/src/ui/presenter.rs` `render_presenter`

After `render_goto_badge`, call `render_goto_autocomplete` when `goto_buffer` is `Some`.

---

### Phase 4 — Branch Overlay Focus & Affordance

**File:** `crates/fireside-tui/src/ui/branch.rs`

#### P4.1 — Focused row tracking

Add `focused_option: usize` to the branch overlay render path.
In `App`, add `branch_focused_option: usize`, reset to 0 when a new branch node is entered.
`↑/↓` keys in `AppMode::Presenting` when `node.branch_point().is_some()` adjust `branch_focused_option`.

#### P4.2 — Styled focused row

In `render_branch_overlay`, draw each option row as:

```
  ▌ [a]  Developer Track                   ← if focused
    [b]  Designer Track                    ← normal
```

- Left accent bar: 2-char `▌ ` in `border_active` on focused row.
- Focused row bg: `surface`.
- Letter key: `[X]` rendered as `on_surface` bg, `heading_h2` key, `on_surface` label.
- Row separator: `─────` line in `border_inactive`.

#### P4.3 — Enter / letter key selection

`Enter` in presenting mode when branch visible → activate focused option.
Letter key skips focus and selects directly (existing behaviour).

---

### Phase 5 — Editor Structured Sections

This is the largest phase. Replaces the right-side form pane with four named sections following Exploration 05.

#### P5.1 — New detail panel layout

**File:** `crates/fireside-tui/src/ui/editor.rs`

Replace the right-panel form rendering with `render_detail_panel(frame, area, node, session, theme, view_state)`:

```
┌──────────────────────────────────────────────────────┐
│ node-id  [1/14]  •  * unsaved                        │  ← 1 row header
├─ METADATA ───────────────────────────────────────────┤
│ Layout      [ Default ] [ Split H ] [ Split V ] …    │  ← chip row (horizontal scroll)
│ Transition  [ None ]    [ Slide L ] [ Slide R ] …    │
├─ CONTENT BLOCKS (2) ─────────────────────────────────┤
│  1. Heading  "Hello, Fireside!"                       │
│  2. Text     "Fireside is a portable format…"         │
│     [i]=edit  [a]=add after  [d]=delete               │
├─ TRAVERSAL ──────────────────────────────────────────┤
│  next         →  setup                               │
│  branch-point    (none)                              │
├─ SPEAKER NOTES (empty) ──────────────────────────────┤
│  (no notes – press o to add)                         │
└──────────────────────────────────────────────────────┘
```

Section headers use `border_inactive` horizontal rule + label in `muted` style.

#### P5.2 — Metadata chip row

Reuse the concept from the existing picker overlay but rendered inline.
Chip row shows all enum variants; active one has `heading_h1` fg + `surface` bg.
Adjacent chips are dimmed `footer` fg.
`←/→` cycle; `l/L` and `t/T` remain active for compatibility.

#### P5.3 — Content blocks summary list

Iterate `node.content`: emit numbered rows with `block_type_glyph(block)` + truncated preview text.
`block_type_glyph`: `H` heading, `T` text, `C` code, `L` list, `I` image, `D` divider, `X` extension.
Selected block row (when `focus == NodeDetail`) has highlighted bg.

#### P5.4 — Traversal section

Read-only. Shows:

- `next → <id>` or `next → (sequential)`
- `after  → <id>` or `after → (n/a)`
- `branch-point → (none)` or `branch-point → <prompt>`

`Enter` on a row with a target ID jumps `editor_selected_node` to that node.

#### P5.5 — Speaker notes editable area

When notes section has focus and user presses `o`:

- Enter inline text edit (existing `EditorInlineTarget::SpeakerNotes`).
- Notes textarea expands to fill the section.
- `Esc` commits changes.

#### P5.6 — Undo/Redo chips in footer

**File:** `crates/fireside-tui/src/ui/editor.rs`

Replace current `undo=yes redo=no` text in the footer with:

```
[ Z undo ]  [ Y redo ]   •   e/Esc=present   ?=help   EDITING
```

- When `can_undo` is false: `Z undo` in `border_inactive` fg + `Modifier::DIM`.
- When available: `Z undo` in `success` fg.
- Same for `Y redo`.

---

### Phase 6 — Graph Edge Colour Coding

**File:** `crates/fireside-tui/src/ui/graph.rs`

#### P6.1 — Classify edges

Add `EdgeKind` enum: `Next | Branch | After | Goto`.
In `graph_item_lines(session, idx)`:

1. Determine this node's outgoing edge type by inspecting `node.traversal`:
   - Has `branch_point` → `Branch`.
   - Has explicit `after` → `After`.
   - Has explicit `next` that skips sequential order → `Goto`.
   - Otherwise → `Next`.

#### P6.2 — Coloured edge spans

Arrow text (`──▶`) rendered with:

- `Next` → `heading_h1` (blue).
- `Branch` → `heading_h3` (gold), label shows `[a/b/c]`.
- `After` → `success` (green).
- `Goto` → `error` (red).

Branch node box border rendered in `heading_h3`.
Current-node box border rendered in `border_active` + BOLD.

#### P6.3 — Legend in minimap area

Add a 4-row legend below the minimap:

```
── next   (blue)
── branch (gold)
── after  (green)
── goto   (red)
```

---

### Phase 7 — Compact Breakpoint ≤80 cols

**File:** `crates/fireside-tui/src/design/tokens.rs` → `Breakpoint`

Ensure `Breakpoint::Compact` triggers at `width ≤ 80`.

**File:** `crates/fireside-tui/src/ui/editor.rs`

At `Breakpoint::Compact`:

- Check `app.editor_node_list_visible` (new bool, default `false`).
- If hidden: render right panel 100% width.
- If shown: render node list as top overlay for 30% height.
- Footer hints: glyph-only (`n ? e q`).

**File:** `crates/fireside-tui/src/app.rs`

Add `editor_node_list_visible: bool` to `App`.
`KeyCode::Char('n')` in `AppMode::Editing && breakpoint == Compact` → toggle.

**File:** `crates/fireside-tui/src/ui/presenter.rs`

At `Breakpoint::Compact`:

- Suppress left/right padding (full-width content area).
- Footer hints: icon-only line.

---

### Phase 8 — Polish & Delight

#### P8.1 — Quit confirmation inline

**File:** `crates/fireside-tui/src/app.rs` / `ui/presenter.rs`

Replace the current `AppMode::Quitting` full-frame handling with a 1-row inline confirmaton banner at the bottom of the main area:

```
  Save and quit? [y]es  [n]o  [s]ave first  [Esc] cancel
```

Render this inside `render_presenter` when `AppMode::Quitting` (for presenter) and `render_editor` when quitting from editor.
No separate full-screen clearing required.

#### P8.2 — Welcome / no-file screen

**File:** `crates/fireside-cli/src/commands/session.rs` or `app.rs`

When no file is given (or file not found), display a single-page welcome presentation generated in-memory with a sample branching graph demonstrating Fireside's features.
This acts as a "zero-state onboarding" experience.

#### P8.3 — Transition polish

**File:** `crates/fireside-tui/src/ui/presenter.rs` `transition_lines`

For `Transition::SlideLeft`/`SlideRight`, interpolate using smooth easing (`ease_out_cubic`):

```rust
fn ease_out_cubic(t: f32) -> f32 { 1.0 - (1.0 - t).powi(3) }
```

Apply to the column offset so the slide decelerates into position.

---

## Implementation Order

```
Phase 1  (P1.1–P1.4)  — Mode identity & chrome     ← start here
Phase 4  (P4.1–P4.3)  — Branch overlay affordance
Phase 2  (P2.1–P2.2)  — Help panel
Phase 3  (P3.1–P3.2)  — GotoNode autocomplete
Phase 5  (P5.1–P5.6)  — Editor restructure
Phase 6  (P6.1–P6.3)  — Graph edge colours
Phase 7               — Compact breakpoint
Phase 8               — Polish
```

Phases 1 + 4 land first because they directly affect the non-tech-savvy user path and are the smallest changes with the highest trust-building value.

---

## New Files

| Path                                   | Purpose                                                               |
| -------------------------------------- | --------------------------------------------------------------------- |
| `crates/fireside-tui/src/ui/chrome.rs` | Shared persistent chrome: `render_mode_badge`, `render_flash_message` |

## Modified Files

| Path                                      | Changes                                                                            |
| ----------------------------------------- | ---------------------------------------------------------------------------------- |
| `crates/fireside-tui/src/ui/presenter.rs` | Add mode badge call, goto autocomplete call, quit-inline banner                    |
| `crates/fireside-tui/src/ui/progress.rs`  | Branch-segment colouring, adaptive footer hints                                    |
| `crates/fireside-tui/src/ui/help.rs`      | Right-panel layout, section tabs                                                   |
| `crates/fireside-tui/src/ui/branch.rs`    | Focused row state, styled row rendering                                            |
| `crates/fireside-tui/src/ui/editor.rs`    | Structured sections, chip rows, undo/redo chips                                    |
| `crates/fireside-tui/src/ui/graph.rs`     | Edge colour coding, legend                                                         |
| `crates/fireside-tui/src/app.rs`          | `flash_message`, `branch_focused_option`, `editor_node_list_visible`, goto helpers |
| `crates/fireside-tui/src/ui/mod.rs`       | Export `chrome` module                                                             |

## Tests to Write

Each phase should add at minimum:

- 1 unit test verifying the new helper computes correct output (e.g., `any_node_in_bucket_is_branch`, `goto_matches`, `help_panel_rect`).
- 1 integration snapshot test (using `insta` or text comparison) for the main layout function.
- Existing `hello_smoke.rs` smoke test must continue to pass.

## Known Risks

| Risk                                                                                                        | Mitigation                                                                                                 |
| ----------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| Phase 5 editor restructure is large; may conflict with in-flight editor inline edit                         | Land Phase 5 as its own atomic PR; keep existing picker overlay code as fallback until sections are proven |
| Help panel at ≤80 cols degrades to full-screen (existing behaviour)                                         | Explicit width guard in `help_panel_rect`; covered by `Breakpoint::Compact` test                           |
| Graph edge classification may be wrong for edge cases (e.g. sequential nodes with explicit `next` override) | Write a dedicated table-driven test for `classify_edge(node, idx, session)`                                |
| `ease_out_cubic` transition changes may affect snapshot tests                                               | Gate transition tests with `#[cfg(feature = "transition-tests")]`                                          |

---

## Progress Tracking

**Overall Status:** Completed — 100 %

### Sub-tasks

| ID  | Description                                                         | Status                          |
| --- | ------------------------------------------------------------------- | ------------------------------- |
| 0.1 | Accept / refine five design gaps (GAP-1 through GAP-5)              | Complete — resolved in this doc |
| 1.1 | `ui/chrome.rs`: `render_mode_badge` + PRESENTING badge in presenter | Complete                        |
| 1.2 | `progress.rs`: branch-segment colouring                             | Complete                        |
| 1.3 | `progress.rs`: adaptive footer hints                                | Complete                        |
| 1.4 | `app.rs`: flash message system                                      | Complete                        |
| 2.1 | `help.rs`: right-panel layout                                       | Complete                        |
| 2.2 | `help.rs`: section tabs + key navigation                            | Complete                        |
| 3.1 | `app.rs`: `goto_matches` helper                                     | Complete                        |
| 3.2 | `presenter.rs`: autocomplete strip render                           | Complete                        |
| 4.1 | `branch.rs` + `app.rs`: focused option state                        | Complete                        |
| 4.2 | `branch.rs`: styled focused row + separators                        | Complete                        |
| 4.3 | `app.rs`: Enter / arrow key selection                               | Complete                        |
| 5.1 | `editor.rs`: detail panel layout skeleton                           | Complete                        |
| 5.2 | `editor.rs`: metadata chip row                                      | Complete                        |
| 5.3 | `editor.rs`: content blocks summary list                            | Complete                        |
| 5.4 | `editor.rs`: traversal section                                      | Complete                        |
| 5.5 | `editor.rs`: speaker notes editable area                            | Complete                        |
| 5.6 | `editor.rs`: undo/redo chips in footer                              | Complete                        |
| 6.1 | `graph.rs`: `classify_edge` + `EdgeKind`                            | Complete                        |
| 6.2 | `graph.rs`: coloured edge spans + branch box                        | Complete                        |
| 6.3 | `graph.rs`: legend in minimap area                                  | Complete                        |
| 7.1 | `editor.rs` + `app.rs`: compact breakpoint routing                  | Complete                        |
| 7.2 | `presenter.rs`: compact presenter layout                            | Complete                        |
| 8.1 | Quit confirmation inline banner                                     | Complete                        |
| 8.2 | Welcome / no-file screen                                            | Complete                        |
| 8.3 | `ease_out_cubic` transition easing                                  | Complete                        |

Notes:

- Sub-task 3.1 is complete with `goto_matches` implemented in presenter module scope.
- Sub-task 5.1 is complete with sectioned editor detail rendering and metadata/content/traversal/notes grouping.
- Sub-task 6.2 is complete with branch-node header marker/styling polish and edge classification tests.

### 2026-02-20 Progress Log

- Implemented shared presenter chrome with persistent mode badge in `ui/chrome.rs` and integrated it into presenter rendering.
- Added goto autocomplete strip in presenter mode and moved goto badge placement to avoid overlap with persistent mode badge.
- Implemented branch overlay focus state with arrow-key navigation and Enter activation, including focused-row affordance and row separators.
- Switched help overlay to context-preserving right-side panel on wide terminals with 4-section jump model and compact fallback behavior.
- Added adaptive progress hints by width tier and branch-aware segment coloring in footer progress visualization.
- Added compact editor node-list toggle behavior and updated footer undo/redo chips to `[Z undo]` and `[Y redo]` with dim unavailable states.
- Added graph edge classification and colorized edge rendering with minimap legend.
- Completed graph branch-box polish with branch-node header emphasis and added table-driven edge-classification coverage in `ui/graph.rs` tests.
- Added `ease_out_cubic` transition easing for `SlideLeft` / `SlideRight` in presenter transition rendering.
- Added global flash message lifecycle in `app.rs` + `ui/chrome.rs` with timed expiry and dirty-duration warning behavior.
- Added inline quit confirmation banner rendering in presenter/editor contexts with `y/n/s/Esc` handling.
- Updated event-loop ticking in `fireside-cli` session runner to support timed UI updates when flash/dirty timers are active.
- Updated compact presenter rendering to use full-width content area at `Breakpoint::Compact`.
- Refactored editor detail pane to structured sections (METADATA / CONTENT BLOCKS / TRAVERSAL / SPEAKER NOTES) with metadata chips and per-block summaries.
- Added no-arg welcome flow via in-memory graph session: `fireside` now launches a welcome presentation; `fireside edit` with no path opens welcome when no local project exists.
- Validation complete: `cargo fmt --check`, `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, and `cargo test -p fireside-tui --no-fail-fast` all passed.
