# Task 17 — TUI: ViewMode + container layouts via ratatui

**Depends on:** 07, 09, 10
**Crates:** fireside-tui
**Phase:** 4 — the big one; budget a full week

## Goal

Rendering keys off the spec model: `view-mode` controls the frame, `container.layout` controls arrangement. The obsolete node-level `Layout` enum stops driving rendering. This is also the flagship UX task — the result should feel deliberate and consistent, not translated.

## Ratatui mapping (APIs verified via Context7 against ratatui 0.30)

| Spec concept | Ratatui implementation |
|---|---|
| `view-mode: default` | existing chrome: header, content area, footer (`ui/chrome.rs`) |
| `view-mode: fullscreen` | content gets the full frame; chrome hidden except a minimal key hint |
| `container.layout: "stack"` (default) | `Layout::vertical` with `Constraint::Length(h)` per child (measured content height) |
| `container.layout: "columns"` | `Layout::horizontal` with `Constraint::Fill(1)` per child and `.spacing(1)` — equal columns, left-to-right in array order per the spec |
| `container.layout: "center"` | `Rect::centered(Constraint::Percentage(..), Constraint::Length(content_height))` or `Layout` with `Flex::Center` |
| unknown layout string | render as `stack` (spec: layout is a hint) |

## Steps

1. In `crates/fireside-tui/src/render/` add container layout handling (extend `render/blocks.rs` Container arm + `render/layout.rs`). Nested containers recurse — no depth limit beyond what the area allows.
2. Presenter (`ui/presenter.rs`) and editor preview consume `node.resolved_view_mode(graph.defaults)` (Task 07 helper). Fullscreen hides chrome.
3. Legacy `Layout` translation in ONE place (a `fn legacy_layout_hint(Layout) -> ...` in `render/layout.rs`): `Fullscreen`/`CodeFocus` → treat as `view-mode: fullscreen`; `Center`/`Title` → center the node's content area; all others → default. Delete the per-layout rendering arms that no longer apply. Mark `Layout` `#[deprecated]` in core only if it doesn't break `-D warnings` in CI — otherwise leave a doc comment pointing at ADR-0002.
4. **Design consistency pass** (use `DesignTokens` for every value):
   - identical horizontal padding for content in default and fullscreen modes;
   - column gutters = 1 cell everywhere; centered blocks never exceed 80% width;
   - heading/code/list styling unchanged (already token-driven).
5. Update `design/templates.rs` so editor templates emit container-based layouts, not node `layout` values.
6. Tests: unit-test the layout math (areas returned for stack/columns/center at known sizes); update existing render tests.

## Do NOT

- Remove the `Layout` enum from core (ADR-0002 in Task 19 owns its retirement timeline).
- Introduce new color or style constants outside `design/tokens.rs`.
- Touch transition animations.

## Acceptance

```bash
cargo test -p fireside-tui
cargo run -q -p fireside-cli -- present docs/examples/hello.json --plain
# Manual: cargo run -p fireside-cli -- present docs/examples/hello.json
#   - intro + thanks render centered; layout-demo renders two columns side by side;
#   - code-demo opens fullscreen; toggling view mode at runtime still works.
```
