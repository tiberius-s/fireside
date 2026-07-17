# Feature Specification: Modern TUI Leverage

**Feature Branch**: `007-modern-tui-leverage`

**Created**: 2026-07-17

**Status**: Draft

**Input**: User description: "Modern TUI leverage: add mouse support (click a map node to goto, click a branch option to choose; keyboard remains primary, footer still teaches keys), synchronized output (BeginSynchronizedUpdate/EndSynchronizedUpdate around frame draws to eliminate transition flicker, no-op when terminal doesn't support it), OSC 8 hyperlinks for link-bearing text blocks, and resume-from-fingerprint (persist last position per deck in a dotfile keyed by the deck's content fingerprint via main.rs::fingerprint, so a crashed or interrupted presentation reopens where it left off). This is P2 from .claude/plans/2026-07-12-strategic-improvement-plan.md, the last remaining item of that phase-1 strategic plan."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Click to navigate (Priority: P1)

A presenter viewing the deck map (the list of all slides) clicks directly on a slide's row instead of arrowing to it and pressing Enter. On a slide with a branch choice, the presenter clicks the desired option instead of pressing its number key. Keyboard navigation still works exactly as before — mouse is an additional way in, not a replacement.

**Why this priority**: This is the headline, most visible "modern TUI" capability and the one presenters will notice first; it's also the one the plan calls out as trivially cheap (one crossterm capture flag).

**Independent Test**: Can be fully tested by opening the map screen, clicking a non-current slide's row, and confirming the presentation jumps to it — and separately, at a branch-point slide, clicking an option and confirming the same choice a keyboard select would produce.

**Acceptance Scenarios**:

1. **Given** the map screen is open, **When** the presenter clicks a row for a slide other than the current one, **Then** the presenter jumps to that slide and returns to the normal presenting view, identical to selecting that row and pressing Enter.
2. **Given** the current slide is a branch point with reveal fully finished, **When** the presenter clicks one of the displayed options, **Then** that option is chosen exactly as if its key had been pressed.
3. **Given** any presenting screen, **When** the presenter uses only the keyboard and never touches the mouse, **Then** every existing keyboard behavior is unchanged.
4. **Given** the current slide still has unrevealed content, **When** the presenter clicks a branch option, **Then** the click has no special branch-selection effect (reveal is not yet resolvable by mouse — mirrors the existing keyboard rule that branch keys advance reveal instead of choosing early).

---

### User Story 2 - Resume after a crash or interruption (Priority: P2)

A presenter is running a deck when their terminal is killed, the machine sleeps, or they simply quit partway through. When they relaunch the same deck, they land back on the slide they were last viewing instead of at the beginning.

**Why this priority**: Losing your place mid-presentation is a real, painful failure mode with no current mitigation; the plan calls this "very presenter-first."

**Independent Test**: Can be fully tested by presenting a deck, navigating to a slide partway through, force-quitting the process, relaunching against the same deck file, and confirming the presenter resumes on the same slide.

**Acceptance Scenarios**:

1. **Given** a presenter has navigated to a slide other than the first, **When** the process ends without a clean "presentation finished" state (killed, crashed, or quit early), **Then** relaunching the presenter on the same deck file starts on that same slide.
2. **Given** a presenter reaches the deck's end normally, **When** they relaunch the same deck later, **Then** they start from the beginning again (a completed run does not leave a stale mid-deck resume point).
3. **Given** the deck file's content has changed since the last run (different fingerprint), **When** the presenter relaunches it, **Then** the presenter starts from the beginning rather than resuming a position that may no longer exist or make sense.
4. **Given** a saved resume position names a slide that no longer exists after a content change, **When** it would otherwise be used, **Then** the presenter falls back to the beginning rather than erroring.
5. **Given** a presenter wants to intentionally restart from the beginning, **When** they use the documented way to do so, **Then** the deck opens at the first slide regardless of any saved resume position.

---

### User Story 3 - Flicker-free transitions (Priority: P3)

A presenter moving between slides, especially with the fade transition or on a slower terminal emulator, never sees a half-drawn or torn frame — the screen appears to update atomically.

**Why this priority**: Purely invisible polish with no new interaction surface — safe, cheap, and universally beneficial, but lower presenter-facing value than navigation or resume.

**Independent Test**: Can be fully tested by driving rapid slide transitions in a terminal that supports atomic screen updates and confirming no partial-frame artifacts appear, and separately confirming presenting still works normally in a terminal that does not support the capability.

**Acceptance Scenarios**:

1. **Given** a terminal that supports atomic screen updates, **When** the presenter transitions between slides, **Then** no partially-drawn frame is ever visible.
2. **Given** a terminal that does not support atomic screen updates, **When** the presenter transitions between slides, **Then** presenting behaves exactly as it does today (no error, no degraded behavior beyond the pre-existing lack of the capability).

---

### User Story 4 - Clickable links in slide text (Priority: P4)

A slide's text contains a link (e.g. a reference URL). On a terminal that supports clickable links, the presenter can click (or cmd/ctrl-click, per terminal convention) the link text to open it in a browser, without the raw URL cluttering the visible slide.

**Why this priority**: Narrowest audience of the four — only decks that actually include links benefit — and it is the only one of the four that requires deciding new authoring syntax, not just runtime behavior.

**Independent Test**: Can be fully tested by authoring a deck with a link in a text block, presenting it in a terminal that supports clickable hyperlinks, and confirming the link text is clickable and opens the correct destination; and separately confirming a terminal without that support still shows the link's label as plain readable text.

**Acceptance Scenarios**:

1. **Given** a text block whose content includes a link in the deck's supported link syntax, **When** it is presented in a terminal that supports clickable terminal links, **Then** the link's label renders as a distinctly-styled, clickable region that opens the link's destination.
2. **Given** the same content, **When** it is presented in a terminal that does not support clickable terminal links, **Then** the link's label still renders as readable text (the presentation does not break or show raw escape codes).
3. **Given** a link's destination is not a well-formed URL, **When** the deck is validated, **Then** the author is warned, consistent with how other content mistakes are surfaced today.

---

### Edge Cases

- What happens when the mouse is clicked outside any interactive row/option (e.g. in the margin, on the footer, on a text block)? Nothing happens — no navigation, no error.
- What happens if a presenter clicks rapidly (double-click, or a click during a fade transition)? At most one navigation happens per completed click; a click during a transition is treated the same as a keypress would be at that moment.
- What happens to resume state for the built-in demo deck (no backing file)? No resume position is recorded or read, since resume is keyed to a deck file's content.
- What happens if two different decks happen to fingerprint to the same value? Content fingerprinting already carries this theoretical, negligible risk elsewhere in the system (write-back conflict detection); resume inherits the same accepted tolerance.
- What happens on the very first-ever launch of a given deck (no prior resume record)? Starts at the first slide, same as today.
- What happens when a link's label text is very long or the terminal window is narrow? It wraps/clips exactly like any other styled text today; only the clickable region styling is new.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST let a presenter navigate to any slide from the map screen by clicking its row, producing the same result as selecting that row via keyboard and confirming.
- **FR-002**: The system MUST let a presenter choose a branch option by clicking it, producing the same result as pressing that option's keyboard shortcut.
- **FR-003**: The system MUST leave all existing keyboard-driven navigation and choice behavior fully intact and unchanged when mouse support is added.
- **FR-004**: The system MUST continue to teach keyboard controls via the footer as the primary, always-visible contract; mouse support MUST NOT replace or hide that guidance.
- **FR-005**: The system MUST persist a presenter's current slide position per deck, keyed to the deck's content (not its file path), so the position record only applies while the content is unchanged.
- **FR-006**: The system MUST offer the presenter a way to relaunch a deck at its persisted position when one exists for that deck's current content.
- **FR-007**: The system MUST provide a documented way for a presenter to start from the beginning regardless of any saved position.
- **FR-008**: The system MUST NOT resume into a position that no longer exists in the current content, falling back to the beginning instead.
- **FR-009**: The system MUST NOT persist or use a resume position for a presentation that has no backing deck file.
- **FR-010**: The system MUST eliminate visibly torn or partially-drawn frames during slide transitions on terminals that support atomic screen updates.
- **FR-011**: The system MUST behave exactly as it does today, with no error or degradation, on terminals that do not support atomic screen updates.
- **FR-012**: The system MUST support an authoring syntax for a link (label plus destination) inside text-bearing content blocks.
- **FR-013**: The system MUST render a link's label as a distinctly-styled, clickable region on terminals that support clickable terminal links.
- **FR-014**: The system MUST render a link's label as plain readable text, with no visible escape codes or raw URLs, on terminals that do not support clickable terminal links.
- **FR-015**: The system MUST warn the author during validation if a link's destination is not a well-formed URL.
- **FR-016**: Every new mouse or link interaction MUST degrade gracefully (no crash, no hang) on a terminal or terminal emulator that does not support the underlying capability.

### Key Entities

- **Resume Record**: The last-viewed slide for a specific deck's content, identified by the deck's existing content fingerprint (the same fingerprint mechanism already used to detect on-disk changes), plus the identity of the slide itself.
- **Link**: A label and a destination URL, expressible inside a text-bearing content block, rendered as clickable when the terminal supports it and as plain text otherwise.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A presenter can navigate the entire deck — map jumps and branch choices — using only mouse clicks, with zero keyboard input, and reach the same slides they would via keyboard.
- **SC-002**: A presenter who force-quits mid-deck and relaunches the same deck lands on the same slide in 100% of trials where the content has not changed.
- **SC-003**: Zero visible partial-frame artifacts occur across 50 consecutive slide transitions on a terminal that supports atomic updates.
- **SC-004**: A deck authored with links presents identically well (readable, no broken output) across both a terminal that supports clickable links and one that does not.
- **SC-005**: None of the four capabilities regresses any existing keyboard-only presenting workflow, verified by the existing scenario/tmux-smoke test suite continuing to pass unmodified in its keyboard-only assertions.

## Assumptions

- Mouse clicks are additive on top of the existing keyboard contract; no existing key binding changes meaning or is removed.
- The map screen and branch-point menu are the only two places mouse clicks perform navigation in this phase — clicking elsewhere (body text, footer, help/edit screens) is inert, consistent with "mouse is additive" scoping in the source plan.
- The resume record is a small, host-local record separate from the deck file itself (the plan's own phrasing: "a dotfile keyed by content fingerprint"); it is not part of the portable deck format and is not synced or shared.
- A presentation that reaches its natural end is considered "completed" and does not leave a resume position pointing mid-deck; only an interrupted (non-terminal) session leaves one.
- Link authoring syntax reuses the deck's existing lightweight inline-markdown convention rather than inventing a new block kind, matching how other inline styling (e.g. list-item markdown) is already handled.
- "Terminal supports clickable terminal links" and "terminal supports atomic screen updates" are capabilities the presenting library can query or safely attempt-and-ignore; no new user-facing configuration is introduced to toggle them.
- This feature is additive to protocol/content model (a new optional link syntax) and additive to runtime behavior (mouse, sync output, resume) — it does not change any existing validated deck's meaning.
