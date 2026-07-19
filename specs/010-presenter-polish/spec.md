# Feature Specification: Presenter Polish

**Feature Branch**: `010-presenter-polish`

**Created**: 2026-07-18

**Status**: Draft

**Input**: User description: "Presenter polish: four small UX feedback fixes for the presenter/authoring flow, from .claude/plans/2026-07-18-ux-polish-plan.md Wave 2 (spec 010). (a) Resume toast: on launch that resumes mid-deck, flash "Resumed where you left off — --restart starts over". (b) Exit summary: on quitting the TUI with q, print one line after the TUI closes, e.g. "Presented 5/7 slides in 12:30." using seen-slide count and elapsed timer the app already tracks. (c) Reserved-key validator warning: new Layer-2 validation rule `reserved-branch-key` (warning severity) that flags a branch option `key` colliding with the reserved global presenter keys (e f g h j k m n p q s t), since such a branch option can never fire; document the rule in the validation spec docs. (d) `art text` width guard: when generated banner width exceeds 76 columns (the existing ascii-art-too-wide threshold), print a note to stderr with the measured width, matching the existing skip-note behavior in `new --banner`; stdout art output stays unchanged. (e) Wizard momentum: interactive `fireside new` ends by asking "Present it now? [Y/n]" and execs present on yes."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Catch a dead branch key before it ships (Priority: P1)

An author writes a deck with a branch point and, without realizing it, assigns
a branch option the same single-letter key that the presenter already uses
globally (for example `q` to quit, or `e` to open quick-edit). Today nothing
tells them; the mistake only surfaces live, in front of an audience, when
pressing the advertised key does the wrong thing (this exact bug shipped in
the built-in demo deck and had to be hotfixed). `fireside validate` should
catch it during authoring instead.

**Why this priority**: This is the highest-leverage fix — it prevents the
same class of bug from recurring, and it's a pure authoring-time check with
no runtime behavior change, so it's low-risk to ship first.

**Independent Test**: Author a deck with a branch option whose key is one of
the reserved presenter keys, run `fireside validate` against it, and confirm
a warning-level diagnostic names the colliding key and the offending node.
Can be verified without touching the TUI at all.

**Acceptance Scenarios**:

1. **Given** a branch point with an option keyed `e`, **When** the deck is
   validated, **Then** a warning diagnostic reports that `e` is a reserved
   presenter key and the branch option can never fire.
2. **Given** a branch point where every option key is unreserved (e.g. `1`,
   `y`, `x`), **When** the deck is validated, **Then** no `reserved-branch-key`
   diagnostic is produced.
3. **Given** a branch point with an option keyed `e`, **When** the deck is
   presented anyway (warnings don't block presenting), **Then** pressing `e`
   opens quick-edit as before, not the branch option — matching the warning's
   claim.

---

### User Story 2 - See how the rehearsal went (Priority: P2)

A presenter runs through their deck to rehearse, quits with `q`, and wants
quick feedback on coverage and pacing without digging through notes: how much
of the deck they actually saw, and how long it took.

**Why this priority**: Free, always-on feedback using data the app already
tracks (visited-node count, elapsed time) — pure win, no new state needed,
independent of every other story here.

**Independent Test**: Launch a known deck, view a subset of its slides, press
`q`, and confirm the terminal shows one summary line with the correct
seen/total slide count and elapsed time after the TUI has fully closed.

**Acceptance Scenarios**:

1. **Given** a 7-slide deck where the presenter has viewed 5 distinct slides
   over 12 minutes 30 seconds, **When** they press `q`, **Then** after the
   TUI closes the terminal shows a line reading `Presented 5/7 slides in
   12:30.`
2. **Given** the presenter quits at the very first slide without advancing,
   **When** the TUI closes, **Then** the summary still prints, reporting
   1/N slides seen.
3. **Given** the presenter exits via `Ctrl+C` or a fatal error instead of
   `q`, **When** the process ends, **Then** no summary line is required (this
   story only covers the graceful `q` quit path).

---

### User Story 3 - Know at a glance that a session resumed (Priority: P3)

A presenter re-launches a deck they quit mid-way through last time. The app
silently resumes at the saved position — which is convenient, but also
disorienting if they'd forgotten they were mid-deck, since there's no visual
difference between "resumed" and "the deck just starts here."

**Why this priority**: A one-line, already-existing-mechanism (flash message)
fix for a real but lower-frequency confusion than the exit summary or the
authoring-time bug class in P1.

**Independent Test**: Present a deck, quit partway through (leaving a resume
record), relaunch the same deck, and confirm a flash message announces the
resume and how to opt out, without needing to inspect any other feature.

**Acceptance Scenarios**:

1. **Given** a deck was quit partway through on a prior run (a resume record
   exists), **When** it's launched again without `--restart`, **Then** a
   flash message reads `Resumed where you left off — --restart starts over`.
2. **Given** a deck has no resume record (first-ever launch, or it was
   already completed), **When** it's launched, **Then** no resume flash
   appears.
3. **Given** a resume record exists, **When** the deck is launched with
   `--restart`, **Then** no resume flash appears (the session starts fresh,
   consistent with `--restart`'s existing behavior).

---

### User Story 4 - Get from idea to rehearsal in one flow (Priority: P4)

Someone runs the interactive `fireside new` wizard to scaffold a deck. Today
they still have to notice the output path and type a second command to see
it. Ending the wizard with an offer to present immediately keeps momentum for
first-time and casual users.

**Why this priority**: Nice-to-have friction reduction for the interactive
wizard path specifically (not the non-interactive `fireside new <name>`
path); doesn't affect any existing content or validation behavior.

**Independent Test**: Run the interactive wizard to completion, answer yes at
the final prompt, and confirm the presenter launches on the just-created deck
without a separate command.

**Acceptance Scenarios**:

1. **Given** the interactive wizard has just created a deck, **When** it
   prompts `Present it now? [Y/n]` and the user presses Enter (or types `y`),
   **Then** the presenter launches immediately on that deck.
2. **Given** the same prompt, **When** the user answers `n`, **Then** the
   wizard exits normally without launching the presenter, as it does today.
3. **Given** the non-interactive form (`fireside new <name>`), **When** it
   completes, **Then** no present prompt appears — this story only changes
   the interactive wizard.

---

### User Story 5 - Know an ASCII banner won't fit before pasting it (Priority: P5)

Someone runs the standalone `fireside art text` command to generate a banner
for hand-pasting into a deck. If the rendered banner is wider than decks are
allowed to be, they'd only find out later from `fireside validate`. A note at
generation time closes that gap, matching the equivalent warning `new
--banner` already gives.

**Why this priority**: Smallest, most self-contained story — a stderr note
alongside output that's otherwise unchanged. Lowest independent user impact
of the five.

**Independent Test**: Run `fireside art text` with a phrase long enough to
exceed the width threshold and confirm a note appears on stderr while stdout
still contains the full, untruncated art.

**Acceptance Scenarios**:

1. **Given** a phrase that renders wider than 76 columns, **When** `fireside
   art text <phrase>` runs, **Then** stderr shows a note naming the measured
   width, and stdout still contains the complete banner unchanged.
2. **Given** a phrase that renders at or under 76 columns, **When** the
   command runs, **Then** no stderr note appears.

---

### Edge Cases

- A branch option has no `key` at all (label-only selection) — not subject to
  `reserved-branch-key` (there's no key to collide).
- A resumed session's saved node no longer exists in the current deck (deck
  edited since last run) — this story doesn't change that existing fallback
  behavior; the resume toast only fires when a resume position was actually
  applied.
- The presenter quits with `q` having seen only the starting slide and no
  others — exit summary still reports correctly (1/N).
- A deck has a branch point where multiple options collide with reserved keys
  — each colliding option gets its own diagnostic, not just the first.
- The wizard's present-now prompt is declined, or stdin is non-interactive
  (piped input) — falls back to the current no-prompt exit rather than
  hanging on a read.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The validator MUST emit a warning-severity diagnostic when a
  branch option's `key` matches one of the reserved global presenter keys
  (`e`, `f`, `g`, `h`, `j`, `k`, `m`, `n`, `p`, `q`, `s`, `t`), naming the
  colliding key, the owning node, and the option's label.
- **FR-002**: The reserved-key warning MUST NOT block presenting — decks with
  this warning validate as presentable, consistent with warning-severity
  diagnostics elsewhere in the system.
- **FR-003**: The reserved-key rule's meaning and the reserved key list MUST
  be documented alongside the other validation rules in the project's
  published validation reference.
- **FR-004**: On a graceful quit (`q`) from the presenter, the system MUST
  print exactly one summary line after the terminal UI has closed, stating
  how many distinct slides were seen out of the deck's total, and elapsed
  time formatted as minutes:seconds.
- **FR-005**: The exit summary MUST NOT print on non-graceful exits (process
  termination, fatal error) — only on the `q` quit path.
- **FR-006**: On launching a presentation that resumes from a previously
  saved position, the system MUST show a one-time flash message stating that
  the session resumed and how to start over (`--restart`).
- **FR-007**: The resume flash MUST NOT appear when there is no saved
  position to resume from, or when `--restart` was used for this launch.
- **FR-008**: The interactive `fireside new` wizard MUST, after successfully
  creating a deck, ask whether to present it now, defaulting to yes on a bare
  Enter.
- **FR-009**: Answering yes to the present-now prompt MUST launch the
  presenter on the newly created deck without requiring a separate command;
  answering no MUST leave the wizard's existing exit behavior unchanged.
- **FR-010**: The present-now prompt MUST only appear for the interactive
  wizard flow, not for the non-interactive `fireside new <name>` form.
- **FR-011**: `fireside art text` MUST print a note to stderr, naming the
  measured width, whenever the generated banner's width exceeds the deck
  authoring width threshold (76 columns).
- **FR-012**: The width-guard note MUST NOT alter stdout — the full banner
  text is still written to stdout unchanged and remains pasteable, whether or
  not the note fires.

### Key Entities

- **Reserved presenter key**: One of the fixed set of single-character keys
  the presenter UI already treats as global commands (quit, help, map,
  quick-edit, notes, timer, etc.), regardless of deck content.
- **Resume position**: A host-local record of the last slide a presenter
  reached in a given deck, used to relaunch at that point instead of the
  start.
- **Session summary**: The seen-slide count, deck total, and elapsed time for
  one presentation run, surfaced once at graceful quit.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every branch-key/global-key collision that would previously
  only surface live during a presentation is instead caught by
  `fireside validate` before the deck is ever presented.
- **SC-002**: 100% of graceful (`q`) presenter exits show a session summary
  with accurate seen/total counts and elapsed time, verified against the
  deck's actual node count and a controlled elapsed duration.
- **SC-003**: A presenter relaunching a deck they quit mid-way through can
  tell, without consulting any other tool or file, that they resumed rather
  than started over, and how to override it.
- **SC-004**: A first-time user can go from `fireside new` to a running
  presentation of their new deck without typing a second command.
- **SC-005**: An author generating a too-wide banner via `fireside art text`
  learns this before pasting it into a deck, not after running `validate`.

## Assumptions

- The reserved presenter key set is exactly the twelve keys already handled
  globally by the presenter's key dispatch (`e f g h j k m n p q s t`); this
  feature does not change which keys are reserved, only adds a check against
  the existing set.
- "Seen" for the exit summary means distinct slides visited this session
  (the same count already shown live in the header as "X/Y seen"), not total
  keypresses or revisits.
- The 76-column width threshold for `art text` is the same threshold
  `ascii-art-too-wide` already validates against and `new --banner` already
  applies — this feature does not introduce a new or different limit.
- The present-now prompt in the wizard follows the same y/N-style convention
  as the wizard's existing `Add an ASCII title banner? [y/N]` prompt, except
  defaulting to yes per the feature description ("[Y/n]").
- Non-interactive or piped stdin at the present-now prompt is treated as "no"
  (or otherwise falls back to the wizard's current exit) rather than
  blocking — consistent with how the rest of the wizard already only runs
  interactively.
