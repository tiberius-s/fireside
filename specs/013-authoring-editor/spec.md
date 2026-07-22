# Feature Specification: Authoring Editor (`fireside edit`)

**Feature Branch**: `013-authoring-editor`

**Created**: 2026-07-21

**Status**: Draft

**Input**: User description: "WYSIWYG block editor (`fireside edit`) — a full-screen, mouse-first, block-based authoring studio for Fireside decks. Two commitments: (1) WYSIWYG by construction — the editing canvas reuses the presenter's exact rendering, so what you edit is exactly what your audience will see; (2) a block editor, not a text editor — a slide is a stack of discrete, clickable blocks (heading, text, code, list, picture, divider, text art, columns/box/stack) edited through small forms, never through raw JSON or graph/node vocabulary. Explicit bar: foolproof and easy to use for people who cannot edit JSON or think in graph structures. Design brief: `.claude/plans/2026-07-19-wysiwyg-editor-plan.md` (rev 3)."

## Clarifications

### Session 2026-07-21

- Q: Should the editing canvas render at the terminal's actual current
  size, or always at a fixed 80-column width? → A: Render at the real
  pane/window size (matching the presenter's own behavior), with an
  optional toggle to preview at a standard fixed size.
- Q: Can a block drag be started by pressing anywhere on the block, or
  only on a dedicated drag-handle icon? → A: Anywhere on the block; the
  handle icon is a visual cue only, not the only grab point.
- Q: What concrete scale/latency bar should "remains responsive" for
  large decks mean? → A: Decks up to 500 slides; every editor
  interaction (selection, navigation, undo, drag) completes within
  100ms.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Edit a slide's content without touching JSON (Priority: P1)

A presenter who has never seen the underlying file format opens an
existing deck in the editor, clicks on a piece of text or a heading on a
slide, changes the wording through a small on-screen form, and saves.
They never see a node id, a JSON key, or the word "graph." The editor
looks like the presentation they will give — clicking a block edits
exactly what the audience will see, in place.

**Why this priority**: This is the smallest slice that proves the whole
promise of the feature — content correction without technical knowledge —
and is useful in isolation even before any structural editing exists.

**Independent Test**: Open a deck with at least one text and one heading
block, select each in turn, change its wording via the block's form, save,
and confirm — by presenting the deck — that the audience-facing output
changed exactly as edited and nothing else did. Then undo the edit and
confirm the original wording is restored.

**Acceptance Scenarios**:

1. **Given** a valid deck open in the editor, **When** the user clicks a
   text block, **Then** the block shows a clear selected state and a
   small set of plain-language actions, including an edit action.
2. **Given** a block's edit form is open, **When** the user changes the
   text and confirms, **Then** the canvas immediately shows the updated
   wording rendered exactly as the presenter would show it.
3. **Given** unsaved changes exist, **When** the user saves, **Then** the
   deck file is updated and the unsaved-changes indicator clears.
4. **Given** a just-made edit, **When** the user undoes it, **Then** the
   block's previous wording is restored and is visibly, immediately
   correct on the canvas.
5. **Given** the same task, **When** performed using only the mouse vs.
   only the keyboard, **Then** both paths succeed without requiring the
   other input method.

---

### User Story 2 - Add, remove, and rearrange blocks on a slide (Priority: P2)

An author builds out a slide's content by adding new blocks (a bullet
list, a code sample, a divider, an image placeholder, and so on),
deleting ones they no longer want, and dragging blocks into a different
order — all by clicking and dragging on the canvas, choosing from a
plain-language list of block types.

**Why this priority**: Once content can be edited (P1), the next most
valuable step is composing new content, which is most of what building a
talk from scratch requires.

**Independent Test**: Starting from a slide with two blocks, add a new
block of a different kind, delete one of the original blocks, and drag
the remaining blocks into a new order — using only the mouse — then
confirm the slide presents in the new order with the right content, and
that every step could be undone back to the starting state.

**Acceptance Scenarios**:

1. **Given** a slide is open, **When** the user clicks an "add a block"
   control, **Then** a list of block kinds is shown in plain language
   (no technical kind names) and choosing one inserts it with placeholder
   content and opens its edit form immediately.
2. **Given** a block is selected, **When** the user chooses delete,
   **Then** the block is removed and a reversible confirmation ("Deleted
   — Undo") appears rather than a blocking dialog.
3. **Given** two or more blocks on a slide, **When** the user drags one
   to a new position, **Then** a clear indicator shows where it will land
   before release, and the slide reflects the new order after release.
4. **Given** a drag is in progress, **When** the user presses the cancel
   key, **Then** the block returns to its original position with no
   change made.

---

### User Story 3 - Restructure the deck: slides, branches, and reveal steps (Priority: P3)

An author manages the deck's structure — adding, removing, and reordering
slides; turning a slide into a branch point with multiple answers; setting
what a slide leads to next or marking it as an ending; and controlling how
many "reveal" steps a slide's content is split into — all through named,
clickable pickers rather than typed identifiers.

**Why this priority**: This is what turns a linear list of edited slides
(P1/P2) into a complete, presentable deck with branching paths, which is
part of the product but less immediately necessary than being able to
edit and compose content at all.

**Independent Test**: Starting from a 3-slide linear deck, turn one slide
into a branch point with two answers pointing at two different existing
slides, reorder the remaining linear slides by dragging, and set one
slide's content to reveal in two steps — using only named pickers and
drag — then confirm the deck presents the branch correctly and nothing
requires typing an identifier.

**Acceptance Scenarios**:

1. **Given** a slide, **When** the user opens its destination control,
   **Then** they choose the next slide by name from a list (with a
   preview), never by typing an id.
2. **Given** a slide, **When** the user turns it into a branch point,
   **Then** they can add named answers, each pointing at a slide chosen
   by name, and the slide presents the branch correctly afterward.
3. **Given** slides in a straight run, **When** the user drags one to
   reorder it, **Then** the run's order updates to match; **When** the
   user instead drags a slide across a branch's boundary, **Then** the
   action is refused with a plain-language explanation and a way to
   perform the intended change correctly.
4. **Given** a slide with multiple pieces of content, **When** the user
   sets reveal steps, **Then** the step numbers shown are always
   consecutive starting at one, regardless of the order steps were
   assigned.
5. **Given** any structural change, **When** the user previews the deck
   in-place, **Then** they can step through it as the audience would and
   return to editing exactly where they left off, without saving first.

---

### User Story 4 - Never lose work (Priority: P4)

An author's session ends unexpectedly (crash, accidental quit, power
loss) or a change turns out to be wrong. In every case, the author can
recover: an autosaved draft is offered back on next open, quitting with
unsaved changes always asks first, and any single action — however far
back — can be undone.

**Why this priority**: This is a safety net around P1–P3, valuable
throughout but not itself a reason to open the editor — it protects work
already described in the higher-priority stories.

**Independent Test**: Make several edits without saving, force-quit the
process, reopen the same deck, and confirm the editor offers to restore
the unsaved draft with a clear choice between the draft and the last
saved file; separately, make 20 edits in a row and confirm every one can
be undone back to the original, in order.

**Acceptance Scenarios**:

1. **Given** unsaved changes and a forced quit, **When** the deck is
   reopened, **Then** the editor offers a choice between the recovered
   draft and the last saved file, with when each was last touched shown
   in plain language.
2. **Given** unsaved changes, **When** the user tries to quit normally,
   **Then** they are asked to save, discard, or keep editing before
   anything is lost.
3. **Given** a long sequence of edits, **When** the user repeatedly
   undoes, **Then** each step reverses exactly one prior action, in
   order, for at least 100 prior actions.
4. **Given** the deck file was changed by something else (e.g. the
   presenter's quick-edit) since the editor opened it, **When** the
   author tries to save, **Then** the conflict is surfaced clearly rather
   than one edit silently overwriting the other.

---

### Edge Cases

- Opening a path that exists but fails to parse as a deck: the editor
  refuses to open and points the author at the same diagnostic report
  produced elsewhere in the product, plus one line naming the fix-first
  tool.
- Opening a path that exists and parses but has other outstanding issues
  (e.g. a slide nobody can reach): the editor opens normally, showing the
  issue in plain language, since fixing that kind of issue is part of
  what the editor is for.
- Opening a path that does not exist yet: the editor offers to create a
  new deck, reusing the same starting templates as deck creation
  elsewhere in the product.
- Running in an environment without an interactive terminal: the editor
  refuses to start with a clear message rather than failing unreadably.
- A terminal window smaller than the editor's minimum usable size: the
  editor shows a single clear message asking for a bigger window instead
  of drawing a broken or overlapping layout.
- A terminal that cannot report mouse hover position: hover-only cues are
  never the only way to discover or perform an action — everything
  remains reachable by clicking and by keyboard.
- Deleting a slide that other slides point to: the slides that pointed to
  it are updated to point past it, and the author is told this happened.
- A slide with no blocks at all: the canvas shows one clear, large
  "add your first block" target rather than empty space.
- Very small or very large decks (a single slide; a deck of up to 500
  slides with deep branching): outline, navigation, and undo remain
  responsive — every interaction (selection, navigation, undo, drag)
  completes within 100ms — and every slide stays reachable in the slide
  list.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a full-screen editing view, opened
  by naming an existing or new deck file, distinct from the presenting
  view.
- **FR-002**: The editing canvas MUST render each slide's content exactly
  as the presenting view would render it, with no separate or
  approximated rendering for authoring. The canvas MUST render at the
  window's actual current size (matching how the presenting view itself
  behaves), and MUST offer an optional toggle to preview the slide at a
  standard fixed size for authors who want to check the common case
  without resizing their window.
- **FR-003**: Every slide MUST be represented as an ordered stack of
  discrete, individually selectable blocks; there is no free-form text
  buffer or markup view.
- **FR-004**: Users MUST be able to select any block by clicking it, and
  a selected block MUST show a distinct visual state plus a small set of
  available actions.
- **FR-005**: Users MUST be able to edit a selected block's content
  through a small on-screen form specific to that block's kind, with
  explicit confirm and cancel actions, and never by typing raw markup or
  structural syntax.
- **FR-006**: The system MUST support at least these block kinds, each
  presented to the user only by a plain-language name: heading, text,
  code, list, picture, divider, text art, and a layout block for
  side-by-side / centered / stacked groups of other blocks.
- **FR-007**: Users MUST be able to add a new block to a slide at a
  chosen position by choosing its kind from a plain-language list; the
  new block MUST start with placeholder content and open for editing
  immediately.
- **FR-008**: Users MUST be able to delete a block, with the deletion
  reversible through undo and confirmed via a non-blocking, reversible
  notice rather than a blocking dialog.
- **FR-009**: Users MUST be able to reorder blocks on a slide by dragging,
  with a visible indicator of the drop position before release and a way
  to cancel the drag before it completes. A drag MUST be startable by
  pressing anywhere on the block, not only on a dedicated handle; a
  handle icon MAY still be shown as a visual cue that dragging is
  possible.
- **FR-010**: Users MUST be able to add, delete, duplicate, and reorder
  slides, and to rename a slide's title.
- **FR-011**: Users MUST be able to set what a slide leads to next by
  choosing another slide by name (with a preview), or by marking the
  slide as an ending; typed identifiers MUST NOT be required anywhere in
  the editor.
- **FR-012**: Users MUST be able to turn a slide into a branch point with
  multiple named answers, each targeting another slide chosen by name,
  and to turn a branch point back into a single-path slide.
- **FR-013**: Reordering slides within an unbranched run MUST update the
  deck's flow to match the new order; attempting to drag a slide across a
  branch's boundary MUST be refused with a plain-language explanation and
  a way to complete the intended change through the correct control.
- **FR-014**: Users MUST be able to control how many reveal steps a
  slide's content is split into; displayed step numbers MUST always be
  consecutive starting at one regardless of assignment order.
- **FR-015**: Users MUST be able to preview the deck as the audience will
  see it — including branch and reveal behavior — from within the editor
  without saving first, and return to editing at the same point afterward.
- **FR-016**: The system MUST support undo and redo of every edit and
  structural change, covering at least the 100 most recent actions, with
  a visible, clickable undo control in addition to a keyboard shortcut.
- **FR-017**: Every destructive action (delete, discard) MUST be
  reversible via undo and MUST be described in words, never by an icon
  alone.
- **FR-018**: The system MUST track whether there are unsaved changes,
  show this state at all times, and require an explicit save action to
  write those changes to the deck file.
- **FR-019**: On quitting with unsaved changes, the system MUST ask the
  user to save, discard, or continue editing before any work is lost.
- **FR-020**: The system MUST periodically preserve an in-progress,
  unsaved draft such that, after an abrupt termination, reopening the
  same deck offers a clear choice between recovering the draft and
  opening the last saved file, showing when each was last touched.
- **FR-021**: If the deck file on disk has changed since the editor
  opened it (e.g. edited elsewhere), the system MUST detect this at save
  time and surface the conflict rather than silently discarding either
  version.
- **FR-022**: Writes to the deck file MUST leave the file in a valid,
  fully-written state even if interrupted mid-write.
- **FR-023**: The editor MUST NOT allow the deck to be left, by any
  editor-produced change, in a state with a dangling destination, a
  duplicated slide identity, a slide that is simultaneously a branch
  point and a simple next-slide, or reveal steps with gaps; any other
  outstanding issue MUST be visible in a status area and MUST be
  clickable to locate.
- **FR-024**: The system MUST never display, in any editor screen, a
  node/graph identifier, a raw data-format key, or an internal block-kind
  name; all editor-facing vocabulary MUST be plain, presentation-oriented
  language (slides, blocks, answers, endings, "goes to").
- **FR-025**: Opening a path that exists but fails to parse as a deck
  MUST be refused with the same diagnostic report produced elsewhere in
  the product, plus guidance on how to see the full report.
- **FR-026**: Opening a path that exists, parses, but has other
  outstanding issues MUST succeed, with those issues shown in the status
  area rather than blocking entry.
- **FR-027**: Opening a path that does not yet exist MUST offer to create
  a new deck using the product's existing starting templates.
- **FR-028**: The system MUST refuse to start in a non-interactive
  environment (no attached terminal) with a clear, human-readable message.
- **FR-029**: Below the editor's minimum usable window size, the system
  MUST show a single clear message asking for a larger window instead of
  drawing a degraded or overlapping layout.
- **FR-030**: At rest (nothing selected, no form open), the editing
  screen MUST show no more than approximately seven interactive controls;
  additional, contextual controls MUST appear only once something
  relevant is selected and disappear when it is not.
- **FR-031**: Every action reachable by mouse (click, drag) MUST also be
  reachable by keyboard alone, and every action MUST remain fully usable
  using only keyboard input, without requiring the mouse.
- **FR-032**: Every user action MUST produce an immediate, visible
  response (selection highlighting, a drop indicator while dragging, a
  confirmation notice, etc.); no action may appear to do nothing.
- **FR-033**: The system MUST NOT introduce any hover-only affordance —
  every capability discoverable via hover MUST also be discoverable and
  usable via click, for environments that cannot report hover position.

### Key Entities

- **Deck**: The complete presentation being authored — its slides, their
  order and connections, and its title/metadata. What the editor opens,
  edits, and saves.
- **Slide**: A single screen of the deck; has a title, an ordered list of
  blocks, an optional set of reveal steps, and either a single "next"
  destination, a set of named branch answers, or neither (an ending).
- **Block**: One discrete, independently editable unit of a slide's
  content, of one of the supported kinds (heading, text, code, list,
  picture, divider, text art, or a layout grouping of further blocks).
- **Branch answer**: A named choice on a branch-point slide, pointing at
  another slide.
- **Draft**: An automatically preserved, unsaved snapshot of in-progress
  editing, distinct from the saved deck file, used only for crash
  recovery.
- **Edit history entry**: One undoable/redoable step in the current
  editing session.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A person with no prior exposure to the deck's underlying
  file format, and no particular comfort with terminal applications, can
  build a 5-slide deck containing one branch point and one multi-step
  reveal, present it, and save it — in under 10 minutes — using only the
  mouse (typing only inside text fields).
- **SC-002**: The same 10-minute task in SC-001 can also be completed
  using only the keyboard, with no step requiring the mouse.
- **SC-003**: At rest, the editing screen never shows more than
  approximately 7 interactive controls; every additional control that
  appears is tied to something the user has selected.
- **SC-004**: In a review of every editor screen, no node/graph
  identifier, raw data-format key, or internal block-kind name is ever
  shown to the user.
- **SC-005**: Any single edit, however many actions ago (up to at least
  100), can be undone, restoring the deck to exactly its prior state.
- **SC-006**: Terminating the editor process abruptly at any point loses
  at most a few seconds of unsaved work, and the deck file itself is
  never left unreadable.
- **SC-007**: No sequence of edits made through the editor can ever
  produce a deck with a dangling destination, a duplicated slide
  identity, a slide that is both a branch point and a plain next-slide,
  or reveal steps with a gap in their numbering.
- **SC-008**: For every slide in a representative set of decks, the
  editor's at-rest rendering of that slide is pixel-for-pixel (character-
  for-character) identical to how the presenting view renders the same
  slide at the same window size.
- **SC-009**: In a deck of up to 500 slides with deep branching, every
  editor interaction (selecting a block or slide, navigating, undoing,
  starting or completing a drag) completes within 100ms.

## Assumptions

- The editor operates on decks in the product's existing native file
  format and does not introduce a new file format or a second on-disk
  representation.
- The editor is a single-user, single-window, terminal-based application,
  consistent with the rest of the product; it does not add networked or
  multi-user collaboration.
- "Mouse support" assumes a terminal environment capable of reporting
  mouse press/drag/release events; hover-position reporting is treated as
  an enhancement some environments will lack (see Edge Cases and FR-033),
  not a requirement.
- Users have already installed and can launch the product; account
  creation, licensing, and distribution are out of scope for this
  feature.
- Copying text out of the editor via the terminal's own text-selection
  mechanism may require a modifier-key bypass in some terminals, which is
  an acceptable, documentable limitation rather than a defect.
- Reordering blocks or slides across independent decks, or dragging a
  block from one slide to a different slide, is out of scope for this
  feature; moving content between slides is accomplished by other means
  outside this feature's scope.
- Multiple people editing the same deck at the same time is out of scope;
  the conflict handling in FR-021 covers a single author encountering one
  stale save, not concurrent multi-author editing.
