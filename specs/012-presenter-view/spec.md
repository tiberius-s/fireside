# Feature Specification: Dual-Screen Presenter View

**Feature Branch**: `012-presenter-view`

**Created**: 2026-07-20

**Status**: Draft

**Input**: User description: "Dual-screen presenter view — speaker notes on the laptop, deck fullscreen on the extended display, without the audience ever seeing speaker notes. Promoted from the 2026-07-19 UX audit's addendum A-2 by explicit user decision; scoped in that plan's 'Wave 4' section as spec candidate 012-presenter-view."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Follow my own notes while the deck is on the projector (Priority: P1)

A presenter has their laptop connected to a projector or a second display. They
drag the presentation window to the external display and make it fullscreen.
On their laptop screen, in a second terminal window, they run a command that
shows them the current slide's title, its speaker notes, what's coming next,
and how far into a reveal sequence they are — all without that information
ever appearing on the projected screen.

**Why this priority**: This is the entire point of the feature. Every other
story only matters if this core loop works: present on one screen, read notes
on the other, and know the two are in sync.

**Independent Test**: Open a deck that has speaker notes on some slides in one
terminal (the presenter), and run the companion notes command pointed at the
same deck file in a second terminal. Advance and reveal in the presenter;
confirm the notes window updates to match within about half a second, without
ever appearing in the presenter's own window.

**Acceptance Scenarios**:

1. **Given** a presenter window showing slide 1 of a deck with notes attached
   to every slide, **When** the presenter advances to slide 2, **Then** the
   notes window updates to show slide 2's title and notes within about half a
   second, and the presenter's own on-screen output never displays notes text.
2. **Given** a slide with a multi-step reveal sequence, **When** the presenter
   reveals the next step, **Then** the notes window's reveal-progress
   indicator (e.g. "3/5 revealed") updates to match.
3. **Given** the presenter is at a branch point with multiple choice options,
   **When** the notes window is showing that same moment, **Then** it shows
   the available choice options instead of a single "next slide" title.

---

### User Story 2 - Know immediately if I've lost the connection to my presentation (Priority: P2)

A presenter is mid-talk when something goes wrong — they close the
presenter window by accident, it crashes, or they haven't started it yet.
The notes window they're relying on should never show stale information as if
it were current; it should clearly say the presenter isn't running.

**Why this priority**: A notes display that silently freezes on the last
known slide is worse than no notes display — the presenter would trust
information that's gone stale mid-talk, which is a worse failure mode than a
plainly-stated disconnect.

**Independent Test**: Start the presenter and the notes window together,
confirm they're in sync, then forcibly stop the presenter process. Confirm the
notes window switches to a clear "not running" state within about two seconds,
without needing to be restarted itself.

**Acceptance Scenarios**:

1. **Given** a notes window in sync with a running presenter, **When** the
   presenter process is killed or exits abnormally, **Then** the notes window
   shows a plain message that the presenter isn't running, with a hint on how
   to start it, within about two seconds.
2. **Given** a notes window is started before any presenter for that deck has
   started, **When** it first opens, **Then** it shows the same
   "not running yet" state rather than an error.
3. **Given** the presenter exits normally (the presenter quits the deck via
   its own quit key), **When** the notes window is watching, **Then** it also
   settles into the "not running" state rather than showing a crash or error.

---

### User Story 3 - Trust the notes even if I edit the deck mid-talk (Priority: P3)

A presenter is using the live quick-edit feature to tweak a slide's wording
during a rehearsal, or the deck file changes on disk for some other reason
while both windows are open. The notes window should pick up the edited
content rather than showing stale text, and should never crash or show a
raw error if the two windows are briefly looking at different versions of the
deck during a reload.

**Why this priority**: This is a real but secondary scenario — most talks
aren't edited live, but rehearsal and the demo's own quick-edit feature make
it common enough that showing wrong notes (or crashing) would be embarrassing
in front of an audience, even though it's not the core loop.

**Independent Test**: With both windows open and in sync, edit the currently
displayed slide's notes text via quick-edit in the presenter and save.
Confirm the notes window picks up the new text. Then, in the moment right
after a save when the two windows may briefly disagree about which slide
exists, confirm the notes window never shows a crash or raw error — at worst
it shows a brief "waiting for presenter" state.

**Acceptance Scenarios**:

1. **Given** a deck file that changes on disk (e.g. via quick-edit save),
   **When** the notes window is watching that file, **Then** it reloads its
   own copy and reflects the updated notes text without needing to be
   restarted.
2. **Given** a moment where the presenter has moved to a slide that doesn't
   yet exist in the notes window's stale copy of the deck (a brief reload
   race), **When** the notes window tries to resolve that slide, **Then** it
   shows a benign "waiting for presenter…" state rather than an error or
   crash.

---

### Edge Cases

- What happens when the notes window is pointed at a deck file that doesn't
  have a matching presenter running for it, versus one where a *different*
  deck's presenter is running? The notes window keys its lookup to the exact
  deck file it was given, so it only ever reflects a presenter running
  against that same file — a presenter running a different deck is
  invisible to it and it stays in the "not running" state.
- What happens when the current slide has no speaker notes at all? The notes
  window shows the slide title and next-slide/branch information as normal,
  with a plain indication that this slide has no notes (not a blank or
  broken-looking panel).
- What happens if the notes window is launched without an interactive
  terminal (piped, scripted, non-tty)? It fails the same way the presenter
  itself does for the same condition — a plain one-line message, no raw
  crash output.
- What happens at the very last slide, where there is no "next slide"? The
  notes window shows that this is the final slide instead of a broken or
  empty "next" field.
- What happens if two notes windows are opened for the same deck at once?
  Both are read-only followers of the same state; both simply show the same
  information. Not a scenario the feature needs to prevent.
- What happens if the presenter's clock and the notes window's clock disagree
  about "how long has this talk been running"? The elapsed timer is
  presenter-owned and read by the notes window as-is; the notes window does
  not run its own independent clock that could drift from it.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a way to view the current slide's title,
  its speaker notes, the next slide's title (or the available choices if the
  presenter is at a branch point), reveal progress, and elapsed presentation
  time, in a window separate from the one the audience sees.
- **FR-002**: The audience-facing presenter window MUST NOT render speaker
  notes as part of this feature (the existing single-screen notes panel is a
  separate, already-shipped affordance and is unaffected).
- **FR-003**: The notes window MUST reflect the presenter's navigation
  (advancing, going back, jumping via the map, branch choices) and reveal
  steps, updating within about half a second of the change happening in the
  presenter.
- **FR-004**: The notes window MUST detect when its associated presenter is
  not running — whether it never started, exited cleanly, or stopped
  unexpectedly — and switch to a plainly worded "not running" state within
  about two seconds of that becoming true.
- **FR-005**: The notes window MUST NOT write to, modify, or otherwise affect
  the deck file, the presenter's own state, or any other presentation
  artifact — it is read-only with respect to the presentation.
- **FR-006**: The notes window MUST pick up content changes made to the deck
  file while it's running (e.g. a live edit) without requiring the user to
  restart the notes window.
- **FR-007**: If the notes window cannot resolve the presenter's reported
  current position against its own copy of the deck (e.g. because of a brief
  reload timing mismatch), it MUST show a benign waiting/pending state rather
  than an error, crash, or stale-but-unmarked information.
- **FR-008**: The system MUST allow a presenter to launch directly into a
  fullscreen presentation view (today's fullscreen toggle, available as a
  launch-time option) so the deck can be handed straight to a projector
  without a manual key-press step.
- **FR-009**: The notes window MUST be launchable without any interactive
  configuration beyond pointing it at the same deck file the presenter is
  using, and MUST require no code or protocol change to the deck file format
  itself.
- **FR-010**: Launching the notes window without an interactive terminal
  (e.g. piped input/output) MUST fail with the same style of plain,
  one-line guidance the presenter itself already gives in that situation,
  not a raw crash.
- **FR-011**: The notes window MUST show its own key-based controls (at
  minimum, how to quit) discoverably, consistent with how the presenter
  teaches its own controls.
- **FR-012**: When the current slide has no speaker notes, the notes window
  MUST say so plainly rather than leaving a panel blank or looking broken.
- **FR-013**: On the final slide of a presentation path, the notes window
  MUST indicate there is no next slide rather than showing an empty or
  broken "next" field.

### Key Entities

- **Presentation session**: The live state of one running presenter process
  for one deck — which slide it's on, how far into a reveal sequence, how
  long it's been running, and whether it's still alive. Exists only while a
  presenter is running; disappears (or is understood to be gone) when the
  presenter exits, whether cleanly or not.
- **Notes window**: A separate, read-only view of a presentation session,
  associated with exactly one deck file, showing the presenter's current
  position translated into notes-relevant information (current notes, next
  title/choices, reveal progress, elapsed time, liveness).
- **Deck**: The existing presentation document (unchanged by this feature) —
  the notes window reads the same file the presenter is showing, including
  its speaker notes content, titles, branch structure, and reveal steps.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A presenter can navigate and reveal content in their
  presentation and see the corresponding notes update in the separate notes
  window within half a second, in a normal live demo.
- **SC-002**: If the presenter process stops for any reason, the notes window
  reflects "not running" within two seconds, every time, with no manual
  action required from the user.
- **SC-003**: In a live audience-facing setup, 100% of speaker-notes content
  is confined to the notes window — none of it appears in the window an
  audience would see, across every slide type (plain, branch, reveal, final
  slide, no-notes slide).
- **SC-004**: A presenter who has never used the feature before can start
  both windows and understand what they're looking at without external
  documentation, based solely on what each window shows and teaches about its
  own controls.
- **SC-005**: A live edit to a slide's notes text is visible in the notes
  window without restarting it, in a normal rehearsal workflow.

## Assumptions

- The presenter and the notes window run on the same machine, reading the
  same local deck file from the same local filesystem — this feature is not
  a network or multi-machine remote-control feature.
- Exactly one presenter process is the "source of truth" for a given deck's
  session at a time; the feature does not need to arbitrate between two
  simultaneous presenters of the same deck file (an unusual, unsupported
  scenario, not one this feature needs to detect or block).
- The notes window can have any number of simultaneous read-only viewers of
  the same session without conflict, since none of them write anything.
- Users of this feature are the same population as the existing presenter
  audience (comfortable running a second terminal command); no new
  onboarding path beyond documenting the two-window workflow is in scope.
- "Speaker notes" refers to the existing `speaker-notes` content already
  supported by decks today — this feature changes how/where notes are
  displayed, not what notes are or how they're authored.
- This feature is about *where* notes appear, not about adding any new
  authoring capability, new deck content type, or change to the presentation
  file format itself.
