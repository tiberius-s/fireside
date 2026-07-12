# Fireside Strategic Improvement Plan — 2026-07-12

## Executive Summary

Fireside's rewrite delivered what ADR-004 promised: a 4-crate workspace where
spec and implementation genuinely agree (all 7 block kinds render, dual
Rust/Node validators share rule names, 100 tests pass), and a presenting
experience a non-technical person can drive from the footer alone. The
strategic gap is no longer presenting — it is **authoring**. ADR-004's own
trade-off section says it: "Editing decks means editing JSON by hand until an
editor returns." Every comparable tool (presenterm, slides, patat) authors in
Markdown; Fireside asks non-technical presenters to hand-write graph JSON.
Phase 1 should (1) close the authoring gap in stages that reuse the live-reload
loop already built, (2) ship the two visual follow-ups ADR-004 explicitly
deferred — terminal image rendering and, with it, bounded ASCII art — via the
spec-first extension process, and (3) harden the protocol with a shared
conformance fixture corpus and a small list of spec ambiguities found in this
audit. Everything below respects protocol 0.1.0 (additive 0.1.x only) and the
crate boundary table in AGENTS.md.

---

## 1. Current-State Analysis

### What is verifiably solid

- **Architecture.** `fireside-core` (pure model) → `fireside-engine`
  (Session state machine + Layer-2 validation) → `fireside-tui` (TEA, pure
  render, all styling through `theme.rs::Tokens`) → `fireside-cli`
  (present/validate/new/demo). Boundaries are enforced by a written table
  (AGENTS.md), not convention.
- **Spec/impl agreement.** All 7 block kinds render in
  `crates/fireside-tui/src/render/blocks.rs`, including `highlight-lines`,
  `show-line-numbers`, and all three container layouts. Columns fall back to
  stack below a width threshold — small-terminal behavior is designed, not
  accidental (`narrow_columns_fall_back_to_stack` test).
- **Validation parity by design.** `fireside-engine/src/validation.rs`
  mirrors `protocol/validate.mjs` rule-for-rule: unique-node-ids,
  valid-traversal-target, next/branch-point conflict, branch-options,
  reachability, self-loops, trivial-cycles, dead-end-branch. Parity is
  *claimed* via matching rule names but not *tested* (see §3).
- **Test posture.** 100 tests: engine semantics unit-tested in
  `session.rs`/`validation.rs`, TUI scenario suite drives real key events
  through `App::update` against `TestBackend`, CLI covered e2e, plus tmux
  smoke for real terminals.

### Known, deliberate gaps (ADR-004 follow-ups)

- Images render as a text placeholder box (`blocks.rs:54`) — terminal image
  rendering named as a "deliberate polish follow-up."
- No authoring surface at all; `fireside new` emits a starter deck, then the
  presenter is on their own in a JSON file.
- `fade` transition and syntax highlighting have shipped since ADR-004;
  image rendering is the remaining named follow-up.

### Competitive survey (what presenters expect elsewhere)

| Tool | Format | Features Fireside lacks |
|---|---|---|
| presenterm (Rust) | Markdown | kitty/sixel/iterm2 images, incremental reveal (pauses), PDF export, speaker-notes second window |
| slides (Go) | Markdown | Markdown authoring, code execution blocks |
| patat (Haskell) | Pandoc | incremental reveal, auto-advance, wrap/margins config |
| sli.dev (web) | Markdown | presenter view, recording, themes ecosystem |

**Fireside's unique position:** nobody else does graph-structured, branching
presentations. That is the moat. The table above says the costs of entry are
(a) Markdown-adjacent authoring, (b) real images, (c) incremental reveal.

### Protocol ambiguities found in this audit (spec, `protocol/main.tsp`)

1. **BranchOption.key uniqueness is unspecified.** Two options with key
   `"a"` at one branch point: validator silence, engine behavior undefined.
   Needs a validation rule (error or warning) in both validators.
2. **Empty Traversal object.** `"traversal": {}` (neither `next` nor
   `branch-point`) — is it a terminal node or invalid? The spec defines the
   string, object, and absent forms but not the empty object. Recommend: a
   validator warning, engine treats as terminal.
3. **choose() contract.** §Operations says "push and navigate" but never
   states the option must belong to the *current* node's branch point.
   Implementations could accept a forged option. One sentence fixes it.
4. **ViewMode toggle persistence.** Spec says engines SHOULD allow toggling;
   it doesn't say whether the toggle persists across node transitions or
   resets to the node/default value. The engine has picked a behavior —
   document it in Appendix B.
5. **ListBlock items and Markdown.** TextBlock explicitly allows inline
   Markdown; ListBlock's `items: string[]` is silent. The renderer has made a
   choice; the spec should state it.
6. **Image width/height overflow.** `width: 500` in an 80-column terminal:
   clamp, clip, or reject? Currently engine-defined; Appendix B should say
   "engines MUST clamp to the content area."
7. **History growth is unbounded** (long presentations with goto loops).
   Non-issue in practice; worth one line in Appendix B ("engines MAY cap").

None of these are wire-format changes — they are spec-text and validator
additions, all 0.1.x-safe.

---

## 2. Prioritized Improvements

### P0 — Authoring path (the ADR-004 acknowledged debt)

The presenter-first north star currently ends at "present." A non-technical
person cannot *make* a deck. Staged approach, cheapest-first, each stage
useful on its own:

- **Stage A (days): `fireside validate --watch`.** Reuse the existing file
  watcher (`main.rs::Watcher`) + caret parse errors. The authoring loop
  becomes: editor on the left, watch pane on the right, errors appear as you
  save. Zero new architecture.
- **Stage B (days): richer `fireside new`.** Interactive scaffold — ask
  title/author, offer 2–3 templates (linear talk, branching demo, workshop),
  emit commented example nodes. The starter deck already exists
  (`main.rs::starter_deck`); this is prompts + templates.
- **Stage C (week+): quick-edit in the TUI.** A modal that edits the current
  node's text/heading blocks in place and writes the file (which live-reload
  then picks up — the round-trip already works). Not a full editor: no
  structural edits, no undo. This needs **ADR-005** because ADR-004
  explicitly deleted the editor; the ADR should scope what "editor returns"
  means and what it forever excludes.
- **Stage D (later, own decision): Markdown authoring frontend.** A
  `fireside import deck.md` compiler (headings→nodes, a fence syntax for
  branch points) that emits protocol JSON. This attacks the authoring gap
  without touching the wire format and matches how every competitor authors.
  Prototype before committing — the branch-point syntax is the hard part.

### P1 — Terminal images + bounded ASCII art (the deferred polish)

- **ASCII art, bounded to the window.** Two layers:
  - *Engine-only (no spec change):* code blocks whose language is `text`,
    `ascii`, or absent already render monospace; add centering and
    graceful horizontal clipping with the existing ellipsis marker
    (`blocks.rs` clip helpers). Ship immediately.
  - *Spec-first (0.1.x additive):* an optional `fit?: "clip" | "center" | "shrink"`
    field on CodeBlock, specified in main.tsp and registered per the
    extension process. Old engines ignore unknown fields (ADR-004 guarantees
    this), so it is backwards compatible. `shrink` for ASCII art means
    dropping to a braille/half-block downscale only if trivially achievable —
    otherwise clip+center is honest and portable.
- **Real images.** `ratatui-image` (kitty graphics / sixel / iTerm2 /
  half-block fallback) behind capability detection, with the current
  placeholder as the universal fallback. **Flags:** (1) adds `image` +
  `ratatui-image` to `fireside-tui`'s permitted deps — AGENTS.md boundary
  table needs a deliberate amendment; (2) MSRV ≤ 1.88 must be verified before
  adoption; (3) protocol already has `width`/`height` in cells, so no spec
  change needed beyond the clamp rule (ambiguity #6).

### P1 — Incremental reveal ("fragments")

The single most-expected presenter feature Fireside lacks. Requires spec
work: either a `reveal?: int32` field on ContentBlock variants (0.1.x
additive, ignored by old engines — degrades to "everything visible," which is
correct) or a first-class step model in 0.2.0. Recommend the additive field,
specified first, with `next()` consuming reveal steps before advancing nodes.
This changes the `next()` contract, so it must go through the spec, not ship
as engine behavior.

### P2 — Modern TUI leverage

- **Mouse support** where it's discoverable: click a map node to goto, click
  a branch option to choose. Keyboard remains the primary contract (footer
  teaches keys); mouse is additive. Crossterm mouse capture is one flag.
- **Synchronized output** (`BeginSynchronizedUpdate`) to eliminate any
  transition flicker — cheap, invisible when unsupported.
- **OSC 8 hyperlinks** for link-bearing text blocks.
- **Resume**: persist last position per deck (a dotfile keyed by content
  fingerprint — `main.rs::fingerprint` already exists) so a crashed or
  interrupted presentation reopens where it left off. Very presenter-first.
- *Not recommended now:* kitty keyboard protocol, background images, heavy
  animation — cost exceeds presenter value.

### P2 — Protocol & workflow hardening

- **Shared conformance fixture corpus.** `protocol/fixtures/{valid,invalid}/*.json`
  consumed by *both* `validate.mjs` and the Rust validation tests, asserting
  identical rule IDs fire. Turns the claimed parity into a tested invariant,
  and becomes the seed of a conformance suite any third-party engine can run.
- **Property tests** (proptest, dev-dep only): serde round-trip on arbitrary
  Graphs; Session invariants (history reflects actual path, visited ⊆ nodes)
  under arbitrary op sequences.
- **Robustness fixtures:** deep container nesting (spec says "engines MAY
  impose limits" — pick one and test it), multi-codepoint/emoji/CJK width in
  headings and columns, 1,000-node deck load time, rapid reload with
  half-saved (invalid) JSON mid-edit.
- **CI additions:** `cargo msrv verify` (the 1.88 promise is currently
  untested), `cargo deny` already present via deny.toml — confirm it runs in CI.

### P3 — Later, each behind its own decision

Export (HTML via the docs renderer, or PDF), second-terminal speaker view,
theme variants, `goto` fuzzy-search by title. Explicitly out of phase 1.

---

## 3. Phase 1 Roadmap (2–3 weeks)

**Week 1 — Spec hardening + quick authoring wins**
1. Spec patch 0.1.1: fix the 7 ambiguities (§1 above) in `main.tsp` +
   `docs/src/content/docs/spec/`; add `branch-option-key-uniqueness` and
   `empty-traversal` rules to both validators; regenerate `tsp-output/`.
2. Shared fixture corpus wired into both test suites.
3. `fireside validate --watch` + interactive `fireside new` templates.
4. ASCII art engine-side: center + clip code blocks; scenario tests at 80×24.

**Week 2 — Images + ADR-005**
5. Spike `ratatui-image`: MSRV check, terminal coverage matrix (kitty,
   iTerm2, sixel, plain), fallback behavior. Go/no-go by mid-week.
6. If go: image rendering behind capability detection, clamp rule, scenario
   tests with the placeholder fallback path.
7. Write ADR-005 (authoring returns, scoped) — decide Stage C vs jumping to
   Stage D based on Week 1 learnings.
8. Docs: CLI reference page + keyboard reference (see §5).

**Week 3 — Reveal + editor stage C + polish**
9. Spec 0.1.x: `reveal` field proposal; implement `next()` step-consumption
   behind it; scenario tests (footer must show reveal state — every keypress
   gets feedback per the Outcome rule).
10. Quick-edit modal (if ADR-005 approved) or Markdown-import prototype.
11. Mouse on map/branch menu + synchronized output.
12. Resume-from-fingerprint.

**Definition of done for the phase:** a non-technical presenter can scaffold
a deck, see errors live while editing, present it with images and an ASCII
diagram at 80×24, and reveal bullets one at a time — verified by driving the
real TUI (tmux smoke), not just TestBackend.

---

## 4. Open Questions / Prototyping Needed

1. **ratatui-image MSRV and dep weight** — hard gate; verify before any code.
2. **Markdown branch-point syntax** — no prior art for branching in Markdown
   decks; needs a throwaway prototype and a taste test.
3. **`reveal` semantics at branch points** — do reveal steps precede the
   branch menu? Spec question to settle before implementing.
4. **Quick-edit write-back** — writing JSON preserving the author's key
   order/formatting is lossy with serde_json; decide whether canonical
   reformatting on save is acceptable (probably yes — document it).
5. **ASCII `shrink` fit** — is braille downscaling worth it, or is
   clip+center the honest answer? Prototype only if cheap.

---

## 5. Documentation Audit & Plan

**Accurate today:** spec §1–§4, §6 mirror main.tsp; hello.json is canonical
and tested; mental-models and vocabulary pages are unusual strengths.

**Gaps, in priority order:**
1. **No CLI reference.** present/validate/new/demo, exit codes, `--watch`.
   Generate the skeleton from clap definitions to prevent drift.
2. **No keyboard/presenting guide.** The footer teaches keys in-app; the docs
   should have the same table plus map/notes/fullscreen/goto workflows —
   this is the page a nervous presenter reads the night before.
3. **Sidebar skips §5** (astro.config.mjs jumps §4 → §6). Either restore the
   missing section or renumber — as published it looks like an error.
4. **One guide total.** Add "Authoring a branching deck" (hand-written JSON
   today, updated as P0 lands) and "Images and ASCII art" once P1 ships.
5. **No conformance page.** Once the fixture corpus exists, document how a
   third-party engine claims conformance — this is what makes the protocol
   real beyond the reference implementation.
6. **Spec appendices** get the ambiguity resolutions (clamp rule, history
   cap, ViewMode persistence) rather than burying them in engine code.

Structure already serves the three audiences (guides / spec / reference);
no reorganization needed — only filling.
