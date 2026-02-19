# UI Component Inventory

Comprehensive mapping of every widget in the Fireside TUI (presentation
and editor modes) to concrete ratatui primitives and implementation
patterns. Each entry is implementation-ready: a developer can pick up
any component and build it from this spec.

**Crate dependencies:** `ratatui`, `crossterm`, `syntect`, `two-face`,
`textwrap`, `tui-input` (editor text fields).

**Conventions:**

- State mutation only in `App::update`; all rendering is stateless.
- `DesignTokens` → `Theme` bridge provides per-component colors.
- `Breakpoint` (Compact / Standard / Wide) drives padding and
  constraint selection.
- All layout splits use `Layout::default().direction(...).constraints(...)`.

---

## 1. Content Block Renderers

Every content block is rendered via `render_block()` in
`render/markdown.rs`. Each variant produces `Vec<Line<'_>>` that the
caller wraps in a `Paragraph`. The sections below document the widget
composition and styling for each block kind.

### 1.1 HeadingBlock

**Purpose:** Render H1–H6 headings with level-dependent color and
indentation.

**Ratatui primitives:**

- `Line` with `Span::styled(text, Style)`.
- Consumed by parent `Paragraph::new(lines)`.

**Styling:**

| Level | Token        | Modifier | Indent |
| ----- | ------------ | -------- | ------ |
| 1     | `heading_h1` | `BOLD`   | 0      |
| 2     | `heading_h2` | `BOLD`   | 2      |
| 3     | `heading_h3` | `BOLD`   | 4      |
| 4+    | `heading_h3` | `BOLD`   | 6      |

**Props/inputs:**

- `level: u8` — heading level (1–6).
- `text: &str` — heading text content.
- `theme: &Theme` — for color resolution.

**ASCII mockup:**

```
  Hello, Fireside!          ← H1, cyan, bold, no indent
    What is Fireside?       ← H2, green, bold, 2-space indent
      Architecture          ← H3, yellow, bold, 4-space indent
```

**Special behaviors:**

- No wrapping; headings are single-line. If the heading exceeds
  the area width, `Paragraph::wrap(Wrap { trim: false })` from the
  parent handles it.
- Heading text is rendered as a single `Line` (not split across lines).

---

### 1.2 TextBlock

**Purpose:** Render paragraph body text with word-wrapping.

**Ratatui primitives:**

- `textwrap::wrap(body, width)` → multiple `Line` items.
- Each `Line` contains a single `Span::styled(...)`.
- Consumed by parent `Paragraph`.

**Styling:**

| Token        | Modifier | Notes                   |
| ------------ | -------- | ----------------------- |
| `foreground` | none     | Default body text color |

**Props/inputs:**

- `body: &str` — paragraph text.
- `theme: &Theme` — `foreground` token.
- `width: u16` — available area width for wrapping.

**ASCII mockup:**

```
  Fireside is a portable format for branching
  presentations and lessons. It supports rich
  content blocks, themes, and interactive paths.
```

**Special behaviors:**

- Word-wrapping via `textwrap::wrap` respects the area width.
- Inline markdown (bold, italic, links) is not yet parsed; the text
  is rendered as plain spans. Future: parse inline markdown into
  multiple `Span` items with `BOLD`, `ITALIC`, `UNDERLINED` modifiers.

---

### 1.3 CodeBlock

**Purpose:** Render source code with syntax highlighting, optional line
numbers, and highlight-line markers.

**Ratatui primitives:**

- `syntect::easy::HighlightLines` → per-line `Vec<Span>` with RGB colors.
- Fallback: `Span::styled(line, Style { fg: code_fg, bg: code_bg })`.
- Consumed by parent `Paragraph`.
- Future: `Block::bordered()` wrapping with language label title.

**Styling:**

| Token             | Role                              |
| ----------------- | --------------------------------- |
| `code_fg`         | Fallback text color (no syntect)  |
| `code_bg`         | Background fill for code area     |
| `border_inactive` | Border around code block (future) |
| `syntax_theme`    | syntect theme name string         |

**Props/inputs:**

- `language: Option<&str>` — language hint for syntect.
- `source: &str` — raw source code.
- `highlight_lines: Vec<u32>` — 1-based lines to visually emphasize.
- `show_line_numbers: bool` — whether to prefix each line with number.
- `theme: &Theme` — token resolution.

**ASCII mockup (with line numbers and highlight):**

```
  ┌─ rust ──────────────────────────────┐
  │  1 │ fn main() {                    │
  │  2 │     println!("Hello");   ← highlighted
  │  3 │ }                              │
  └─────────────────────────────────────┘
```

**ASCII mockup (without border, current implementation):**

```
  fn main() {
      println!("Hello");
  }
```

**Special behaviors:**

- When `language` is recognized by syntect, each span gets per-token
  RGB foreground from the theme. Unrecognized languages fall back to
  monochrome `code_fg`.
- `highlight_lines`: lines in this set get a distinct background tint
  (e.g., `surface` token with increased lightness) to draw the eye.
  Not yet implemented — requires per-line style override.
- `show_line_numbers`: right-aligned gutter with `muted` color,
  separated by `│`. Not yet implemented — currently all lines have
  equal left margin.
- Future: bordered code block using `Block::bordered().title(lang)`.

**Implementation notes:**

- `highlight_code()` in `render/code.rs` loads `two_face::syntax`
  and `two_face::theme` on every call. For performance, cache the
  `SyntaxSet` and `ThemeSet` in a `OnceCell` or pass them in.
- Long lines are not wrapped; they extend beyond the area. Consider
  horizontal scrolling or soft-wrap for code in future.

---

### 1.4 ListBlock

**Purpose:** Render ordered or unordered lists with nested sub-items
and depth-dependent bullet markers.

**Ratatui primitives:**

- `Line` per item with two `Span`s: marker + text.
- Marker `Span` has `DIM` modifier.
- Recursive call for `item.children` with incremented depth.

**Styling:**

| Token        | Role          | Modifier |
| ------------ | ------------- | -------- |
| `foreground` | Item text     | none     |
| `foreground` | Bullet/number | `DIM`    |

**Bullet progression by depth:**

| Depth | Unordered | Ordered |
| ----- | --------- | ------- |
| 0     | `•`       | `1. `   |
| 1     | `◦`       | `1. `   |
| 2+    | `▪`       | `1. `   |

**Props/inputs:**

- `ordered: bool` — ordered (numbers) or unordered (bullets).
- `items: &[ListItem]` — text + optional children.
- `theme: &Theme` — `foreground` token.
- (internal) `depth: usize` — recursion depth for indentation.

**ASCII mockup:**

```
  • First point
  • Second point
    ◦ Sub-point A
    ◦ Sub-point B
      ▪ Deep nested
  • Third point
```

**Special behaviors:**

- Indentation is `"  ".repeat(depth)` (2 spaces per level).
- Ordered numbering restarts at each nesting level.
- There is no max depth limit, but at depth 3+ the indentation
  may push content off-screen on compact terminals.

---

### 1.5 ImageBlock

**Purpose:** Render a placeholder for images in the terminal with alt
text and optional caption. Terminals cannot display images natively; this
renders a text-based representation.

**Ratatui primitives:**

- `Line` with `Span::styled(...)` — multiple lines.
- Consumed by parent `Paragraph`.

**Styling:**

| Token             | Role             | Modifier |
| ----------------- | ---------------- | -------- |
| `border_inactive` | Placeholder text | `DIM`    |
| `foreground`      | Caption text     | `ITALIC` |

**Props/inputs:**

- `src: &str` — image path or URL.
- `alt: &str` — alt text for accessibility.
- `caption: Option<&str>` — caption text below image.
- `theme: &Theme` — token resolution.

**ASCII mockup:**

```
  🖼  [Image: assets/architecture.png]
      A diagram of the TEA architecture
  Caption: The Elm Architecture in Fireside
```

**Special behaviors:**

- The `🖼` emoji prefix provides a visual hint. Falls back gracefully
  in terminals without emoji support (renders as placeholder glyph).
- Future: integrate with `viuer` or Kitty/iTerm2 inline image
  protocols for actual image rendering in supported terminals.
- Alt text only renders if non-empty.

---

### 1.6 DividerBlock

**Purpose:** Render a horizontal rule using box-drawing characters.

**Ratatui primitives:**

- Single `Line` with `Span::styled("─".repeat(width), style)`.

**Styling:**

| Token             | Role       |
| ----------------- | ---------- |
| `border_inactive` | Rule color |

**Props/inputs:**

- `width: u16` — area width for the rule.
- `theme: &Theme` — `code_border` / `border_inactive` token.

**ASCII mockup:**

```
  ──────────────────────────────────────────────
```

**Special behaviors:**

- Fills the entire available width with the `─` character.
- Provides visual separation between content sections.

---

### 1.7 ContainerBlock

**Purpose:** Render a group of nested content blocks with an optional
layout hint. Delegates recursively to `render_node_content`.

**Ratatui primitives:**

- Calls `render_node_content(children, theme, width)` recursively.
- Future: split into sub-areas using `Layout` when layout hint is
  provided (e.g., `"row"` → `Direction::Horizontal`).

**Styling:**

- Inherits from child blocks. No container-specific styling yet.
- Future: optional `Block::bordered()` wrapper with layout label.

**Props/inputs:**

- `layout: Option<&str>` — layout hint (e.g., `"row"`, `"column"`).
- `children: &[ContentBlock]` — nested content blocks.
- `theme: &Theme` — passed through to child renderers.
- `width: u16` — available area width.

**ASCII mockup (default — vertical stacking):**

```
  ┌─ container ─────────────────────────┐
  │  ## Left Column                     │
  │  Some content here                  │
  │                                     │
  │  ## Right Column                    │
  │  More content here                  │
  └─────────────────────────────────────┘
```

**Special behaviors:**

- Currently renders children as a flat vertical list with blank line
  separators (identical to top-level node content).
- Future: `"row"` layout splits the area horizontally using
  `Layout::default().direction(Direction::Horizontal)` and renders
  child groups side-by-side.
- Future: `"grid"` layout with `Table` or manual `Layout` composition.

---

### 1.8 ExtensionBlock

**Purpose:** Render an unknown extension type as a fallback placeholder.
If a `fallback` content block is provided, render that instead.

**Ratatui primitives:**

- Placeholder: single `Line` with `Span::styled("[extension block]", style)`.
- With fallback: recursive `render_block(fallback, ...)`.

**Styling:**

| Token      | Role                 | Modifier |
| ---------- | -------------------- | -------- |
| `DarkGray` | Placeholder text     | `DIM`    |
| (varies)   | Fallback block style | (varies) |

**Props/inputs:**

- `extension_type: &str` — type identifier (e.g., `"acme.table"`).
- `fallback: Option<&ContentBlock>` — optional fallback block.
- `payload: &serde_json::Value` — extension-specific data (ignored in
  base renderer).
- `theme: &Theme` — passed to fallback renderer.

**ASCII mockup (no fallback):**

```
  [extension block]
```

**ASCII mockup (with fallback text block):**

```
  This content is shown when the acme.table extension
  is not supported by the rendering engine.
```

**Special behaviors:**

- The current implementation does not check for `fallback`; it always
  renders the placeholder. Planned: check `fallback.is_some()` and
  delegate to `render_block` for the fallback content block.
- Future: plugin architecture allowing engines to register extension
  renderers by type string.

---

## 2. Chrome Components

Components that frame the content area and provide navigation context.

### 2.1 Progress Bar

**Purpose:** Display current node position, total count, and elapsed
presentation time in the footer.

**Ratatui primitives:**

- `Paragraph::new(Line::from(vec![...spans...]))`.
- Three `Span` items: node info (bold), padding, timer.
- Rendered in a `Constraint::Length(1)` footer row.

**State requirements:**

- `current: usize` — current node index (0-based).
- `total: usize` — total number of nodes.
- `elapsed_secs: u64` — seconds since presentation start.

**Inputs:**

- `session.current_node_index()` → current.
- `session.graph.nodes.len()` → total.
- `app.start_time.elapsed().as_secs()` → elapsed.
- `theme.footer` → text color.

**Actions emitted:** None (display only). Future: click → `EnterGotoMode`.

**ASCII mockup:**

```
 Node 3 / 12                                          02:15
```

**Implementation notes:**

- Node info is `BOLD`, timer is regular weight.
- Padding is computed as `area.width - node_info.len() - time_info.len()`
  to right-align the timer.
- Located in `ui/progress.rs` → `render_progress_bar()`.

---

### 2.2 Breadcrumb Trail

**Purpose:** Show the navigation history path so the presenter knows
how they arrived at the current node, especially after branching.

**Ratatui primitives:**

- `Paragraph::new(Line::from(vec![...spans...]))`.
- Each crumb is a `Span::styled(node_label, style)`.
- Separator `Span::styled(" → ", muted_style)` between crumbs.

**State requirements:**

- `engine.history()` → `&[usize]` — history stack.
- `graph.nodes` — for label resolution.

**Inputs:**

- `session.traversal.history()` — indices in traversal order.
- `session.graph.nodes[idx].id` — node ID or fallback index label.

**Actions emitted:** None (display only).

**ASCII mockup:**

```
 title → what-is → branch-demo → themes
```

**Styling:**

| Element    | Token     | Modifier |
| ---------- | --------- | -------- |
| Past nodes | `muted`   | none     |
| Current    | `primary` | `BOLD`   |
| Separator  | `muted`   | `DIM`    |

**Implementation notes:**

- Truncate from the left when the breadcrumb exceeds available width:
  `... → branch-demo → themes`.
- Integrate into footer or as a second footer row
  (`Constraint::Length(1)` above the progress bar).
- Only show when history is non-empty (i.e., when the user has
  navigated beyond the first node).

---

### 2.3 Status Line

**Purpose:** Show the current mode, dirty flag, and undo/redo state
for the editor. Not visible in presentation mode.

**Ratatui primitives:**

- `Paragraph::new(Line::from(vec![...spans...]))`.
- Multiple `Span` items for each indicator.

**State requirements:**

- `app.mode` — current `AppMode`.
- `session.dirty` — modification flag.
- `command_history.can_undo()` / `can_redo()` — undo/redo availability.

**Inputs:**

- `app.mode` — mapped to display string.
- `session.dirty` — `"● modified"` or empty.
- `CommandHistory` — undo/redo indicator counts.

**Actions emitted:** None (display only).

**ASCII mockup:**

```
 ● modified  │  EDIT  │  ↩ 3  ↪ 1  │  title [H1 center]
```

**Styling:**

| Element      | Token     | Modifier |
| ------------ | --------- | -------- |
| Dirty dot    | `error`   | `BOLD`   |
| Mode label   | `primary` | `BOLD`   |
| Undo/redo    | `muted`   | none     |
| Node summary | `body`    | none     |

**Implementation notes:**

- Separator `│` between sections styled with `muted`.
- In editor mode this replaces or augments the progress bar area.
- `can_undo()`/`can_redo()` counts give users confidence about
  available undo depth.

---

### 2.4 Mode Indicator (Tab Strip)

**Purpose:** Show available modes (Present / Edit / Graph) with the
active mode highlighted. Enables mode switching context.

**Ratatui primitives:**

- `Tabs::new(vec!["Present", "Edit", "Graph"])`.
- `.select(active_index)`.
- `.highlight_style(Style::default().fg(primary).add_modifier(BOLD))`.
- `.divider(" │ ")`.

**State requirements:**

- `app.mode` — determines which tab is active.

**Inputs:**

- `AppMode::Presenting` → index 0.
- `AppMode::Editing { .. }` → index 1.
- `AppMode::GraphView { .. }` → index 2.

**Actions emitted:** None directly. Future: mouse click on tab →
`Action::SwitchMode(mode)`.

**ASCII mockup:**

```
 Present │ Edit │ Graph
 ────────
```

**Styling:**

| Element    | Token     | Modifier              |
| ---------- | --------- | --------------------- |
| Active tab | `primary` | `BOLD` + `UNDERLINED` |
| Inactive   | `muted`   | none                  |
| Divider    | `muted`   | `DIM`                 |

**Implementation notes:**

- Position at the top of the screen in a `Constraint::Length(1)` row.
- Only visible when editor or graph modes are enabled. In
  present-only mode, omit to maximize content area.
- Tab hotkeys shown as tooltips on hover (future mouse support):
  `Ctrl-P`, `Ctrl-E`, `Ctrl-G`.

---

### 2.5 Footer (Composite)

**Purpose:** Unified footer bar combining progress, status, and
navigation context. Adapts based on the active mode.

**Ratatui primitives:**

- `Layout::default().direction(Direction::Vertical).constraints([...])`.
- Vertical stack of 1–2 rows at the bottom of the screen.
- Contents vary by mode.

**Layout by mode:**

| Mode                   | Footer Row 1                    | Footer Row 2             |
| ---------------------- | ------------------------------- | ------------------------ |
| Presenting             | Progress bar (2.1)              | (none)                   |
| Presenting (branching) | Progress bar + branch indicator | (none)                   |
| GotoNode               | Go-to prompt                    | (none)                   |
| Editing                | Status line (2.3)               | Node summary + Save hint |
| GraphView              | Legend + navigation hints       | (none)                   |

**Inputs:** Delegates to child components.

**ASCII mockup (presenting):**

```
 Node 3 / 12                                          02:15
```

**ASCII mockup (presenting with branch):**

```
 Node 5 / 12  ⑂ branch-demo                          02:15
```

**ASCII mockup (goto mode):**

```
 Go to node: 7_
```

**ASCII mockup (editing):**

```
 ● modified  │  EDIT  │  ↩ 3  ↪ 1  │  Ctrl-S: Save
```

**Implementation notes:**

- The `⑂` branch indicator appears when the current node has a
  `traversal.branch_point`. Use `theme.accent` color.
- Go-to mode replaces the progress bar entirely while active, then
  reverts on confirm or cancel.
- `Constraint::Length(1)` for single-row footer,
  `Constraint::Length(2)` when breadcrumb trail is shown.

---

## 3. Overlay Components

Modal/popup components rendered on top of the content area.

### 3.1 Help Overlay

**Purpose:** Display a categorized keybinding reference as a centered
popup over the presentation content.

**Ratatui primitives:**

- `Clear` — clears the area behind the popup.
- `Block::default().title(" Keybindings ").borders(Borders::ALL)`.
- `Paragraph::new(lines)` — keybinding rows inside the block.
- Centered via `centered_popup(area, 50, 60)` helper.

**State requirements:**

- `app.show_help: bool` — toggle flag.

**Inputs:**

- `KEYBINDINGS` const array: `&[(&str, &str)]` — key + description.
- `theme.heading_h2` — border color.
- `theme.heading_h1` + `BOLD` — key text.
- `theme.foreground` — description text.

**Actions emitted:**

- `Action::ToggleHelp` (on `?` key press).

**ASCII mockup:**

```
┌─ Keybindings ───────────────────────────────┐
│                                             │
│  → / l / Space / Enter    Next node         │
│  ← / h                    Previous node     │
│  g                        Go to node        │
│  a-f                      Choose branch     │
│  ?                        Toggle this help  │
│  q / Esc                  Quit              │
│                                             │
└─────────────────────────────────────────────┘
```

**Implementation notes:**

- Located in `ui/help.rs` → `render_help_overlay()`.
- Popup is 50% width, 60% height of the terminal area.
- Rendered last, after content and footer, so it appears on top.
- Future: categorize keybindings by mode context (navigation,
  branching, editing).

---

### 3.2 Branch Selector

**Purpose:** Display the branch point options below the node content
when the current node has a `traversal.branch-point`. Options are
rendered inline (not as a modal popup).

**Ratatui primitives:**

- `Line` items appended to the node content `Vec<Line>`.
- Prompt `Line` with `Span::styled(prompt_text, accent_style)`.
- Option `Line`s: `Span::styled("[key]", key_style)` + `Span::styled(label, body_style)`.

**State requirements:**

- Current node's `traversal.branch_point` — presence and contents.

**Inputs:**

- `node.branch_point()` → `Option<&BranchPoint>`.
- `bp.prompt` — text displayed above options.
- `bp.options` — `Vec<BranchOption>` with `key`, `label`, `target`.
- `theme.accent` — key badge color.
- `theme.on_background` — label text color.

**Actions emitted:**

- `Action::ChooseBranch(key)` — when user presses `a`–`f`.

**ASCII mockup:**

```
  ## Choose Your Path

  Fireside supports branching presentations
  with traversal overrides.

  ──────────────────────────────────────────

  What would you like to explore?

   [a] Themes & Colors
   [b] Content Blocks
   [c] Skip to end

 Node 5 / 8                              02:15
```

**Styling:**

| Element       | Token     | Modifier |
| ------------- | --------- | -------- |
| `[key]` badge | `accent`  | `BOLD`   |
| Option label  | `body`    | none     |
| Prompt text   | `primary` | none     |

**Implementation notes:**

- Branch options render as extra `Line` items in the node content,
  not as a separate widget. This keeps them in the scroll flow.
- Key badge `[a]` uses 3 characters; pad label to align across
  options.
- The divider above the branch prompt could be a `DividerBlock`.
- Mouse click on an option label → `ChooseBranch(key)` (future).
- Max 6 options (`a`–`f`) per branch point.

---

### 3.3 Go-to Prompt

**Purpose:** Accept numeric input for jumping to a specific node by
number. Replaces the footer during `GotoNode` mode.

**Ratatui primitives:**

- `Paragraph::new(Line::from(vec![label_span, buffer_span, cursor_span]))`.
- `frame.set_cursor_position((x, y))` for blinking cursor.

**State requirements (in `AppMode::GotoNode`):**

- `buffer: String` — digits entered so far.

**Inputs:**

- `buffer` from `AppMode::GotoNode { buffer }`.
- `theme.primary` — label color.
- `theme.on_background` — digit color.
- `footer` area rect for positioning.

**Actions emitted:**

- `Action::GotoDigit(n)` — on `0`–`9` key press.
- `Action::GotoConfirm` — on `Enter`.
- `Action::GotoCancel` — on `Esc`.

**ASCII mockup:**

```
 Go to node: 12_
```

**Styling:**

| Element       | Token           | Modifier |
| ------------- | --------------- | -------- |
| `Go to node:` | `primary`       | `BOLD`   |
| Digits        | `on_background` | none     |
| Cursor `_`    | `accent`        | blink    |

**Implementation notes:**

- Cursor position: `area.x + "Go to node: ".len() + buffer.len()`.
- Use `frame.set_cursor_position()` for native terminal cursor blink.
- Input is 1-indexed (user-friendly); conversion to 0-indexed happens
  in `App::update` via `num.saturating_sub(1)`.
- Number > total nodes: engine returns `EngineError::InvalidTraversal`,
  which is silently caught and no movement occurs.
- Empty buffer + Enter: `parse::<usize>` fails, returns to
  `AppMode::Presenting`.

---

### 3.4 Command Palette (Future)

**Purpose:** Fuzzy-search action picker for all available commands.
Similar to VS Code's Ctrl-P or Sublime Command Palette.

**Ratatui primitives:**

- `Clear` — background clearing.
- `Block::bordered().title(" Command Palette ")` — popup frame.
- `Paragraph::new(input.value())` (via `tui-input` crate) — search field.
- `List::new(filtered_items).highlight_style(...)` + `ListState` — result list.
- Centered via `centered_popup(area, 60, 50)`.

**State requirements:**

```rust
struct CommandPaletteState {
    input: tui_input::Input,
    items: Vec<PaletteItem>,
    filtered: Vec<usize>,   // indices into items
    list_state: ListState,
}

struct PaletteItem {
    label: String,
    shortcut: Option<String>,
    action: Action,
}
```

**Inputs:**

- All available `Action` variants with display names and shortcuts.
- `input.value()` — current filter text.
- Fuzzy match score for ranking.

**Actions emitted:**

- The `Action` associated with the selected `PaletteItem`.
- `Action::ClosePalette` — on `Esc`.

**ASCII mockup:**

```
┌─ Command Palette ───────────────────────┐
│                                         │
│  > goto_                                │
│                                         │
│  >> Go to node           g              │
│     Go to first node     H              │
│     Go to last node      L              │
│                                         │
│  3 results                              │
└─────────────────────────────────────────┘
```

**Styling:**

| Element       | Token     | Modifier          |
| ------------- | --------- | ----------------- |
| Input text    | `body`    | none              |
| `>` prompt    | `accent`  | `BOLD`            |
| Selected `>>` | `primary` | `BOLD` + reversed |
| Shortcut hint | `muted`   | none              |
| Result count  | `muted`   | `DIM`             |

**Implementation notes:**

- Use `tui-input::Input` for text field state and event handling.
- Fuzzy matching: simple substring match initially; upgrade to
  `nucleo` or `fuzzy-matcher` crate for scored matching.
- Render input with `Paragraph::new(input.value()).scroll((0, scroll))`.
- List uses `ListState` for selection tracking with `render_stateful_widget`.
- Trigger: `Ctrl-P` or `:` (vim-style).

---

### 3.5 Confirmation Dialog

**Purpose:** Yes/no prompt for destructive actions (delete node, discard
unsaved changes, quit with dirty flag).

**Ratatui primitives:**

- `Clear` — clears popup background.
- `Block::bordered().title(" Confirm ").border_style(error_style)`.
- `Paragraph::new(message_lines)` — question text.
- Key hints as `Span` items at the bottom.

**State requirements:**

```rust
struct ConfirmState {
    message: String,
    on_confirm: Action,
    on_cancel: Action,
}
```

**Inputs:**

- `message` — the question text (e.g., `"Discard unsaved changes?"`).
- `on_confirm` — `Action` to dispatch on `y` / `Enter`.
- `on_cancel` — `Action` to dispatch on `n` / `Esc`.

**Actions emitted:**

- `on_confirm` action — on `y` or `Enter`.
- `on_cancel` action — on `n` or `Esc`.

**ASCII mockup:**

```
┌─ Confirm ────────────────────────┐
│                                  │
│  Unsaved changes will be lost.   │
│  Discard and quit?               │
│                                  │
│  [y] Yes    [n] No    [Esc]      │
└──────────────────────────────────┘
```

**Styling:**

| Element      | Token     | Modifier |
| ------------ | --------- | -------- |
| Border       | `error`   | none     |
| Message text | `body`    | none     |
| `[y]` badge  | `error`   | `BOLD`   |
| `[n]` badge  | `success` | `BOLD`   |
| `[Esc]`      | `muted`   | none     |

**Implementation notes:**

- Popup size: `centered_popup(area, 40, 30)`.
- Rendered last in the draw stack so it appears above everything.
- Border uses `error` token to signal danger.
- Accessible: Esc always cancels, so accidental key presses don't
  trigger destructive actions.

---

## 4. Editor Components

Editing-specific components for the three-panel editor layout defined
in the UX flows. The three-panel layout uses:

```rust
Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(20),  // Node list sidebar
        Constraint::Percentage(45),  // Block editor
        Constraint::Percentage(35),  // Preview panel
    ])
```

### 4.1 Node List Sidebar

**Purpose:** Scrollable list of all nodes in the graph with selection
highlight. Shows node IDs or index-based labels.

**Ratatui primitives:**

- `List::new(items)` with `ListState` for scrollable selection.
- `.highlight_style(Style::default().fg(on_primary).bg(primary))`.
- `.highlight_symbol("● ")`.
- `Block::bordered().title(" Nodes ")`.
- `frame.render_stateful_widget(list, area, &mut list_state)`.

**State requirements:**

```rust
struct NodeListState {
    list_state: ListState,
    selected: usize,
}
```

**Inputs:**

- `session.graph.nodes` — node list for labels.
- `session.current_node_index()` — mark the active presenter node.
- `session.dirty` — dirty indicator in footer.

**Actions emitted:**

- `Action::SelectNode(index)` — on `j`/`k`/↑/↓ + Enter.
- `Action::AddNode` — on `A`.
- `Action::DeleteNode` — on `D` (triggers confirmation dialog).

**ASCII mockup:**

```
┌─ Nodes ──────┐
│ ● title      │
│   what-is    │
│   code-demo  │
│   layouts    │
│   branch-demo│
│   themes     │
│   blocks     │
│   thanks     │
│              │
│              │
│──────────────│
│ ● modified   │
└──────────────┘
```

**Styling:**

| Element             | Token                        | Modifier |
| ------------------- | ---------------------------- | -------- |
| Selected node       | `on_primary` on `primary` bg | `BOLD`   |
| Current (presenter) | `accent`                     | none     |
| Unselected          | `body`                       | none     |
| Border (focused)    | `border_active`              | none     |
| Border (unfocused)  | `border_inactive`            | none     |
| Dirty indicator     | `error`                      | `BOLD`   |

**Implementation notes:**

- Use `ListState::select(Some(index))` when selection changes.
- Nodes without IDs display as `[1]`, `[2]`, etc. using 1-based
  display indices.
- The `● modified` footer inside the block uses `session.dirty`.
- Scrollbar: use `Scrollbar::new(ScrollbarOrientation::VerticalRight)`
  rendered with `render_stateful_widget` paired with a `ScrollbarState`.

---

### 4.2 Block Editor

**Purpose:** Form-based list of content blocks for the selected node.
Each block shows its kind badge, a summary of its content, and
supports selection for editing, reordering, and deletion.

**Ratatui primitives:**

- `List::new(items)` + `ListState` — block list with selection.
- Each `ListItem` is a `Line` with kind badge `Span` + summary `Span`.
- `Block::bordered().title(" Content Blocks ")`.
- Toolbar hint row at bottom: `Paragraph::new(Line::from(toolbar_spans))`.

**State requirements:**

```rust
struct BlockEditorState {
    list_state: ListState,
    selected_block: usize,
    editing_field: Option<EditField>,
}

enum EditField {
    HeadingText,
    HeadingLevel,
    TextBody,
    CodeSource,
    CodeLanguage,
    ListItems,
    ImageSrc,
    ImageAlt,
    ImageCaption,
    ContainerLayout,
}
```

**Inputs:**

- `session.current_node().content` — the blocks to display.
- `theme` — styling tokens.

**Actions emitted:**

- `Action::SelectBlock(index)` — on `j`/`k`.
- `Action::EditBlockField(field)` — on `Enter`.
- `Action::AddBlock` — on `a` (opens block type picker).
- `Action::DeleteBlock` — on `d` (with confirmation).
- `Action::MoveBlockUp` — on `K`.
- `Action::MoveBlockDown` — on `J`.

**ASCII mockup:**

```
┌─ Content Blocks ──────────────────────┐
│                                       │
│  [1] 🅗 heading  "Hello, Fireside!"   │
│                                       │
│  [2] 🅣 text     "A portable format"  │
│                                       │
│  [3] 🅣 text     "Press → to advance" │
│                                       │
│                                       │
│  [a]dd  [d]elete  [J↓/K↑] reorder    │
└───────────────────────────────────────┘
```

**Styling:**

| Element        | Token                        | Modifier |
| -------------- | ---------------------------- | -------- |
| Selected block | `on_primary` on `primary` bg | `BOLD`   |
| Kind badge     | `accent`                     | `BOLD`   |
| Summary text   | `body`                       | none     |
| Toolbar hints  | `muted`                      | `DIM`    |

**Implementation notes:**

- Summary is a truncated version of the primary field:
  - `heading` → first 30 chars of `text`.
  - `text` → first 30 chars of `body`.
  - `code` → `language` or `"code"`.
  - `list` → `"{n} items"`.
  - `image` → basename of `src`.
  - `divider` → `"───"`.
  - `container` → `"{n} children"`.
  - `extension` → `extension_type`.
- Block numbers are 1-indexed for the user display.
- When the node has zero blocks, show centered message:
  `"No content blocks. Press [a] to add."`.

---

### 4.3 Property Inspector

**Purpose:** Form for editing node-level metadata: id, layout,
transition, traversal overrides, and speaker notes.

**Ratatui primitives:**

- Vertical stack of form fields using `Layout::default().direction(Vertical)`.
- Each field: `Paragraph::new(Line::from(vec![label, value]))`.
- Active field: `tui-input::Input` for text editing.
- `Block::bordered().title(" Node Properties ")`.

**State requirements:**

```rust
struct PropertyInspectorState {
    selected_field: usize,
    editing: Option<PropertyField>,
    input: Option<tui_input::Input>,
}

enum PropertyField {
    Id,
    Layout,
    Transition,
    TraversalNext,
    TraversalAfter,
    SpeakerNotes,
}
```

**Inputs:**

- `session.current_node()` — all node fields.
- `Layout::all()` — for layout enum picker.
- `Transition::all()` — for transition enum picker.

**Actions emitted:**

- `Action::EditNodeProperty(field)` — on Enter.
- `Action::ConfirmEdit` — on Enter in text input mode.
- `Action::CancelEdit` — on Esc.

**ASCII mockup:**

```
┌─ Node Properties ─────────────────────┐
│                                       │
│  id          [title               ]   │
│  layout      [center         ▾]      │
│  transition  [fade           ▾]      │
│  next        [none                ]   │
│  after       [none                ]   │
│                                       │
│  speaker-notes:                       │
│  ┌────────────────────────────────┐   │
│  │ Remember to pause after this   │   │
│  │ slide for questions.           │   │
│  └────────────────────────────────┘   │
│                                       │
└───────────────────────────────────────┘
```

**Styling:**

| Element         | Token           | Modifier |
| --------------- | --------------- | -------- |
| Label           | `muted`         | none     |
| Value (view)    | `body`          | none     |
| Value (editing) | `on_background` | none     |
| Input border    | `border_active` | none     |
| Dropdown `▾`    | `accent`        | none     |

**Implementation notes:**

- Layout and transition fields use a dropdown-style picker: pressing
  Enter opens a mini-list of enum variants rendered as a `List` popup
  anchored to the field.
- Speaker notes field is a multi-line text area. Use `Paragraph`
  with `Wrap` + manual scroll offset for editing.
- The inspector can be a toggle panel (like `Ctrl-I`) or a tab within
  the block editor area.

---

### 4.4 Validation Panel

**Purpose:** Display graph validation diagnostics (errors and warnings)
as a collapsible list anchored at the bottom of the editor.

**Ratatui primitives:**

- `List::new(items)` + `ListState`.
- Each `ListItem` is a `Line` with severity icon + message.
- `Block::bordered().title(" Diagnostics (2) ")`.
- Collapsible: `Constraint::Length(0)` when hidden,
  `Constraint::Percentage(25)` when visible.

**State requirements:**

```rust
struct ValidationPanelState {
    diagnostics: Vec<Diagnostic>,
    list_state: ListState,
    expanded: bool,
}
```

**Inputs:**

- `validate_graph(&session.graph)` → `Vec<Diagnostic>`.
- `diagnostic.severity` — `Error` or `Warning`.
- `diagnostic.message` — human-readable description.
- `diagnostic.node_id` — optional node context.

**Actions emitted:**

- `Action::ToggleValidation` — on `Ctrl-D` or toolbar button.
- `Action::GoToNode(index)` — on Enter, jump to the node with the
  diagnostic.

**ASCII mockup (expanded):**

```
┌─ Diagnostics (2) ─────────────────────────────┐
│  ✗ Node 'x' traversal.next references unknown │
│    node 'missing'                              │
│  ⚠ Node 'y' is unreachable from start         │
└────────────────────────────────────────────────┘
```

**ASCII mockup (collapsed):**

```
 ✗ 1 error, ⚠ 1 warning                [Ctrl-D]
```

**Styling:**

| Element  | Token                 | Modifier     | Icon |
| -------- | --------------------- | ------------ | ---- |
| Error    | `error`               | `BOLD`       | `✗`  |
| Warning  | `heading_h3` (yellow) | none         | `⚠`  |
| Node ref | `accent`              | `UNDERLINED` | —    |

**Implementation notes:**

- Re-run validation on every graph mutation (or debounce to
  on-save / on-idle).
- The collapsed view fits in a single `Constraint::Length(1)` row
  showing a summary count.
- Pressing Enter on a diagnostic selects the referenced node in the
  node list sidebar (cross-component communication via `Action`).

---

### 4.5 Settings Form

**Purpose:** UI for editing presentation settings: theme selection,
syntax theme, default layout/transition, and document metadata.

**Ratatui primitives:**

- Vertical stack of form field components (see §6.5 Form Field).
- `Block::bordered().title(" Settings ")`.
- Scrollable if content exceeds panel height.
- `List` for enum-value fields (theme list, layout options).

**State requirements:**

```rust
struct SettingsFormState {
    selected_field: usize,
    editing: bool,
    fields: Vec<SettingsField>,
}

enum SettingsField {
    Title,
    Author,
    Date,
    Description,
    Theme,
    Font,
    DefaultLayout,
    DefaultTransition,
    SyntaxTheme,
}
```

**Inputs:**

- `session.graph.metadata` — current values.
- `available_themes()` — for theme picker.
- `Layout::all()` / `Transition::all()` — for default selectors.

**Actions emitted:**

- `Action::UpdateMetadata(field, value)` — on confirm.
- `Action::CancelEdit` — on Esc.

**ASCII mockup:**

```
┌─ Settings ────────────────────────────┐
│                                       │
│  title       [Hello, Fireside!   ]    │
│  author      [Tiberius           ]    │
│  date        [2026-02-19         ]    │
│  theme       [dracula        ▾]      │
│  font        [JetBrains Mono    ]    │
│  default-layout     [center  ▾]      │
│  default-transition [fade    ▾]      │
│  syntax-theme       [base16  ▾]      │
│                                       │
└───────────────────────────────────────┘
```

**Implementation notes:**

- Settings form is accessible via a dedicated panel or overlay
  (not integrated into the block editor).
- Trigger: `Ctrl-,` (common settings shortcut).
- Changes to metadata mutate `session.graph.metadata` via `Command`
  variants and mark the session dirty.

---

### 4.6 Block Type Picker

**Purpose:** Centered overlay for selecting a new content block type
when adding a block via `a`.

**Ratatui primitives:**

- `Clear` → `Block::bordered().title(" Add Block ")`.
- `List::new(block_types)` + `ListState` for selection.
- OR: static `Paragraph` with numbered entries.

**State requirements:**

```rust
struct BlockPickerState {
    list_state: ListState,
}
```

**Inputs:**

- Hardcoded list of 7 core block kinds + extension.
- Each entry: number/key, kind label, short description.

**Actions emitted:**

- `Action::InsertBlock(kind)` — on `1`–`7` or Enter with selection.
- `Action::CancelEdit` — on Esc.

**ASCII mockup:**

```
┌─ Add Block ──────────────────────┐
│                                  │
│  [1] heading    Heading (H1-H6)  │
│  [2] text       Prose text       │
│  [3] code       Source code      │
│  [4] list       Bullet/numbered  │
│  [5] image      Image + caption  │
│  [6] divider    Horizontal rule  │
│  [7] container  Nested blocks    │
│                                  │
│  Press 1-7 or Esc to cancel      │
└──────────────────────────────────┘
```

**Styling:**

| Element     | Token     | Modifier |
| ----------- | --------- | -------- |
| Number key  | `accent`  | `BOLD`   |
| Kind label  | `primary` | none     |
| Description | `muted`   | none     |
| Footer hint | `muted`   | `DIM`    |

**Implementation notes:**

- Popup size: `centered_popup(area, 40, 40)`.
- After selection, a new block of the chosen kind is inserted after
  the currently selected block with default/empty content.
- Block defaults:
  - `heading`: `{ level: 2, text: "" }`.
  - `text`: `{ body: "" }`.
  - `code`: `{ language: None, source: "", ... }`.
  - `list`: `{ ordered: false, items: [] }`.
  - `image`: `{ src: "", alt: "", caption: None }`.
  - `divider`: `Divider`.
  - `container`: `{ layout: None, children: [] }`.

---

## 5. Graph View Components

Components for the graph exploration mode (proposed in UX Flow 5).

### 5.1 ASCII Graph Renderer

**Purpose:** Render the graph structure as a box-drawing diagram showing
nodes as boxes and edges as connecting lines/arrows.

**Ratatui primitives:**

- `canvas::Canvas` widget (ratatui `canvas` feature) with custom
  draw closure, OR manual `Line`/`Span` composition for full control.
- Recommended: manual `Vec<Line>` approach for fine-grained box-drawing
  character placement.
- `Paragraph::new(lines).scroll((row_offset, col_offset))` for viewport.

**State requirements:**

```rust
struct GraphViewState {
    selected: usize,
    scroll_offset: (u16, u16),  // (row, col)
    layout_cache: Vec<NodeBox>,
}

struct NodeBox {
    node_index: usize,
    label: String,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}
```

**Inputs:**

- `session.graph.nodes` — all nodes for box labels.
- `session.graph.node_index` — ID → index mapping.
- Node `traversal.next`, `traversal.branch_point` — edges.
- `session.current_node_index()` — mark current with `●`.

**Actions emitted:**

- `Action::SelectNode(index)` — on `j`/`k`.
- `Action::GoToPresenter(index)` — on Enter.
- `Action::GoToEditor(index)` — on `e`.
- `Action::PanGraph(dx, dy)` — on `h`/`l` arrow keys.
- `Action::FitToScreen` — on `f`.

**ASCII mockup:**

```
┌──────────┐    ┌──────────┐    ┌──────────┐
│  title   │───→│ what-is  │───→│code-demo │
│   (H1)   │    │  (list)  │    │  (code)  │
└──────────┘    └──────────┘    └──────────┘
                                     │
                                     ▼
                                ┌──────────┐
                                │●branch-  │
                                │  demo    │
                                └─┬──┬──┬──┘
                           a ╱   │b │  │ ╲ c
                            ╱    │  │  │  ╲
                   ┌────────┐ ┌──┴──┐ ┌────┴──┐
                   │ themes │ │block│ │thanks │
                   └────────┘ └─────┘ └───────┘
```

**Styling:**

| Element          | Token           | Modifier       |
| ---------------- | --------------- | -------------- |
| Current node `●` | `primary`       | `BOLD`         |
| Selected box     | `accent` border | reversed       |
| Sequential edge  | `muted`         | none           |
| Branch edge      | `accent`        | none           |
| Override edge    | `heading_h3`    | `DIM` (dotted) |
| Node label       | `body`          | none           |
| Node sublabel    | `muted`         | `DIM`          |

**Implementation notes:**

- Layout algorithm: layered/Sugiyama-style.
  1. Topological sort from first node.
  2. Assign layers (depth from root).
  3. Minimize edge crossings within layers.
  4. Render boxes at grid positions.
  5. Draw edges with box-drawing characters.
- For MVP, a simple left-to-right horizontal chain with branch
  fan-out below is sufficient.
- Node box width: `max(label.len() + 4, 12)`.
- Node box height: 3 (top border, label, bottom border).
- Sublabel in parentheses: primary content block kind.
- Cache the layout computation; invalidate on graph mutation.

---

### 5.2 Graph Navigator

**Purpose:** Keyboard-driven cursor for selecting and navigating between
nodes in the graph view.

**Ratatui primitives:**

- No dedicated widget; modifies rendering style of the selected
  `NodeBox` in the graph renderer.
- Selection highlight applied as `.bg(accent)` reversed style on the
  selected box border.

**State requirements:**

- `graph_view.selected: usize` — index of highlighted node.

**Inputs:**

- `j`/`k` — move selection to next/previous node by index.
- `h`/`l` — pan viewport when graph exceeds screen.
- `Enter` — jump to selected node.

**Actions emitted:**

- See §5.1 actions.

**Implementation notes:**

- Arrow-key navigation follows node index order (not spatial position).
- Future: spatial navigation where `h`/`l` move to the nearest node
  left/right, and `j`/`k` move up/down.
- Viewport auto-scrolls to keep the selected node visible.

---

### 5.3 Mini-map

**Purpose:** Small overview of the entire graph in the corner of the
presenter view, showing the current position in context.

**Ratatui primitives:**

- `Block::bordered()` — thin border frame.
- Custom rendering of simplified box-drawing nodes as single
  characters (`□` normal, `■` current, `◆` branching).
- Positioned in bottom-right corner:
  `Rect { x: area.width - 20, y: area.height - 8, width: 18, height: 6 }`.

**State requirements:**

- Same as graph view but read-only.

**Inputs:**

- `session.graph` — node count, edge structure.
- `session.current_node_index()` — current position marker.

**Actions emitted:** None (display only). Future: click node → `GoToNode`.

**ASCII mockup (in corner of presenter):**

```
                    ┌─ map ──────┐
                    │ □─□─□─□    │
                    │       ↓    │
                    │      ■←┐  │
                    │     ╱ ╲│  │
                    │    □   □   │
                    └────────────┘
```

**Styling:**

| Element   | Token             |
| --------- | ----------------- |
| Current ■ | `primary`         |
| Normal □  | `muted`           |
| Branch ◆  | `accent`          |
| Edges     | `border_inactive` |

**Implementation notes:**

- Toggle visibility with `m` key.
- Scale: each node is 1–2 characters wide. For large graphs, nodes
  are compressed to fit the mini-map area.
- Semi-transparent background using `surface` token color.
- Only useful when graph has more than ~5 nodes.

---

## 6. Shared Primitives

Reusable building blocks composed into higher-level components.

### 6.1 Styled Block

**Purpose:** A `Block::bordered()` container with a title, consistent
border styling, and focus-aware coloring.

**Ratatui primitives:**

- `Block::bordered().title(title).border_style(border_style).style(bg_style)`.
- `block.inner(area)` for computing the inner content rect.

**State requirements:**

- `focused: bool` — whether this panel has input focus.

**Inputs:**

- `title: &str` — block title.
- `focused: bool` — changes border color.
- `theme` — `border_active` vs `border_inactive`.

**Actions emitted:** None (structural only).

**ASCII mockup (focused):**

```
┌─ Title ──────────────────────┐  ← border_active color
│  (content area)              │
└──────────────────────────────┘
```

**ASCII mockup (unfocused):**

```
┌─ Title ──────────────────────┐  ← border_inactive color
│  (content area)              │
└──────────────────────────────┘
```

**Implementation notes:**

- Factory function:
  ```rust
  fn styled_block(title: &str, focused: bool, theme: &Theme) -> Block<'_> {
      let border_color = if focused {
          theme.border_active  // from DesignTokens
      } else {
          theme.border_inactive
      };
      Block::bordered()
          .title(format!(" {title} "))
          .border_style(Style::default().fg(border_color))
          .style(Style::default().bg(theme.background))
  }
  ```
- Note: `Theme` currently doesn't expose `border_active`/`border_inactive`
  directly; use `DesignTokens` or extend `Theme` to include them.

---

### 6.2 Scrollable Area

**Purpose:** Content area with a vertical scrollbar for content that
exceeds the visible height.

**Ratatui primitives:**

- `Paragraph::new(lines).scroll((row_offset, 0))` — content with offset.
- `Scrollbar::new(ScrollbarOrientation::VerticalRight)` — visual indicator.
- `ScrollbarState::new(total_lines).position(current_offset)`.
- `frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state)`.

**State requirements:**

```rust
struct ScrollState {
    offset: u16,
    total_lines: u16,
    visible_lines: u16,
}
```

**Inputs:**

- `lines: Vec<Line>` — the full content.
- `area.height` — visible line count.
- Scroll events: `ScrollUp` / `ScrollDown`.

**Actions emitted:**

- `Action::ScrollUp` / `Action::ScrollDown`.
- Or handled internally by adjusting `offset`.

**ASCII mockup:**

```
│  Line 1 content                    ▲│
│  Line 2 content                    ┃│
│  Line 3 content                    ┃│
│  Line 4 content                    █│
│  Line 5 content                    ┃│
│  Line 6 (visible bottom)          ▼│
```

**Implementation notes:**

- Scrollbar is only visible when `total_lines > visible_lines`.
- Mouse `ScrollUp`/`ScrollDown` events adjust `offset` by 1–3 lines.
- Page Up/Down adjust by `visible_lines - 1`.
- Clamp `offset` to `0..=total_lines.saturating_sub(visible_lines)`.

---

### 6.3 Key Badge

**Purpose:** Render a keyboard shortcut as a highlighted, visually
distinct inline element. Used in help overlays, branch options,
toolbar hints, and picker items.

**Ratatui primitives:**

- `Span::styled(format!("[{key}]"), badge_style)`.
- `badge_style`: `Style::default().fg(accent).add_modifier(BOLD)`.

**State requirements:** None (stateless).

**Inputs:**

- `key: &str` — the key text (e.g., `"a"`, `"Ctrl-S"`, `"Enter"`).
- `theme.accent` — badge foreground color.

**Actions emitted:** None (display only).

**ASCII mockup:**

```
  [a] Themes & Colors
  [Ctrl-S] Save
  [?] Help
```

**Styling:**

| Variant    | Token    | Modifier |
| ---------- | -------- | -------- |
| Normal key | `accent` | `BOLD`   |
| Danger key | `error`  | `BOLD`   |
| Muted key  | `muted`  | none     |

**Implementation notes:**

- Factory function:
  ```rust
  fn key_badge<'a>(key: &'a str, theme: &Theme) -> Span<'a> {
      Span::styled(
          format!("[{key}]"),
          Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
      )
  }
  ```
- Width: `key.len() + 2` (for brackets).
- Used extensively in help overlay, branch selector, block type picker,
  toolbar hints, and confirmation dialogs.

---

### 6.4 Icon / Type Badge

**Purpose:** Small inline indicator showing a content block's kind as
a colored abbreviation or icon. Used in the block editor list.

**Ratatui primitives:**

- `Span::styled(badge_text, badge_style)`.

**State requirements:** None (stateless).

**Inputs:**

- `kind: &str` — the content block kind.

**ASCII mockup:**

```
  H1  heading
  T   text
  </>  code
  •   list
  🖼  image
  ──  divider
  [ ] container
  ⚙   extension
```

**Styling:**

| Kind      | Badge  | Color                  |
| --------- | ------ | ---------------------- |
| heading   | `H{n}` | `heading_h1` / h2 / h3 |
| text      | `T`    | `body`                 |
| code      | `</>`  | `code_fg`              |
| list      | `•`    | `body`                 |
| image     | `🖼`   | `muted`                |
| divider   | `──`   | `muted`                |
| container | `[ ]`  | `accent`               |
| extension | `⚙`    | `muted`                |

**Implementation notes:**

- Fixed-width 4 characters (padded) for alignment in lists.
- Color-coding provides at-a-glance block type identification.

---

### 6.5 Form Field

**Purpose:** Reusable label + value pair for property editors and
settings forms. Supports view mode (display value) and edit mode
(text input).

**Ratatui primitives:**

- View mode: `Line::from(vec![label_span, value_span])`.
- Edit mode: `tui-input::Input` rendered as
  `Paragraph::new(input.value()).scroll((0, scroll)).block(Block::bordered())`.
- `frame.set_cursor_position(...)` for cursor in edit mode.

**State requirements:**

```rust
struct FormFieldState {
    label: String,
    value: String,
    editing: bool,
    input: Option<tui_input::Input>,
}
```

**Inputs:**

- `label: &str` — field label.
- `value: &str` — current value (display).
- `editing: bool` — whether the field is in edit mode.
- `input: &tui_input::Input` — edit mode state (when editing).

**Actions emitted:**

- `Action::ConfirmEdit` — on Enter in edit mode.
- `Action::CancelEdit` — on Esc in edit mode.

**ASCII mockup (view):**

```
  id          title
```

**ASCII mockup (editing):**

```
  id          [title_              ]
              ↑ cursor
```

**Styling:**

| Element      | Token           | Modifier |
| ------------ | --------------- | -------- |
| Label        | `muted`         | none     |
| Value (view) | `body`          | none     |
| Value (edit) | `on_background` | none     |
| Input border | `border_active` | none     |
| Cursor       | `accent`        | blink    |

**Implementation notes:**

- Label column width: fixed at 14 characters (right-padded) for
  alignment across all fields.
- Use `Layout::default().direction(Horizontal).constraints([
  Constraint::Length(14), Constraint::Min(1)
])` for label/value split.
- `tui-input` handles cursor movement, insertion, deletion.
- `input.visual_cursor()` provides cursor X offset for
  `set_cursor_position`.

---

### 6.6 Toast / Notification

**Purpose:** Transient status messages displayed briefly after an
action completes (save, undo, error, theme change).

**Ratatui primitives:**

- `Paragraph::new(Line::from(message_spans))`.
- Positioned in the top-right corner:
  `Rect { x: area.width - msg.len() - 4, y: 1, width: msg.len() + 4, height: 1 }`.
- `Clear` to clear background behind the message.

**State requirements:**

```rust
struct ToastState {
    message: String,
    severity: ToastSeverity,
    shown_at: Instant,
    duration: Duration,
}

enum ToastSeverity {
    Info,
    Success,
    Warning,
    Error,
}
```

**Inputs:**

- `message: &str` — notification text.
- `severity` — determines color.
- `Instant::now()` — for auto-dismiss timing.

**Actions emitted:**

- `Action::DismissToast` — auto-triggered after `duration` elapses
  (checked in `Tick` handler).

**ASCII mockup:**

```
                                   ╭─────────────────╮
                                   │ ✓ Saved         │
                                   ╰─────────────────╯
```

**Styling:**

| Severity | Token        | Icon |
| -------- | ------------ | ---- |
| Info     | `primary`    | `ℹ`  |
| Success  | `success`    | `✓`  |
| Warning  | `heading_h3` | `⚠`  |
| Error    | `error`      | `✗`  |

**Implementation notes:**

- Default duration: 2 seconds. Errors: 4 seconds.
- Auto-dismiss: `Action::Tick` handler checks
  `shown_at.elapsed() >= duration` and clears the toast.
- Only one toast at a time; new toasts replace existing ones.
- Position: top-right corner with 1-cell margin from edges.
- Rounded corners `╭╮╰╯` for visual distinction from rectangular
  panels. Falls back to `┌┐└┘` in terminals without Unicode support.

---

## Component Dependency Map

```
┌─────────────────────────────────────────────────────────────┐
│ App::view() dispatches to one of:                          │
├──────────────┬──────────────────┬──────────────────────────┤
│  Presenter   │  Editor          │  Graph View              │
│  (§1, §2)    │  (§4, §1)        │  (§5)                    │
├──────────────┴──────────────────┴──────────────────────────┤
│ Overlays (§3) — rendered last on top of any mode           │
├────────────────────────────────────────────────────────────┤
│ Shared Primitives (§6) — used by all components above      │
├────────────────────────────────────────────────────────────┤
│ DesignTokens / Theme — color resolution layer              │
├────────────────────────────────────────────────────────────┤
│ Layout Engine (compute_areas / templates) — area slicing   │
└────────────────────────────────────────────────────────────┘
```

## Summary Table

| #   | Component               | Section | Status                    | Primary Widget                    |
| --- | ----------------------- | ------- | ------------------------- | --------------------------------- |
| 1.1 | HeadingBlock renderer   | §1      | Implemented               | `Span` in `Line`                  |
| 1.2 | TextBlock renderer      | §1      | Implemented               | `Span` + `textwrap`               |
| 1.3 | CodeBlock renderer      | §1      | Implemented (partial)     | `syntect` → `Span`                |
| 1.4 | ListBlock renderer      | §1      | Implemented               | `Span` with recursion             |
| 1.5 | ImageBlock renderer     | §1      | Implemented (placeholder) | `Span`                            |
| 1.6 | DividerBlock renderer   | §1      | Implemented               | `Span`                            |
| 1.7 | ContainerBlock renderer | §1      | Implemented (basic)       | recursive call                    |
| 1.8 | ExtensionBlock renderer | §1      | Implemented (placeholder) | `Span`                            |
| 2.1 | Progress bar            | §2      | Implemented               | `Paragraph`                       |
| 2.2 | Breadcrumb trail        | §2      | Planned                   | `Paragraph`                       |
| 2.3 | Status line             | §2      | Planned                   | `Paragraph`                       |
| 2.4 | Mode indicator          | §2      | Planned                   | `Tabs`                            |
| 2.5 | Footer (composite)      | §2      | Partial                   | `Layout` + children               |
| 3.1 | Help overlay            | §3      | Implemented               | `Clear` + `Block` + `Paragraph`   |
| 3.2 | Branch selector         | §3      | Planned                   | `Line`/`Span` inline              |
| 3.3 | Go-to prompt            | §3      | Implemented               | `Paragraph` + cursor              |
| 3.4 | Command palette         | §3      | Planned (future)          | `tui-input` + `List`              |
| 3.5 | Confirmation dialog     | §3      | Planned                   | `Clear` + `Block`                 |
| 4.1 | Node list sidebar       | §4      | Planned                   | `List` + `ListState`              |
| 4.2 | Block editor            | §4      | Planned                   | `List` + `ListState`              |
| 4.3 | Property inspector      | §4      | Planned                   | `tui-input` + `Layout`            |
| 4.4 | Validation panel        | §4      | Planned                   | `List` + `ListState`              |
| 4.5 | Settings form           | §4      | Planned                   | Form fields                       |
| 4.6 | Block type picker       | §4      | Planned                   | `Clear` + `List`                  |
| 5.1 | ASCII graph renderer    | §5      | Planned                   | Manual `Line`/`Span`              |
| 5.2 | Graph navigator         | §5      | Planned                   | Style modifier                    |
| 5.3 | Mini-map                | §5      | Planned (future)          | `Block` + custom draw             |
| 6.1 | Styled block            | §6      | Partial (used implicitly) | `Block::bordered()`               |
| 6.2 | Scrollable area         | §6      | Planned                   | `Paragraph::scroll` + `Scrollbar` |
| 6.3 | Key badge               | §6      | Used (not extracted)      | `Span`                            |
| 6.4 | Icon / type badge       | §6      | Planned                   | `Span`                            |
| 6.5 | Form field              | §6      | Planned                   | `tui-input` + `Layout`            |
| 6.6 | Toast / notification    | §6      | Planned                   | `Paragraph` + timer               |
