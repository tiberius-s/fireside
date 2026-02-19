## Plan: Fireside TUI — UI/UX Full Initiative

The TUI has a solid TEA architecture and a working presentation pipeline, but significant gaps remain: no branch point UI, no editor mode, no mouse support, placeholder image/extension rendering, unwired design tokens & templates, and no transition animations. This plan spans six phases — from competitive research through to documentation — covering everything needed to deliver a polished, dual-mode (present + edit) TUI with full content block rendering, ASCII graph visualization, and best-practice documentation. Each phase builds on the prior one and can be shipped incrementally.

---

### Phase 0: Competitive Research & UX Design Foundation

**Goal:** Establish design direction with a formal analysis and define user flows before writing UI code.

**Steps**

1. **Competitive analysis document** — Create `docs/src/content/docs/reference/competitive-analysis.md` analyzing these tools:
   - **Terminal presenters:** presenterm, slides (Go), lookatme (Python), patat (Haskell)
   - **Web-based:** Slidev, Reveal.js, Marp, Spectacle
   - **Interactive/lesson platforms:** Jupyter, Observable, Scrimba, Exercism
   - **TUI editors:** Helix, Zed (for UI patterns), Vim/Neovim plugin ecosystems
   - For each tool, evaluate: navigation model, content rendering, theming, branching/interactivity support, editing capabilities, keyboard/mouse balance, visual polish
   - Conclude with a **"steal these ideas"** section — specific patterns to adopt in Fireside

2. **User flow definitions** — Create `memory-bank/ux-flows.md` documenting key user journeys:
   - First-run experience (`fireside new` → `fireside present`)
   - Linear presentation flow (navigate, progress, finish)
   - Branching presentation flow (encounter branch → choose → backtrack → rejoin)
   - Editing flow (open editor → modify node → preview → save)
   - Graph exploration flow (open graph view → navigate structure → jump to node)
   - Theme customization flow (import theme → preview → apply)

3. **UI component inventory** — Create `memory-bank/ui-components.md` cataloguing every widget needed across both modes, mapped to ratatui primitives:
   - Content block renderers (heading, text, code, list, image, divider, container, extension)
   - Chrome components (progress bar, breadcrumb trail, status line, mode indicator)
   - Overlay components (help popup, branch selector, goto prompt, command palette)
   - Editor components (property panel, block editor, graph view, validation panel, settings form)

**Verification:** Review documents for completeness; no code changes in this phase.

---

### Phase 1: Presentation Mode Polish

**Goal:** Make presentation mode visually excellent and functionally complete — this is what users see first.

**Steps**

1. **Wire design tokens into the renderer** — Replace direct `Theme` usage in `crates/fireside-tui/src/render/markdown.rs` with `DesignTokens` from `crates/fireside-tui/src/design/tokens.rs`. The `DesignTokens` struct already has 25 semantic color tokens and WCAG contrast checking — use them for all content block styling. Keep the `Theme` ↔ `DesignTokens` roundtrip (`to_theme()` / `from_theme()`) as the bridge.

2. **Wire node templates into the presenter** — Replace the simple `compute_areas()` in `crates/fireside-tui/src/render/layout.rs` with the `TemplateAreas` system from `crates/fireside-tui/src/design/templates.rs` for layout variants that have corresponding templates (Title, TwoColumn, CodeFocus, Quote, etc.). The templates already compute `main`, `secondary`, and `footer` rects — the presenter in `crates/fireside-tui/src/ui/presenter.rs` just needs to use them.

3. **Implement split layout rendering** — The `two_column_split()` helper already exists in `crates/fireside-tui/src/render/layout.rs` but is never called. Wire `Layout::SplitHorizontal` and `SplitVertical` to actually render content in two panes. Extend to handle `Container` blocks with layout hints (e.g., a container with `layout: "split-horizontal"` renders its children in columns).

4. **Content block visual treatment** — Enhance each block type in `render_block()`:
   - **Headings:** Add underline borders or box-drawing separators below H1/H2
   - **Code blocks:** Render with a visible border using `Block::bordered()`, add gutter with line numbers when `show-line-numbers: true`, highlight specific lines when `highlight-lines` is set (both fields exist in the model but are currently ignored)
   - **Lists:** Add indentation guides (vertical `│` characters for nested items)
   - **Dividers:** Already fine, consider themed color
   - **Containers:** Render with subtle background or border to visually group children
   - **Images:** Improve placeholder with a bordered frame: `┌─ 🖼 image.png ─┐` with alt text inside
   - **Extension blocks:** Show the `type` identifier and any `fallback` content instead of static `[extension block]`

5. **Branch point UI** — This is the biggest functional gap in presentation mode. When the current node has a `branch_point`, render a branch selector overlay:
   - Display the `prompt` text prominently
   - List each `BranchOption` with its `key` highlighted (e.g., `[a] Option one`, `[b] Option two`)
   - Show a preview snippet of the target node's first content block (title or opening text) beside each option
   - Style with a bordered popup or an inline panel below content, depending on layout
   - Add this as a new UI component in `crates/fireside-tui/src/ui/` (e.g., `branch.rs`)

6. **Navigation indicators** — Enhance the footer in `crates/fireside-tui/src/ui/progress.rs`:
   - Add a **breadcrumb trail** showing the path taken through the graph (use the `history` stack from `TraversalEngine`). Display as `Start → Node A → Node B → Current`
   - Add a **visual progress bar** (not just `Node X / Y` text) using block characters `▓░`
   - Show the current node's `id` alongside the index
   - Indicate if the current node is a branch point or has custom traversal

7. **Speaker notes support** — Add a toggleable speaker notes panel (key: `s`). When active, split the screen to show `speaker_notes` from the current `Node` in a secondary pane below or beside the content. Add a new `Action::ToggleSpeakerNotes` variant.

8. **Mouse support for presentation mode** — Enable `crossterm::EnableMouseCapture` in the event loop (`crates/fireside-cli/src/commands/session.rs`):
   - Click on branch options to select them
   - Click left/right halves of screen for prev/next
   - Scroll wheel for prev/next
   - Add `Action::MouseClick(x, y)` and `Action::MouseScroll(direction)` variants to `crates/fireside-tui/src/event.rs`

9. **Responsive breakpoints** — Use the `Breakpoint` enum (Compact/Standard/Wide) already defined in `crates/fireside-tui/src/design/tokens.rs` to adapt layout spacing and content sizing based on terminal dimensions.

**Verification:** `cargo clippy -- -D warnings`, `cargo test`, manual visual testing at 80×24, 120×40, and 200×60 terminal sizes. Test with branching example files.

---

### Phase 2: Editor Mode Foundation

**Goal:** Deliver a functional in-TUI editor for modifying presentations without touching raw JSON.

**Steps**

1. **Editor state machine** — Add `AppMode::Editing` variant (and sub-modes) to `crates/fireside-tui/src/app.rs`:
   - `Editing::NodeList` — browsing nodes in a sidebar
   - `Editing::NodeEditor` — editing properties/content of a single node
   - `Editing::BlockEditor` — editing a specific content block
   - `Editing::GraphView` — viewing the ASCII graph
   - `Editing::Settings` — configuring presentation properties
   - Add `Action::EnterEditMode`, `Action::EnterPresentMode`, `Action::SwitchEditorPanel` etc. to the action enum
   - Toggle between modes with a key (e.g., `e` from presentation, `Esc` from editor to presentation)

2. **Editor layout** — Three-pane layout:
   - **Left sidebar** (~25% width): Node list with current node highlighted, showing node IDs and first content block summary. Scrollable with `j/k` or arrow keys.
   - **Center panel** (~50% width): Content editor showing the selected node's content blocks as editable forms. Each block shows its `kind` badge and editable fields (text content, heading level, code language, etc.).
   - **Right panel** (~25% width): Property inspector showing node metadata (id, layout, transition, traversal settings, speaker notes). Form-based editing with tab-between-fields navigation.

3. **Wire up Command execution** — Implement `apply()` on each `Command` variant in `crates/fireside-engine/src/commands.rs`:
   - `UpdateNodeContent` → mutate node's content blocks in the `Graph`
   - `AddNode` → insert node, update `node_index`
   - `RemoveNode` → remove node, update indices, fix dangling references
   - `SetTraversalNext` / `ClearTraversalNext` → modify node's `Traversal`
   - Each command should return an inverse command for undo
   - Wire `CommandHistory` to actually execute commands and maintain undo/redo stacks

4. **Undo/redo** — Connect `CommandHistory.undo()` and `CommandHistory.redo()` to the editor's action loop. Bind `Ctrl+Z` → undo, `Ctrl+Shift+Z` or `Ctrl+Y` → redo. Show undo/redo availability in the editor status bar.

5. **Content block CRUD** — In the center editor panel:
   - Add new block: show a kind selector (heading/text/code/list/divider/container/extension)
   - Edit block: inline text editing with cursor movement (use ratatui's `Input` or a custom text widget)
   - Reorder blocks: `Alt+↑` / `Alt+↓` to move blocks up/down
   - Delete block: `d` with confirmation, or `Ctrl+D`
   - Each operation emits a `Command` that goes through `CommandHistory`

6. **Real-time validation** — After each edit command, run `validate_graph()` from `crates/fireside-engine/src/validation.rs` and display `Diagnostic` results in a collapsible validation panel at the bottom of the editor. Color-code by severity (error = red, warning = yellow).

7. **Save workflow** — Implement `PresentationSession::save()` that serializes the `Graph` back to JSON (using the `GraphFile` serde format). Bind to `Ctrl+S`. Show dirty state indicator (modified `●` vs clean `○`) in the status bar. Use the `dirty` flag already on `PresentationSession`.

8. **Implement `fireside edit`** — Replace the `bail!("not yet implemented")` in `crates/fireside-cli/src/commands/session.rs` with a real editor session launch. Reuse the same event loop structure as `run_presentation()` but start in `AppMode::Editing`.

**Verification:** `cargo test` covering command apply/undo roundtrips. Manual testing: create a presentation with `fireside new`, open with `fireside edit`, modify content blocks, save, then verify with `fireside present`.

---

### Phase 3: Graph View & Advanced Editor Features

**Goal:** Add graph visualization and the remaining editor power features.

**Steps**

1. **ASCII graph renderer** — Create `crates/fireside-tui/src/ui/graph.rs` that renders the presentation's node graph as an ASCII diagram:
   - Each node rendered as a box: `┌─[node-id]─┐`
   - Edges drawn with box-drawing characters (`│`, `─`, `├`, `└`, `→`)
   - Linear sequences shown vertically
   - Branch points shown with multiple outgoing edges labeled with option keys
   - Current node highlighted
   - History path shown with a distinct style (bold or colored edges)
   - Scrollable for large graphs

2. **Graph navigation** — From the graph view, press `Enter` on a highlighted node to jump to it in the editor or presenter. Use `j/k/h/l` or arrow keys to navigate the graph. Show a mini-map in a corner panel when in editor mode.

3. **Presentation settings panel** — In `Editing::Settings` sub-mode, provide a form for:
   - Graph metadata: title, author, date, description, version, tags
   - Default layout and transition (from `NodeDefaults`)
   - Theme selection (list available `.toml` themes, preview colors inline)
   - Font preference (use the font detection from `crates/fireside-tui/src/design/fonts.rs`)

4. **Mouse support in editor mode** — Extend mouse handling to editor-specific interactions:
   - Click to select node in sidebar
   - Click to focus content block in editor
   - Click to select field in property inspector
   - Scroll wheel in panels for scrolling

5. **Diff/change tracking** — Visual indicators for modified content:
   - Modified nodes shown with a marker (`*`) in the node list
   - Changed fields highlighted with accent color in property inspector
   - Track against the last-saved state using the `dirty` flag

**Verification:** Render test graphs with various topologies (linear, branching, converging). Test navigation between graph view and editor. Test settings save/load cycle.

---

### Phase 4: Media Rendering & Visual Effects

**Goal:** Replace placeholders with real rendering and add visual polish.

**Steps**

1. **Image rendering** — Add `ratatui-image` dependency (or Sixel/Kitty protocol support) for inline image display in terminals that support it. Graceful fallback to the enhanced placeholder from Phase 1 in unsupported terminals. Update the `Image` arm in `crates/fireside-tui/src/render/markdown.rs` and change `RenderError::ImageLoad` from unused to active.

2. **Transition animations** — Wire up the `Transition` enum from `crates/fireside-core/src/model/transition.rs`. Use the existing `Action::Tick` (currently a no-op) as the animation driver:
   - `Fade` → progressive alpha/dim effect over frames
   - `SlideLeft`/`SlideRight` → horizontal offset animation
   - `Wipe` → progressive column reveal
   - `Dissolve` → random character replacement
   - `Matrix`/`Typewriter` → character-by-character reveal effects
   - Animation duration configurable (e.g., 300ms default, 8-12 frames)

3. **Container block layout rendering** — Implement layout-hint-aware rendering for `Container` blocks:
   - `split-horizontal` → render children in side-by-side columns
   - `split-vertical` → render children in stacked rows
   - Other layout hints → apply corresponding layout logic from the layout engine

4. **Extension block rendering** — For known extension types, render structured content. For unknown types, render the `fallback` content blocks if present, otherwise show the `type` identifier with a styled indicator.

**Verification:** Test with example files containing images, various transitions, nested containers, and extension blocks. Visual testing across iTerm2, Alacritty, Kitty, and basic Terminal.app.

---

### Phase 5: Documentation, Guides & Polish

**Goal:** Make the TUI discoverable, documented, and polished for users.

**Steps**

1. **In-app keyboard shortcut reference** — Expand the help overlay in `crates/fireside-tui/src/ui/help.rs` from 6 entries to a categorized, scrollable reference:
   - Categories: Navigation, Branching, Display, Editor, Graph View, System
   - Show mode-specific shortcuts (dimming inapplicable ones)
   - Make it accessible via `?` in any mode

2. **Keyboard shortcut documentation page** — Create `docs/src/content/docs/reference/keyboard-shortcuts.md` with full keybinding tables for both modes, aligned with the in-app help content.

3. **Presentation design best practices guide** — Create `docs/src/content/docs/guides/presentation-design.md` covering:
   - Structuring content for graph-based presentations
   - When and how to use branching effectively
   - Layout selection guidance (which layouts work for which content types)
   - Audience engagement patterns unique to branching presentations
   - Tips on code blocks, images, and containers
   - Examples of well-structured presentations (reference example files)

4. **Update existing guide pages** — Revise `docs/src/content/docs/guides/deep-dive-fireside-tui.md` to document the new editor mode, graph view, and enhanced presentation features.

5. **Create example presentations** — Add 2-3 example files in `docs/examples/` showcasing:
   - A rich linear presentation using all content block types and layouts
   - A branching presentation with multiple paths and merge points
   - An editorial/lesson format demonstrating the editor workflow

6. **Responsive design testing** — Verify and fix rendering at standard terminal sizes:
   - Compact (80×24) — everything readable, no overflow
   - Standard (120×40) — optimal experience
   - Wide (200×60) — content doesn't float or become unreadable
   - Use `Breakpoint` from design tokens throughout

7. **Accessibility review** — Verify WCAG AA contrast ratios using the `meets_contrast_aa()` function in `crates/fireside-tui/src/design/tokens.rs` for all theme color combinations. Ensure all UI elements have keyboard-accessible equivalents.

**Verification:** `cd docs && npm run build` passes. All example files validate with `fireside validate`. Manual review of all documentation pages.

---

### Phase 6: Integration, Settings & Release Polish

**Goal:** Final integration, configuration system, and release readiness.

**Steps**

1. **Settings system** — Wire the `Settings` struct in `crates/fireside-tui/src/config/settings.rs` to load from a config file (`~/.config/fireside/config.toml` or `fireside.toml` in project root). Include all user-configurable options: default theme, poll timeout, show progress, show timer, editor preferences, animation speed.

2. **`after` traversal support** — Implement the `after` field handling in `crates/fireside-engine/src/traversal.rs`. When a branch path ends, the engine should follow the `after` field to rejoin the main presentation flow.

3. **Hot-reload in presenter** — Watch the presentation file for changes and reload automatically. Useful for the edit → present → edit workflow.

4. **Final integration testing** — End-to-end test: `fireside new` → `fireside edit` (modify) → `fireside present` (verify) → round-trip JSON fidelity. Test on macOS Terminal.app, iTerm2, Alacritty, and Kitty.

5. **Performance profiling** — Ensure smooth rendering at 60fps even for large presentations (100+ nodes). Profile the render loop and optimize hot paths if needed.

---

### Decisions

- **Graph view style:** ASCII box-drawing diagram (static rendered) — avoids the complexity of an interactive canvas in TUI while still giving structural visibility
- **Editor architecture:** Full TUI editor with form-based blocks, property panels, and command-pattern undo/redo — built on the existing `Command`/`CommandHistory` scaffold
- **Research deliverable:** Formal competitive analysis document published in the docs site
- **Phasing rationale:** Presentation mode polish (Phase 1) comes before editor (Phase 2) because it's what users encounter first and validates the rendering pipeline that the editor will reuse
- **Mouse + keyboard balance:** Keyboard is primary (everything keyboard-accessible); mouse is supplementary (click on branch options, click to select in editor, scroll)

### Verification

- Each phase: `cargo clippy -- -D warnings`, `cargo test`, `cargo fmt --check`
- Docs: `cd docs && npm run build`
- Visual: Manual testing at 3 terminal sizes per phase
- Integration: Full `new → edit → present → validate` round-trip after Phase 2+
