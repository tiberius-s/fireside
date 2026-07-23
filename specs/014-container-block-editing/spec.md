# Feature Specification: Container Block Editing

**Feature Branch**: `014-container-block-editing`

**Created**: 2026-07-23

**Status**: Draft

**Input**: User description: "Container-block children (columns/box/stack) are unreachable in the fireside edit TUI editor. Tab/Shift+Tab selection and canvas click hit-testing only operate at depth 1 (top-level blocks within a slide) — a container's own children can never be individually selected, edited, reordered, or deleted. The container's edit form shows a read-only ChildSummary list but Down/Enter and clicking a child row do nothing. This affects 4 of 8 slides in the bundled demo deck (welcome, layout, extras, finale) and defeats the 'no JSON required' promise of the editor for any deck using columns/box/stack layout. Extend block selection and hit-testing to descend into container children, wire up canvas selection glow for a selected child, and make the container's child-edit-form actually open the child's own block-kind form. This is a completion of the existing US2 hit-testing/selection work from spec 013-authoring-editor, not a new user-facing feature area."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Select and edit a block inside a container (Priority: P1)

An author opens a slide built from a container (columns, box, or stack
layout) — for example the bundled demo's "Welcome" slide, which is a
single container holding a title, a tagline, a divider, and closing text.
They select one of those inner pieces of content directly, either by
clicking on its rendered text on the canvas or by cycling to it with
Tab/Shift+Tab, and open its own edit form (the same per-kind form they
already get for a top-level text/heading/etc. block) to change its
content.

**Why this priority**: Without this, every word of text inside a
container slide is permanently unreachable from the block editor — the
single most common reason an author would give up on `fireside edit` and
go back to hand-editing JSON. This is the load-bearing capability; nothing
else in this feature matters if a child still can't be selected and
edited.

**Independent Test**: Open a deck with a container slide, click directly
on a child's rendered text (or Tab to it), confirm it — not the whole
container — becomes the selected element with a visible selection
indicator, open its edit form, change its text, save, and confirm the
change appears both in the editor's canvas and in the presenter view of
the same deck.

**Acceptance Scenarios**:

1. **Given** a slide whose content is a container with several children,
   **When** the author clicks directly on one child's rendered text on the
   canvas, **Then** that child (not the container as a whole) becomes the
   selected element, shown with the same selection indicator used for
   top-level blocks.
2. **Given** a child block is selected inside a container, **When** the
   author presses Enter (or clicks the child's own "Edit" action),
   **Then** the editor opens that child's own block-kind edit form (the
   same form kind it would open for an equivalent top-level block), not
   the container's layout-only form.
3. **Given** a container's edit form is open showing its read-only summary
   list of children, **When** the author selects one of the listed
   children, **Then** the editor opens that specific child's own edit
   form.
4. **Given** a slide with a container that has no children (empty
   container), **When** the author selects it, **Then** Tab/Shift+Tab and
   click behave exactly as before this feature (select the container as a
   whole; there is nothing to descend into).
5. **Given** the author is editing a child's content and saves,
   **When** they reopen the deck later, **Then** the change persists in
   the same way top-level block edits already do.

---

### User Story 2 - Reorder and delete a container's children (Priority: P2)

Having been able to select and edit a container's children (User Story
1), an author now wants to restructure a container slide the same way
they already restructure the rest of a deck: move a child earlier or
later within its container, or remove one entirely, without leaving the
editor or touching JSON.

**Why this priority**: Builds directly on User Story 1 and delivers the
remaining half of the "no JSON required" promise for container slides.
Ranked below User Story 1 because a container slide with unreachable
content is a harder blocker than one whose content is editable but not
yet reorderable.

**Independent Test**: Open a deck with a container holding at least three
children, reorder one of them (drag or an equivalent move action), save,
and confirm the new order appears in both the editor and the presenter
view. Separately, delete one child and confirm it disappears from both
the editor and the presenter view while its siblings remain untouched.

**Acceptance Scenarios**:

1. **Given** a container with multiple children, **When** the author
   reorders one child relative to its siblings using the same
   drag-to-reorder gesture already used for top-level blocks, **Then** the
   container's children are stored in the new order and the presenter
   view reflects it.
2. **Given** a child is selected inside a container, **When** the author
   triggers its delete action, **Then** that child is removed from the
   container, its siblings are unaffected, and the container itself is
   left in place (with however many children remain, including zero).
3. **Given** a container has exactly one child, **When** the author
   deletes it, **Then** the container becomes an empty container (per
   User Story 1's Acceptance Scenario 4), not itself deleted as a
   side-effect.

---

### User Story 3 - Add a new block inside a container (Priority: P3)

An author building a container slide from scratch (for example, starting
a new columns layout for a side-by-side comparison) wants to add new
content blocks inside the container, in addition to editing, reordering,
and deleting ones that are already there.

**Why this priority**: Completes the set of block operations already
available at the top level, but ranked lowest because the two prior
stories already unblock the far more common case — editing an
already-authored container slide (such as every container slide in the
bundled demo deck) — whereas building a container from empty is rarer.

**Independent Test**: Open a container slide (or an empty container),
add a new block inside it using the same "add a block" affordance already
used at the top level, confirm the new block appears as a child of that
container (not as a top-level sibling of it), and confirm it renders
correctly in the presenter view.

**Acceptance Scenarios**:

1. **Given** a container is selected (empty or with existing children),
   **When** the author adds a new block "inside" it, **Then** the new
   block becomes a child of that container, positioned per the author's
   chosen insertion point among any existing children.
2. **Given** a newly added child inside a container, **When** the author
   selects and edits it, **Then** it behaves identically to any other
   container child from User Stories 1 and 2 (selectable, editable,
   reorderable, deletable).

---

### Edge Cases

- What happens when a container's child is itself another container
  (nested containers)? At minimum, the outer container's direct children
  must remain reachable per this feature; behavior for descending a
  second level down is out of scope and should not regress to the current
  "container children are all unreachable" state for the outer level.
- What happens when the terminal is too narrow/short to render a
  container's children individually? The editor should fall back
  gracefully (e.g., no crash, no silently-wrong selection) — same
  minimum-size guard behavior already used elsewhere in the editor.
- What happens when a child selection exists and the author undoes an
  action that removes that child (e.g., an earlier delete is undone
  elsewhere, or the container itself is deleted)? Selection must not point
  at a block that no longer exists; the editor should fall back to
  selecting the nearest valid target instead of erroring.
- What happens when the author double-clicks a child, or uses whatever
  gesture already opens a top-level block's form? It should open that
  child's form exactly as it would for an equivalent top-level block.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The editor MUST allow a container's direct children to be
  individually selected on the canvas, distinct from selecting the
  container as a whole.
- **FR-002**: The editor MUST allow Tab/Shift+Tab cycling to reach every
  direct child of a selected container, in addition to every top-level
  block, without skipping any of them.
- **FR-003**: The editor MUST show the same selection indicator for a
  selected container child that it already shows for a selected top-level
  block.
- **FR-004**: The editor MUST open a selected container child's own
  block-kind edit form (the same form it would show for an equivalent
  top-level block of that kind) when the author triggers edit on it,
  whether from the canvas or from the container's own child summary list.
- **FR-005**: The editor MUST allow a container's children to be
  reordered relative to their siblings, using the same interaction the
  editor already provides for reordering top-level blocks.
- **FR-006**: The editor MUST allow a container's child to be deleted
  independently of its siblings and independently of the container
  itself.
- **FR-007**: The editor MUST allow a new block to be added as a child of
  a container, at an author-chosen position among any existing children.
- **FR-008**: The editor MUST continue to support selecting a container as
  a whole (for its own layout picker) exactly as it does today, including
  when the container has no children.
- **FR-009**: All container-child operations (select, edit, reorder,
  delete, add) MUST integrate with the editor's existing save, undo/redo,
  and crash-recovery draft behavior identically to top-level block
  operations — a child edit is not a second-class change.
- **FR-010**: The documentation describing the editor's block-editing
  capabilities MUST be updated to describe container children as reachable
  once this feature ships (matching whatever this feature actually
  delivers), removing the previously-noted mismatch between the container
  form's own child-summary and its non-functional interaction.

### Key Entities

- **Container block**: An existing block kind (`columns`, `box`, or
  `stack` layout) that holds an ordered list of child content blocks.
  Already selectable and layout-editable as a whole; this feature extends
  reachability to its individual children.
- **Container child**: Any content block nested one level inside a
  container — the same block kinds available at the top level (text,
  heading, text-art, divider, etc.), distinguished only by having a
  container as its parent rather than the slide itself.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every child block on every container-based slide in the
  bundled demo deck (currently 4 of its 8 slides) can be individually
  selected and edited from the block editor, with zero remaining
  unreachable content.
- **SC-002**: An author can go from "container slide with content I want
  to change" to "change saved and visible in the presenter view" using
  only the block editor, with no JSON editing, in under 30 seconds for a
  single-line text change.
- **SC-003**: Reordering or deleting a container's child produces the
  identical on-disk and on-screen result as the equivalent operation on a
  top-level block, verified by a direct comparison of the two flows.
- **SC-004**: The mismatch between the container edit form's own
  documentation/comments and its actual behavior (previously: each side's
  comment named the other as the place this was implemented) no longer
  exists — the code and its own comments agree on where child editing
  happens.

## Assumptions

- This feature covers exactly one level of nesting (a container's direct
  children); children that are themselves containers are addressed only
  to the extent described in Edge Cases, not as a fully recursive
  multi-level editor.
- The existing `BlockPath`/`Target::Block` shapes from spec
  013-authoring-editor's hit-testing contract are reused as-is (they
  already generalize to nested paths); this feature is scoped as
  completing that existing design, not introducing a new selection model.
- No new block kinds, no new container layouts, and no changes to the
  underlying deck file format are introduced — this is purely about
  making already-representable container children reachable through the
  editor UI.
- The bundled demo deck's four container slides (`welcome`, `layout`,
  `extras`, `finale`) serve as the reference fixtures for manual and
  automated verification, since they already exercise this gap in
  practice.
