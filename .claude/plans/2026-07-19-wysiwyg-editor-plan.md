# Fireside — WYSIWYG Block Editor Plan (`fireside edit`, 2026-07-19, rev 3)

> User decisions 2026-07-19: build the Tier 2 editor from the audit
> addendum (`.claude/plans/2026-07-19-fable-ux-audit.md`, A-3) — a
> dedicated full-screen TUI authoring studio — with an explicit bar:
> **foolproof and easy to use for people who cannot edit JSON or think in
> graph structures.**
>
> **Rev 2 (same day):** the user further directed that the editor must not
> feel like a terminal application to someone unfamiliar with terminals —
> it is a **block editor** (discrete visual blocks you select, drag, and
> drop — the Notion/Gutenberg mental model), **mouse-first** with
> drag-and-drop, and rich in visual cues and feedback. Text-based/modal
> interaction is the fallback layer, not the primary one. This inverts the
> presenter's "keyboard taught, mouse additive" posture *for the editor
> only* — record the inversion in ADR-014.
>
> Tier 1 (structural edits bolted onto the presenter) stays rejected;
> Tier 3 (web editor) stays a separate future decision. Spec-kit feature
> candidate: `013-authoring-editor`, full pipeline (`/speckit-specify` →
> `/speckit-clarify` → `/speckit-plan` → `/speckit-tasks` →
> `/speckit-implement`). This plan is the pre-spec design brief.
>
> **Rev 3 (same day, CTO pass):** resolved every point where an
> implementer would otherwise have to invent an answer — outline
> ordering with branches/unreachable slides, the block-form ↔
> `ContentBlock` kind mapping, embedded-present mechanics, open behavior
> for invalid decks, minimum terminal geometry, undo representation, the
> id slug/rename algorithm, the draft-sidecar format, where hit-testing
> gets its geometry, the vocabulary-gate implementation, and the E0
> sequencing decision (audit render fixes first). No scope change.

## Progress Log

_Update this section whenever an item lands or starts. One line per item:
status, date._

- [X] E0 foundations (ADRs, authoring transforms, hit-testing + shared-renderer refactor) — done 2026-07-21
- [X] E1 read-only studio (canvas + outline + toolbar, click/hover/scroll, present) — done 2026-07-21
- [X] E2 block editing (select/edit/add/delete, drag-and-drop reorder, undo, save) — done 2026-07-22
- [ ] E3 structure editing (slides, wiring, choices, reveal staging)
- [ ] E4 foolproofing polish (drafts, empty states, first-run tour, refinements)

## Product definition

`fireside edit <deck>` opens a full-screen authoring studio; `fireside
edit <name>` with no existing file offers to create one (reusing `new`'s
templates). The presenter is untouched — quick-edit stays content-only
there (ADR-005 continues to govern the presenter; ADR-014 scopes full
editing to `fireside edit`).

Opening rules (rev 3): a path that exists but fails to parse as a deck
(malformed JSON or schema violations) is refused at the CLI with the
same report `present` gives plus one line — `Fix the file first —
"fireside validate <path>" shows the full report.` The editor only opens
decks it can parse. A deck that parses but carries semantic (Layer-2)
diagnostics opens normally with the diagnostics in the status banner —
fixing those *is* the editor's job. Create-if-missing triggers only when
the path does not exist; a `.md`/`.markdown` path gets the audit's P0-2
import hint, never the create flow. Non-tty stdin/stdout is refused with
the P0-3 message from day one.

Two commitments define the product:

1. **WYSIWYG by construction.** The editing canvas *is* the presenter's
   renderer — same card, same wrapping, same theme, same reveal staging.
   There is no second rendering path to drift.
2. **A block editor, not a text editor.** A slide is a stack of discrete
   blocks. You click a block to select it, drag it to move it, click `＋`
   to insert one, and edit through small forms. Nobody is ever dropped
   into a buffer full of markup, and nobody needs to memorize keys to make
   progress — every action on screen is visible and clickable.

## Interaction design principles (the anti-overwhelm charter)

These are requirements, not aspirations — the acceptance bar tests them.

1. **Everything visible is clickable; everything clickable has a keyboard
   path.** Mouse-first for approachability; keyboard-complete for SSH,
   accessibility, and power users.
2. **Progressive disclosure.** At rest the screen shows the deck and at
   most ~7 interactive affordances. Contextual actions appear when a
   block or slide is selected, and disappear when it isn't. Advanced
   operations live one click deeper, never on the surface.
3. **One accent color means "you can interact with this."** All
   affordance styling flows through `theme.rs::Tokens` (a small set of new
   tokens: `affordance`, `selection`, `drop-target`, `ghost`).
4. **Every action produces immediate visible feedback.** Selection glow,
   insertion indicator, drop flash, toast. Nothing ever silently happens
   or silently fails (the presenter's flash discipline, extended).
5. **No invisible modes.** The screen always shows what state it's in — a
   drag in progress looks like a drag; an open form looks like a form; a
   breadcrumb shows where you are inside nested blocks.
6. **Words for anything destructive.** `[ Delete ]`, never a bare `🗑`/`x`.
   Iconic chips are fine for safe, frequent actions (`↑` `↓`).
7. **Never punish.** Undo everything (≥100 steps); Esc always backs out of
   exactly one level; destructive actions confirm via undo-toast, not
   blocking dialogs ("Deleted text block — [ Undo ]").

## The screen

```
┌ My Great Talk ● ──────────────── [ + Slide ] [ ▶ Present ] [ Save ] [ ↶ Undo ] [ ? ] ┐
│ Slides            │  Canvas — the slide as it will present                  │
│ ▸ 1 Welcome       │  ╭──────────────────────────────────────────╮          │
│   2 Pick a path ⑂ │  │ ┃▎ A few more touches            [✎]     │ ← selected: accent
│   ├ a) Features   │  │ ┃  ⋮⋮ drag handle in gutter               │   border + handle
│   ├ b) The end    │  │                                          │          │
│   3 Features      │  │   Press Space to reveal…          ◇1     │ ← reveal-step badge
│   4 The end ■     │  │ ── ＋ add a block here ──                 │ ← hover insertion
│                   │  │   The map (m) shows every slide…  ◇2     │          │
│ ＋ new slide      │  ╰──────────────────────────────────────────╯          │
│                   │  Goes to: → Features        [ change ]                 │
├───────────────────┴────────────────────────────────────────────────────────┤
│ ✓ ready to present · 4 slides                                              │
│ Click a block to select · drag ⋮⋮ to move · ? shows every key              │
└────────────────────────────────────────────────────────────────────────────┘
```

- **Toolbar (top).** Deck title (click to rename; `●` = unsaved) and five
  chips: add slide, present, save, undo, help. These are the whole
  top-level surface — principle 2.
- **Outline (left).** Every slide, reusing the map's visual language
  (⑂ choice, ■ ending). Rows are clickable; drag a row to reorder a
  linear run (rewiring follows the drag — see E3). `＋ new slide` is a
  permanent, clickable last row.
  **Ordering (rev 3, deterministic):** depth-first from the deck's start
  node — follow `next` first, then choice options in declared order; a
  slide appears exactly once, at its first visit (cycles terminate for
  free). Slides unreachable from start (a legitimate mid-edit state) are
  listed after a `not linked yet` divider row, in stable id order, so
  nothing the user created can ever vanish from the outline. Numbering
  is position in this order and recomputes after every structural op —
  it is a display coordinate, never an identifier. If the map screen
  already has an ordering function, extract and share it; either way the
  ordering lives in `engine::authoring` with direct unit tests over
  branch / cycle / unreachable fixtures.
- **Canvas (right).** The presenter's rendering of the selected slide,
  overlaid with block affordances (below). Wheel/trackpad scrolls when
  the slide overflows; a scrollbar appears only when needed.
- **"Goes to" strip.** The slide's structural facts in plain words, with
  a `[ change ]` chip. Choice slides show their answers here instead.
- **Status line.** Live validation in plain language (`✓ ready to
  present` / `✗ won't present yet: … — click to view`), clickable to jump
  to the problem. Reuses the existing validator messages.
- **Hint line.** Not a key reference — one sentence of guidance for the
  current context, plus `?` for the full map. Rotates as context changes
  (selection, drag, form). This replaces the presenter-style dense key
  footer, which is exactly what overwhelms terminal newcomers.
- **Minimum geometry (rev 3).** Below 80×24 the editor draws a single
  centered guard — `Fireside edit needs at least an 80×24 window — make
  this one bigger` — and waits for resize; the three panes never
  collapse into overlap. TestBackend scenario at 44×14 pins it.

## The block model on the canvas

- **At rest**: the slide renders clean — pure presenter output. No chrome.
- **Hover** (terminals reporting motion events; see Risks): the block
  under the pointer gets a hairline outline and a `⋮⋮` gutter handle;
  the gap between blocks nearest the pointer shows a
  `── ＋ add a block here ──` line.
- **Click**: selects the block — accent border, gutter handle, and a
  contextual chip row: `[ ✎ Edit ] [ ＋ Add below ] [ ↑ ] [ ↓ ]
  [ Reveal ▾ ] [ Delete ]`. Click elsewhere (or Esc) deselects.
- **Double-click / Enter**: opens the block's form (below).
- **Drag** (press on the block or its handle, then move): the block lifts —
  rendered as a dimmed ghost following the pointer — and a bold insertion
  line snaps between blocks to show exactly where release will drop it.
  Auto-scroll near canvas edges. Release drops (with a brief settle
  flash); **Esc during a drag cancels it** and the block returns. The same
  gesture works in the outline for slides.
- **Reveal staging**: blocks with reveal steps carry a `◇n` badge (edit
  view only — never in present). The `[ Reveal ▾ ]` chip cycles
  none → 1 → … → none; steps auto-compact to 1..n. A `[ ▷ preview ]`
  chip on the canvas header steps the staging live.
- **Empty slide**: one big centered `＋ Add your first block` target.

### Block forms (never syntax)

The forms map 1:1 onto `ContentBlock`'s **eight** kinds (rev 3 — pin
this so nobody invents a ninth or merges two): `heading`, `text`,
`code`, `list`, `image` ("picture"), `divider` ("line"), `ascii-art`
("text art"), `container` ("columns / box / stack" is **one** kind — the
layout picker sets `ContainerLayout::Columns` / `Center` / `Stack`, and
the add palette shows eight cards). The quoted names are the only ones
ever rendered (vocabulary rule below).

Click `✎` (or Enter) and the block's editor opens *in place*, sized to the
block, with `[ Done ]` / `[ Cancel ]` chips:

- **heading / text**: the quick-edit field editor, reused as-is.
- **list**: one item per line; blank lines dropped.
- **code**: language picker + multiline source; Tab inserts spaces
  (dovetails with audit P1-3).
- **text art**: paste area + `[ Generate from a phrase… ]` (in-process
  figlet via a CLI-injected callback); the 76-column width rule is checked
  *before* accepting, with the warning shown in the form.
- **picture placeholder**: path + description fields, the standing
  reminder that terminals show a placeholder frame, and a
  `[ Convert to text art ]` chip (the `art image` path).
- **columns / box**: layout picker (side-by-side / centered / stack);
  children edited by clicking into them — a breadcrumb
  (`Slide 2 ▸ columns ▸ left`) always shows the way back. Full recursion,
  since half-support would break the foolproof promise.
- **line (divider)**: nothing to edit.

The add-block palette (from any `＋`) is a clickable card list — each of
the 8 kinds with a one-line plain-language description — inserting
placeholder content and opening its form immediately.

### Vocabulary rule

Unchanged from rev 1 and enforced by a snapshot grep gate: no node id,
JSON key, kind string, or graph/node/traversal jargon is ever rendered.
Slides, choices, answers, endings, "goes to", reveal steps. Ids are
auto-managed (slugified titles, deduped, renames rewrite every reference
atomically) — invisible, always.

Slug algorithm (rev 3): lowercase the title; map every run of
non-alphanumeric characters to a single `-`; trim leading/trailing `-`;
an empty result falls back to `slide`; dedupe against all existing ids
with `-2`, `-3`, … suffixes. Retitle is **one** `engine::authoring`
transform that rewrites the id and every reference to it (`next` edges,
choice targets, the start id) in the same op — with a proptest that no
rename sequence can ever dangle a reference.

Gate implementation (rev 3): one render-suite test walks every editor
insta snapshot and fails on the denylist regex —
`\b(node|nodes|graph|traversal|kind|id)\b`, the raw kind strings
(`ascii-art`, `container`, `divider`), and any `"` -quoted JSON key.
Editor snapshot fixtures keep those words out of their deck *content*
(cheaper than teaching the gate to tell chrome from content); the
fixture used for preview-fidelity tests is exempt since it renders
presenter output only.

### Structure editing (plain words, pickers, drags)

- **New slide**: toolbar chip or outline row → title prompt → auto-wired
  after the current slide.
- **Reorder slides**: drag in the outline. Within a linear run the wiring
  follows the order. Dragging across a branch boundary is refused with a
  toast explaining why ("Features is one of Pick-a-path's answers — change
  the answer's target instead — [ take me there ]").
- **Delete / duplicate**: chips on the selected outline row. Delete heals
  wiring (predecessors point past it; explained in the undo toast).
- **Wiring**: `[ change ]` on the "Goes to" strip opens a slide picker —
  titles with a live mini-preview of the highlighted target, plus
  `→ a new slide…` and `→ nothing — this is an ending`. No typed ids,
  anywhere, ever.
- **Choices**: `[ Turn into a choice ]` on a slide → prompt field + answer
  rows (label, optional one-letter key — the picker refuses reserved
  presenter keys, surfacing the `reserved-branch-key` rule at authoring
  time — and a target via the same slide picker). `[ Turn back into a
  normal slide ]` keeps the first answer's target. The `next`-xor-choice
  invariant is unrepresentable in this UI.
- **Metadata & notes**: deck title in the toolbar; a `[ Notes ]` chip per
  slide for speaker notes (feeds the Wave 4 dual-screen feature).

## Never lose work

Unchanged from rev 1, now with visible affordances:

- **Undo/redo of everything** — ≥100 snapshots; toolbar `[ ↶ Undo ]` chip
  plus `u`/`U`; destructive actions confirm via undo-toast, not dialogs.
  Representation (rev 3): full `Graph` clones, pushed by
  `EditorApp::update` on each committed op, capped at 100, redo stack
  cleared on any new op; each snapshot carries the selection so undo
  restores view context. Op inversion is explicitly rejected — decks are
  small (the audit's 500-node stress deck clones instantly) and
  snapshots mean the proptests only prove the transforms, not inverses.
- **Esc is layered and safe**: cancels drag → closes form (field only) →
  deselects → offers quit. Committed edits die only by explicit undo.
- **Explicit save** (`[ Save ]` chip / Ctrl+S), honest `●` dirty dot, quit
  prompt with `[ Save ] [ Discard ] [ Keep editing ]` chips.
- **Crash-proof drafts**: autosave to an XDG-state sidecar (path-keyed per
  audit P1-1) every few seconds and on every structural op; restore
  prompt on next open. SIGKILL loses seconds at most.
  Sidecar format (rev 3):
  `$XDG_STATE_HOME/fireside/drafts/<fnv1a64 hex of canonical path>.json`
  (same key scheme as the audit plan's W4-DS-2 session file — share the
  hash helper) holding `{"schema": 1, "deck_path": …, "saved_at": <epoch>,
  "deck": <full deck JSON>}`. On open, if a draft exists whose `deck`
  differs from the file's content, prompt with both timestamps:
  `[ Restore draft ] [ Open saved file ]`. The draft is deleted on
  successful save and on clean quit without unsaved changes; atomic
  temp + rename writes, like every other state file.
- **Atomic writes** (temp + rename) and the quick-edit fingerprint
  conflict guard for the two-editors case.
- **Save is never blocked by validity** — construction prevents most
  invalid states; the rest surface in the clickable status banner.

## Try it without leaving

`[ ▶ Present ]` (or `P`) presents the working graph in-process from the
current slide; `q` returns to the editor exactly where you were. No save
needed. This is the author's single-keystroke loop and the editor's
biggest usability multiplier.

Mechanics (rev 3): `fireside-tui` already owns its event loops —
`present_authoring` initializes the terminal and runs `event_loop`
internally. The editor entry point does the same, and `[ ▶ Present ]`
neither spawns a process nor re-initializes the terminal: inside the
editor loop, build `Session::new(working_graph.clone())`, `goto` the
selected slide, wrap it in the presenter `App`, and run the existing
presenter `event_loop` against the **already-initialized** terminal with
a no-op reload source, `Unavailable` write-back sink, and a no-op
position sink — embedded runs never touch resume state, never write
session-state files, and never print the exit summary. On quit, control
falls back to the editor loop, which repaints. The only enabling change
is making `event_loop` callable from the editor module (visibility, not
refactor); mouse capture is already on for the whole process.

## Acceptance bar (testable)

1. **The 10-minute test, twice.** A user who has never seen JSON *and is
   not comfortable in terminals* creates a 5-slide deck with one choice
   and one reveal, presents it, and saves — once **using only the mouse**
   (typing only inside text fields), once **using only the keyboard**.
   Both are scripted tmux walkthroughs (mouse via injected SGR sequences).
2. **At rest ≤ ~7 visible affordances**; contextual actions appear only on
   selection (snapshot-audited per screen state).
3. Every mouse gesture has visible intermediate state: hover cue (where
   supported), selection border, drag ghost + insertion line, drop flash.
   A drag can always be cancelled with Esc.
4. No jargon ever rendered (snapshot grep gate); destructive actions are
   word-labeled.
5. Any single action is recoverable: Esc backs out one level; undo covers
   ≥100 operations.
6. Kill -9 loses at most the autosave interval; the deck file always
   parses (atomic writes).
7. Editor-produced decks never contain dangling targets, duplicate ids,
   `next`+choice conflicts, or gapped reveal steps (proptest over
   arbitrary op sequences — unrepresentable by construction), and any
   remaining diagnostic was on screen at save time.
8. **Preview fidelity**: the canvas's at-rest buffer for a slide equals
   the presenter's buffer at the same geometry, for every fixture deck
   (the WYSIWYG guarantee as a property test).

## Architecture

**No new dependencies.** Ratatui + crossterm + std cover everything:
crossterm's `EnableMouseCapture` already delivers press/drag/release with
coordinates plus motion (hover) and wheel events, and the presenter
already ships mouse hit-testing (`branch_option_hit`/`map_row_hit`) whose
pattern this generalizes.

| Piece | Lives in | Notes |
| --- | --- | --- |
| Graph transforms (slide add/delete/duplicate/retitle+rewrite, rewire, to/from choice, block insert/move/delete, reveal renumber) | `fireside-engine`, new `authoring` module | Pure `(Graph, Op) -> Result<Graph, AuthoringError>` (thiserror), unit + proptests. Engine owns graph semantics; core stays a passive model. |
| Editor state machine | `fireside-tui`, new `editor` module | Own TEA app: `EditorApp::update` is the sole mutation point. State includes selection, drag (`Idle / Lifting { block } / Over { slot }`), open form, undo stack, draft-timer ticks as messages. Reuses `EditableField` (promoted out of `app.rs`). |
| **Hit-testing** | `fireside-tui` | The presenter's pattern generalized: layout is pure and deterministic, so `update` recomputes the same layout the last frame drew and asks "what interactive region contains (x, y)?" — a `hit(app, area, x, y) -> Option<Target>` function over an enumeration of affordances (toolbar chip, outline row, block, insertion slot, form chip…). Pure, unit-testable, no render-to-update back-channel, TEA intact. Drag = press target + motion resolving to insertion slots + release commit. Geometry source (rev 3): `EditorApp` stores the terminal size — set at startup, updated by every resize event — and `update` computes layout from that stored size, never from the renderer. |
| Editor rendering | `fireside-tui`, `render/editor/` | Canvas calls the *existing* content-rendering path via a small extracted `SlideView` input (E0 refactor), then overlays affordances. New theme tokens for affordance/selection/drop/ghost. Outline reuses map idioms. |
| File I/O (load, save, draft, conflict fingerprint) | `fireside-cli` | Injected closures, the `present_authoring` sink pattern. TUI touches no files. |
| `edit` subcommand + create-if-missing | `fireside-cli/src/edit.rs` | Template reuse from `new.rs`/`templates.rs`; non-tty guard from day one (audit P0-3). |
| Art generation helper | CLI-injected callback | figlet/rascii stay CLI-side per the allowlist. |
| Embedded present | `fireside-tui` | Run the existing presenter loop over the working graph (no reload source, `Unavailable` sink), return to editor state on quit. |

### Decisions needed at `/speckit-clarify`

- **Canvas geometry**: render at the pane's real size (recommended — the
  edit terminal ≈ the show terminal) with a `[ 80×24 room view ]` toggle,
  vs. always-80-col.
- **Hover dependency**: ship assuming motion events (mode 1003 — supported
  by every mainstream emulator + tmux); degrade to click-reveals-affordances
  where absent. Confirm degradation UX.
- **Drag initiation**: drag from anywhere on the block vs. handle-only.
  Recommended: anywhere (bigger target, Notion behavior); handle exists as
  the visual cue that dragging is possible.
- **Slide-drag across branch boundaries**: refuse-with-explanation
  (recommended, above) vs. allow-with-rewiring-prompt.

## Waves

Each wave leaves `main` releasable and is independently useful.

**E0 — foundations (M).** ADR-014: `fireside edit` scope — extends
ADR-004's verbs by explicit user request; supersedes ADR-005 *for the
editor only*; **records the interaction-posture inversion** (editor:
mouse-first, keyboard-complete; presenter: unchanged). ADR-015: authoring
transforms in `fireside-engine` (module-charter note). PATCH constitution
amendment: TEA wording → "each TUI application struct has exactly one
update function", plus the new theme tokens noted under Principle IV's
styling rule. Then: `engine::authoring` (full `Op` enum, transforms,
errors, unit tests + the two proptests), the `SlideView` render refactor
(behavior-neutral, snapshot-pinned), and the `hit()` region enumeration
skeleton with unit tests.

**E1 — read-only studio (M).** Toolbar + outline + canvas + status/hint
lines. Click to select slides and blocks, hover cues, wheel scrolling,
`[ ▶ Present ]`, `?` overlay. No mutations. Already useful as a deck
explorer. TestBackend scenarios (keyboard *and* injected mouse events);
tmux smoke including SGR mouse-sequence injection (prove the technique
here, while the surface is small).

**E2 — block editing (L).** Selection chip row, all 8 block forms, add
palette, delete + undo-toast, **drag-and-drop block reorder** (ghost,
insertion line, auto-scroll, Esc-cancel), undo/redo stack, dirty state,
save (atomic + conflict guard), quit prompt. Drag-and-drop lands here,
not in polish — it is the block-editor identity. Heaviest wave.

**E3 — structure editing (M–L).** Slide create/duplicate/delete/retitle,
outline drag-reorder of linear runs (refusal toast across branch
boundaries), wiring picker, choice builder, ending toggle, metadata +
notes forms. The mouse-only 10-minute test passes here — pin it as the
flagship smoke.

**E4 — foolproofing polish (M).** Draft autosave/restore, empty states,
first-run hint tour (three rotating hint-line messages, dismissed
forever after the first save), status-banner jump-to, drag auto-scroll
tuning, the text-selection escape hatch note (below), docs: new
`guides/editing.md` + VHS tape via `scripts/demos.sh`, quickstart/README/
cli.md updates, bare-invocation teaching line.

Total: L–XL (several focused weeks). E0+E1 is the first PR-able
milestone. The E0 render refactor must not interleave with the audit
plan's `render/` fixes (P1-6/P2-1) — sequence one before the other.

## Test discipline map (constitution VII)

- `engine::authoring`: unit + proptests first (TDD — the invariants are
  the crown jewels).
- `hit()`: table-driven unit tests (region × coordinate → target).
- Every editor screen state: TestBackend scenarios driving **both key
  events and synthetic `MouseEvent`s** (press/move/release sequences for
  drag paths) through `EditorApp::update`; insta snapshots for layouts,
  `contains()` for behavior contracts.
- CLI: e2e for `edit` args, create flow, non-tty refusal.
- Real terminal: tmux smoke per wave — mouse injected as SGR escape
  sequences (`ESC [<0;x;yM` / `m`), validated as a technique in E1 —
  culminating in the two scripted 10-minute tests; wired into
  `scripts/smoke.sh` (audit CH-2) and `verify.sh`.
- Definition of done, per wave (rev 3): `scripts/verify.sh` passes (it
  mirrors every CI job — never a hand-picked subset), the wave's tmux
  smoke ran in a real terminal, `graphify update .` ran, and this file's
  Progress Log line is ticked with the date.

## Risks & mitigations

- **Mouse capture hijacks native text selection/copy** — the classic TUI
  complaint. Mitigate: document Shift+drag (the standard terminal bypass)
  in the help overlay and `guides/editing.md`; all text is also reachable
  through forms for copying.
- **Hover events unsupported** in a minority of environments → all hover
  cues are enhancements over click; nothing is hover-*only* (acceptance
  #3's "where supported" wording is deliberate).
- **Glyph-width surprises** with chip/handle characters → stick to ASCII +
  box-drawing + `⋮ ◇ ⑂ ■ ▸` (already proven in the presenter), snapshot
  the lot.
- **Scope creep toward a full IDE** → the surface is the toolbar's five
  chips plus contextual rows; anything that wants a sixth toolbar chip
  goes through `/speckit-clarify`, not into the code.
- **tmux mouse injection flakiness** in smoke tests → proven in E1 while
  the surface is one screen; if it proves unreliable, TestBackend mouse
  scenarios remain the deep coverage and smoke falls back to
  keyboard-parity paths (every mouse action has one, by principle 1).

## Constitution notes (explicit)

- **New verb** beyond present/validate/new: extends ADR-004 by explicit
  user request (2026-07-19) — exactly Principle II's condition; recorded
  in ADR-014 together with the editor-only mouse-first posture.
- **ADR-005** continues to govern the presenter; ADR-014 makes the editor
  the structural-edit surface.
- **No dependency-allowlist changes.** If a wave wants a widget crate
  (e.g. a drag-drop or textarea helper), that is a stop-and-flag moment
  per Principle III; default answer is "build the small thing in-tree",
  as quick-edit's field editor already proved out.
- **TEA wording** PATCH amendment (bundled with ADR-014, above); the
  hit-testing design deliberately preserves "rendering is pure".
- **Styling**: new affordance/selection/drop/ghost tokens go in
  `theme.rs::Tokens` — no raw colors in render code, per Principle IV.
- **No protocol changes**; MSRV 1.88 unaffected.

## Explicitly out of scope

- Web/static WYSIWYG editor (Tier 3) — the `engine::authoring` op layer
  would serve it (wasm) if that day comes; nothing here blocks it.
- Live re-sync into an open editor while the same deck is being presented
  (the conflict guard makes it safe; making it *pleasant* is later).
- Dragging blocks *between* slides (drag within a slide + cut/paste-style
  move via chips covers the need; cross-slide drag is a v2 gesture).
- Multi-deck projects, asset management, themes, collaborative editing.
- Import-time interactivity — the audit plan's import fixes stay there.

## Suggested order relative to other plans

Audit Wave 1 (P0s) and P1-1 (resume keying) first — small, and P1-1
underpins this plan's draft keying and Wave 4's follower. E0/E1 can then
run in parallel with audit Wave 2 and the dual-screen feature (012),
which touch disjoint code — except inside `fireside-tui/src/render/`:
**decided (rev 3)** the audit's render fixes (P1-6, P2-1) land *before*
the E0 `SlideView` refactor, which then carries them. The audit plan
records the same decision; neither plan interleaves with the other in
that directory.
