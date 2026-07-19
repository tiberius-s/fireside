# Fireside — UX Polish, ASCII Art Quality & Docs Restructure Plan (2026-07-18)

Source: hands-on QA session 2026-07-18 (tmux-driven walkthrough of the demo,
present/new/import/validate/art verbs, live-reload failure paths, resume,
quick-edit, small-terminal degradation) plus a full docs-site audit.
Predecessor: `.claude/plans/2026-07-17-research-and-improvement-plan.md`
(P0/P1 complete; this plan supersedes its remaining "next" items where they
overlap).

## Progress Log

_Update this section whenever an item lands or starts. One line per item:
status, date._

- [x] W1-1 demo deck reserved-key fix — 2026-07-18, key changed `e`→`o`, demo.tape + demo.gif re-recorded
- [x] W1-2 help overlay wrapping — 2026-07-18, overlay now sizes to content instead of a fixed 50 cols; two longest labels shortened; snapshot updated
- [x] W1-3 diagnostics pluralization — 2026-07-18, `report.rs` emits "1 error"/"2 errors"/"0 errors" via new `plural()` helper
- [x] W1-4 friendly file-not-found for present — 2026-07-18, `load()` in main.rs now prints `No deck named X — "fireside new <stem>" creates one.` (also benefits `validate`)
- [x] W2 (spec 010) presenter-polish feature — 2026-07-18, all 5 stories shipped (specs/010-presenter-polish/): reserved-branch-key validator warning (Rust+Node, fixture-parity proven), exit summary, resume toast, wizard present-now prompt, `art text` width guard. `scripts/verify.sh` green.
- [x] W3 (spec 011) art image quality — 2026-07-18, `fireside art image` now
      applies a 2nd/98th-percentile contrast stretch by default (ADR-013,
      constitution → 1.3.0 for the new direct `image` dependency), plus
      `--charset <default|block|slight>`, `--invert`, and `--no-normalize`
      flags, and a stderr warning on unusually low-contrast sources.
      `.github/demo-art.png` replaced with a high-contrast CC0 sunset/tree
      silhouette (was a muddy night photo at ~30% of the brightness range;
      new image at ~66%), `art-image.gif` re-recorded. `scripts/verify.sh`
      green.
- [ ] W4 docs restructure — not started

## Ground rules

- Per `CLAUDE.md`: pure bug fixes / mechanical chores (Wave 1, Wave 4) skip
  the Spec Kit pipeline; behavior-adding work (Waves 2–3) goes through
  `/speckit-specify` → `/speckit-plan` → `/speckit-tasks` →
  `/speckit-implement` with artifacts in `specs/NNN-*/`.
- Any TUI-visible change gets a real tmux smoke test before "done"
  (TestBackend tests miss timing/ordering bugs — see memory
  `feedback_tmux_smoke_catches_timing_bugs`).
- Before handoff: `scripts/verify.sh` (mirrors every CI job), plus
  `graphify update .` after code changes.
- GIF regeneration: `scripts/demos.sh` (vhs; tapes in `.github/*.tape`).

## Wave 1 — bug fixes (no pipeline, one small PR-sized batch)

**W1-1: Demo deck advertises a dead shortcut.**
`crates/fireside-cli/assets/demo.fireside.json:117` gives the first branch
option `"key": "e"`, but global `e` (quick-edit) shadows it — pressing `e`
on the demo's branch slide opens the editor. Change to a non-reserved letter
(`o`); leave `w` as-is. Reserved set to avoid: `e f g h j k m n p q s t`.
Re-record `demo.gif` only if the branch slide's `[e]` badge is visible in it
(check `demo.tape` output before bothering).
Accept: tmux smoke — pressing the advertised key on the demo branch slide
takes the branch.

**W1-2: Help overlay truncates mid-word.**
At 100×30 the `?` overlay clips two rows ("…branch option to│",
"…heading/text on th│"). Overlay rendering lives in
`crates/fireside-tui/src/render/overlays.rs`. Either widen to longest-line
width when space allows, or wrap rows at the overlay's inner width. Also
shorten the two offending label strings — they're the longest by far.
Accept: no clipped glyphs at 80×24 and 100×30; snapshot test updated.

**W1-3: Pluralization.**
`validate` prints "1 error(s), 0 warning(s), 1 note(s)"
(`crates/fireside-cli/src/report.rs`). Emit "1 error", "2 errors", etc.
Accept: unit test on the summary line for 0/1/n.

**W1-4: Friendly file-not-found for present.**
`fireside nope.fireside.json` prints a raw anyhow chain
(`crates/fireside-cli/src/main.rs:188`). When the path doesn't exist, print
one plain-language line and suggest the fix: `No deck named
nope.fireside.json — "fireside new nope" creates one.` Keep exit 1.
Accept: cli_e2e test asserting message shape.

## Wave 2 — Spec Kit feature 010: presenter polish (small, behavior-adding)

One spec covering four related presenter/authoring feedback gaps. All footer
messaging goes through the existing flash mechanism
(`crates/fireside-tui/src/app.rs`, `set_flash`).

- **010-a Resume toast.** On launch that resumes mid-deck, flash
  `Resumed where you left off — --restart starts over`. Silent resume
  disorients presenters who forgot they quit mid-run.
- **010-b Exit summary.** On `q`, print one line after the TUI closes:
  `Presented 5/7 slides in 12:30.` Free rehearsal feedback; uses data the
  app already tracks (seen count, timer).
- **010-c Reserved-key validator warning.** New Layer-2 rule
  `reserved-branch-key` (warning severity) in
  `crates/fireside-engine/src/validation.rs`, next to `unique-branch-keys`
  (~line 182): a branch option `key` in the reserved presenter set can never
  fire. Document in `docs/src/content/docs/spec/validation.md` (rule list at
  ~L68) and note engine-specificity in Appendix D if required by the
  protocol-workflow rules from spec 008.
- **010-d `art text` width guard.** When banner width exceeds 76 columns
  (the `ascii-art-too-wide` threshold), print a note to stderr with the
  measured width — matching `new --banner`'s existing skip-note behavior.
  Stdout art unchanged (still pasteable).
- **010-e Wizard momentum.** Interactive `fireside new` ends with
  `Present it now? [Y/n]` and execs present on yes
  (`crates/fireside-cli/src/new.rs`).

Verify: tmux smoke for a/b/e; validation unit tests + docs for c; e2e for d.

## Wave 3 — Spec Kit feature 011: `art image` output quality

Root cause of "undecipherable" output, verified experimentally this session:
`.github/demo-art.png` occupies only ~29% of the luminance range (2–98th
percentile: 3–77 of 255), and `render_image_ascii`
(`crates/fireside-cli/src/art.rs:46`) passes rascii_art defaults — linear
grayscale→charset mapping lands almost every cell in the two darkest glyphs.
A percentile contrast stretch (2%/98% clip) + the `block` charset produced a
clearly legible flame from the same photo (experiment in session scratchpad,
`asciitest`).

Scope:

1. **Auto contrast-stretch by default.** Load via the `image` crate (pin to
   the same 0.24.x rascii_art uses — version mismatch is a compile error),
   stretch, feed `rascii_art::render_image_to`. Escape hatch:
   `--no-normalize`.
2. **`--charset <default|block|slight>`** and **`--invert`** flags —
   rascii_art 0.4.5 supports both already; just surface them.
3. **Low-range warning.** If the pre-stretch 2–98% span is under ~40% of
   full range, print a stderr note ("this image uses N% of its brightness
   range — results may be muddy; try --invert or a higher-contrast image").
4. **Replace the demo image.** Even fixed, a busy night photo is the wrong
   showcase. Pick a high-contrast, simple-silhouette CC0 subject; keep the
   input-photo-next-to-output presentation in `reference/cli.md` (it's a
   good pattern). Re-record `art-image.gif` via `scripts/demos.sh`.
5. Docs: `reference/cli.md` flag table + `guides/authoring-markdown.md`
   pointer; `new --banner` path unaffected (text banners don't normalize).

Accept: converting `demo-art.png` with defaults yields output where the fire
is recognizable; goldens for stretch math; e2e for flags and warning.

## Wave 4 — docs restructure (docs chore, no pipeline; biggest UX lever)

The site is implementer-first; the audience is presenter-first. Individually
strong pages (`presenting.md`, `authoring-markdown.md`, `cli.md`) are
undermined by the front door.

1. **New page `guides/quickstart.md`** — the README flow, expanded:
   install (`cargo install --path crates/fireside-cli` + truecolor/terminal
   requirements) → `fireside demo` → `fireside new` → present → live-edit →
   `import`. Today `cargo install` appears **nowhere** on the docs site.
2. **Sidebar reorder** (`docs/astro.config.mjs`): Guides → Reference →
   Specification. Quickstart first in Guides.
3. **Landing page rewrite** (`index.md`): open with what a presenter can do;
   "Start Here" routes to quickstart → presenting → authoring-markdown;
   move the spec reading order into a "For implementers" subsection.
4. **Retitle/reframe `getting-started.md`** ("Your First Fireside Graph") as
   the hand-written-JSON deep-dive it actually is; fix its "Run it" section
   to give real commands (`fireside validate my-graph.fireside.json`,
   `fireside my-graph.fireside.json`); cross-link quickstart.
5. **Appendix lettering**: appendices run B, C, D with no A. Either restore
   A or reletter to A, B, C (update sidebar labels + inbound links).
6. Sweep cross-links after the moves (`npm run build` catches broken ones).
7. **Document `image` vs. `ascii-art` blocks (new, surfaced 2026-07-19
   during spec 011 follow-up).** Neither `getting-started.md` nor
   `authoring-markdown.md` currently explains that a plain `image` block
   (`{"kind": "image", "src": ...}`) renders in the terminal as only a
   labeled placeholder frame — no real pixels, a deliberate decision
   (ADR-008: real image rendering is NO-GO) — while `ascii-art` (generated
   via `fireside art text`/`art image`, spec 009/011) is the only way to
   get a photo or banner *visually* into a presented deck.
   `authoring-markdown.md`'s existing "ASCII art" section is three
   sentences with no worked example. Expand it with a full loop (source
   image → `fireside art image` → pasted fence → rendered slide) and add a
   short callout to `getting-started.md` (or its retitled replacement, item
   4 above) clarifying the placeholder-only behavior of plain `image`
   blocks, so a new user doesn't write one expecting to see their photo.

Verify: `docs` CI job / `npm run build` clean; read-through of the new
front-door path start to finish.

## Deferred / watchlist

- **Choice-key precedence.** Global keys shadowing branch keys is contained
  by 010-c; if the global key set ever grows, revisit letting choice keys
  win on branch slides. Needs its own spec — traversal-adjacent semantics.
- **`↓` double-duty** on an overflowing branch slide (scroll vs. selection)
  — cosmetic today; fold into any future scrolling rework.
- **`art text` font options / auto-shrink** — only if users hit the width
  guard often.

## Suggested order

W1 (one sitting, immediate wins) → W3 (highest complaint, self-contained) →
W4 (docs, parallelizable with W3) → W2 (nice-to-haves batch). Each wave
ships independently; nothing blocks anything else except W2's 010-c which
should land before or with any new global keybindings.
