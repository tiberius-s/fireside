# Phase 0 Research: Modern TUI Leverage

## 1. Mouse hit-testing without breaking the pure-render invariant

**Decision**: Extract pure layout/hit-test functions in `fireside-tui::render` —
one for map-screen rows, one for branch-menu options — that take the same
inputs `render::draw` already uses (session state, terminal area) and return
each interactive row's `Rect`. `render::draw` calls them to know where to
paint; a new `App` method calls the *same* functions to know what a click
landed on. Neither path mutates anything, so Constitution Principle IV's
"rendering is pure" / "`App::update` is the only mutator" both hold — the
hit-test function is pure, and only `App::update` (handling a new
`Msg::Mouse`) turns a hit into a state change.

**Rationale**: The alternative — stashing rects in `App` as a side effect of
drawing — would make rendering impure and give `App::update` a second
mutation path (render-time writes), which Principle IV forbids.

**Alternatives considered**: A general-purpose hit-testing/UI-region crate —
rejected, adds a dependency for two screens' worth of geometry that a ~20-line
pure function already covers.

**New `App` state required**: the last known terminal `Size` (not currently
tracked — `app.rs` has no `size`/`Rect` field today). Populated on the first
draw and refreshed on `crossterm::event::Event::Resize`, mirroring how
`scroll`/`view_override` are already plain `App` fields.

**Click semantics**: `MouseEventKind::Down(MouseButton::Left)` triggers the
action immediately (matches the immediacy of every existing keypress);
`Up`/`Drag`/other buttons are ignored. A click on a branch option while
reveal is still pending advances reveal instead of choosing — the same rule
`on_present_key` already applies to branch keys (`app.rs:554-555`) — so mouse
and keyboard stay behaviorally identical at every gate, not just at the
happy path.

## 2. Resume position persistence

**Decision**: `fireside-tui` gains two new optional plumbing points on
`present_authoring`, symmetric to the existing `ReloadSource`/`WriteBackSink`
pattern (`lib.rs`): an initial-node override (used once, via `Session::goto`
right after `Session::new`) and a position-changed callback invoked whenever
the current node changes. `fireside-cli` — which already owns all file I/O
per the crate boundary table — reads/writes a small resume-state file keyed
by the deck's existing content fingerprint (`main.rs::fingerprint`, already
used for reload/write-back conflict detection) mapping fingerprint → last
node id. `fireside-tui` itself never touches a filesystem, preserving
Principle III.

Falling back when a saved node id no longer exists (FR-008) is free: `Outcome::UnknownNode`
already exists and `Session::goto` is already a "guarded no-op" on an unknown
target (`session.rs:216`, test `goto_unknown_node_is_a_guarded_no_op`) — the
CLI just attempts the goto and ignores a non-`Moved` outcome, leaving the
session at its normal default-entry node.

The CLI clears the resume record when a session reaches a normal end (no
further `next`/branch target) rather than only on clean process exit, so a
kill/crash mid-deck is exactly the case that leaves a record (FR-001/FR-002
acceptance).

**Storage location — flagged for review**: no new dependency is used; the
path is built with `std::env`/`std::path` only (checking `XDG_STATE_HOME`,
falling back to `~/.local/state/fireside/resume.json` on Unix-likes),
consistent with "no new dependencies" already promised for this feature.
**Flag per Principle III**: a `dirs`-crate-based cross-platform path (also
correct on Windows/macOS conventions) is the more robust alternative but
would add a new `fireside-cli` dependency and needs an explicit decision;
the manual-path default is used unless told otherwise.

**Restart escape hatch (FR-007)**: a new `--restart` flag on `present`
bypasses the resume lookup for that run without deleting the stored record
(so the *next* unflagged run still resumes normally).

**Alternatives considered**: storing resume state in the engine or
alongside the deck file itself — rejected; the plan explicitly calls it "a
dotfile keyed by content fingerprint," i.e. host-local and separate from the
portable deck format, and engine crates are forbidden any I/O.

## 3. Synchronized output

**Decision**: bracket the existing `terminal.draw(...)` call in
`fireside-tui::event_loop` (`lib.rs:138`) with
`crossterm::terminal::BeginSynchronizedUpdate` /
`EndSynchronizedUpdate` (already present in crossterm 0.29's `command`
module — confirmed by inspection, no new dependency).

**Rationale**: these are just escape-sequence writes (DEC private mode
2026); a terminal that doesn't recognize them ignores them by construction
of the spec, so FR-011 ("no error, no degradation on unsupported terminals")
holds with zero capability detection — the same reasoning already applied
to the `fade` transition's documented fallback (Appendix D).

**Alternatives considered**: querying terminal support before using it —
rejected as unneeded complexity for an escape sequence defined to be inert
when unrecognized.

## 4. OSC 8 hyperlinks — rendering feasibility (the real open question)

Ratatui has no built-in hyperlink span type (unlike bold/italic/underline,
which map to `Modifier` bits) — this needed an actual feasibility check
before committing to it, in the same spirit as the ADR-008 `ratatui-image`
spike.

**Rejected approach**: writing raw OSC 8 escape bytes directly via
`ratatui-crossterm`'s `CrosstermBackend::writer_mut()`. That accessor exists
but is gated behind the `unstable-backend-writer` feature, and its own doc
comment warns: *"writing to the writer may cause incorrect output after the
write. This is due to the way that the Terminal implements diffing
Buffers."* Using it would risk exactly the kind of cross-frame corruption
that made the image spike a NO-GO — rejected without needing to build a
throwaway to confirm; the warning is explicit in the dependency's own docs.

**Chosen approach**: `ratatui-core::buffer::Cell` has a stable, tested,
non-unstable mechanism for exactly this shape of problem —
`CellDiffOption::ForcedWidth`. It already exists to let one `Cell`'s
`symbol` string differ in byte length from its on-screen column width (this
is how ratatui's own buffer diffing already treats wide CJK glyphs
internally, and it has dedicated unit test coverage in
`ratatui-core`'s `buffer.rs`). Plan: the labeled span's first cell gets
`symbol = "<OSC-8-open><label><OSC-8-close>"` with
`diff_option = CellDiffOption::ForcedWidth(label_visible_width)`; the
label's remaining cells are marked `CellDiffOption::Skip` (again, the same
technique already used for wide-character continuation cells). No unstable
ratatui feature, no raw-writer bypass, no new dependency.

**On unsupported terminals**: the OSC 8 open/close sequences are inert
control bytes the terminal doesn't act on; the label text between them
still prints normally (FR-014) — same "invisible-if-unsupported" pattern as
synchronized output.

**Link authoring syntax**: reuses the existing "spec allows inline Markdown
in `text.body` without pinning a subset" latitude already documented in
Appendix D and already used for `**bold**`/`*italic*`/`` `code` `` in
`fireside-tui/src/render/markdown.rs`. Adding `[label](url)` is the same
kind of engine-extension behavior, not a protocol/schema change — confirmed
against `protocol/main.tsp` (no `link` field anywhere in the content-block
schema) and Appendix D's existing "Behavior near the protocol's edges"
section. **No spec version bump, no `tsp-output/` regen** — this stays
inside the same non-normative latitude as the other inline markers; only
`docs/src/content/docs/spec/appendix-engine-extensions.md` gets a new
bullet.

**Malformed-URL validation (FR-015)**: a new WARNING rule alongside the
existing symmetric validator pair (`fireside-engine::validation` +
`protocol/validate.mjs`), extending the shared fixture corpus
(`protocol/fixtures/{valid,invalid}/*.json`) the same way `empty-traversal`
and `reveal-masked-by-container` were added — proven to actually fire by the
same "introduce a deliberate mismatch, confirm it fails, then revert"
discipline used for those two.

## Summary of dependency/protocol impact

- No new crate dependencies anywhere (mouse, sync-output, OSC 8, and resume
  storage all use APIs already present in `crossterm`/`ratatui`/`std`).
- No protocol/schema change, no version bump, no `tsp-output/` regen — the
  only spec-adjacent artifact touched is the non-normative Appendix D.
- One flagged, reviewable choice: the resume-state file path is built
  manually (`std::env`/`std::path`) rather than via a `dirs`-style crate, to
  keep the "no new dependencies" property; a cross-platform-correct
  alternative exists but needs an explicit sign-off if wanted (Principle III).
