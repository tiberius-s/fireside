# Contract: `fireside edit <file>`

New `Command::Edit { file: PathBuf }` variant on the existing `Cli`/`Command`
enum in `crates/fireside-cli/src/main.rs:61`, dispatched to a new
`crates/fireside-cli/src/edit.rs`. Mirrors the existing `present`/`notes`
functions' shape (`main.rs:335,401`): load, validate/guard, hand a
callback-injected sink/source set to `fireside-tui`, unwrap the
`TuiError::NotATty` case specially.

## Preconditions / opening rules (spec FR-025–FR-029)

Evaluated in this order:

1. **Non-interactive terminal** (`!stdout.is_tty() || !stdin.is_tty()`):
   refuse immediately with `TuiError::NotATty`'s existing message, exit 1.
   Same guard `present`/`notes` already apply (`lib.rs:236,370`) — reused,
   not reimplemented, inside the new editor entry point.
2. **Path exists, reads, but fails to parse as a deck**: refuse to open.
   Print the same report `present` prints on a parse failure
   (`report::parse_report`, `main.rs:284`) plus one line: `Fix the file
   first — "fireside validate <path>" shows the full report.` Exit 1. The
   editor never opens a file it cannot parse — unlike `present`'s
   `load()`, this is a hard refusal, not a fallback path.
3. **Path is `.md`/`.markdown`**: the existing import hint
   (`main.rs:279`, "This is a Markdown file — run `fireside import` first")
   — never the create-if-missing flow, even though the path "doesn't
   exist" as a `Graph`.
4. **Path does not exist** (and is not `.md`/`.markdown`): offer to create
   a new deck, reusing `new.rs`'s existing template flow
   (`crates/fireside-cli/src/new.rs`, `templates.rs`) rather than exiting
   with a hint the way `present`'s `load()` does today — `edit`'s
   create-if-missing is an actual flow, not a pointer to run a different
   command.
5. **Path exists, parses, but `fireside-engine::validation::rules()`
   reports diagnostics**: open normally. Diagnostics appear in the
   editor's status banner (spec FR-026) — fixing them is what the editor
   is for, so this is not a refusal case.
6. **Terminal smaller than the editor's minimum usable size**: after
   opening, draw the single centered guard message and wait for resize
   (spec FR-029) — evaluated continuously (a resize below the threshold
   mid-session re-shows the guard), not only at open.

## Behavior

- On open, check for a draft sidecar (`data-model.md`'s Draft sidecar
  section) keyed off the same canonicalized-path scheme as `resume_key`
  (`resume.rs:128`). If a draft exists and differs from the file's parsed
  content, prompt `[ Restore draft ] [ Open saved file ]` before drawing
  the studio, showing both timestamps.
- Runs the editor's own event loop (new, in `fireside-tui::editor`) against
  a `try_init`'d terminal — same initialization pattern as `present_authoring`
  (`lib.rs:255`).
- `[ ▶ Present ]` calls the existing `event_loop` in-process against the
  same terminal (`research.md` §6) — no process spawn, no second
  `try_init`.
- Save (`[ Save ]` / Ctrl+S) writes the deck file via the same
  injected-closure pattern `present_authoring`'s write-back sink already
  uses, atomically (temp file + rename); on success, deletes the draft
  sidecar.
- Quit with unsaved changes prompts `[ Save ] [ Discard ] [ Keep editing ]`
  (spec FR-019) before the process exits; clean quit with no unsaved
  changes deletes the draft sidecar.
- **`edit` does not touch `resume.json` or the live session-state file**
  (`session.rs`) — those are presenter-only state; the editor's embedded
  present (`research.md` §6) explicitly wires a no-op position sink and an
  `Unavailable`-reporting write-back sink so it can never write either.

## Exit codes

- `0`: normal quit (saved or explicitly discarded).
- `1`: refused to open (non-tty, unparseable deck) — same convention as
  `present`/`validate`.

## Out of scope for this contract

- The `fireside new`-reused template *content* (unchanged, this feature
  only reuses the existing flow, does not modify it).
- Any change to `present`, `notes`, or `validate`'s own behavior.
