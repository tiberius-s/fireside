# TUI Implementation Guidelines — Coding Agent Reference

> **Purpose**: This document translates the Penpot design system into actionable implementation
> specifications for coding agents working on Fireside's Rust TUI crates. It covers the Rose Pine
> colour palette, component specifications, UX improvement priorities, and code-level guidance.
>
> **Design Source**: Penpot Design System page — 9 sections, 39 content boards, 31 reusable components.

---

## 1. Rose Pine Colour Palette

The default theme is **Rosé Pine** (Main variant). The `Theme::default()` implementation in
`crates/fireside-tui/src/theme.rs` must be updated from One Dark to Rose Pine.

### 1.1 Complete Palette — Main Variant

| Role           | Hex       | RGB               | CSS Variable      |
| -------------- | --------- | ----------------- | ----------------- |
| base           | `#191724` | `(25, 23, 36)`    | `--color-base`    |
| surface        | `#1f1d2e` | `(31, 29, 46)`    | `--color-surface` |
| overlay        | `#26233a` | `(38, 35, 58)`    | `--color-overlay` |
| muted          | `#6e6a86` | `(110, 106, 134)` | `--color-muted`   |
| subtle         | `#908caa` | `(144, 140, 170)` | `--color-subtle`  |
| text           | `#e0def4` | `(224, 222, 244)` | `--color-text`    |
| love           | `#eb6f92` | `(235, 111, 146)` | `--color-love`    |
| gold           | `#f6c177` | `(246, 193, 119)` | `--color-gold`    |
| rose           | `#ebbcba` | `(235, 188, 186)` | `--color-rose`    |
| pine           | `#31748f` | `(49, 116, 143)`  | `--color-pine`    |
| foam           | `#9ccfd8` | `(156, 207, 216)` | `--color-foam`    |
| iris           | `#c4a7e7` | `(196, 167, 231)` | `--color-iris`    |
| highlight-low  | `#21202e` | `(33, 32, 46)`    | `--color-hl-low`  |
| highlight-med  | `#403d52` | `(64, 61, 82)`    | `--color-hl-med`  |
| highlight-high | `#524f67` | `(82, 79, 103)`   | `--color-hl-high` |

### 1.2 Moon Variant (alternative dark theme)

| Role    | Hex       | RGB               |
| ------- | --------- | ----------------- |
| base    | `#232136` | `(35, 33, 54)`    |
| surface | `#2a273f` | `(42, 39, 63)`    |
| overlay | `#393552` | `(57, 53, 82)`    |
| muted   | `#6e6a86` | `(110, 106, 134)` |
| subtle  | `#908caa` | `(144, 140, 170)` |
| text    | `#e0def4` | `(224, 222, 244)` |
| love    | `#eb6f92` | `(235, 111, 146)` |
| gold    | `#f6c177` | `(246, 193, 119)` |
| rose    | `#ebbcba` | `(235, 188, 186)` |
| pine    | `#3e8fb0` | `(62, 143, 176)`  |
| foam    | `#9ccfd8` | `(156, 207, 216)` |
| iris    | `#c4a7e7` | `(196, 167, 231)` |

### 1.3 Dawn Variant (light theme)

| Role    | Hex       | RGB               |
| ------- | --------- | ----------------- |
| base    | `#faf4ed` | `(250, 244, 237)` |
| surface | `#fffaf3` | `(255, 250, 243)` |
| overlay | `#f2e9e1` | `(242, 233, 225)` |
| muted   | `#9893a5` | `(152, 147, 165)` |
| subtle  | `#797593` | `(121, 117, 147)` |
| text    | `#575279` | `(87, 82, 121)`   |
| love    | `#b4637a` | `(180, 99, 122)`  |
| gold    | `#ea9d34` | `(234, 157, 52)`  |
| rose    | `#d7827e` | `(215, 130, 126)` |
| pine    | `#286983` | `(40, 105, 131)`  |
| foam    | `#56949f` | `(86, 148, 159)`  |
| iris    | `#907aa9` | `(144, 122, 169)` |

### 1.4 Theme Struct Mapping

Update `Theme::default()` in [crates/fireside-tui/src/theme.rs](crates/fireside-tui/src/theme.rs):

```rust
impl Default for Theme {
    fn default() -> Self {
        Self {
            // Base — Rosé Pine base (#191724) and text (#E0DEF4)
            background: Color::Rgb(25, 23, 36),
            foreground: Color::Rgb(224, 222, 244),

            // Surface — Rosé Pine surface (#1F1D2E) and text (#E0DEF4)
            surface: Color::Rgb(31, 29, 46),
            on_surface: Color::Rgb(224, 222, 244),

            // Headings — foam (#9CCFD8), pine (#31748F), gold (#F6C177)
            heading_h1: Color::Rgb(156, 207, 216),
            heading_h2: Color::Rgb(49, 116, 143),
            heading_h3: Color::Rgb(246, 193, 119),

            // Code blocks — overlay bg, text fg, muted border
            code_background: Color::Rgb(38, 35, 58),
            code_foreground: Color::Rgb(224, 222, 244),
            code_border: Color::Rgb(110, 106, 134),

            // Misc content — muted for both
            block_quote: Color::Rgb(110, 106, 134),
            footer: Color::Rgb(110, 106, 134),

            // Chrome — foam for active border, overlay bg for toolbar
            border_active: Color::Rgb(156, 207, 216),
            border_inactive: Color::Rgb(64, 61, 82),
            toolbar_bg: Color::Rgb(31, 29, 46),
            toolbar_fg: Color::Rgb(144, 140, 170),

            // Semantic — iris accent, love error, pine success
            accent: Color::Rgb(196, 167, 231),
            error: Color::Rgb(235, 111, 146),
            success: Color::Rgb(49, 116, 143),

            syntax_theme: String::from("base16-ocean.dark"),
        }
    }
}
```

### 1.5 DesignTokens Mapping

Update `DesignTokens::default()` in [crates/fireside-tui/src/design/tokens.rs](crates/fireside-tui/src/design/tokens.rs):

| Token Field       | Rose Pine Role | Hex       |
| ----------------- | -------------- | --------- |
| `background`      | base           | `#191724` |
| `surface`         | surface        | `#1f1d2e` |
| `primary`         | foam           | `#9ccfd8` |
| `accent`          | iris           | `#c4a7e7` |
| `muted`           | muted          | `#6e6a86` |
| `error`           | love           | `#eb6f92` |
| `success`         | pine           | `#31748f` |
| `on_background`   | text           | `#e0def4` |
| `on_surface`      | text           | `#e0def4` |
| `on_primary`      | base           | `#191724` |
| `heading_h1`      | foam           | `#9ccfd8` |
| `heading_h2`      | pine           | `#31748f` |
| `heading_h3`      | gold           | `#f6c177` |
| `body`            | text           | `#e0def4` |
| `code_fg`         | text           | `#e0def4` |
| `code_bg`         | overlay        | `#26233a` |
| `quote`           | muted          | `#6e6a86` |
| `footer`          | muted          | `#6e6a86` |
| `border_active`   | foam           | `#9ccfd8` |
| `border_inactive` | highlight-med  | `#403d52` |
| `toolbar_bg`      | surface        | `#1f1d2e` |
| `toolbar_fg`      | subtle         | `#908caa` |

---

## 2. Semantic Colour Roles

These are the semantic meanings of each Rose Pine colour in the Fireside TUI:

| Colour      | Hex       | semantic Usage                                                |
| ----------- | --------- | ------------------------------------------------------------- |
| **foam**    | `#9ccfd8` | Primary interactive: active borders, links, focused UI, H1    |
| **pine**    | `#31748f` | Secondary emphasis: success states, H2, Presenting mode badge |
| **iris**    | `#c4a7e7` | Accent: Editing mode badge, accent highlights, redo key       |
| **gold**    | `#f6c177` | Warning/attention: H3, GotoNode mode, branch-ahead flag       |
| **love**    | `#eb6f92` | Error/destructive: error states, over-time timer, quit        |
| **rose**    | `#ebbcba` | Warm accent: Branch mode badge, soft emphasis                 |
| **text**    | `#e0def4` | Primary text: body content, headings, labels                  |
| **subtle**  | `#908caa` | Secondary text: toolbar text, captions, hints                 |
| **muted**   | `#6e6a86` | Tertiary text: footer, disabled states, unfocused borders     |
| **base**    | `#191724` | Background: main terminal background                          |
| **surface** | `#1f1d2e` | Elevated panels: toolbars, sidebars, overlays                 |
| **overlay** | `#26233a` | Code blocks, popup backgrounds, deeply elevated surfaces      |
| **hl-med**  | `#403d52` | Selection, focus highlights, inactive borders                 |
| **hl-high** | `#524f67` | Hover states, stronger highlights                             |

---

## 3. Component Specifications

The design system defines 31 reusable components. Each is implemented as a Ratatui
rendering function. Key specs:

### 3.1 Mode Badges

Located in [crates/fireside-tui/src/ui/chrome.rs](crates/fireside-tui/src/ui/chrome.rs).

| Variant    | Text        | Colour         | Background       |
| ---------- | ----------- | -------------- | ---------------- |
| Presenting | `■ PRESENT` | pine `#31748f` | hl-med `#403d52` |
| Editing    | `✎ EDITING` | iris `#c4a7e7` | hl-med `#403d52` |
| GotoNode   | `⊞ GOTO`    | gold `#f6c177` | hl-med `#403d52` |
| Branch     | `⎇ BRANCH`  | rose `#ebbcba` | hl-med `#403d52` |

- Position: top-right corner, 1-cell padding from edges
- Border: 1px, colour matches text colour
- Font: Source Code Pro or any monospace, bold

### 3.2 Status Chips

| Variant | Text        | Colour          |
| ------- | ----------- | --------------- |
| Saved   | `● saved`   | pine `#31748f`  |
| Unsaved | `● unsaved` | gold `#f6c177`  |
| No File | `○ no file` | muted `#6e6a86` |

- Positioned in editor footer, left-aligned after mode badge

### 3.3 Progress Bar

Located in [crates/fireside-tui/src/ui/progress.rs](crates/fireside-tui/src/ui/progress.rs).

- Segment dots: filled `●` / empty `○`
- Filled colour: foam `#9ccfd8`
- Empty colour: hl-med `#403d52`
- Branch marker: `⎇` in gold `#f6c177`
- End marker: `■` in rose `#ebbcba`
- Position counter: `Node N / M` in subtle `#908caa`
- Next-node indicator: `→ node-title` in subtle
- Elapsed timer: `MM:SS` in muted `#6e6a86`

### 3.4 Buttons

| Variant   | Background       | Text Colour     | Border |
| --------- | ---------------- | --------------- | ------ |
| Primary   | foam `#9ccfd8`   | base `#191724`  | foam   |
| Secondary | hl-med `#403d52` | text `#e0def4`  | muted  |
| Danger    | love `#eb6f92`   | base `#191724`  | love   |
| Disabled  | hl-med `#403d52` | muted `#6e6a86` | muted  |

- Height: 36px (3 terminal rows in design; 1 row in TUI)
- Keybinding chip integrated: e.g., `[Enter] Confirm`

### 3.5 Keybinding Chips

- Background: hl-med `#403d52`
- Text: text `#e0def4`
- Border: subtle `#908caa`
- Single character: `?`, `e`, `v`, `q`, etc.
- Spacing: 1 cell padding per side

### 3.6 Input Fields

| State   | Border Colour   | Background        |
| ------- | --------------- | ----------------- |
| Default | muted `#6e6a86` | surface `#1f1d2e` |
| Active  | foam `#9ccfd8`  | surface `#1f1d2e` |
| Error   | love `#eb6f92`  | surface `#1f1d2e` |

- Width: 200px design (25 chars in TUI)
- Height: 32px design (1 row in TUI)
- Placeholder text: muted `#6e6a86`

### 3.7 Branch Options

Located in [crates/fireside-tui/src/ui/branch.rs](crates/fireside-tui/src/ui/branch.rs).

| State   | Key Chip Colour  | Bar Colour     | Text Colour |
| ------- | ---------------- | -------------- | ----------- |
| Default | subtle `#908caa` | transparent    | text        |
| Focused | foam `#9ccfd8`   | foam `#9ccfd8` | text (bold) |

- Key chips: `a`, `b`, `c` etc. in keybinding chip style
- Focused row: left accent bar (2px design, `│` in TUI) + highlight-med background
- Footer: `↑↓ Navigate · a/b/c Jump · Enter Select`

### 3.8 Block Type Labels

Used in editor detail pane for content block identification.

| Block Type | Icon | Colour           |
| ---------- | ---- | ---------------- |
| Heading    | `H1` | foam `#9ccfd8`   |
| Text       | `¶`  | text `#e0def4`   |
| Code       | `<>` | gold `#f6c177`   |
| List       | `•`  | text `#e0def4`   |
| Image      | `🖼` | iris `#c4a7e7`   |
| Divider    | `──` | muted `#6e6a86`  |
| Container  | `⬛` | subtle `#908caa` |
| Extension  | `⚡` | rose `#ebbcba`   |

### 3.9 Content Block Rendering

Located in [crates/fireside-tui/src/render/blocks.rs](crates/fireside-tui/src/render/blocks.rs).

| Block Kind | Style Details                                                      |
| ---------- | ------------------------------------------------------------------ |
| Heading H1 | foam `#9ccfd8`, bold, underline decoration                         |
| Heading H2 | pine `#31748f`, bold                                               |
| Heading H3 | gold `#f6c177`, bold                                               |
| Body text  | text `#e0def4`, regular                                            |
| Code block | overlay `#26233a` bg, muted `#6e6a86` border, syntect highlighting |
| Blockquote | muted `#6e6a86` border + text, left bar `│` in muted               |
| List items | text colour, `•` or `1.` prefix                                    |
| Divider    | `───` in muted `#6e6a86`                                           |
| Footer     | muted `#6e6a86` text                                               |

---

## 4. Spacing & Layout

### 4.1 Spacing Scale (terminal cells)

| Token | Value | Usage                                   |
| ----- | ----- | --------------------------------------- |
| `XS`  | 1     | Tight: inline padding, compact gaps     |
| `SM`  | 2     | Standard: section spacing, list indent  |
| `MD`  | 3     | Medium: header-to-content gap           |
| `LG`  | 4     | Generous: panel padding on wide screens |
| `XL`  | 6     | Maximum: title page vertical padding    |

### 4.2 Responsive Breakpoints

| Breakpoint | Condition                  | Content Width | H Padding |
| ---------- | -------------------------- | ------------- | --------- |
| Compact    | `≤80 cols` or `≤24 rows`   | 96%           | XS (1)    |
| Standard   | `≤120 cols` or `≤40 rows`  | 85%           | SM (2)    |
| Wide       | `>120 cols` and `>40 rows` | 75%           | LG (4)    |

### 4.3 Layout Templates

The `NodeTemplate` enum in [crates/fireside-tui/src/design/templates.rs](crates/fireside-tui/src/design/templates.rs)
maps `Layout` variants to area computations. Each template defines:

- Content width percentage per breakpoint
- Vertical padding
- Secondary area placement (for speaker notes, split layouts)

---

## 5. UX Improvement Specifications (Priority Order)

### Tier 1 — Low Complexity (implement first)

#### UX-01: Persistent Mode Indicator

- **Status**: Partially implemented (`ModeBadgeKind` exists in `chrome.rs`)
- **Spec**: Always-visible badge in top-right shows current mode
- **Variants**: Present (pine), Editing (iris), Goto (gold), Branch (rose)
- **File**: [crates/fireside-tui/src/ui/chrome.rs](crates/fireside-tui/src/ui/chrome.rs)
- **Work needed**: Verify colour mapping matches Rose Pine. Add Branch badge variant.

#### UX-02: Progress Bar Upgrade

- **Status**: Implemented in `progress.rs`
- **Spec**: Show next-node ID/title for sequential flow; gold `⎇ BRANCH` indicator when
  branch ahead; rose `■ END` at end of path
- **File**: [crates/fireside-tui/src/ui/progress.rs](crates/fireside-tui/src/ui/progress.rs)
- **Work needed**: Add next-node title display. Add branch-ahead and end-of-path indicators.
  Update colours to Rose Pine.

#### UX-08: Undo/Redo Visual State

- **Spec**: Show undo/redo availability in editor footer bar
- **States**: Both available (normal), no redo (redo chip disabled), no history (both disabled)
- **Key colours**: Z → foam `#9ccfd8`, Y → iris `#c4a7e7`
- **Disabled**: Highlight-low bg, 40% opacity, strikethrough
- **Active**: Highlight-med bg, full opacity
- **File**: [crates/fireside-tui/src/ui/chrome.rs](crates/fireside-tui/src/ui/chrome.rs) (new component)
- **Work needed**: New `render_undo_redo_chips()` function. Integrate into editor footer.

#### UX-13: Session Timeline

- **Spec**: Compact horizontal strip between content and footer
- **Shows**: Last N visited nodes with IDs and labels
- **Branch transitions**: Marked with `⎇` in gold
- **Current node**: Highlighted in pine
- **Toggle**: `Ctrl+H`
- **File**: New file [crates/fireside-tui/src/ui/timeline.rs](crates/fireside-tui/src/ui/timeline.rs)
- **Work needed**: New module. Track visited nodes in `App` state. Render as scrollable strip.

#### UX-14: Focus/Zen Mode

- **Spec**: `Ctrl+F` toggles distraction-free mode
- **Hides**: Footer, progress bar, mode badge — content fills 100%
- **Shows**: Only the node content on base background
- **File**: [crates/fireside-tui/src/app.rs](crates/fireside-tui/src/app.rs) (new `show_zen_mode: bool` field)
- **Work needed**: Add field to `App`. Gate chrome rendering. Add `Action::ToggleZenMode`.

#### UX-15: Presenter Timer & Pace Guide

- **Spec**: Extend existing elapsed timer with target duration
- **Colours**: Pine (on pace), gold (slightly behind), love (over time)
- **Footer display**: `04:32 / 15:00 ● On pace`
- **File**: [crates/fireside-tui/src/ui/progress.rs](crates/fireside-tui/src/ui/progress.rs)
- **Work needed**: Add `target_duration` to App/Session. Colour-code timer. Add pace indicator.

#### UX-17: Micro-Interactions & Polish

- **Spec**: Subtle feedback animations for state changes
- **Mode badge**: Cross-fade transition (2-frame, 100ms)
- **Save flash**: Pulse green on save, settle to steady state
- **Focus slide**: Node list highlight bar animates to new position
- **File**: Various UI files
- **Work needed**: Frame-based animation system in `App::update` via `Action::Tick`.
  Use intermediate render states during transitions.

### Tier 2 — Medium Complexity

#### UX-03: Branch Overlay Affordance

- **Status**: Implemented in `branch.rs`
- **Spec**: Styled key chips, focus highlight with accent bar, footer keybinding hints
- **File**: [crates/fireside-tui/src/ui/branch.rs](crates/fireside-tui/src/ui/branch.rs)
- **Work needed**: Verify key chip styling matches design. Add accent bar on focused option.
  Update colours to Rose Pine.

#### UX-04: Metadata Selectors (Editor Mode)

- **Spec**: Replace arrow-cycling selectors with visible option chips
- **Before**: `◀ Default ▶` (one-at-a-time cycling)
- **After**: All options visible as chips, active one highlighted in accent
- **File**: [crates/fireside-tui/src/ui/editor.rs](crates/fireside-tui/src/ui/editor.rs)
- **Work needed**: Redesign layout/transition picker rendering. Show all options as horizontal chips.

#### UX-05: Context-Preserving Help

- **Status**: Implemented (help slides from right in overlay)
- **Spec**: Content stays visible but dimmed. Help panel slides from right.
- **File**: [crates/fireside-tui/src/ui/help.rs](crates/fireside-tui/src/ui/help.rs)
- **Work needed**: Verify dimming effect. Update key reference colours. Ensure context preservation.

#### UX-06: Graph Edge Colour Coding

- **Spec**: Edges coloured by relationship type in graph overlay
- **Colours**: Next → foam, Branch → gold, After → iris, Goto → rose
- **Legend**: Rendered at bottom of graph overlay
- **File**: [crates/fireside-tui/src/ui/graph.rs](crates/fireside-tui/src/ui/graph.rs)
- **Work needed**: Classify edge types. Apply distinct colours. Add legend.

#### UX-07: GotoNode Visual Feedback

- **Spec**: Auto-completing dropdown filtered as you type
- **Node IDs**: Gold `#f6c177`, name text in white
- **Focused row**: Highlight-med background
- **Border**: Gold indicates GotoNode mode
- **Match count**: `N / M match` in subtle text
- **File**: [crates/fireside-tui/src/app.rs](crates/fireside-tui/src/app.rs) (GotoNode rendering section)
- **Work needed**: Add autocomplete dropdown rendering. Filter nodes as user types.
  Show match count. Highlight focused result.

#### UX-09: Compact Breakpoint (≤80 cols)

- **Status**: Partially implemented (breakpoint detection exists)
- **Spec**: Adaptive layout for small terminals
- **Key features**: Node list becomes overlay toggled by `n`, compressed footer showing
  only icons (`→← ? v g q`), title bar with node count
- **File**: [crates/fireside-tui/src/ui/editor.rs](crates/fireside-tui/src/ui/editor.rs)
- **Work needed**: Compact-mode rendering paths for editor. `n` key toggle. Compressed footer.

#### UX-10: Breadcrumb Navigation Trail

- **Spec**: Shows navigation path with branch points marked
- **Format**: `intro → basics ⎇ branching → advanced`
- **`⎇` separator**: Gold, marks branch choices
- **Jump**: Click/key to jump to any ancestor
- **Shortcut**: `Ctrl+←` jumps to last branch point
- **Truncation**: `...` when path exceeds available width
- **File**: New file [crates/fireside-tui/src/ui/breadcrumb.rs](crates/fireside-tui/src/ui/breadcrumb.rs)
- **Work needed**: Track navigation path in `App` state. New render function.
  New action `JumpToBranchPoint`.

#### UX-11: Node Preview on Hover/Focus

- **Spec**: Preview panel appears when focus changes in branch overlay
- **Shows**: Node title, content summary, block count, layout, next node
- **File**: [crates/fireside-tui/src/ui/branch.rs](crates/fireside-tui/src/ui/branch.rs)
- **Work needed**: Preview panel rendering. Fetch target node content for preview.

#### UX-12: Command Palette

- **Spec**: VS Code-style `Ctrl+P` or `:` command palette
- **Features**: Fuzzy matching against all actions, mode-aware filtering
- **Rendering**: Centered overlay, text input at top, scrollable results list
- **Result format**: `Action name | keybinding | category`
- **File**: New file [crates/fireside-tui/src/ui/command_palette.rs](crates/fireside-tui/src/ui/command_palette.rs)
- **Work needed**: New overlay system. Action registry with descriptions/categories.
  Fuzzy matcher. New `AppMode::CommandPalette` or overlay flag.

#### UX-16: Content Block Minimap

- **Spec**: Thin vertical strip on right edge of editor detail pane
- **Block type colours**: Heading=foam, Text=text, Code=gold, List=text,
  Image=iris, Divider=muted, Container=subtle, Extension=rose
- **Current block**: Highlighted indicator
- **File**: New file [crates/fireside-tui/src/ui/minimap.rs](crates/fireside-tui/src/ui/minimap.rs)
- **Work needed**: New component. Render block overview. Scroll-linked to detail pane.

---

## 6. Architecture Rules for Implementation

### 6.1 Crate Boundaries (enforced)

| Crate             | Owns                                         | Must NOT contain                  |
| ----------------- | -------------------------------------------- | --------------------------------- |
| `fireside-core`   | Protocol types, serde, wire format           | I/O, UI, rendering, validation    |
| `fireside-engine` | Loading, validation, traversal, undo/redo    | `ratatui`, `crossterm`, rendering |
| `fireside-tui`    | Ratatui UI, rendering, themes, design tokens | Business logic, direct file I/O   |
| `fireside-cli`    | `main()`, terminal lifecycle, clap dispatch  | State, rendering, business logic  |

### 6.2 TEA Pattern

```
crossterm::Event
  → App::handle_event()
    → map_key_to_action(key, &mode)
      → Option<Action>
        → App::update(&mut self, action)  // SOLE MUTATION POINT
          → App::view(&self, frame)       // PURE RENDER
```

- **All state changes** go through `App::update()`. Never mutate `App` fields outside `update()`.
- **All rendering** is a pure function of `&App`. Render functions take `&App` (or specific fields) and `&mut Frame`.
- **`needs_redraw`** gate: Only call `view()` when `take_needs_redraw()` returns `true`.

### 6.3 Adding New Actions

1. Add variant to `Action` enum in [crates/fireside-tui/src/event.rs](crates/fireside-tui/src/event.rs)
2. Add key mapping in [crates/fireside-tui/src/config/keybindings.rs](crates/fireside-tui/src/config/keybindings.rs)
3. Handle in `App::update()` in [crates/fireside-tui/src/app.rs](crates/fireside-tui/src/app.rs)
4. Update help text in [crates/fireside-tui/src/ui/help.rs](crates/fireside-tui/src/ui/help.rs)

### 6.4 Adding New UI Components

1. Create new file in `crates/fireside-tui/src/ui/` (e.g., `breadcrumb.rs`)
2. Add `pub mod breadcrumb;` in [crates/fireside-tui/src/ui/mod.rs](crates/fireside-tui/src/ui/mod.rs)
3. Implement `render_*()` function taking `&App` (or specific fields) + `&mut Frame` + `Rect`
4. Call from `render_presenter()` or `render_editor()` as appropriate
5. Use `DesignTokens::from_theme(&app.theme)` for semantic colour access
6. Respect breakpoint: `let bp = Breakpoint::from_size(area.width, area.height);`

### 6.5 Adding New Overlay State

For overlays (command palette, node preview, etc.):

1. Add boolean field to `App`: `show_command_palette: bool`
2. Add `Action::ToggleCommandPalette` variant
3. In `App::update()`, toggle the boolean
4. In `view()`, render the overlay on top of existing content when active
5. Overlays should dim the background content (50% opacity effect via darker style)
6. Handle Esc to dismiss the overlay

### 6.6 Coding Standards Reminders

- No `unwrap()`/`expect()` in library code
- `#[must_use]` on all value-returning functions
- `///` doc comments on all public items
- `//!` module-level docs on all modules
- Use `Result`/`Option` for fallible operations
- `tracing::warn!` for recoverable render failures — never panic
- After structural graph mutation, call `Graph::rebuild_index()`
- `LazyLock` statics for `SYNTAX_SET`/`THEME_SET` — never re-init per render

---

## 7. Testing Approach

### 7.1 Theme Tests

```rust
#[test]
fn rose_pine_default_contrasts() {
    let theme = Theme::default();
    let tokens = DesignTokens::from_theme(&theme);
    // WCAG AA requires 4.5:1 for normal text
    assert!(tokens.meets_contrast_aa(tokens.body, tokens.background));
    assert!(tokens.meets_contrast_aa(tokens.heading_h1, tokens.background));
    assert!(tokens.meets_contrast_aa(tokens.heading_h2, tokens.background));
    assert!(tokens.meets_contrast_aa(tokens.heading_h3, tokens.background));
}
```

### 7.2 Component Render Tests

Use Ratatui's `TestBackend` for snapshot tests:

```rust
use ratatui::{backend::TestBackend, Terminal};

#[test]
fn mode_badge_presenting() {
    let backend = TestBackend::new(20, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| {
        render_mode_badge(f, f.area(), &ModeBadgeKind::Presenting, &Theme::default());
    }).unwrap();
    // Assert cell colours match expected Rose Pine values
}
```

### 7.3 Integration Tests

- Test theme file loading: ensure JSON theme files override defaults correctly
- Test breakpoint detection: verify correct breakpoint at boundary sizes
- Test keybinding dispatch: verify action mapping per mode

---

## 8. File Change Checklist

When implementing any UX improvement, update these files:

- [ ] `theme.rs` — Colours if not yet Rose Pine
- [ ] `design/tokens.rs` — Token defaults if not yet Rose Pine
- [ ] `event.rs` — New `Action` variants
- [ ] `config/keybindings.rs` — Key mappings for new actions
- [ ] `app.rs` — New state fields, `update()` handler, `view()` call
- [ ] `ui/*.rs` — New or modified render functions
- [ ] `ui/help.rs` — Updated keybinding reference table
- [ ] `ui/mod.rs` — Module re-exports for new UI files
- [ ] `lib.rs` — Public re-exports if needed
- [ ] Tests — Unit + integration tests for new functionality
- [ ] `memory-bank/progress.md` — Status update
- [ ] `memory-bank/activeContext.md` — Current state update

---

## 9. Implementation Priority

Recommended order for a coding agent:

1. **Theme update** (§1.4) — Change `Theme::default()` to Rose Pine. This affects everything.
2. **UX-01** Mode Indicator — Verify/fix colours (quick win)
3. **UX-02** Progress Bar — Add next-node and branch indicators
4. **UX-08** Undo/Redo chips — New footer component
5. **UX-03** Branch Overlay — Polish existing implementation
6. **UX-17** Micro-interactions — Animation primitives in tick handler
7. **UX-14** Zen Mode — Simple toggle, high UX impact
8. **UX-15** Presenter Timer — Extend existing elapsed timer
9. **UX-06** Graph Edge Coding — Classify and colour edges
10. **UX-07** GotoNode — Autocomplete dropdown
11. **UX-05** Help Overlay — Verify context preservation
12. **UX-09** Compact Breakpoint — Adaptive layout paths
13. **UX-04** Metadata Selectors — Redesign picker UI
14. **UX-13** Session Timeline — New component
15. **UX-10** Breadcrumb — New navigation component
16. **UX-11** Node Preview — Preview panel in branch overlay
17. **UX-12** Command Palette — Major new feature
18. **UX-16** Minimap — Editor enhancement

---

## 10. Graph Edge Colour Reference (UX-06)

For the graph overlay in [crates/fireside-tui/src/ui/graph.rs](crates/fireside-tui/src/ui/graph.rs):

| Edge Type | Description                        | Colour | Hex       |
| --------- | ---------------------------------- | ------ | --------- |
| Next      | Default linear path between nodes  | foam   | `#9ccfd8` |
| Branch    | User-chosen branching path         | gold   | `#f6c177` |
| After     | Reconnects branches to common node | iris   | `#c4a7e7` |
| Goto      | Non-local jump to arbitrary node   | rose   | `#ebbcba` |

Legend should be rendered at the bottom of the graph overlay.

---

## 11. Accessibility Requirements

From the Contrast Ratios and Focus Order boards:

- All text on `base` background must meet WCAG AA (4.5:1 contrast ratio)
- Interactive elements must have visible focus indicators (foam border)
- Keyboard navigation must follow logical tab order
- Screen reader: mode changes announced via terminal bell or title update
- Use the WCAG utilities in `DesignTokens`: `meets_contrast_aa()`, `contrast_ratio()`
- Validate all colour combinations during theme loading

### Verified Contrast Ratios (Rose Pine Main on #191724 base)

| Colour | Hex       | Ratio  | AA Pass                    |
| ------ | --------- | ------ | -------------------------- |
| text   | `#e0def4` | 12.5:1 | ✅                         |
| subtle | `#908caa` | 5.2:1  | ✅                         |
| foam   | `#9ccfd8` | 9.6:1  | ✅                         |
| iris   | `#c4a7e7` | 7.2:1  | ✅                         |
| gold   | `#f6c177` | 9.8:1  | ✅                         |
| love   | `#eb6f92` | 6.3:1  | ✅                         |
| pine   | `#31748f` | 3.2:1  | ⚠️ Use on surface only     |
| muted  | `#6e6a86` | 3.1:1  | ⚠️ Non-essential text only |

> **Note**: Pine and muted are below 4.5:1 on base. Use pine on `surface` (#1f1d2e)
> or pair with bold text (3:1 for large text). Use muted only for decorative/non-essential
> text like footers and disabled states.
