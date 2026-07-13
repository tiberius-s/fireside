# Tasks: ASCII art centering and clipping

**Input**: Design documents from `/specs/005-ascii-art-centering/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included — Test Discipline requires scenario coverage for
user-visible TUI state, plus unit coverage per contracts/code-block-rendering.md's
invariants.

**Organization**: Single-file change (`blocks.rs`), so tasks are grouped
by user story but largely sequential — US1 and US2 share the same
`code()` rewrite, split by which behavior each task's tests prove.

## Phase 1: Setup

- [X] T001 Read the current `code()` implementation and its callers in `crates/fireside-tui/src/render/blocks.rs` once more immediately before editing, to confirm line numbers haven't shifted since planning

## Phase 2: User Story 1 - Presenter's ASCII diagram reads as deliberate (Priority: P1)

**Goal**: ASCII-art-classified code blocks size to content and center; explicit-language code blocks are unchanged.

**Independent Test**: Per spec.md US1 acceptance scenarios — narrow no-language/`"text"`/`"ascii"` code blocks center; a `"rust"` block with the same content stays full-width.

- [X] T002 [US1] In `crates/fireside-tui/src/render/blocks.rs`, rewrite `code()` per contracts/code-block-rendering.md: compute `is_ascii_art` classification, compute `box_width` (natural content width for ASCII art, `full_width` otherwise) before building any lines, use `box_width` (not `width`) for the top-rule fill calculation, the per-row `avail` clip width, and the bottom rule
- [X] T003 [US1] In the same rewrite, after building the box's lines, apply a uniform leading `Span::raw` pad to every line (top rule, each content row, bottom rule) equal to `(full_width - box_width) / 2` whenever `box_width < full_width`; verify non-ASCII-art blocks get zero pad (i.e. `box_width == full_width` always for them, so this branch is a no-op and existing output is byte-for-byte unchanged)
- [X] T004 [P] [US1] Add unit test `ascii_art_code_block_centers_to_its_content_width` in `blocks.rs`'s `#[cfg(test)] mod tests`: a `language: None` block with a short multi-line source at a wide `width`; assert the rendered box width is less than the given width and every line shares the same leading pad
- [X] T005 [P] [US1] Add unit test `text_and_ascii_language_strings_center_like_no_language` in `blocks.rs`: same content with `language: Some("text".into())` and separately `Some("ascii".into())`; assert identical centering behavior to the `None` case
- [X] T006 [P] [US1] Add unit test `explicit_language_code_block_stays_full_width` in `blocks.rs`: same narrow content with `language: Some("rust".into())`; assert the box stretches to the full given width with no leading pad, and diff this test's expected output against the pre-feature behavior to prove zero regression
- [X] T007 [US1] Add a new scenario test in `crates/fireside-tui/src/render/mod.rs`'s `TestBackend` suite at 80×24: a node whose only content block is a narrow ASCII-art code block; assert the rendered screen shows the art horizontally centered within the content area (per quickstart.md Scenario 4 / SC-001)

**Checkpoint**: Narrow ASCII art centers; explicit-language code is provably unchanged.

---

## Phase 3: User Story 2 - Oversized ASCII art degrades gracefully (Priority: P2)

**Goal**: ASCII art wider than the available width caps and clips instead of breaking.

**Independent Test**: Per spec.md US2 acceptance scenarios — an oversized fixture clips with a visible marker and never panics across multiple widths.

- [X] T008 [US2] Add unit test `oversized_ascii_art_caps_and_clips_with_ellipsis` in `blocks.rs`: a `language: None` block with a line far wider than a narrow given `width`; assert the box width caps at the given width (no pad) and the overflowing line ends with the existing ellipsis marker (reusing `clip`/`clip_spans` — no new assertions about clipping mechanics beyond confirming they still fire)
- [X] T009 [P] [US2] Add unit test `ascii_art_never_panics_across_a_range_of_widths` in `blocks.rs`: render the same oversized fixture at several widths (including very small, e.g. 1-10 columns) in a loop; assert no panic (the test itself succeeding is the assertion)
- [X] T010 [US2] Add unit test `empty_ascii_art_code_block_does_not_collapse_or_panic` in `blocks.rs`: `language: None`, empty `source`; assert the box still renders a top and bottom rule at least as wide as the label, no panic

**Checkpoint**: Oversized and empty ASCII art both degrade safely.

---

## Phase 4: Regression & Composition

- [X] T011 Run the existing `centered_code_keeps_its_internal_alignment` test unmodified; confirm it still passes (proves composition with `container { layout: "center" }` per FR-007, contracts/code-block-rendering.md's Composition section)
- [X] T012 Run all other existing `blocks.rs` and `render/mod.rs` tests that exercise `code`/`ContentBlock::Code` (e.g. `code_renders_rules_line_numbers_and_clipping`, `highlight_lines_dim_the_rest_and_keep_focus_bright`, `code_gets_syntax_colors_from_the_theme`) unmodified; confirm all still pass, proving SC-002 (zero regression to explicit-language rendering)

## Phase 5: Polish & Cross-Cutting

- [X] T013 Run `cargo test -p fireside-tui` and `cargo test --workspace`; both must be clean
- [X] T014 Run `cargo clippy --workspace --all-targets`; must stay silent
- [X] T015 Update `.claude/plans/2026-07-12-strategic-improvement-plan.md`'s Progress Log: mark "Week 1 ASCII art engine-side (center/clip)" done, with a technical summary matching the style of existing entries
- [X] T016 Update the `project_strategic_plan_2026_07` memory file and `MEMORY.md` index to reflect all of Week 1 being complete

---

## Dependencies & Execution Order

- **Phase 1** has no dependencies.
- **Phase 2 (US1)**: T002 and T003 are the core rewrite and must land together/sequentially (same function). T004-T006 depend on T002-T003 existing to test against, but are independent of each other (different test functions) — marked [P]. T007 depends on T002-T003.
- **Phase 3 (US2)**: depends on Phase 2's `code()` rewrite existing (T002-T003) but is otherwise independent of Phase 2's specific tests. T008 and T010 are sequential-safe but independent tests; T009 is independent — marked [P].
- **Phase 4**: depends on all of Phase 2 and 3 being implemented (it's the regression check on the finished rewrite).
- **Phase 5**: depends on everything above.

## Implementation Strategy

**MVP scope**: Phase 1 + Phase 2 (US1) delivers the entire user-visible
value — centered ASCII art with explicit-language code provably
unchanged. Phase 3 (US2, P2) hardens the edge case. Phase 4 is the
regression proof this project's "whole stage at a time" convention
requires before calling anything done. Phase 5 is mandatory wrap-up.
