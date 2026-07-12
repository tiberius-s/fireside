# Feature Specification: Quick-Edit Modal for Text and Heading Blocks

**Feature Branch**: `002-quick-edit-modal`

**Created**: 2026-07-12

**Status**: Draft

**Input**: User description: "Quick-edit modal for the current node's
text/heading blocks in fireside-tui, per ADR-005
(.claude/adrs/adr-005-quick-edit-modal-scope.md). Scope: while presenting a
deck, the presenter can open a modal that shows the current node's editable
text/heading block content, edit it in a text area, and save. On save, the
TUI does NOT write the file itself — it emits the edited content back
through a write-back callback owned by fireside-cli, symmetric to the
existing ReloadSource callback used for live-reload. fireside-cli serializes
the whole Graph via serde_json and writes it to disk; the existing file
watcher then picks up the change and reloads normally. Canonical
reformatting of the whole file on save is accepted/expected behavior.
Explicitly out of scope: adding/removing/reordering nodes, editing
traversal/branch-point/branch-options, undo/redo, multi-node batch edits,
and editing image/code/divider/list/columns blocks."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Fix a typo without leaving the presenter (Priority: P1)

A presenter is rehearsing or live-editing a deck. They notice a typo or an
awkward sentence in a heading or paragraph on the slide currently on screen.
Instead of switching to a text editor, finding the right node by id, editing
the JSON, and switching back, they open a quick-edit modal directly from the
presenter, change the text, save, and see the slide update in place.

**Why this priority**: This is the entire feature — the single most common
authoring friction (a small wording fix) currently requires leaving the tool
entirely. Everything else is refinement of this one loop.

**Independent Test**: Present a deck containing a node with a heading and a
text block. Open the quick-edit modal on that node, change the heading text,
save, and confirm the on-screen slide reflects the new text and the
underlying deck file on disk contains the change.

**Acceptance Scenarios**:

1. **Given** a presenter is viewing a node with at least one text or heading
   block, **When** they open the quick-edit modal, **Then** the modal shows
   the current content of each editable block on that node, clearly
   associated with its block.
2. **Given** the quick-edit modal is open with a block's text changed,
   **When** the presenter saves, **Then** the modal closes, the on-screen
   slide shows the updated text, and the deck file on disk contains the
   updated text.
3. **Given** the quick-edit modal is open, **When** the presenter cancels
   instead of saving, **Then** the modal closes and neither the on-screen
   slide nor the deck file on disk change.

---

### User Story 2 - Trust that only the intended text changed (Priority: P2)

A presenter saves a quick edit to one node. They want confidence that the
rest of their deck — every other node, every branch, every other block —
is untouched by the save, even though the whole file is rewritten on disk.

**Why this priority**: Without this guarantee, a presenter cannot trust the
feature for anything beyond a toy deck; one bad save touching unrelated
content would be worse than the status quo of manual editing.

**Independent Test**: Present a multi-node deck with branches, save a quick
edit to one text block on one node, and confirm every other node's content,
every traversal/branch-point, and every non-text/heading block are
byte-for-byte unchanged in meaning (formatting/key-order may differ).

**Acceptance Scenarios**:

1. **Given** a deck with multiple nodes and branches, **When** the presenter
   edits and saves a single text block on one node, **Then** every other
   node's content and every node's traversal/branch structure are
   semantically unchanged after the save.
2. **Given** a node containing a mix of block kinds (e.g. heading, text,
   code, image), **When** the presenter saves a quick edit to the heading,
   **Then** the code and image blocks on that same node are unchanged.

---

### User Story 3 - Know when there's nothing to quick-edit (Priority: P3)

A presenter is on a node made entirely of a code block and an image (no text
or heading content). They try to open the quick-edit modal out of habit.

**Why this priority**: A minor polish item — the feature should not
confuse a presenter into thinking editing is possible where it isn't, but
this doesn't block the core loop for nodes that do have editable content.

**Independent Test**: Present a node with only non-editable block kinds
(code, image, divider, list, or a container of those), attempt to open the
quick-edit modal, and confirm the presenter gets clear feedback that there
is nothing on this node to quick-edit, rather than a blank or broken modal.

**Acceptance Scenarios**:

1. **Given** the current node has no text or heading blocks, **When** the
   presenter tries to open the quick-edit modal, **Then** the presenter
   sees a clear message that this node has no editable content, and no
   empty/blank modal appears.

### Edge Cases

- What happens if the presenter opens the modal, edits text, and the
  underlying deck file changes on disk (e.g. someone else, or the
  presenter in another window, edits it) before they save? The save MUST
  NOT silently discard the concurrent on-disk change; the presenter should
  be warned and given the choice to overwrite or cancel, rather than data
  loss happening invisibly.
- What happens when a presenter tries to save an empty string into a
  required text field? The system MUST accept it (an empty heading/text
  body is valid content per the protocol) — it MUST NOT invent a
  non-empty-content validation rule this feature doesn't own.
- What happens when the node has a `container` block whose children include
  text/heading blocks nested inside a stack/columns layout? Nested
  text/heading blocks inside containers ARE editable through the modal
  (the modal edits by block, not by node-top-level-only); the container
  layout itself is not edited.
- What happens if the save produces a file that is no longer valid per
  deck validation rules (e.g. some other constraint)? This cannot happen
  from a content-only text edit under this feature's scope, since editing a
  string field cannot violate any current validation rule (uniqueness,
  traversal, reachability); no special handling is required beyond what
  already exists.
- What happens if the write to disk fails (e.g. permissions, disk full)?
  The presenter MUST see a clear error and their in-progress edit MUST NOT
  be silently lost — they should be able to retry the save or copy their
  edited text out.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The presenter MUST be able to open a quick-edit modal while
  presenting, scoped to the currently displayed node.
- **FR-002**: The modal MUST show every heading and text block on the
  current node (including those nested inside container layouts), each
  editable independently, pre-filled with its current content.
- **FR-003**: The presenter MUST be able to edit a block's text content in
  the modal using free-form multi-line text entry.
- **FR-004**: The presenter MUST be able to save their edits, applying all
  changed blocks on the current node at once.
- **FR-005**: The presenter MUST be able to cancel out of the modal without
  any change being applied to the on-screen presentation or the deck file.
- **FR-006**: On save, the system MUST persist the change to the deck's
  source file such that reopening or reloading the deck shows the edited
  content.
- **FR-007**: On save, the system MUST leave every other node, every
  block that is not a directly- or nested-edited text/heading block, and
  every traversal/branch structure semantically unchanged.
- **FR-008**: After a successful save, the presenter MUST see the updated
  content reflected on screen without manually reloading or restarting the
  presentation.
- **FR-009**: The system MUST NOT allow the quick-edit modal to add,
  remove, or reorder nodes or blocks, or to change traversal, branch-point,
  or branch-option structure. These remain edit-file-by-hand operations
  outside this feature.
- **FR-010**: The system MUST NOT provide undo/redo for quick-edit changes;
  correcting a mistaken save is done by editing again or by external
  version control, not by an in-app history.
- **FR-011**: When the current node has no text or heading blocks (directly
  or nested in a container), the system MUST tell the presenter there is
  nothing to quick-edit rather than opening an empty or non-functional
  modal.
- **FR-012**: Code, image, divider, list, and container-layout blocks MUST
  remain read-only through this modal; the feature MUST NOT expose an edit
  affordance for them.
- **FR-013**: If the deck's source file changes on disk between when the
  modal was opened and when the presenter saves, the system MUST detect the
  conflict and let the presenter choose to overwrite or abandon their edit,
  rather than silently discarding either version.
- **FR-014**: If persisting the save fails for any reason, the system MUST
  report the failure to the presenter and MUST NOT discard their edited
  text, so they can retry.

### Key Entities

- **Current node**: the node the presenter is viewing when they open the
  modal; the only node the quick-edit modal ever touches.
- **Editable block**: a heading or text content block on the current node
  (including nested inside a container's children), identified so an edit
  can be written back to the exact same block.
- **Quick-edit session**: the in-progress, unsaved edits held in the modal
  between opening and saving/cancelling; discarded entirely on cancel.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A presenter can correct a typo in a heading or paragraph and
  see it live on screen in under 15 seconds, without leaving the
  presentation or opening a separate editor.
- **SC-002**: After any quick-edit save, 100% of the deck's other nodes and
  non-text/heading blocks are unchanged in meaning, verified across decks
  with branches and nested containers.
- **SC-003**: A presenter attempting to quick-edit a node with no editable
  content gets clear feedback every time — never a blank or unresponsive
  modal.
- **SC-004**: No quick-edit save ever silently loses a concurrent external
  edit to the same file, or silently loses the presenter's own in-progress
  edit on a save failure.

## Assumptions

- "Text area" editing means free-form multi-line plain text input for a
  block's string content; no rich-text or Markdown-preview affordance is
  required beyond what the presenter already understands from how that
  text renders on the slide.
- Saving reformats the entire deck file to the tool's canonical output
  formatting (key order, whitespace); this is accepted, documented behavior
  per ADR-005, not a defect.
- This feature does not touch the protocol/wire format — no new JSON
  fields, no schema change — since it only edits existing string values
  already defined by the protocol.
- Concurrent-edit detection (FR-013) compares the file's state at open time
  against its state at save time; the exact mechanism (e.g. modification
  time/size fingerprint) reuses whatever the existing live-reload watcher
  already uses to detect changes, per ADR-005's write-back design.
- This feature is scoped entirely by ADR-005
  (`.claude/adrs/adr-005-quick-edit-modal-scope.md`); any request that
  falls outside that ADR's accepted scope is out of scope for this spec
  and requires a new ADR, not an amendment here.
