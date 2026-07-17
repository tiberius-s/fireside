# Fireside Strategic Improvement Plan — 2026-07-12

## Progress Log

_Update this section (don't just rely on git log) whenever a plan item lands
or starts. One line per item: status, commit(s), date._

- [x] P0 Stage A — `fireside validate --watch` — landed `580fc0d` (spec at
      `specs/001-validate-watch/`), 2026-07-12.
- [x] P0 Stage B — interactive `fireside new` templates — landed `a96a590`,
      2026-07-12 (no spec dir; went through implementation directly).
- [x] P0 Stage C — quick-edit modal in TUI — landed 2026-07-12 (uncommitted
      on main). ADR-005 (`.claude/adrs/adr-005-quick-edit-modal-scope.md`)
      scoped it; full speckit pipeline run at `specs/002-quick-edit-modal/`
      (spec → plan → tasks → implement, all 26 tasks done). Shipped:
      `Screen::Edit` modal in `fireside-tui` (open with `e`, edits
      heading/text blocks incl. nested in containers, `Ctrl+S`/`Esc`),
      `present_authoring`/`WriteBackSink`/`WriteBackError` in
      `fireside-tui`'s public API (present/present_watching now thin
      wrappers), `Watcher::write_back` in `fireside-cli` reusing the
      existing mtime+size fingerprint for conflict detection. 8 new tests,
      123 total passing, clippy silent. Two real bugs found only by the
      manual real-terminal (tmux) smoke walk — neither caught by the
      TestBackend suite — and fixed: (1) `write_back` was resyncing its
      fingerprint on success, which made the *next* poll see "no change"
      and silently skip the reload, leaving stale content on screen after
      a successful save; fixed by leaving the fingerprint stale on success
      so the ordinary reload path picks it up, exactly like an external
      edit. (2) a same-tick ordering race: reload was checked before the
      pending-save was consumed, so on the tick where Ctrl+S flips
      `Screen::Edit` back to `Present`, reload ran first, resynced the
      fingerprint to any external change, and the conflict check then saw
      no conflict — silently overwriting a concurrent external edit with
      no warning; fixed by handling the pending save before the reload
      check every iteration, and by keeping the modal open (edit intact)
      on any save failure so a conflict is retryable (Ctrl+S again to
      overwrite, Esc to abandon) rather than a silent loss either way.
      **Lesson for next time:** for any feature involving the live-reload
      loop or file-watch timing, run the tmux smoke walk *before* declaring
      done — TestBackend tests exercise `App` in isolation and cannot catch
      event-loop ordering or fingerprint-timing bugs.
- [x] P0 Stage D — Markdown authoring frontend (`fireside import`) — landed
      2026-07-12 (uncommitted on main). Branch-fence syntax (link list in a
      ` ```branch ` fence) chosen via a user taste-test of three concrete
      mockups before ADR-006 (`.claude/adrs/adr-006-markdown-import.md`)
      was written; full speckit pipeline at `specs/003-markdown-import/`
      (spec → plan → tasks → implement, all 27 tasks done). Added
      `pulldown-cmark` to `fireside-cli`'s permitted deps (constitution
      v1.0.0 → v1.1.0, MSRV-1.88-verified). Shipped: new
      `crates/fireside-cli/src/import.rs` module — `##` headings become
      nodes (slugified, deduped ids), H3-H6/paragraphs/code/lists/images/
      dividers convert to content blocks, byte-range source slicing
      preserves inline Markdown verbatim in paragraphs, a two-pass design
      (collect ids, then build+resolve) supports branch links to
      later-appearing sections, nested lists and unresolved/malformed
      branch syntax are rejected with a line number rather than silently
      mishandled, output is validated with the existing
      `fireside_engine::validate` before any write. New `fireside import
      <input.md> [output]` CLI verb. 24 new tests (14 unit in `import.rs`,
      2 CLI e2e, plus the pre-existing `slugify` refactor test), 139 total
      passing, clippy silent. Verified end-to-end in a real terminal
      (tmux): linear import, branching import (including pressing the
      author-declared hotkey and watching the route trace), an unresolved
      branch target, a nested list, and a headingless document all produce
      the exact messages/behavior specified.
- [x] Week 1 spec patch 0.1.1 (7 ambiguities) + validator rules — landed
      2026-07-12 (uncommitted on main). ADR-007
      (`.claude/adrs/adr-007-spec-patch-0-1-1.md`) recorded the decision
      after reading the actual reference implementation: six of the seven
      audit ambiguities (branch-key uniqueness severity, empty-traversal
      terminal handling, choose() option scoping, ViewMode persistence,
      list-item Markdown, unbounded history) turned out to already be
      settled reference behavior, just undocumented (or, for branch-key
      uniqueness, documented at the wrong severity in `validation.md`) —
      only one ambiguity (the empty traversal object `{}`) needed new
      validator code. Full speckit pipeline at
      `specs/004-spec-patch-0-1-1/` (spec → plan → tasks → implement, all
      35 tasks done). Shipped: protocol version 0.1.1 (`Versions` enum in
      `main.tsp` gains `v0_1_1`, purely additive, `tsp-output/`
      regenerated); a new symmetric `empty-traversal` WARNING rule in both
      `fireside-engine/src/validation.rs` and `protocol/validate.mjs`
      (verified byte-for-byte matching diagnostics between the two CLIs);
      `validation.md`'s `unique-branch-keys` doc fix (was misclassified as
      "Recommended," now correctly "Required" matching its actual Error
      severity); spec-text additions across `traversal.md`,
      `appendix-engine-guidelines.md`, `appendix-content-blocks.md`, and
      `main.tsp` doc comments covering all seven ambiguities. 4 new tests
      (3 Rust unit tests + 1 fixture-corpus integration test), 143 total
      passing, clippy silent, docs site `astro check` clean.
- [x] Week 1 shared fixture corpus — landed 2026-07-12 alongside the spec
      patch above (same feature/ADR, bundled since the corpus exists to
      test the rules the patch touches). `protocol/fixtures/{valid,invalid}/*.json`
      (10 fixtures, each isolating exactly one Layer-2 rule) plus a single
      `protocol/fixtures.expected.json` read by BOTH a new Rust
      integration test (`crates/fireside-engine/tests/fixtures.rs`) and a
      new Node script (`protocol/run-fixtures.mjs`, wired as
      `npm run test:fixtures --prefix protocol`) — turning "the Rust and
      Node validators agree" from a claimed invariant (matching rule-name
      strings) into a tested one (identical rule-id sets per fixture).
      Required exporting `validate` from `protocol/validate.mjs` and
      guarding its `main()` call behind an `import.meta.url` check so the
      module can be imported without hijacking the process — a real bug
      the corpus work surfaced immediately. **Verified the corpus actually
      catches divergence**, not just passes: deliberately renamed a rule
      string in only the Rust validator, confirmed the fixture test failed
      with a clear mismatch message, then reverted.
- [x] Week 1 ASCII art engine-side (center/clip) — landed 2026-07-12
      (uncommitted on main). No ADR (no crate boundary/dependency/protocol
      change — pure rendering-quality fix). Full speckit pipeline at
      `specs/005-ascii-art-centering/` (spec → plan → tasks → implement,
      all 16 tasks done). Shipped: `crates/fireside-tui/src/render/blocks.rs`'s
      `code()` now classifies a code block as ASCII art when its language
      is absent, `"text"`, or `"ascii"`; ASCII-art blocks size their box
      (top rule, every content row, bottom rule) to their own natural
      content width and center that box within the available width via a
      uniform leading pad on every line, while any other language keeps
      today's full-width left-aligned rendering byte-for-byte unchanged.
      Oversized ASCII art caps to the available width and clips with the
      existing ellipsis marker — no new clipping logic, reused
      `clip`/`clip_spans` as the plan required. 7 new tests (6 unit in
      `blocks.rs`, 1 `TestBackend` scenario at 80×24), 150 total passing,
      clippy silent. Verified visually in a real terminal (tmux) at
      80×24: a small ASCII cat face renders as a compact, genuinely
      centered box inside the card, not stretched — confirming the
      assertions match what a presenter would actually see. This closes
      out all of Week 1 (spec patch, fixture corpus, and ASCII art).
- [x] P1 terminal images (`ratatui-image`) spike — **NO-GO**, 2026-07-12.
      ADR-008 (`.claude/adrs/adr-008-ratatui-image-msrv-spike.md`) records
      the finding: every `ratatui-image` release compatible with
      Fireside's current `ratatui 0.30` unconditionally (not
      feature-gated, not pinnable around) pulls in `icy_sixel` →
      `quantette` → `wide`/`safe_arch`, which require rustc 1.89/1.90.
      Verified empirically with a real `cargo +1.88 build` (not just
      declared `rust-version` metadata) — it fails outright. The only
      MSRV-1.88-compatible version (`ratatui-image` v8.x) forces a
      `ratatui` downgrade to 0.29, incompatible with the 0.30 already used
      throughout `fireside-tui` — rejected as disproportionate. Terminal
      images remain a placeholder-box gap, unchanged from ADR-004. No
      code changed; this was a pure spike in a throwaway scratch project.
      Re-spike only if Fireside's MSRV rises past 1.90 or `icy_sixel`
      ships a lower-MSRV release.
- [x] P1 incremental reveal (`reveal` field) — landed 2026-07-12
      (uncommitted on main). ADR-009
      (`.claude/adrs/adr-009-incremental-reveal.md`) recorded the design
      before any code, resolving the plan's two open questions: reveal
      always fully precedes branch-point/next-target checks on `next()`
      (unconditionally, even on terminal nodes), and steps are ordinal
      over the distinct positive `reveal` values actually used in a
      node's content — not raw magnitudes — so a gap in an author's
      numbering (e.g. `1` then `3`) can never produce a keypress that
      reveals nothing. Full speckit pipeline at
      `specs/006-incremental-reveal/` (spec → plan → tasks → implement,
      all 50 tasks done). Shipped: `reveal?: int32` (`@minValue(0)`)
      spread via a shared `Revealable` TypeSpec model into all seven
      `ContentBlock` variants, protocol bumped to 0.1.2 (additive);
      `fireside-core::Node::reveal_levels()` (pure, recursive, walks
      `Container` children); `fireside-engine::Session` gained
      `Outcome::Revealed`, a `reveal_level` field reset on every node
      entry (`next`/`choose`/`goto`/`back`, including `back`'s own
      bypass of the shared navigation helper), and `next()`/`choose()`
      both gate correctly on pending reveal (`choose()` itself now
      rejects while reveal is pending, not just the TUI's key routing);
      `fireside-tui` hides not-yet-revealed blocks structurally (no
      reserved layout space — verified specifically inside a `columns`
      container), shows a "N/M revealed" footer badge only while
      pending, and — a real gap caught while writing the branch-point
      scenario test, not anticipated in the plan — routes *any*
      branch-selection keypress (not just the generic "next" key) to
      continue revealing instead of silently doing nothing, per FR-007.
      New symmetric `reveal-masked-by-container` WARNING rule in both
      validators, extending the existing fixture corpus (12 fixtures
      now, up from 10). 19 new tests (169 total passing), clippy silent,
      docs site `astro check` clean. Verified live in tmux: bullets
      reveal one Space-press at a time with correct footer feedback, the
      branch menu stays hidden until reveal is exhausted, and `Enter`
      then correctly chooses once it appears — plus a separate tmux
      check confirming `hello.json` (no reveal marks) renders byte-for-
      byte unchanged, no footer badge.
- [X] P2 mouse / synchronized output / resume / OSC 8 hyperlinks — done
      2026-07-17. Full speckit pipeline at `specs/007-modern-tui-leverage/`,
      50/50 tasks done, 188 tests total (up from 169), clippy silent. Four
      independent stories, no shared Foundational phase (a first for this
      repo): (1) mouse — click a map row or branch option, hit-tested via
      pure layout functions shared with `render::draw` so a click can never
      disagree with what's on screen; verified live in tmux via injected
      SGR mouse escape sequences (`tmux send-keys -H`). (2) resume — a
      host-local `resume.json` (XDG state dir, manual `std::env`/`std::path`
      construction, no new dependency) keyed by the existing content
      fingerprint; `fireside-tui` gained `initial_node`/`PositionSink`
      params on `present_authoring` (no file I/O in the crate itself, per
      the boundary table); verified live in tmux: SIGKILL mid-deck then
      relaunch reopened on the same slide, reaching the terminal node
      cleared the record. (3) synchronized output — `BeginSynchronizedUpdate`/
      `EndSynchronizedUpdate` bracket every `terminal.draw`, no capability
      query needed (inert-if-unsupported by the escape sequence's own
      design). (4) OSC 8 hyperlinks — new `[label](url)` inline syntax
      (engine-extension latitude, no protocol/schema change), rendered via
      `ratatui-core`'s `CellDiffOption::ForcedWidth` (confirmed live in tmux
      via a raw hexdump of the actual escape bytes sent to the terminal);
      required bumping the transitively-resolved `ratatui-core` 0.1.0 →
      0.1.2 (`ForcedWidth` doesn't exist in 0.1.0) — flagged mid-session and
      approved by the user, still MSRV-1.88-safe, ~7 new transitive crates
      compiled. New symmetric `malformed-link-url` WARNING rule in both
      validators, fixture corpus now 14. One discovered nuance: a
      multi-word link label renders as separately-OSC8-wrapped adjacent
      regions (word-wrap splits it into multiple spans), not one continuous
      region — still fully clickable across the whole label, just
      structurally split. Not tested against an actual OSC-8-incapable
      terminal (none available in this environment); the fallback claim
      rests on the escape sequence being spec-inert when unrecognized, the
      same reasoning already used for synchronized output and the `fade`
      transition.
- [ ] P2 protocol & workflow hardening (property tests, robustness fixtures,
      CI additions) — not started.

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
  _claimed_ via matching rule names but not _tested_ (see §3).
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

| Tool              | Format   | Features Fireside lacks                                                                         |
| ----------------- | -------- | ----------------------------------------------------------------------------------------------- |
| presenterm (Rust) | Markdown | kitty/sixel/iterm2 images, incremental reveal (pauses), PDF export, speaker-notes second window |
| slides (Go)       | Markdown | Markdown authoring, code execution blocks                                                       |
| patat (Haskell)   | Pandoc   | incremental reveal, auto-advance, wrap/margins config                                           |
| sli.dev (web)     | Markdown | presenter view, recording, themes ecosystem                                                     |

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
   states the option must belong to the _current_ node's branch point.
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
person cannot _make_ a deck. Staged approach, cheapest-first, each stage
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
  - _Engine-only (no spec change):_ code blocks whose language is `text`,
    `ascii`, or absent already render monospace; add centering and
    graceful horizontal clipping with the existing ellipsis marker
    (`blocks.rs` clip helpers). Ship immediately.
  - _Spec-first (0.1.x additive):_ an optional `fit?: "clip" | "center" | "shrink"`
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
- _Not recommended now:_ kitty keyboard protocol, background images, heavy
  animation — cost exceeds presenter value.

### P2 — Protocol & workflow hardening

- **Shared conformance fixture corpus.** `protocol/fixtures/{valid,invalid}/*.json`
  consumed by _both_ `validate.mjs` and the Rust validation tests, asserting
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

**Week 2 — Images + ADR-005** 5. Spike `ratatui-image`: MSRV check, terminal coverage matrix (kitty,
iTerm2, sixel, plain), fallback behavior. Go/no-go by mid-week. 6. If go: image rendering behind capability detection, clamp rule, scenario
tests with the placeholder fallback path. 7. Write ADR-005 (authoring returns, scoped) — decide Stage C vs jumping to
Stage D based on Week 1 learnings. 8. Docs: CLI reference page + keyboard reference (see §5).

**Week 3 — Reveal + editor stage C + polish** 9. Spec 0.1.x: `reveal` field proposal; implement `next()` step-consumption
behind it; scenario tests (footer must show reveal state — every keypress
gets feedback per the Outcome rule). 10. Quick-edit modal (if ADR-005 approved) or Markdown-import prototype. 11. Mouse on map/branch menu + synchronized output. 12. Resume-from-fingerprint.

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
