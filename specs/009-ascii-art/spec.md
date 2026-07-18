# Feature Specification: ASCII art content block

**Feature Branch**: `009-ascii-art`

**Created**: 2026-07-18

**Status**: Draft

**Input**: User description: "Add an additive `ascii-art` content block kind to the Fireside protocol (bump to 0.1.3). All ASCII art is generated at authoring time. Two authoring-time conversion paths land in the CLI only: text-to-banner and image-to-ASCII conversion, per the GO decision in ADR-011. The block reuses the existing centered/clipped rendering path from the earlier ASCII-art-centering feature, with zero new rendering dependencies. Validation warns on art that's too wide or empty. Reveal support is uniform with every other content block, atomic only (no per-line reveal)."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
-->

### User Story 1 - Author turns a title into a stylized banner (Priority: P1)

An author wants a slide to open with a large stylized rendering of a short
word or phrase (e.g. a section title like "BREAK" or a product name),
instead of the plain heading text every other slide uses. Today the only
way to get this is to hand-draw the letters character by character, or
find an external website and copy-paste the result. The author instead
runs a single command against the phrase and gets ready-to-use art they
can drop straight into their deck.

**Why this priority**: This is the most common and lowest-effort ASCII
art use case (a stylized title), and the one named first in the feature's
own motivation. Without it, the feature offers no more value than
hand-typing art already did.

**Independent Test**: Run the banner-generation command against a short
phrase; confirm it produces multi-line stylized text output. Paste that
output into a deck's ascii-art block and present it: confirm it appears
centered and sized to itself, not stretched to the full slide width.

**Acceptance Scenarios**:

1. **Given** a short phrase of ordinary letters and numbers, **When** the
   author runs the banner-generation command against it, **Then** the
   command produces multi-line stylized text output representing that
   phrase as large block letters.
2. **Given** the generated banner output pasted into a deck's ascii-art
   block, **When** the slide is presented, **Then** the art appears
   centered and sized to its own widest line, using the same visual
   treatment as any other ASCII art in Fireside.
3. **Given** a phrase containing a character the banner style has no
   letterform for (e.g. an emoji or non-Latin character), **When** the
   author runs the command, **Then** it still produces output for every
   character it does recognize rather than failing outright, so one
   unsupported character doesn't block the whole phrase.

---

### User Story 2 - Author converts an existing image into ASCII art (Priority: P1)

An author has a small image on disk — a logo, a diagram, a screenshot —
and wants a text-based rendering of it to include in a slide, since
Fireside presents in a terminal and doesn't display real images. Today
there is no way to do this at all short of manually recreating the image
as text. The author instead runs a single command against the image file
and gets ready-to-use text art.

**Why this priority**: Equal in value to banner generation — both are
named as the two authoring-time conversion paths this feature exists to
provide — but ranked alongside rather than above it because a presenter
with no images to convert gets full value from banner generation alone.

**Independent Test**: Run the image-conversion command against a small
local image file; confirm it produces multi-line text output shaped like
the image's contents. Paste that output into a deck's ascii-art block and
present it: confirm it renders like any other ASCII art.

**Acceptance Scenarios**:

1. **Given** a readable local image file, **When** the author runs the
   image-conversion command against it, **Then** the command produces
   multi-line plain-text output whose shading approximates the image.
2. **Given** a path to a file that doesn't exist or isn't a readable
   image, **When** the author runs the image-conversion command, **Then**
   the command reports a clear error and does not crash.

---

### User Story 3 - Ascii-art appears alongside other content, on its own or hand-authored (Priority: P2)

An author wants to place ascii-art in a slide as a first-class piece of
content, the same way they place headings, text, or code blocks —
whether that art came from one of this feature's generation commands or
was typed/pasted in by hand (e.g. an ASCII diagram copied from
elsewhere). The block also needs to work with the presentation's existing
progressive-reveal feature, so an author can time an art reveal to their
narration exactly like any other content.

**Why this priority**: Establishes ascii-art as ordinary content rather
than a special case tied only to the two generator commands, and confirms
it composes with a feature (reveal) that already applies to every other
block kind. Lower priority than the two generation paths because a
hand-typed or already-existing art string is a narrower need.

**Independent Test**: Author a deck with an ascii-art block containing
hand-typed art (no generator command involved) and confirm it presents
correctly. Separately, mark an ascii-art block to reveal at a later step
in a node that also has other content; confirm it stays hidden until that
step and then appears as a whole, not line by line.

**Acceptance Scenarios**:

1. **Given** an ascii-art block whose content was typed directly rather
   than produced by either generation command, **When** the slide is
   presented, **Then** it renders identically to generated art — the
   block's origin is not distinguishable at presentation time.
2. **Given** an ascii-art block marked to reveal at a later step, **When**
   the presenter has not yet reached that step, **Then** none of the
   block's lines are shown and it occupies no layout space.
3. **Given** the same block once its reveal step is reached, **When** it
   becomes visible, **Then** every line of the art appears together in
   the same action — never a partial, line-by-line reveal.

---

### User Story 4 - Author is warned about an oversized or empty art block before presenting (Priority: P3)

An author's generated or hand-typed art is wider than fits inside the
presentation card, or an ascii-art block was accidentally left with no
content at all. Checking the deck warns them about this before they
present, instead of them discovering a clipped or blank slide live.

**Why this priority**: A safety net, not core value — the two generation
commands and manual authoring already work without it. Ranked lowest
because nothing about the primary use cases depends on it.

**Independent Test**: Author a deck with one ascii-art block wider than
the checker's width threshold and another with empty content. Run the
deck checker: confirm each produces a distinct warning naming the
offending block, and confirm an ordinary, correctly sized art block
produces neither.

**Acceptance Scenarios**:

1. **Given** an ascii-art block whose widest line exceeds the width that
   fits inside the presentation card, **When** the deck is checked,
   **Then** a warning identifies the block as too wide.
2. **Given** an ascii-art block with no art content, **When** the deck is
   checked, **Then** a warning identifies the block as empty.
3. **Given** an ascii-art block within normal width and non-empty,
   **When** the deck is checked, **Then** neither warning is produced for
   it.

---

### Edge Cases

- An ascii-art block is nested inside a container (e.g. a two-column
  layout or boxed callout) that is itself reveal-gated to a later step
  than the block: the existing reveal-masked-by-container warning
  (introduced by the reveal feature) applies to ascii-art exactly as it
  does to every other block kind — no new warning is needed.
- A phrase given to the banner-generation command produces no recognized
  characters at all (e.g. only unsupported symbols): the command reports
  that no output could be produced rather than emitting a blank block
  silently.
- An author opens a deck containing an ascii-art block in a presenting
  tool built before this feature existed: the tool cannot safely guess
  what the block means (unlike the earlier reveal feature, this is a new
  block kind, not an optional field on an existing one), so it must
  refuse to open the deck with a clear compatibility message rather than
  silently skipping or misrendering the art. See FR-011.
- A deck that never uses an ascii-art block behaves identically before
  and after this feature ships.
- Generated or hand-typed art contains trailing blank lines or trailing
  spaces on a line: rendering treats it the same as any other multi-line
  text block — trailing whitespace does not itself trigger the
  too-wide warning.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Authors MUST be able to include a dedicated ascii-art
  content block in a slide's content, distinct from an ordinary code
  block, consisting of pre-rendered multi-line text.
- **FR-002**: An ascii-art block MUST render centered within the
  available content width and sized to its own widest line, using the
  same visual treatment slides already use for language-less ASCII-art
  code blocks — never stretched to the slide's full width.
- **FR-003**: Authors MUST be able to generate ascii-art content from a
  short text phrase as a large stylized banner via a dedicated command,
  without hand-drawing letters or using an external tool.
- **FR-004**: Authors MUST be able to generate ascii-art content from a
  local image file via a dedicated command, without hand-drawing the art
  or using an external tool.
- **FR-005**: Authors MUST be able to attach an optional plain-language
  description to an ascii-art block, for reference by anyone who can't
  see the rendered art.
- **FR-006**: An ascii-art block MUST support the same optional
  progressive-reveal marking already available on every other content
  block kind, with no new concept for the author to learn.
- **FR-007**: A reveal-marked ascii-art block MUST appear or stay hidden
  as one indivisible unit — never partially, line by line.
- **FR-008**: Checking a deck MUST warn the author when an ascii-art
  block's widest line exceeds the width that fits inside the presentation
  card, identifying the specific block.
- **FR-009**: Checking a deck MUST warn the author when an ascii-art
  block has no art content, identifying the specific block.
- **FR-010**: A deck that contains no ascii-art blocks MUST look and
  behave exactly as decks do today — no visual or behavioral change.
- **FR-011**: A deck containing an ascii-art block MUST be clearly
  rejected, with an explicit compatibility message, by any presenting
  tool built before this feature existed — this is a new content kind,
  not an optional addition to an existing one, and silently
  misinterpreting it would be worse than refusing it.
- **FR-012**: Ascii-art content MUST consist of plain text only — no
  embedded color or formatting codes — since Fireside's visual styling is
  controlled entirely by the presentation's own theme, not by content
  authors.
- **FR-013**: The banner-generation command MUST still produce output for
  every character it recognizes in a phrase that also contains characters
  it does not recognize, rather than failing the whole phrase over one
  unsupported character; if it recognizes none of the phrase, it MUST
  report that no output could be produced instead of emitting a blank
  block.
- **FR-014**: The image-conversion command MUST report a clear,
  actionable error — not a crash — when given a path that does not exist
  or is not a readable image.

### Key Entities

- **Ascii-art block**: A content block representing pre-rendered,
  plain-text ASCII/text art, with the same optional reveal marker every
  other content block has and an optional plain-language description.
  Distinct from a code block — it has no source-code-listing behavior and
  no "language" concept.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An author can produce a ready-to-use stylized text banner
  from a phrase with a single command and no external tool or website.
- **SC-002**: An author can produce ready-to-use ASCII art from a local
  image with a single command and no external tool or website.
- **SC-003**: Every deck that existed before this feature shipped
  presents identically after it ships, verified directly rather than
  assumed.
- **SC-004**: An author discovers an oversized or empty ascii-art block
  by checking their deck, before ever presenting it live.
- **SC-005**: An author who already knows how progressive reveal works
  can apply it to an ascii-art block with zero new concepts to learn.

## Assumptions

- The two generation commands are authoring-time conveniences that print
  ready-to-use art for the author to place into a deck; they do not
  themselves edit a deck file automatically. Deck editing remains a
  manual (or existing quick-edit / import) step.
- No color or per-line reveal is in scope — ascii-art is plain
  monospace text revealed as a whole, matching how every other content
  block already behaves.
- The width threshold used for the "too wide" warning matches the
  presentation card's usable width at the standard supported terminal
  size, consistent with how the existing ASCII-art centering feature
  already reasons about available width.
- This feature deliberately breaks forward compatibility for decks that
  use it: opening such a deck in an older, ascii-art-unaware version of
  Fireside is expected to fail with a clear message, not degrade
  gracefully — an explicit, accepted trade-off, unlike every prior
  additive protocol change this project has shipped.
