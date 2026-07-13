# Feature Specification: Incremental reveal

**Feature Branch**: `006-incremental-reveal`

**Created**: 2026-07-12

**Status**: Draft

**Input**: User description: "Incremental reveal (\"fragments\"): a new optional `reveal` field on every content block that lets an author mark a block as hidden until the presenter has pressed \"next\" enough times at the current node. Design decided in ADR-009: ordinal-over-distinct-values step semantics, reveal always precedes branch points and the terminal/next edge, reveal resets on every node entry (not history-aware), branch selection is blocked while reveal is pending, footer must show reveal progress, hidden blocks are structurally absent from render output, and a new `reveal-masked-by-container` validator warning."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Presenter reveals bullets one at a time (Priority: P1)

A presenter is narrating a slide with several bullet points. Instead of the
audience seeing the whole list at once (spoiling what's coming next), each
bullet appears only when the presenter presses the "next" key, in sync with
their narration. Once every marked bullet has appeared, the same key moves
the presentation on to whatever would normally come next (another slide, or
a choice, or the end of the path) — the presenter uses one familiar key for
both "reveal more" and "move on."

**Why this priority**: This is the single feature named as the most-missed
capability compared to every other presentation tool surveyed. Without it,
the entire "incremental reveal" value proposition doesn't exist — it is the
whole feature, not an enhancement to it.

**Independent Test**: Build a deck with one node containing three bullets
marked to reveal progressively. Start presenting: confirm only the first
bullet (and any unmarked content) is visible. Press "next" twice more:
confirm each press reveals exactly one more bullet, and a footer indicator
reflects progress. Press "next" a final time: confirm the presentation now
moves to the node's actual next destination, not a fourth phantom reveal
step.

**Acceptance Scenarios**:

1. **Given** a node whose content includes bullets marked to reveal at
   different points, **When** the presenter first arrives at the node,
   **Then** only the unmarked (always-visible) content and the earliest
   marked group are shown.
2. **Given** a node with reveal-marked content still pending, **When** the
   presenter presses "next", **Then** exactly one more group of content
   becomes visible and the presenter does not advance to a different node.
3. **Given** a node where every reveal-marked group has already been shown,
   **When** the presenter presses "next" again, **Then** the presentation
   behaves exactly as it would without any reveal marks on that node
   (follows its normal destination, asks for a choice, or reports the end
   of the path).
4. **Given** an author marks reveal groups with gaps in their numbering
   (e.g. the first group and the third group, with no second group used
   anywhere on the node), **When** the presenter presses "next" through
   the sequence, **Then** every press reveals something — no press is ever
   wasted waiting for a group number that was never used.
5. **Given** a deck with no reveal marks anywhere, **When** it is opened in
   this or any older, reveal-unaware presenting tool, **Then** every slide
   looks and behaves exactly as it did before this feature existed — full
   content, visible immediately, every time.

---

### User Story 2 - Reveal composes with side-by-side layouts (Priority: P2)

An author builds a two-column comparison slide (e.g. "before" vs. "after")
where the right-hand column should only appear after the presenter has
walked through the left-hand column. The still-hidden column must not leave
a blank gap or throw off the layout of the column that is showing — the
visible column should look intentional on its own, not like something is
missing.

**Why this priority**: Side-by-side layouts are one of the three existing
container arrangements and a natural pairing with progressive reveal (a
comparison is a classic reveal use case), but the feature is usable and
valuable without this specific composition, so it ranks below the core
mechanic.

**Independent Test**: Build a node with a two-column container where one
column is marked to reveal after the other. Confirm the first render shows
only the visible column, using the space a single column would use — not a
two-column layout with an empty second slot. Advance the reveal step and
confirm the second column now appears alongside the first, in the normal
two-column arrangement.

**Acceptance Scenarios**:

1. **Given** a two-column container where one column's content is
   reveal-gated and the other is not, **When** the presenter first arrives,
   **Then** only the ungated column is shown, without reserving empty space
   for the hidden one.
2. **Given** the state in the previous scenario, **When** the presenter
   reveals the gated column, **Then** both columns now appear side by side
   in their originally authored arrangement.

---

### User Story 3 - Author is warned about a reveal mistake before presenting (Priority: P3)

An author nests a block inside a group (e.g. a boxed callout) and marks the
outer group to appear at reveal step 2, but leaves an inner block marked to
appear at reveal step 1 — not realizing the inner block can never actually
appear before the outer group does, since the group itself is still hidden
at step 1. Checking the deck warns them about this before they present,
instead of them discovering the confusion live on stage.

**Why this priority**: This is an authoring-safety nicety, not a
presenting-time behavior — it improves the authoring experience but nothing
breaks for an audience if it's skipped, so it is the lowest priority of the
three.

**Independent Test**: Build a deck with a group marked to reveal at step 2
containing a child marked to reveal at step 1. Run the deck checker:
confirm it reports a warning naming the affected block, and confirm a deck
without this pattern produces no such warning.

**Acceptance Scenarios**:

1. **Given** a group reveal-marked at step 2 with an inner block
   reveal-marked at step 1, **When** the deck is checked, **Then** a
   warning identifies the inner block and explains it cannot appear earlier
   than its enclosing group.
2. **Given** a group and its children where every child's reveal mark is
   equal to or later than the group's own, **When** the deck is checked,
   **Then** no such warning is produced.

---

### Edge Cases

- A node has reveal-marked content AND is also a branch point (asks the
  presenter to choose between paths): the presenter must finish revealing
  everything before the choice becomes available — pressing a choice key
  early has no effect on the choice; it continues revealing instead. See
  FR-006, FR-007.
- A node has reveal-marked content AND is a dead end (no destination at
  all): the presenter can still step through every reveal via "next";
  only once everything is revealed does "next" report the end of the
  path. See FR-008.
- The presenter reveals everything on a node, goes back to the previous
  slide, then returns to this node again (by any means): the node starts
  over with nothing beyond its first reveal group shown, exactly as on a
  first visit — the presenter's earlier progress on that node is not
  remembered. See FR-009.
- Reveal marks are used inside a node whose content also includes entirely
  unmarked blocks: unmarked content is always visible, unaffected by reveal
  state, mixed freely with marked content. See FR-002.
- Two or more blocks share the exact same reveal mark: they appear
  together on the same "next" press, as one group. See FR-003.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Authors MUST be able to mark any individual content block
  (heading, text, code, list, image, divider, or group/container) as
  hidden until a specific point in the presenter's progression through the
  node, using an optional marker on that block. A block with no marker, or
  an explicit "no delay" marker, MUST behave exactly as all content does
  today: visible the instant the node is entered.
- **FR-002**: Unmarked content on a node with some reveal-marked content
  MUST remain visible at all times, unaffected by the node's reveal
  progress.
- **FR-003**: Multiple blocks sharing the identical reveal marker MUST
  become visible together, on the same presenter action, as a single
  group.
- **FR-004**: The sequence of reveal steps for a node MUST be derived from
  the distinct reveal markers actually used on that node's content, in
  ascending order — not from the raw numeric values of those markers.
  Consequently, no gap or irregularity in an author's chosen marker values
  can ever cause a presenter action to reveal nothing.
- **FR-005**: While any reveal step remains for the current node, the
  presenter's "advance" action MUST reveal exactly the next step's content
  and MUST NOT perform any other kind of advancement (moving to a
  different node, or presenting a choice) in that same action.
- **FR-006**: Once every reveal step for the current node has been shown,
  the presenter's "advance" action MUST behave exactly as it would if the
  node had no reveal marks at all — following the node's destination,
  presenting a choice, or reporting the end of the path, as applicable.
- **FR-007**: If a node has both reveal-marked content and offers the
  presenter a choice between destinations, the choice MUST NOT be
  selectable — by any means the presenter would normally use to choose —
  until every reveal step on that node has been shown. Attempting to
  choose early MUST instead continue revealing.
- **FR-008**: If a node has reveal-marked content and no destination at
  all (a dead end), the presenter MUST still be able to step through every
  reveal via the ordinary "advance" action; only after every step is shown
  MUST the system report that the path has ended.
- **FR-009**: A node's reveal progress MUST reset to "nothing beyond the
  always-visible content shown" every time the presenter arrives at that
  node, regardless of how they arrived (moving forward, choosing a path,
  jumping directly, or going back) and regardless of whether that node was
  fully revealed on any earlier visit.
- **FR-010**: Whenever a node has reveal steps not yet fully shown, the
  presenter-facing status area MUST indicate that more content remains to
  reveal and MUST reflect how many of the node's reveal steps have been
  shown so far. Nodes that use no reveal marks at all MUST show nothing
  different from today.
- **FR-011**: A block whose reveal step has not yet been reached MUST NOT
  occupy any visual space in the rendered slide — it is treated as absent,
  not as present-but-invisible. This applies within side-by-side (column)
  arrangements: an unrevealed column MUST NOT reserve width for content
  that is not yet shown.
- **FR-012**: A deck-checking pass MUST warn the author when a block
  nested inside a group is marked to reveal at an earlier step than the
  group itself, since such a block can never actually appear before its
  enclosing group does. The warning MUST NOT be raised for a deck that
  contains no such pattern.
- **FR-013**: A deck created before this feature existed, or created
  without using any reveal marks, MUST present identically after this
  feature ships — no visual or behavioral change for decks that don't use
  it.
- **FR-014**: A deck that uses reveal marks MUST remain fully presentable
  (all content eventually visible, in the authored order) when opened in a
  presenting tool that does not understand reveal marks at all — such a
  tool MUST show all of that node's content immediately, which is
  considered a safe, acceptable degrade rather than an error.

### Key Entities

- **Reveal marker**: An optional value attached to a content block,
  identifying which step in a node's reveal sequence the block belongs to.
  Absence means "always visible." Distinct marker values used within one
  node, in ascending order, define that node's reveal steps.
- **Reveal progress**: Per-node, transient presenter state — how far
  through that node's reveal sequence the presenter currently is. Not
  saved, not carried between visits to the same node, not shared across
  nodes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A presenter can reveal a three-bullet list one bullet per
  keypress, using the same single key they already use to advance slides,
  with zero additional keys to learn.
- **SC-002**: Every keypress during a reveal sequence produces a visible
  change on screen — no keypress during an in-progress reveal ever appears
  to do nothing.
- **SC-003**: A deck authored entirely without reveal marks shows no
  difference in appearance or behavior compared to before this feature
  existed — verified by direct comparison, not just by intent.
- **SC-004**: An author who nests a reveal mark inside a later-revealing
  group is told about the mistake before presenting, without needing to
  present the deck to discover it.
- **SC-005**: A presenter can tell, at a glance at the footer, whether a
  slide has more content left to reveal and roughly how much.

## Assumptions

- Reveal applies only to content blocks within a node; it does not extend
  to branch-point options, node titles, or any other non-content-block
  part of the document.
- No fade, animation, or other visual transition accompanies a reveal step
  — content simply appears, consistent with how the rest of this project's
  transition model works (transitions are pacing hints, not animation
  requirements).
- Editing a slide's text/heading content (the existing quick-edit feature)
  operates on the full node regardless of current reveal progress; reveal
  state is a presenting-time concern, not an authoring-time restriction.
- Reveal progress is intentionally not part of a presenter's navigation
  history — going back to a previously fully-revealed node shows it from
  the start again. This is a deliberate simplification, not an oversight.
