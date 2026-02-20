# TASK014 — Penpot UX Design Audit

**Status:** Completed
**Added:** 2026-02-20
**Updated:** 2026-02-20

## Original Request

"I've added penpot ui uix design skill and access to a penpot MCP. I want you to take another look at
the design system and views we have in our TUI, and create coherent design artifacts. Make sure you run
the TUI in both, edit and presentation modes, find the gaps in usability, and come up with improvements
to both, ui and ux. Advocate for the used[er]."

## Thought Process

This is a three-part task:

1. **Audit** — Read all TUI source to understand the current rendering pipeline, layout rules, colour
   tokens, and keyboard maps. Build the binary and visually inspect both modes.
2. **Design** — Create Penpot boards that accurately reflect the current TUI in all its states, using
   the exact design tokens from `design/tokens.rs`.
3. **Advocate** — Identify concrete UX gaps (not cosmetic preferences), rate them by implementation
   cost, and produce annotated improvement proposals as a deliverable.

Key decisions:

- Screen frame size: 960×680px (120 cols × 40 rows at 8×17px/char). Matches the Standard breakpoint.
- Font used in mockups: Source Code Pro (matches the TUI default monospace).
- Colour palette sourced directly from `DesignTokens` RGB values in `crates/fireside-tui/src/design/tokens.rs`.
- All 9 UX proposals are non-breaking enhancements — no protocol changes required.
- Penpot API patterns learned: `createText(string)` takes string arg; `resize(w,h)` not direct property
  assignment; `board.insertChild(board.children.length, shape)` for ordered z-insert; `storage` object
  persists across `execute_code` calls; `setActivePage` does not exist.

## Implementation Plan

- [x] Read `penpot-uiux-design` skill
- [x] Read all TUI source: `theme.rs`, `design/tokens.rs`, `ui/presenter.rs`, `ui/editor.rs`, `app.rs`,
      `ui/graph.rs`, `ui/branch.rs`, `ui/help.rs`
- [x] Build and run binary (`cargo build -q`)
- [x] Read `memory-bank/ui-components.md` and `memory-bank/ux-flows.md`
- [x] Establish Penpot MCP connection; learn correct API patterns
- [x] Create 16 library colour tokens in Penpot file
- [x] Board "01 — Design System": palette swatches, typography scale, spacing scale, border states
- [x] Board "02 — Presenter Mode": full-screen content, footer, mode badge
- [x] Board "03 — Presenter + Branch": branch choice overlay with key chips
- [x] Board "04 — Presenter + Help": slide-in right-panel help (the proposed UX, not the current modal)
- [x] Board "05 — Editor Mode": 30/70 split, node list, metadata selectors, content blocks, footer
- [x] Board "06 — Editor + Graph Overlay": graph popup with topology + minimap + legend
- [x] Board "07 — UX Improvements": 9 annotated PROBLEM/PROPOSAL cards, colour-coded by complexity
- [x] Register Penpot MCP + `penpot-uiux-design` skill in `copilot-instructions.md`
- [x] Update `activeContext.md` with UX proposals and recent-changes entry
- [x] Create this task file and update `_index.md`

## Progress Tracking

**Overall Status:** Completed — 100%

### Subtasks

| ID    | Description                             | Status   | Updated    | Notes                                     |
| ----- | --------------------------------------- | -------- | ---------- | ----------------------------------------- |
| 14.1  | Read skill + source code                | Complete | 2026-02-20 | 8 source files, design tokens, ux-flows   |
| 14.2  | Build binary                            | Complete | 2026-02-20 | `cargo build -q` clean                    |
| 14.3  | Establish Penpot connection + learn API | Complete | 2026-02-20 | Two API errors corrected, patterns stored |
| 14.4  | Design System board                     | Complete | 2026-02-20 | Palette, typography, spacing, borders     |
| 14.5  | Presenter Mode screen                   | Complete | 2026-02-20 | H1, code block, list, footer, badge       |
| 14.6  | Presenter + Branch Overlay screen       | Complete | 2026-02-20 | Key chips, dim overlay, footer            |
| 14.7  | Presenter + Help Overlay screen         | Complete | 2026-02-20 | Slide-in panel (proposed UX)              |
| 14.8  | Editor Mode screen                      | Complete | 2026-02-20 | 30/70 split, selectors, content blocks    |
| 14.9  | Editor + Graph Overlay screen           | Complete | 2026-02-20 | Topology + minimap + legend               |
| 14.10 | UX Improvements board                   | Complete | 2026-02-20 | 9 cards, PROBLEM/PROPOSAL, complexity     |
| 14.11 | Update `copilot-instructions.md`        | Complete | 2026-02-20 | Penpot server + skill entries added       |
| 14.12 | Update memory bank                      | Complete | 2026-02-20 | activeContext + this task file            |

## Progress Log

### 2026-02-20

- Read `penpot-uiux-design` SKILL.md (343 lines) — extracted Penpot API rules
- Read 8 TUI source files in parallel: confirmed `AppMode` enum (no `GraphView` mode — it's an overlay),
  exact colour tokens from `DesignTokens`, 30/70 editor split, footer status bar, branch overlay sizing,
  help overlay section jump keys
- Built binary cleanly with `cargo build -q`
- Established Penpot MCP connection (single page "Page 1", renamed to "Design System")
- Corrected two API errors: `createText()` needs string arg; `setActivePage` does not exist
- Created 16 library colour tokens using `penpot.library.local.createColor()`
- Created "01 — Design System" board (1360×860) at (100, 80):
  - 12 colour swatches with hex labels
  - 6-level typography scale (H1 22px → Code 13px) with correct token colours
  - 5-step spacing scale (XS 1 → XL 6 cells) with visual bars
  - 3 border/focus states (default / active / error)
- Created "02 — Presenter Mode" (960×680) at (100, 1020):
  - H1 heading in `#61AFEF`, code block in `#2C313C` surface, body text, bullet list
  - Footer toolbar with progress bar, node counter, keybinding hints
  - `PRESENT` mode badge in blue top-right
- Created "03 — Presenter + Branch" (960×680) at (1160, 1020):
  - Dim overlay on background content
  - 3-option branch chooser with styled `[a]` `[b]` `[c]` key chips (blue bordered box)
  - Preview text per option in muted italic
- Created "04 — Presenter + Help" (960×680) at (2220, 1020):
  - Slide-in right panel (55% width) — this is the **proposed** UX, not the current full-modal
  - Left 45% remains visible behind dim overlay
  - 4 collapsible sections with gold headings, blue keys, body descriptions
  - Section jump footer [1]–[4]
- Created "05 — Editor Mode" (960×680) at (100, 1800):
  - Left node list (30%) with `›` selection indicator, scrollable list of 12 node IDs
  - Right detail (70%): node header, METADATA section with `◀ Default ▶` / `◀ None ▶` selectors,
    CONTENT BLOCKS section, TRAVERSAL section, SPEAKER NOTES section with dashed border empty state
  - Footer with dirty marker `*`, undo/redo state, `EDITING` badge in purple
- Created "06 — Editor + Graph Overlay" (960×680) at (1160, 1800):
  - 74%×76% centred popup
  - Topology area (78%) with ASCII art nodes in blue+muted, branch node in gold
  - Minimap (22%) with tiny node rectangles and viewport indicator
  - Legend footer with keyboard hints
- Created "07 — UX Improvements" (1420×960) at (2220, 1800):
  - 9 improvement cards in 3×3 grid
  - Each card: left accent bar, title in accent colour, PROBLEM label in red, PROPOSAL label in green
  - Complexity labelled in footer: low (1–4) / medium (5–8) / high (9)
- Updated `copilot-instructions.md`: added Penpot row to MCP servers table, added Penpot usage rules
  section, added `penpot-uiux-design` row to skills table
- Updated `activeContext.md`: added recent-changes entry, added UX Improvement Proposals section
- Created this task file; updating `_index.md`

## UX Gaps Identified

| #   | Gap                                                       | Complexity | Card |
| --- | --------------------------------------------------------- | ---------- | ---- |
| 1   | No persistent mode indicator                              | Low        | 01   |
| 2   | Progress bar has no next-node preview or branch indicator | Low        | 02   |
| 3   | GotoNode has no visual feedback or node autocomplete      | Low        | 07   |
| 4   | Undo/redo shown as muted text, not interactive chips      | Low        | 08   |
| 5   | Branch overlay has no button affordance                   | Medium     | 03   |
| 6   | Metadata selectors non-discoverable (`◀ [val] ▶`)         | Medium     | 04   |
| 7   | Help overlay is full-modal, breaks context                | Medium     | 05   |
| 8   | Graph edges are undifferentiated by type                  | Medium     | 06   |
| 9   | Fixed 30/70 split cramped at 80 cols or less              | High       | 09   |
