# Feature Specification: Live Validation While Authoring (`validate --watch`)

**Feature Branch**: `001-validate-watch`

**Created**: 2026-07-12

**Status**: Draft

**Input**: User description: "Add `fireside validate --watch` — a live-reload
validation mode for authoring. Today `fireside validate <file>` runs once and
exits. Presenters need a tight edit-save-see-errors loop while hand-editing
deck JSON, without re-running the command after every save. Reuse the
existing file watcher built for `fireside present` and the existing
caret-style parse error rendering. Stay within the `validate` verb (a flag,
not a new subcommand); CLI-only, plain stdout output; no protocol changes;
no ADR needed."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See errors immediately after saving (Priority: P1)

A presenter is hand-editing a deck's JSON file in an editor. They run
`fireside validate --watch deck.fireside.json` in a terminal alongside their
editor. Every time they save the file, the terminal updates to show either a
confirmation that the deck is valid, or the specific problems found —
without the presenter re-running any command.

**Why this priority**: This is the entire feature. Without it there is
nothing to ship — today's `validate` requires a manual re-run after every
edit, which is the friction this feature exists to remove.

**Independent Test**: Run `fireside validate --watch` against a deck file,
edit the file to introduce a semantic error (e.g. a dangling traversal
target) and save, and confirm the terminal shows the error without further
input. Fix the error and save again, and confirm the terminal shows a
success confirmation.

**Acceptance Scenarios**:

1. **Given** `validate --watch` is running against a valid deck file, **When**
   the presenter saves an edit that introduces a semantic validation error,
   **Then** the terminal displays the validation diagnostics for that error
   within one poll cycle, without the presenter taking any further action.
2. **Given** `validate --watch` is running and the terminal is currently
   showing an error, **When** the presenter saves a fix for that error,
   **Then** the terminal displays a success confirmation within one poll
   cycle.
3. **Given** `validate --watch` has just started, **When** no file changes
   have occurred yet, **Then** the terminal shows the validation result for
   the file's current on-disk contents (the presenter does not have to
   save once before seeing a first result).

---

### User Story 2 - Get a precise location for JSON syntax errors (Priority: P2)

While hand-editing JSON, a presenter introduces a syntax mistake (a missing
comma, an unclosed brace). `validate --watch` shows exactly where the mistake
is — the line, column, and a caret pointing at the offending character —
instead of a generic "invalid JSON" message.

**Why this priority**: JSON syntax errors are the most common and most
frustrating mistake for a non-technical or semi-technical author to
self-diagnose. A caret-pointed location turns a multi-minute hunt into a
glance. This reuses rendering that already exists elsewhere in the CLI, so
it is high-value, low-cost.

**Independent Test**: Save a file with a deliberately broken JSON syntax
error (e.g. a trailing comma) while `validate --watch` is running, and
confirm the output shows the line/column and a caret under the specific
character, matching the format already used elsewhere in the CLI for parse
errors.

**Acceptance Scenarios**:

1. **Given** `validate --watch` is running, **When** the presenter saves a
   file containing malformed JSON, **Then** the output shows the line
   number, column number, and a caret marker pointing at the error location.

---

### User Story 3 - Keep working through transient save states (Priority: P3)

Some editors write files in multiple steps (e.g. write to a temp file, then
rename) or briefly leave a file empty or partially written mid-save.
`validate --watch` should not crash or show a misleading "broken" result for
these transient states — it waits for a stable, readable file and then
re-validates.

**Why this priority**: This is a robustness requirement, not new
user-visible behavior on the happy path. It matters because editor save
behavior varies and a false "broken" flash would erode trust in the tool,
but it is lower priority than the core loop and the error-location detail.

**Independent Test**: Simulate a rapid multi-write save (write empty, then
write full content in quick succession) and confirm `validate --watch`
settles on the correct final result without showing a spurious crash or a
permanently stuck state.

**Acceptance Scenarios**:

1. **Given** `validate --watch` is running, **When** the watched file is
   briefly unreadable during a save (e.g. mid-write), **Then** the watcher
   does not crash and re-checks the file on the next poll instead of exiting
   or hanging.

### Edge Cases

- What happens when the watched file is deleted while `validate --watch` is
  running? The tool should report that the file is missing (not crash) and
  continue watching in case it reappears (e.g. the editor recreated it).
- What happens when the presenter passes `--watch` together with a file that
  does not exist yet? The tool should report the file is missing on the
  first check and continue watching, so a presenter can start the watcher
  before the file is first saved.
- How does the system handle a file that is valid JSON but produces many
  validation diagnostics? All diagnostics are shown, matching today's
  non-watch `validate` output format (errors, warnings, and info, with
  counts).
- What happens when the presenter interrupts `validate --watch` (Ctrl-C)?
  The process exits cleanly, matching the interrupt behavior already used by
  `fireside present`'s watch loop.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `validate` command MUST accept a `--watch` flag.
- **FR-002**: When `--watch` is not passed, `validate` MUST behave exactly
  as it does today (single check, then exit) — this feature MUST NOT change
  default behavior.
- **FR-003**: When `--watch` is passed, the system MUST perform an initial
  validation check and display its result immediately, without waiting for
  a file change.
- **FR-004**: While running with `--watch`, the system MUST detect when the
  watched file's contents change on disk and MUST re-run validation
  automatically, without requiring the presenter to restart the command.
- **FR-005**: On each re-validation, the system MUST display either a clear
  success confirmation (deck is valid) or the full set of validation
  diagnostics (errors, warnings, info) for the current file contents.
- **FR-006**: When the watched file contains malformed JSON (a parse
  failure), the system MUST report the line, column, and a caret pointing at
  the error location, consistent with the parse-error format already used
  elsewhere in the CLI.
- **FR-007**: When the watched file contains valid JSON that fails semantic
  validation rules, the system MUST report the same diagnostics (rule name,
  severity, message, affected node where applicable) that non-watch
  `validate` reports today.
- **FR-008**: The system MUST NOT crash or exit when the watched file is
  transiently unreadable (e.g. mid-save); it MUST continue watching and
  re-check on the next detected change.
- **FR-009**: The system MUST NOT crash or exit when the watched file does
  not exist (yet, or is deleted mid-run); it MUST report that the file is
  missing and continue watching for it to appear.
- **FR-010**: The presenter MUST be able to stop `validate --watch` with a
  standard interrupt (Ctrl-C), and the process MUST exit cleanly.
- **FR-011**: Output MUST be plain stdout text (no interactive terminal UI),
  consistent with the CLI-only, non-TUI nature of the `validate` command.

### Key Entities

- **Watched file**: the deck JSON file passed to `validate --watch`; tracked
  by a change fingerprint (modification time and size) so unchanged content
  is not needlessly re-validated.
- **Validation result**: either a success confirmation or an ordered list of
  diagnostics (severity, rule identifier, message, and location/node when
  applicable), the same shape already produced by non-watch `validate`.
- **Parse error**: a specific validation result variant carrying a precise
  source location (line, column) and a caret-formatted rendering, used when
  the file is not valid JSON at all.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A presenter can see the validation result of a saved edit
  without typing any further command — the loop is edit, save, look at the
  terminal.
- **SC-002**: A JSON syntax mistake is diagnosed with an exact line and
  column on the first save that introduces it, every time.
- **SC-003**: `validate --watch` runs indefinitely across at least 50
  consecutive saves (a realistic authoring session) without crashing or
  requiring a restart.
- **SC-004**: Existing `validate` behavior (without `--watch`) is unchanged
  — every current non-watch validation scenario produces identical output
  before and after this feature ships.

## Assumptions

- The watcher re-checks the file on a short poll interval (consistent with
  the existing `fireside present` watch loop) rather than using OS-level
  file-change notifications; this matches the precedent already in the
  codebase and avoids adding a new dependency.
- "Live" means sub-second-to-a-few-seconds latency after a save, matching
  the perceived responsiveness of the existing present-mode live reload —
  not a strict real-time guarantee.
- The terminal is cleared or the new result is otherwise made clearly
  distinguishable from the previous result on each re-validation, so the
  presenter isn't left scrolling to find the latest output; the exact
  presentation (clear-and-redraw vs. append with separators) is a design
  decision for the planning phase, not fixed here.
- No new CLI subcommand is introduced; this ships entirely as a flag on the
  existing `validate` verb.
- No protocol or wire-format change is required; this is authoring tooling
  only.
