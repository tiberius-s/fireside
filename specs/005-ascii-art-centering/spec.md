# Feature Specification: ASCII art centering and clipping

**Feature Branch**: `005-ascii-art-centering`

**Created**: 2026-07-12

**Status**: Draft

**Input**: User description: "ASCII art engine-side rendering: center and gracefully clip code blocks whose language is absent, \"text\", or \"ascii\", per the strategic plan's P1 ASCII art item, engine-only layer, no spec change."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A presenter's ASCII diagram reads as deliberate, not broken (Priority: P1)

A presenter includes a small ASCII diagram or plain-text figure in a slide
(the only way to author ASCII art in Fireside today — a code block with no
language, or `"text"`/`"ascii"`). Today it renders inside a box stretched
to the full width of the content area, leaving a large blank strip beside
the art and making the slide look like a rendering bug rather than an
intentional figure. The presenter wants the art to appear centered and
sized to itself, the way a deliberately placed diagram should look.

**Why this priority**: This is the entire value of the feature — it's the
only user-visible outcome. Without it, nothing changes for a presenter.

**Independent Test**: Author a deck with a narrow ASCII-art code block (no
language) on a slide by itself; present it and confirm the art appears
centered within the content area, sized to its own content rather than
stretched full-width.

**Acceptance Scenarios**:

1. **Given** a code block with no language attribute containing a narrow
   multi-line ASCII figure, **When** the slide is presented, **Then** the
   figure's box is sized to its own widest line and is horizontally
   centered within the available content width.
2. **Given** a code block with language `"text"` or `"ascii"` and the same
   narrow content, **When** the slide is presented, **Then** it centers
   identically to the no-language case (all three forms are treated the
   same way).
3. **Given** a code block with an explicit programming-language attribute
   (e.g. `"rust"`) and the same narrow content, **When** the slide is
   presented, **Then** the box still stretches to the full available
   width and stays left-aligned — existing source-code-listing behavior is
   unchanged.

---

### User Story 2 - Oversized ASCII art degrades gracefully instead of breaking the layout (Priority: P2)

A presenter's ASCII art is wider than the terminal it's being presented
in (a diagram authored on a wide monitor, presented on a narrower one, or
at a small terminal size). The presenter needs the slide to still be
usable — no panic, no runaway layout, no silently truncated content with
no indication anything was cut.

**Why this priority**: A real but less common situation than the primary
centering case; correctness here prevents a broken presentation rather
than improving a working one.

**Independent Test**: Author an ASCII-art code block wider than a small
terminal (e.g. 40 columns); present at that width and confirm the art is
clipped with a visible cut marker on the affected lines rather than
overflowing, wrapping unexpectedly, or crashing.

**Acceptance Scenarios**:

1. **Given** an ASCII-art code block whose widest line exceeds the
   available content width, **When** presented, **Then** the box caps to
   the available width (no centering gap) and every line wider than that
   width is cut with a visible marker at the point of the cut.
2. **Given** the same oversized fixture, **When** presented at several
   different terminal widths in sequence (as a presenter resizing their
   window would experience), **Then** rendering never panics and content
   remains readable at every size.

### Edge Cases

- A code block classified as ASCII art placed inside a `container` with
  centered layout: the two centering behaviors (the code block's own
  self-sizing/centering, and the container's whole-unit centering) must
  compose correctly — the art still reads as one centered unit, not
  double-shifted or misaligned relative to other centered content on the
  same slide.
- An ASCII-art code block with line numbers or highlighted lines enabled:
  those features continue to work exactly as they do for ordinary code
  listings; only the box's horizontal size and position are affected by
  this feature.
- An empty ASCII-art code block (zero-width content): sizes to a sane
  minimum (at least its label) rather than collapsing to nothing or
  panicking.
- Case sensitivity: only the literal strings `"text"` and `"ascii"` (and
  an absent language) count as ASCII art; any other casing (`"Text"`,
  `"ASCII"`) is treated as an explicit language and does not center. This
  matches how `language` is already compared elsewhere in the renderer,
  so behavior is predictable and consistent across the codebase.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A code block MUST be classified as ASCII art when its
  language attribute is absent, or is exactly `"text"`, or is exactly
  `"ascii"`.
- **FR-002**: An ASCII-art code block's rendered box (its top rule, every
  content row, and its bottom rule) MUST be sized to the natural width of
  its widest content line (including any row prefix such as a
  line-number gutter), not stretched to the full available width.
- **FR-003**: An ASCII-art code block's box MUST be horizontally centered
  as a single unit within the available content width.
- **FR-004**: A code block with any language other than absent, `"text"`,
  or `"ascii"` MUST continue to render exactly as it does today: box
  stretched to the full available width, left-aligned, no centering.
- **FR-005**: When an ASCII-art code block's natural width exceeds the
  available content width, its box MUST cap to the available width (no
  centering offset applied in that case) and each overflowing content
  line MUST be visibly cut rather than silently truncated, wrapped, or
  allowed to overflow the layout.
- **FR-006**: Line-number gutters and highlighted-line emphasis/dimming
  MUST continue to function unchanged for ASCII-art-classified code
  blocks.
- **FR-007**: When an ASCII-art code block is nested inside a container
  with centered layout, the composed result MUST still present the art as
  a single visually centered unit, without breaking the container's
  existing whole-unit-centering guarantee for its other children.
- **FR-008**: Rendering MUST NOT panic or produce a malformed layout for
  any ASCII-art code block content, including empty content or content
  wider than the smallest realistically supported terminal width.

### Key Entities

- **ASCII-art code block**: A code content block whose language
  classification (per FR-001) marks it for the sizing/centering behavior
  in this feature, as opposed to an ordinary source-code-listing code
  block.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An ASCII-art code block presented alone on a slide at a
  standard terminal size (80 columns by 24 rows) is visually centered and
  legible, verified by rendering the slide and confirming placement.
- **SC-002**: 100% of existing code-block rendering behavior for
  explicit-language code blocks is unchanged — no existing test of
  source-code-listing rendering needs to change to accommodate this
  feature.
- **SC-003**: Presenting an oversized ASCII-art figure at a small terminal
  size never crashes the presenter and always shows a visible indication
  that content was cut.

## Assumptions

- No new content-block kind or protocol field is introduced; this feature
  only changes how the existing `code` block renders when its language
  matches the ASCII-art classification. This is the "engine-only, no spec
  change, ship immediately" layer from the strategic plan; the separate
  spec-first `fit` field (a `"clip" | "center" | "shrink"` override) is
  explicitly out of scope for this feature.
- "Available content width" means whatever width is already being passed
  to block rendering at the call site (accounting for any enclosing
  container's inner width) — this feature does not change how that width
  is computed elsewhere in the renderer.
- Existing clipping/ellipsis behavior (used for long lines in ordinary
  code listings today) is reused as-is for the oversized-ASCII-art case;
  no new visual cut-marker style is introduced.
