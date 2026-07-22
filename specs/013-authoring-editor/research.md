# Phase 0 Research: Authoring Editor (`fireside edit`)

All Technical Context fields in `plan.md` are resolved — no
`NEEDS CLARIFICATION` markers remain. This document records the decisions
that needed verification against the actual codebase (not just the design
brief's assumptions) before planning could proceed, plus the two open
questions the design brief flagged, both already resolved during
`/speckit-clarify` and reproduced here for traceability.

## 1. Hit-testing precedent

**Decision**: Generalize the presenter's existing pure hit-testing pattern
(`fireside-tui/src/render/hits.rs`) into the editor's `hit()` function,
rather than inventing a new mechanism.

**Rationale**: `branch_option_hit` and `map_row_hit`
(`crates/fireside-tui/src/render/hits.rs:26,51`) already prove the pattern
this feature needs: recompute the same pure layout the last frame drew, then
ask which region contains `(col, row)`. Both are free functions taking
`&App` and a `Rect`, calling the same layout helpers `draw`/`draw_content`
use (`content_inner`, `node_lines`, `map::hit_test`) — zero render-to-update
back-channel, so the TEA invariant (rendering is pure) holds automatically
if the editor's `hit()` follows the identical shape: `hit(app: &EditorApp,
area: Rect, col: u16, row: u16) -> Option<Target>`, over an enumeration of
every interactive region (toolbar chip, outline row, block, insertion slot,
form chip, drag-drop target).

**Alternatives considered**: A stateful hover/hit registry built during
render and consulted by `update` — rejected, since it would require a
render-to-update back-channel the constitution's TEA invariant forbids
("rendering is pure"). Recomputing layout on every hit-test call is cheap
(pure functions over data already in `EditorApp`, no I/O) at the deck sizes
in scope (≤500 slides, SC-009).

## 2. `EditableField` reuse

**Decision**: Reuse `fireside-tui::app::EditableField`
(`crates/fireside-tui/src/app.rs:97`) for heading/text block editing inside
the new block-form editor, promoting it out of `app.rs` into a shared
location the `editor` module can also depend on.

**Rationale**: `EditableField` already owns exactly the multi-line
buffer/cursor state the quick-edit modal needs (`buffer: Vec<String>`,
character-indexed `cursor: (usize, usize)`, `insert_char`/`backspace`/
`move_up`/`move_down`/`newline`) — precisely the text-editing primitive the
design brief's heading/text block forms need, and precisely what ADR-005
already scoped quick-edit to. No new text-buffer type is needed.

**Alternatives considered**: A separate text-editing struct for the editor
— rejected as needless duplication of already-correct, already-tested
cursor/buffer logic (Unicode-char-boundary handling in particular is easy
to get wrong twice).

## 3. Draft sidecar keying and hashing

**Decision**: Reuse the exact `fnv1a64` + canonicalized-path-keying scheme
already implemented twice in `fireside-cli` (`session.rs:56`'s `fnv1a64`,
`resume.rs:128`'s `resume_key`), rather than adding a new hash or key
scheme for editor drafts.

**Rationale**: Both existing call sites solve the identical problem this
feature has — "one file per deck, keyed by the deck's canonicalized
absolute path, hashed to a filesystem-safe name under an XDG-state
subdirectory" — for session state and resume state respectively. The draft
sidecar is a third instance of the same shape (a single-writer,
per-deck, occasionally-touched cache file), not a new contract; sharing the
hash helper (extracting `fnv1a64` to a small shared location, or calling
`resume::resume_key` directly) avoids a third independent implementation of
path canonicalization + hashing.

**Alternatives considered**: A `DefaultHasher` — rejected, not stable
across Rust versions (same reasoning that ruled it out for session state,
per the design brief). Reusing `resume.json`'s single shared map file for
drafts — rejected, same reasoning ADR-015 already recorded against sharing
it for session state: draft writes are frequent and per-deck, while
`resume.json` is a rarely-touched, whole-map, last-writer-wins file; mixing
the two would put draft churn into a file every other code path treats as
cold.

## 4. Canvas rendering geometry *(clarified 2026-07-21)*

**Decision**: The canvas renders at the pane's real, current size — the
same behavior the presenter itself already has — with an optional toggle to
preview at a fixed standard size.

**Rationale**: Preserves the WYSIWYG guarantee literally: the editor shows
exactly what presenting from that same window, right now, would show. A
fixed-size toggle covers the "check the common case" need without making it
the default and without adding a second default rendering path.

**Alternatives considered**: Always rendering at a fixed 80-column width —
rejected as user-facing behavior in `/speckit-clarify` (breaks the literal
WYSIWYG promise when the author's terminal isn't 80 columns).

## 5. Block drag-initiation target *(clarified 2026-07-21)*

**Decision**: A block drag can start from a press anywhere on the block,
not only its `⋮⋮` handle; the handle remains a visual affordance cue.

**Rationale**: Matches Notion/Gutenberg-style block editors and gives the
"never seen a terminal" target user (spec SC-001/SC-002) a much larger,
more forgiving hit target than a single-character handle glyph would.

**Alternatives considered**: Handle-only drag initiation — rejected in
`/speckit-clarify` as unnecessarily precision-demanding for the target
user, with no offsetting benefit (click-to-select still works everywhere
on the block either way, so there's no ambiguity between "select" and
"start drag" to avoid).

## 6. Embedded present mechanics

**Decision**: `[ ▶ Present ]` calls the presenter's existing `event_loop`
(`crates/fireside-tui/src/lib.rs:279`) directly, against the
already-initialized terminal, rather than spawning a process or
re-initializing the terminal.

**Rationale**: `fireside-tui::lib.rs` already separates terminal
initialization from the event loop across `present_authoring`
(`lib.rs:204`), `present_impl` (`lib.rs:226`), and `present_watching`
(`lib.rs:165`) — all three build an `App`/`Session` and hand it to
`event_loop`. The editor's embedded present does the same: build
`Session::new(working_graph.clone())`, `goto` the selected slide, wrap it in
`App`, and call `event_loop` with a no-op `ReloadSource`, `WriteBackSink` of
a variant that reports `Unavailable`, and no position-sink writes — so
embedded runs never touch resume state or the exit summary. The only
production code change is visibility (`event_loop` becomes `pub(crate)` or
`pub`, callable from the new `editor` module) — not a refactor of the
function itself.

**Alternatives considered**: A second, editor-owned event loop for preview
— rejected; would duplicate the presenter's entire key/mouse handling
surface and risk exactly the rendering drift WYSIWYG-by-construction exists
to prevent.

## 7. `SlideView` extraction scope

**Decision**: Extract a `SlideView` input type consumed by both
`render/content.rs`'s existing `draw_content`
(`crates/fireside-tui/src/render/content.rs:187`) and the new
`render/editor/canvas.rs`, rather than parameterizing `draw_content` itself
with editor-specific concerns.

**Rationale**: Keeps the presenter's rendering code path completely
unaware the editor exists — `SlideView` is the seam, not a shared "mode"
flag threaded through content rendering. This is the mechanism that makes
SC-008 (canvas/presenter pixel-identical rendering) a structural guarantee
rather than a discipline to maintain by hand, and matches the design
brief's explicit ordering requirement: this refactor must be
behavior-neutral and snapshot-pinned, and must land *after* the
2026-07-19 UX audit's `render/` fixes (P1-6, P2-1) — both confirmed
already merged in commit `790bd29` and its predecessors, so this ordering
constraint is already satisfied; E0 can proceed.

**Alternatives considered**: A second rendering path for the editor canvas
— explicitly rejected by the spec (FR-002) and the design brief's core
"WYSIWYG by construction" commitment.

## 8. Outline ordering algorithm reuse

**Decision**: Implement the depth-first-from-start, unreachable-slides-
after-divider ordering once in `engine::authoring`, with its own direct
unit tests over branch/cycle/unreachable fixtures — not extracted from
`render/map.rs`.

**Rationale**: Checked `render/map.rs`: its slide-ordering is fused into
`layout()` / `build()` (`crates/fireside-tui/src/render/map.rs:115,443`),
which simultaneously computes the rail-diagram's lanes, tracks, and spine
for the map screen's braided-rail visualization — it is not a standalone
"list of slides in order" function, so a clean extraction would mean
refactoring the map screen's rendering internals as a prerequisite, which
is out of scope here and risks regressing a screen this feature doesn't
otherwise need to touch. Implementing the outline order fresh in
`engine::authoring` (pure function, no rendering concerns, the exact
algorithm the design brief specifies: `next` before choice options in
declared order, first-visit wins, unreachable slides after a divider in
stable id order) is the lower-risk path for E0; it stays a candidate for a
later shared extraction if the map screen is ever refactored for other
reasons, but that is not this feature's job.

**Alternatives considered**: Extracting/refactoring `map.rs`'s traversal
now — rejected as scope creep into an unrelated screen's internals for a
feature already large enough (L–XL) without it. Two independently
maintained ordering implementations — rejected outright: the map screen
already displays slides in *its own* rail order (which reflects the same
underlying graph structure), so the editor's outline number and the map's
implicit order describe the same traversal even without shared code; what
matters is that `engine::authoring`'s version is correct and tested against
the design brief's algorithm, not that the two screens share a function
pointer.
