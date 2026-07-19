# Fireside — Full-Project UX Audit (2026-07-19, Fable)

Source: hands-on audit session 2026-07-19 — tmux-driven walkthrough of every
CLI verb (demo, present, new interactive + templated, import, validate, art
text/image), degraded-condition testing (44×14 / 40×12 / 80×24 / resize /
SIGKILL / no-truecolor / non-tty), live-reload and quick-edit failure paths,
resume-key experiments, a Markdown-import stress corpus, a control-character
"evil deck" probe, a 500-node performance deck, plus source review of
`app.rs`/`lib.rs`/`resume.rs`/`watch.rs`/`main.rs`/`validation.rs`/`blocks.rs`,
workspace deps, CI workflows, and a docs-site walk.

Predecessors: `.claude/plans/2026-07-17-research-and-improvement-plan.md`
(streams A–F, all closed or descoped) and
`.claude/plans/2026-07-18-ux-polish-plan.md` (waves W1–W4, all done).
**Nothing already checked off there is re-reported here.** Where a finding
touches a predecessor decision (e.g. the descoped release pipeline B-7), that
is said explicitly rather than silently re-litigated.

Repro commands assume a release build
(`cargo build --release -p fireside-cli`) and a scratch directory.

**Rev 2 (2026-07-19, CTO pass):** tightened every item that left an
implementation decision open, so an implementer needs no further judgment
calls — P1-1 record format and migration rule, P1-3 tab-stop semantics,
P1-4 table algorithm, P1-6 flash-overflow rule, P2-3 rate limit, the
W4-DS session-state file (now a separate per-deck file, **not** a
resume.json extension — see W4-DS-2 for why), the follower's data flow,
cross-plan sequencing with the editor plan's E0 refactor, and a
Definition of Done. No findings added, removed, or re-prioritized.

## Progress Log

_Update this section whenever an item lands or starts. One line per item:
status, date._

- [x] P0-1 quickstart install block — 2026-07-19
- [x] P0-2 `.md` hint on present/validate — 2026-07-19 (`load()` is shared by
      `present` and one-shot `validate`, so both get the hint; `validate
      --watch`'s separate `watch_report` path still shows the raw parse
      report — out of scope for the P0 fix, watch mode isn't a first-five-
      minutes path)
- [x] P0-3 non-tty guard + `try_init` — 2026-07-19
- [ ] P1-1 path-keyed resume records
- [ ] P1-2 shorthand `--restart`
- [ ] P1-3 tab expansion at render
- [ ] P1-4 import GFM handling + stderr notes + docs table
- [ ] P1-5 H1-only import detection
- [ ] P1-6 footer flash wrap / segment drop
- [ ] P1-7 friendly missing-file errors for `import` / `art image`
- [ ] P1-8 reveal limitation documented (marker syntax = Fresh #2, needs scope decision)
- [ ] P2-1 ascii-art frame label / alt caption
- [ ] P2-2 help overlay bottom-row clipping
- [ ] P2-3 unknown-key feedback flash
- [ ] P2-4 demo quick-edit save affordance
- [ ] P2-5 quick-edit Esc double-tap guard
- [ ] P2-6 "Saved" flash survives self-reload
- [ ] P2-7 bare-invocation help omissions
- [ ] P2-8 presenting.md fallback row
- [ ] P2-9 mouse wheel scroll
- [ ] CH-1 dead workspace deps + image version
- [ ] CH-2 `scripts/smoke.sh` + CI wiring
- [ ] CH-3 error-path e2e tests (lands with Wave 1/2 fixes)
- [ ] W4-DS dual-screen presenter view — **added to scope 2026-07-19 by user
      decision** (promoted from addendum A-2); see "Wave 4 — scoped feature"
      below. Spec-kit feature candidate `012-presenter-view`.
- [ ] WYSIWYG authoring editor — planned separately in
      `.claude/plans/2026-07-19-wysiwyg-editor-plan.md` (user decision
      2026-07-19, Tier 2 selected). Spec-kit feature candidate
      `013-authoring-editor`.

## Executive summary

The presenter itself is in strong shape: validation messages are exemplary,
the reload guard refuses broken decks with a clear flash, branch/reveal/map
interactions all give feedback, a 500-node deck presents with no lag, a 12 MP
photo converts to ASCII in 0.1 s, and the terminal survives SIGKILL. The two
predecessor passes clearly worked.

What they missed clusters into four themes:

1. **The front door is still broken for the stated audience.** The docs
   quickstart's install section cannot succeed as written (no clone step),
   `fireside talk.md` — the single most natural command for anyone coming
   from presenterm/patat/slides — dies with a JSON parse error, and a
   non-tty invocation produces a raw Rust panic. A non-technical user hits
   one of these in their first five minutes.
2. **Resume is fragile in exactly the "night before the talk" scenario.**
   The resume key is the file's (mtime, length), so fixing one typo orphans
   your saved position — silently (verified with `touch`).
3. **Import silently mangles common Markdown.** Tables become raw pipe text
   on the slide, footnotes leak as literal `[^1]` blocks, intro text before
   the first `##` vanishes, and a deck of `#`-per-slide headings is told "no
   headings found". And import cannot express reveal at all — the flagship
   presenter feature is unreachable from the Markdown path.
4. **A false-negative validation class exists**: control characters. Tab
   characters in code blocks are schema-valid, validate clean, and render
   with all indentation silently deleted — gofmt'd Go code presents flat.

Everything else is papercuts (flash truncation at 80 cols, the audience-visible
`─ ascii-art ─` label, anyhow chains on two subcommands) and hygiene (five
dead workspace dependencies, no automated tmux smoke).

---

## Wave 1 — P0: the first-five-minutes failures

**P0-1: Docs quickstart install cannot succeed as written.**
Observed: `docs/src/content/docs/guides/quickstart.md` "Install" section is
exactly ` cargo install --path crates/fireside-cli ` — no `git clone`, no
`cd`. A new visitor landing on the docs site (the W4 "front door") runs it in
an arbitrary directory and gets a cargo path error. The README has the full
three-line sequence; the quickstart never got it.
Who: every brand-new user arriving via the docs site rather than GitHub.
Fix: copy the README's clone/cd/install block into quickstart.md verbatim.
One-line docs change; do first.
Related, structural: install still requires a Rust toolchain at all. The
minimal release pipeline (B-7) was **descoped by explicit user decision
2026-07-18** — not re-litigating it, but flagging plainly: the project's #1
priority ("someone who has never touched a terminal can install Fireside")
is unreachable while `cargo` is the only path. If that priority is real,
B-7 (or brew tap / `cargo binstall`) needs to come back. **Requires the user
to reverse a recorded decision — do not implement without that.**

**P0-2: `fireside talk.md` fails with JSON-parser output.**
Observed: `fireside stress.md` →
`✗ stress.md is not a valid deck / 1 │ # My Conference Talk / ^ expected value`.
Every comparable tool (presenterm, patat, slides, lookatme) presents a
Markdown file directly, so this is the first command a migrating or
Markdown-first user types; the response reads as "your talk is gibberish".
Who: authors, in their very first session.
Fix (minimum): in `load()` (`crates/fireside-cli/src/main.rs:222`), when the
path ends in `.md`/`.markdown`, print
`This is a Markdown file — run "fireside import talk.md" first, then "fireside talk.fireside.json"`
instead of the parse report. Exit 1 unchanged.
Fix (better, see Fresh ideas #1): compile it in memory and present it.
The hint version is a bug-fix-sized change with no scope implications; the
direct-present version is a scope addition under ADR-004 and needs the user
to ask for it.

**P0-3: Non-tty invocation panics with a raw Rust backtrace message.**
Observed: `echo q | fireside demo` →
`thread 'main' panicked at ratatui-0.30.0/src/init.rs:299: failed to initialize terminal: Os { code: 6, ... "Device not configured" }`
plus the `RUST_BACKTRACE` note, and `[?1049l` splattered first. Triggers
whenever stdin/stdout isn't a terminal: piping, CI, some IDE run buttons,
`fireside demo > log`. This is the exact "frightening rather than helpful"
class the error-handling review targets, and it comes from `ratatui::init()`
(the panicking variant) — the constitution's no-panic rule is honored in
our code but delegated to a panicking dependency call at the boundary.
Who: presenters in unusual environments; anyone scripting.
Fix: before initializing, check `std::io::stdout().is_tty()` (crossterm's
`tty::IsTty` — already a permitted dependency, no allowlist change) in
`present_authoring`/`present` callers, and bail with one plain line:
`fireside needs an interactive terminal to present — run it directly in your terminal, not through a pipe.`
Also switch to `ratatui::try_init()` so any residual init failure flows
through `TuiError::Io` instead of panicking.
Test: cli_e2e case running `fireside demo` with piped stdio asserting the
message and exit 1 — this is also the CI gap that would have caught it.

## Wave 2 — P1: presenter-facing correctness

**P1-1: Any edit to the deck silently discards the resume position.**
Observed: present `my-great-talk.fireside.json`, advance to slide 2, quit
(resume record written); `touch` the file; relaunch → back at slide 1, no
toast, no explanation. Cause: `resume::fingerprint_key` is the file's
(mtime, length) (`crates/fireside-cli/src/resume.rs:115`), captured once at
launch (`main.rs:293`). Consequences beyond the headline:
- Fixing a typo the night before the talk loses your place — the exact
  moment resume exists for.
- A quick-edit save mid-presentation re-keys the file, so subsequent
  position writes go under a stale key and the *next* launch can't resume.
- Orphaned entries accumulate in `resume.json` forever (clear only happens
  on reaching a terminal node under the same key).
Who: presenters, at the highest-stakes moment.
Fix: key records by canonicalized absolute path; keep the fingerprint
*inside* the record as a staleness annotation, not the key. On load: if the
recorded node id still exists in the (possibly edited) deck, resume there —
`Session::goto` already guards unknown ids, so a deleted node degrades to
start-from-the-top for free. Migrate/ignore old-format keys silently, prune
entries whose path no longer exists.
Concrete format (rev 2, no open decisions): the store stays one JSON
object; the key becomes the canonicalized absolute path
(`std::fs::canonicalize`, `to_string_lossy`); the value becomes
`{"node_id": "...", "updated": <epoch secs>, "fingerprint": "<mtime>:<len>"}`
where `fingerprint` is a staleness *annotation* (available for a future
"deck changed since you left" toast), never compared during lookup.
Migration is mechanical, no version field needed: legacy keys are bare
`<mtime>:<len>` fingerprints and never begin with a path separator, so on
every save drop any entry whose key is not an absolute path, plus any
entry whose keyed path no longer exists on disk. Contract doc
(`specs/007.../contracts/resume-state-format.md`) needs a matching update —
it's a local cache, not wire format, so no protocol spec/ADR required.
Test: unit tests on the new keying + a tmux smoke: present → quit → edit
file → relaunch → still on the same slide.

**P1-2: The resume toast teaches a flag the taught command rejects.**
Observed: toast says `Resumed where you left off — --restart starts over`,
but `fireside deck.json --restart` (the form every user actually types,
since the docs teach `fireside <file>`) →
`error: unexpected argument '--restart' found / tip: to pass '--restart' as a value, use '-- --restart'` —
clap-speak with an actively harmful tip. Only the longhand
`fireside present deck.json --restart` works.
Who: presenters following the app's own hint.
Fix: add `#[arg(long)] restart: bool` to the shorthand `Cli.file` form in
`main.rs` and pass it through. cli_e2e case for
`fireside <file> --restart` parse success (present can't run headless, but
`--help`-level parse and a missing-file run assert the flag is accepted).

**P1-3: Tab characters are silently deleted at render time — validator
false-negative class (control characters).**
Observed: a deck whose code block is gofmt'd Go
(`"source": "func main() {\n\tfmt.Println(...)"`) validates
`✓ no problems found` and presents with every line flush-left — all
indentation gone. Same for `\t` in text bodies ("Tab\there" → "Tabhere").
Raw ESC (``) in strings is neutralized by ratatui (no terminal
corruption — good), but renders as leftover `[31m` gunk. Tabs are the
common case: any pasted Go, Makefile, or tab-indented snippet.
Who: authors pasting real code; the audience sees flat code on stage.
Fix: expand `\t` to the **next 4-column tab stop** (column-aware — a tab
after 3 chars inserts 1 space, after 4 chars inserts 4; a fixed
4-space substitution would misalign mid-line tabs) during line preparation
in `fireside-tui/src/render/` (code path in `blocks.rs`, text via
`markdown.rs`) — a rendering fix, no protocol change, no new dependency.
Column position counts display width of the preceding text
(`unicode-width`, already permitted), not byte or char count.
Optionally add a symmetric Layer-2 warning (`control-characters-in-text`,
warning severity) — **that** is a validator-rule addition and per the spec
008 workflow needs the rule implemented in both `validation.rs` and
`protocol/validate.mjs`, fixtures in `protocol/fixtures/`, and a
`docs/.../spec/validation.md` entry. The render fix alone is enough to close
the user-facing hole; ship it first.
Test: TestBackend scenario with a tabbed code block asserting indentation
survives; fixture-parity if the validator rule is added.

**P1-4: `fireside import` silently mangles common Markdown.**
Observed on a stress file (all verified, `import` exits 0 with no mention):
- **Tables** become a `text` block containing raw `| pipe | rows |`, which
  then renders literally on the slide (checked live).
- **Footnotes**: `Thanks![^1]` keeps the marker as literal text and the
  definition becomes its own visible `text` block (`[^1]: ...`).
- **Text between the H1 title and the first `##`** is dropped entirely.
- **Task lists** keep literal `[x]`/`[ ]` markers inside list items.
- **Strikethrough** passes through as literal `~~old idea~~` (renderer has
  no strike support either).
- **Blockquotes** silently flatten to plain text (acceptable, but
  undocumented — the guide's conversion table has no row for any of these).
Root cause: pulldown-cmark is instantiated without extension `Options`
(no `ENABLE_TABLES`/`ENABLE_FOOTNOTES`/`ENABLE_TASKLISTS`/
`ENABLE_STRIKETHROUGH` in `import.rs`), so GitHub-flavored constructs
degrade to their CommonMark fallback text instead of being recognized.
Who: authors — they discover each of these on screen, possibly on stage.
Fix, in order of value:
1. Enable the GFM extension options so these constructs are *recognized*,
   then handle each deliberately: tables → monospace-aligned `code` block
   (no new block kind needed; render already centers code); task lists →
   list items with `☐`/`☑` prefixes; footnotes → drop with a stderr note;
   strikethrough → drop the markers.
   Table→code algorithm (rev 2): collect the cells' plain text (inline
   formatting stripped — a table cell is not a place for bold in v1, note
   it on stderr if markers were dropped); per column, take the max cell
   width; pad each cell right to that width; join cells with two spaces;
   after the header row emit one rule line of `─` at the full joined
   width; the pipe characters and the `---` alignment row from the source
   do not survive. Emit as `code` with `language: None`, no line numbers.
   Width = `chars().count()` — **not** `unicode-width`, which is on the
   TUI allowlist but not the CLI's (Principle III); char count is exact
   for the ASCII/Latin tables that dominate and degrades to mild
   misalignment (never corruption) for wide glyphs. Do not add the
   dependency for this.
2. Emit a stderr note per dropped/transformed construct with line numbers,
   in the same voice as the nested-list rejection (which is excellent).
3. Warn (don't silently drop) on content before the first `##`.
4. Document every conversion + non-conversion in the guide's table.
No new dependency, no protocol change. A real `table` *block kind* would be
a protocol addition (TypeSpec + ADR + version bump + both validators) —
**requires spec/ADR first per constitution Principle I**; the code-block
fallback makes it unnecessary for v1.

**P1-5: A `#`-per-slide Markdown file is told "no ## headings found".**
Observed: a file with multiple `# Slide` headings (the presenterm/patat
convention) → `no ## headings found — at least one is required to produce a
deck`, even though the file is full of headings.
Who: anyone migrating an existing terminal-deck file.
Fix (minimum): detect that H1s exist and say so:
`found 2 "#" headings but slides start at "##" — either demote them or note that the first "#" becomes the deck title`.
Fix (better): when a document has 2+ H1s and no H2s, treat H1s as slides
(unambiguous intent). Pure import-frontend change, no protocol impact.

**P1-6: Footer flash messages truncate at the recommended 80-column size.**
Observed twice at 80×24 (the documented comfortable minimum):
- Save-conflict flash renders `...Ctrl+S again to overwrite, Esc to disc` —
  the instruction for the *abandon* choice is cut mid-word.
- On a reveal slide the key footer ends `... ? help  ·  q` — "quit" clipped.
Who: presenters mid-incident — the conflict message is exactly when they
need the full sentence.
Fix (decided, rev 2): while a flash is showing, the footer shows **only
the flash** — key hints are suppressed for the flash's lifetime (the
footer already owns the row, and a presenter mid-incident needs the
sentence, not the hints). A flash still longer than one row wraps onto a
second row borrowed from the bottom of the content area, word-wrapped,
never truncated mid-word; the content area reflows for those frames.
For the key-hint footer itself (no flash showing), drop lowest-priority
segments whole (`e edit` first, then `m map`) before ever clipping
glyphs. Cover with TestBackend scenarios at 80×24; W1-2 fixed this class
for the help overlay but the footer/flash line was missed.

**P1-7: `import` and `art image` still leak anyhow chains on missing files.**
Observed: `fireside import nope.md` and `fireside art image nope.png` →
`Error: could not read ... / Caused by: No such file or directory (os error 2)`.
W1-4 gave `present`/`validate` the friendly one-liner; these two verbs were
missed, so the polish is inconsistent within one tool.
Who: authors; the raw chain is the "frightening" class already fixed elsewhere.
Fix: same treatment — one plain line, no chain
(`No file named nope.md — check the path.`), exit 1. cli_e2e assertions
mirroring `validate_missing_file_suggests_creating_it`.

**P1-8: Reveal is unreachable from the Markdown path.**
Observed: `import.rs` hard-codes `reveal: None` at every block construction
site; no marker syntax exists; the "what v1 doesn't carry over" note doesn't
mention reveal either. Incremental reveal is a README flagship feature and
the demo's showpiece — but the recommended authoring path can't produce it,
and the limitation isn't even stated.
Who: authors who saw the demo and write Markdown.
Fix (minimum, docs-only): add reveal to the limitations note + guide.
Fix (real): a pause marker — `<!-- reveal -->` between blocks assigns
ascending reveal levels to subsequent blocks in that node (presenterm uses
`<!-- pause -->`; convention is established). Import-frontend only: the
protocol field already exists, validators already check it. Behavior-adding
→ Spec Kit pipeline, but no protocol/ADR work.

## Wave 3 — P2: papercuts and polish

**P2-1: The audience sees the literal label `─ ascii-art ─`, and `alt` is
ignored.**
Observed: every ascii-art block (demo title, `new --banner` output) renders
inside a frame captioned with the block-kind name
(`blocks.rs:271`, hard-coded string); the `alt` field is accepted by the
protocol and discarded by the renderer (`_alt`).
Who: the audience — implementation jargon on the title slide of every
banner deck.
Fix: render ascii-art unframed/unlabeled (it's *art*, not a code listing —
the centering already distinguishes it), or caption with `alt` when
present. Snapshot updates; tmux smoke of the demo title slide.

**P2-2: Small terminals lose the `q quit` row of the help overlay.**
Observed at 44×14: the overlay clips its *bottom rows* — `q quit` and
`press any key to close` are the two things cut. W1-2 fixed 80×24/100×30;
below-minimum sizes degrade in the worst order.
Fix: when height-constrained, drop middle rows before last rows, or pin
`q quit · any key closes` as the overlay's fixed footer line.

**P2-3: Unknown keys (notably Esc) are silent on the Present screen.**
Observed: Esc, Tab, and random letters on a non-branch slide do nothing —
no flash. The constitution promises "every blocked action gives feedback",
and Esc is the panic key a lost presenter reaches for.
Fix: catch-all arm in `on_flow_key`/`on_present_key` flashing
`Press ? to see the keys` (rate-limited: once shown, don't re-trigger for
2 s of further unknown keys — track the last such flash's instant in
`App`). Keep reveal-pending behavior (any key reveals) as is.

**P2-4: The demo advertises quick-edit it can't complete.**
Observed: demo footer teaches `e edit`; editing works but Ctrl+S →
`Can't save — this deck has no file to save to`. The presenter finds out
after typing.
Fix: either suppress `e` from the demo footer (present-without-sink already
knows the sink is `Unavailable`), or open the modal with a banner line
`Demo deck — edits preview but can't be saved`.

**P2-5: Quick-edit Esc discards a multi-field edit instantly.**
Observed: type a paragraph into the modal, press Esc (or reflexively Esc to
"close the popup") — everything is gone, no confirmation, no undo. The
conflict path carefully preserves edits (FR-013) but plain Esc doesn't.
Fix: if any buffer differs from its initial value, first Esc flashes
`Unsaved changes — Esc again to discard, Ctrl+S to save`; second Esc within
the flash window discards.

**P2-6: "Saved" confirmation is never actually seen.**
Observed: Ctrl+S → `Saved` flash is replaced within ~250 ms by `Reloaded`
(the deliberate self-reload of the written file re-flashes). The presenter's
lasting impression of a save is the word "Reloaded".
Fix: in `on_reload`, when the reload was triggered by our own write-back
(watcher can flag the first post-save poll), keep/set the flash to `Saved`
instead. Cosmetic, but it's the confirmation moment of the marquee feature.

**P2-7: Bare-invocation teaching text omits `art image` and `present --restart`.**
Observed: the no-args help lists `art text` but not `art image`; `demo`,
`new`, `import`, `validate` are all there. Minor asymmetry, one line.

**P2-8: `presenting.md` documents an impossible branch fallback.**
Observed: the branch-keys table says Space/→ "advance without choosing, if
the branch has a fallback" — but `next-branch-point-conflict` makes
next+branch-point an *error*, so no such deck validates; at a branch, Space
always flashes `This slide asks for a choice`. Remove/reword the row.

**P2-9: Mouse wheel does nothing.**
Observed: click targets work (map rows, branch options), but
`MouseEventKind::ScrollUp/ScrollDown` are ignored while long slides
advertise `▼ more (↓)`. Wheel-scrolling the content (and the map) is the
gesture the click support trains users to expect. Additive, same
constitution posture as click (Principle II: keyboard remains the taught
contract).

## Codebase health, CI, dependencies

**CH-1: Five dead `[workspace.dependencies]` entries + one stale version.**
Observed: `tracing`, `tracing-subscriber`, `textwrap`, `plist`, `font-kit`
are declared at the workspace root and consumed by **no crate** (verified
against every `crates/*/Cargo.toml`); none is on any Principle III
allowlist. They cost nothing at compile time but misrepresent the
dependency surface (`font-kit`/`plist` look alarming in an audit — this
audit included). Separately, workspace `image = "0.25"` conflicts with
what actually builds: `fireside-cli` declares `image = "0.24"` *directly*
(not `workspace = true`), resolving 0.24.9 to match `rascii_art`.
Fix: delete the five dead entries; either fix the workspace entry to
`"0.24"` and have the cli consume `{ workspace = true }`, or delete the
workspace entry — one source of truth either way. Pure manifest hygiene;
`cargo tree` diff should be empty.

**CH-2: No automated tmux smoke exists — the constitution's smoke
discipline is manual memory.**
Observed: `tmux` appears nowhere in `scripts/` or CI; every smoke run so
far has been ad-hoc (and per project memory, smoke tests are what catch
the timing/ordering bugs TestBackend can't). TUI-visible paths with *no*
scripted real-terminal coverage: live-reload swap, reload-refusal flash,
quick-edit save/conflict/retry, resume toast, SIGKILL recovery, exit
summary.
Fix: `scripts/smoke.sh` — detached tmux, drive the release binary through
demo-walk / live-edit / broken-save / resume, `capture-pane` + grep
assertions (the exact loop this audit ran by hand). Run it in `rust.yml`
on ubuntu (tmux is a one-line apt install) and from `verify.sh`. This also
gives constitution Principle VII's fourth bullet a checkable artifact.

**CH-3: Error-path e2e gaps that let P0-3 ship.**
`cli_e2e.rs` (31 tests) covers happy paths and message shapes well, but has
no non-tty invocation test, no `.md`-presented-by-mistake test, and no
shorthand-flag parse test — each maps to a Wave 1/2 finding above. Add
alongside the fixes.

**Otherwise healthy.** Layering holds (TUI does zero file I/O — reload,
write-back, and position persistence are all caller-injected sinks; this is
textbook). Post-split file sizes are reasonable (largest prod file is
`validation.rs` at 1089 lines, all rule + test code). Performance: 500-node
deck validates in 5 ms and presents with instant navigation and a scrolling
map; 12 MP PNG → ASCII in 0.1 s; live-reload latency is the 250 ms poll.
No unmaintained-dependency alarms beyond noting `figlet-rs` (1.0.0, slow
release cadence) and `rascii_art` (0.4.5) are small, ADR-gated, and easy to
vendor if abandoned — no action.

## Docs

- **Quickstart install** — P0-1 above (missing clone step).
- **presenting.md fallback row** — P2-8 above.
- **authoring-markdown.md conversion table** — add rows for what tables /
  blockquotes / footnotes / task lists / strikethrough become (P1-4), and a
  reveal limitation note (P1-8).
- The rest of the W4 restructure holds up in a fresh walk: quickstart →
  presenting → authoring-markdown reads in the right order, the
  `image`-vs-`ascii-art` placeholder callout exists, and cli.md's flag/exit
  tables match the clap definitions (spot-checked every verb).

## Competitive context (brief)

Surveyed from knowledge of presenterm, patat, lookatme, slides
(maaslalani), and sli.dev as of early 2026 — worth a fresh check before
acting on specifics:

- **Everyone presents `.md` directly**; Fireside's compile step is unique
  friction (P0-2, Fresh #1). Its unique *strengths* — branching, two-layer
  validation, quick-edit, resume, the map — none of the others have.
- **Pause/fragment markers are table stakes** (presenterm `<!-- pause -->`,
  patat fragments); Fireside has the runtime feature but no authoring path
  (P1-8).
- **All ship binaries** (brew/scoop/releases); see P0-1's structural note.
- **presenterm renders real images** via kitty/iTerm2/sixel. Fireside's
  ADR-008 NO-GO is a defensible v1 stance, and `art image` is a charming
  substitute — no change recommended, but expect the comparison.
- Normal elsewhere too (so not a Fireside defect): no PDF export in the
  TUI-native tools without a companion command; theme systems usually come
  later (Fireside's single-theme v1 decision is fine).

## Fresh ideas (not in either predecessor plan)

1. **`fireside talk.md` presents directly** — import in memory, watch the
   `.md`, recompile+swap on save (the reload guard already refuses broken
   compiles gracefully). This makes Fireside's live-reload loop *better*
   than presenterm's for Markdown authors and erases the biggest
   competitive friction. **Scope addition under ADR-004 — needs the user to
   ask for it; flag, don't build.** Quick-edit would stay disabled for
   `.md`-backed sessions (write-back targets JSON only) — surface that in
   the footer.
2. **`<!-- reveal -->` import marker** (P1-8's real fix).
3. **Speaker notes from Markdown** — `<!-- notes: ... -->` or a
   `> Note:`-prefixed blockquote per section → `speaker-notes`. Closes
   another "v1 doesn't carry over" item with zero protocol work.
4. **Code fence info-string passthrough** — ` ```rust {1,3} ` →
   `highlight-lines: [1,3]`, and expose `show-line-numbers` via
   ` ```rust numbered `. The protocol fields exist; import just never sets
   them (discovered while confirming line numbers are opt-in).
5. **First-run environment check** — on TUI start, if `COLORTERM` isn't
   `truecolor`/`24bit`, flash once: `Colors may look off — set
   COLORTERM=truecolor`. Cheaper than a `fireside doctor` subcommand and
   catches the README's requirement at the moment it matters. (Verified the
   presenter still *renders* fine under tmux's 256-color downgrade, so a
   flash, not a refusal.)
6. **Rehearsal stats in the exit summary** — per-slide dwell times are
   implicitly known (position-change timestamps); `--rehearse` could print
   a per-slide table after `Presented 5/7 slides in 12:30.` for pacing
   practice. Presenter-first in spirit; still a scope addition to run past
   the user.
7. **`import --force`** — the documented regenerate loop is "delete the old
   file first"; an explicit overwrite flag is the standard affordance and
   keeps the safe default.
8. **Map search** — on 100+-node decks (workshops), `/` filter in the map.
   Low priority; the map already auto-scrolls to the current node.

## Constitution flags (summary)

- **Release binaries (P0-1 note), direct `.md` present (Fresh #1),
  rehearsal mode (Fresh #6)**: scope additions / reversals of recorded user
  decisions (ADR-004 scope, B-7 descope) — require explicit user say-so.
- **Any new validator rule** (control-characters, import-related): must land
  symmetrically in `validation.rs` + `protocol/validate.mjs` + fixtures +
  spec docs (Principle I / spec 008 workflow).
- **A real `table` block kind**: protocol change — TypeSpec + ADR + version
  bump + both validators. Recommended *avoided* via the code-block fallback
  in P1-4.
- Everything else in this plan uses already-permitted dependencies and
  existing crate boundaries; no allowlist amendments needed (tty check is
  crossterm, GFM options are pulldown-cmark features, tab expansion is
  string handling).

## Wave 4 — scoped feature: dual-screen presenter view (spec 012 candidate)

_Promoted from addendum A-2 by user decision 2026-07-19. Behavior-adding →
full Spec Kit pipeline (`/speckit-specify` → plan → tasks → implement),
plus an ADR for the shared session-state file contract and an ADR noting
the ADR-004 scope extension (user-requested, so the gate is satisfied)._

Goal: notes on the laptop, deck fullscreen on the extended display —
without the audience ever seeing speaker notes (A-1).

Scoped work, in dependency order:

1. **W4-DS-1 — land P1-1 first (path-keyed session state).** Rev 2 note:
   with W4-DS-2's dedicated session file (keyed by canonical path from
   day one) this is no longer the *hard* dependency it was when session
   state was going to live in the resume record — but keep the order
   anyway: P1-1 establishes the canonical-path keying convention and the
   prune/migration behavior the session store copies, and it fixes a
   shipping bug regardless. Do the resume-keying fix as its own change,
   then build on it.
2. **W4-DS-2 — session-state contract (ADR).** Decided (rev 2): live
   session state gets its **own file per deck**, not a new field in
   `resume.json`. Rationale for the ADR: the heartbeat rewrites its file
   on every 250 ms poll tick, and `resume.json` is a shared
   read-modify-write store across *all* decks — heartbeat traffic there
   would race two concurrent presentations (last-writer-wins over the
   whole map) and churn a file the rest of the code treats as a cold
   cache. Location:
   `$XDG_STATE_HOME/fireside/sessions/<fnv1a64 hex of canonical path>.json`
   — FNV-1a 64-bit implemented in ~6 lines beside the store (std-only,
   stable across processes; `DefaultHasher` is not guaranteed stable
   across Rust versions and `watch::fingerprint` is an `(mtime, len)`
   pair, not a hash — neither fits). Contents:
   `{"schema": 1, "deck_path": "...", "node_id": "...", "reveal_step": n,
   "reveal_total": n, "elapsed_secs": n, "heartbeat": <epoch secs>}` —
   heartbeat refreshed on every poll tick, not just on movement. One
   writer (the presenting process), N readers. Atomic writes (temp file
   in the same directory + rename). Deleted on clean presenter exit; a
   reader treats missing-file and stale-heartbeat (> 2 s old) identically
   as "presenter not running". `resume.json` is untouched by this feature
   beyond the P1-1 rekeying. Document in `contracts/` for the spec;
   host-local, **not** protocol-versioned.
3. **W4-DS-3 — presenter side.** Widen `PositionSink` (or add a sibling
   sink) so the CLI can persist reveal step and heartbeat; presenter still
   performs zero file I/O itself. `--fullscreen` launch flag: start with
   `view_override = Fullscreen` (one-liner; the `f` toggle already exists).
4. **W4-DS-4 — `fireside notes <deck>` follower.** New TUI screen in
   `fireside-tui` (rendering only; polling closure injected from the CLI at
   the watcher's 250 ms cadence, same pattern as `watch.rs`). Data flow
   (rev 2): the follower loads the deck itself through the same `load()`
   path, watches the deck file too (so a quick-edit save updates the
   notes), and resolves the session file's `node_id` against its own
   loaded graph — current node's `speaker-notes`, next title via the
   node's `next` edge, or the choice options at a branch. It writes no
   files, ever. A `node_id` it can't resolve (presenter and follower
   mid-reload skew) renders as a benign "waiting for presenter…" state,
   never an error. Same non-tty guard as P0-3. Shows:
   current slide title + its `speaker-notes`, next-slide title (or the
   branch options when the presenter is at a choice), reveal progress
   (`3/5 revealed`), elapsed timer, and a clear stale state
   (`Presenter not running — start "fireside <deck>" in another window`)
   when the heartbeat stops (>2 s). `q` quits. Footer teaches its keys
   (Principle II applies to the follower too).
5. **W4-DS-5 — tests + smoke.** Unit tests for the state read/write;
   TestBackend scenarios for follower states (notes, no-notes, branch,
   stale); tmux smoke driving two panes — presenter in one, follower in the
   other — asserting the follower tracks navigation and reveals, and goes
   stale on presenter kill. Extends CH-2's `scripts/smoke.sh`.
6. **W4-DS-6 — docs.** New section in `guides/presenting.md` ("Presenting
   with two screens": drag the deck terminal to the projector, OS
   fullscreen, `fireside notes` on the laptop) + cli.md entry + quickstart
   pointer. Update the A-1 caveat: notes panel (`s`) documented as the
   single-screen/rehearsal path, `fireside notes` as the on-stage path.

Acceptance: with a deck open in one terminal and `fireside notes` in
another, navigating/revealing in the presenter updates the follower within
~500 ms; killing the presenter flips the follower to the stale state within
~2 s; the audience-facing window never renders notes. No new dependencies;
no protocol change; crate boundaries unchanged.

Constitution flags: ADR-004 scope extension (user-requested — record in the
ADR); session-state contract ADR; no allowlist changes.

## Addendum (2026-07-19, user follow-up): speaker-notes privacy, dual-screen presenter view, WYSIWYG editing

**A-1: Speaker notes are visible to everyone in the room.**
Observed: `s` opens a ≤6-row panel at the bottom of the *same* terminal
frame (`render/content.rs:246–277`). There is exactly one window; if the
terminal is on (or mirrored to) the projector, the audience reads the
notes. `presenting.md` calls notes "meant for you" but no mechanism makes
that true — today the feature is only safe while rehearsing.

**A-2: Dual-screen presenter view (notes on laptop, deck fullscreen on the
external display) — effort: S–M, the architecture nearly anticipates it.**
_Status: promoted to scoped work — see "Wave 4 — scoped feature" above._
A terminal window cannot span displays, so the proven TUI pattern
(presenterm's speaker-notes mode) is **two processes**: the deck presented
in a terminal window dragged to the projector display and OS-fullscreened;
a follower window on the laptop. Design that fits Fireside as-is:
- Presenter side needs *zero* new plumbing: `PositionSink` already fires on
  every node change, and the CLI already persists the current node id to
  `$XDG_STATE_HOME/fireside/resume.json` immediately on every move.
- New `fireside notes <deck>` follower: polls the session-state file at the
  watcher's existing 250 ms cadence (same pattern as `watch.rs`) and renders
  current slide's `speaker-notes`, current + next title, and elapsed time.
  Shows "presenter not running / disconnected" when the record goes stale.
- Extend the resume/session record with reveal step, elapsed, and a
  heartbeat timestamp so the follower can show `3/5 revealed` and the timer.
  _(Superseded in rev 2: the scoped W4-DS-2 puts session state in its own
  per-deck file instead of the resume record — the Wave 4 section governs.)_
- **Depends on P1-1** (path-keyed resume state): with fingerprint keying, a
  quick-edit save mid-talk would silently detach the follower.
- Polish: a `--fullscreen` launch flag (start in the existing `f` view mode)
  for the projector window.
Boundaries hold: follower rendering in `fireside-tui`, all file I/O in
`fireside-cli` via the existing closure-injection pattern; no new
dependencies; session state stays host-local (like resume), so **no
protocol change** — but the session-state file becomes a two-reader
contract and should get an ADR. Constitution: scope addition under ADR-004;
the user has now explicitly asked, which satisfies that gate — run it
through the Spec Kit pipeline.

**A-3: Richer live editing / WYSIWYG for authors who can't edit JSON —
three tiers, rising cost.**
Quick-edit is content-only *by recorded decision* (ADR-005: no structural
edits); anything below supersedes it and needs a new ADR, not silent creep.
- **Tier 1 (M): form-based structure editing in the TUI.** Grow quick-edit
  + the map screen: add-slide-after-this (title prompt, auto-wired `next`),
  delete slide (with re-wiring), add/remove/reorder blocks via a
  kind-picker with sensible defaults, edit branch options (label/key/
  target) as a form. Everything flows through the existing TEA update +
  write-back sink, and the reload guard already validates every save — the
  safety story is built. Risk is presenter-surface creep (Principle II:
  simplicity beats surface area); mitigate by hanging structural ops off
  the map screen or a separate `fireside edit <deck>` entry point rather
  than adding keys to Present.
- **Tier 2 (L): a dedicated `fireside edit` authoring TUI** — outline pane,
  live slide preview, block palette, edge wiring. Keeps the presenter lean;
  weeks of work; still a TUI, so it only partially serves the
  "never touched a terminal" audience.
- **Tier 3 (the real WYSIWYG answer): a web editor — and there's a cheap
  on-ramp.** A local `--web` server would need an HTTP dependency
  (**requires constitution amendment**). But a *static* editor page on the
  existing Astro docs site — open/save `.fireside.json` via the browser's
  File System Access API, drag-drop nodes, form-based blocks — needs **zero
  Rust changes and zero new Rust dependencies**, ships on the existing
  Pages pipeline, and can reuse `protocol/validate.mjs` verbatim in the
  browser since the semantic validator is already plain JS. The
  protocol-first design makes this unusually cheap for what it is.
Recommendation: the Markdown-path fixes (P1-4/5/8, Fresh #1–4) already
serve most "I don't want JSON" authors; do Tier 1 next as a spec-kit
feature with the ADR-005-superseding ADR; treat the static web editor as a
separate product-direction decision — it can be prototyped without touching
the workspace.

## Definition of done (applies to every item above)

- `scripts/verify.sh` passes — it mirrors every CI job; do not substitute
  a hand-picked subset of checks.
- Any change touching a TUI-visible path gets a real tmux smoke run
  before it is called done (TestBackend cannot catch reload/ordering/
  timing bugs — project rule, learned the hard way).
- New/changed behavior lands with tests at the constitution-VII layer
  named in the item (unit / TestBackend scenario / cli_e2e / tmux smoke).
- `graphify update .` after code changes.
- Tick the item's Progress Log line in this file (status + date).

## Suggested order

Wave 1 in one sitting (P0-1 docs line + P0-2 hint + P0-3 tty guard are each
small); then P1-1/P1-2 together (both resume-adjacent, one tmux smoke
covers both); then P1-3 (render fix only) and P1-6 (both footer/render,
snapshot-heavy); then the import batch P1-4/P1-5/P1-8-docs as one
`import`-focused PR; P1-7 rides along with anything. Wave 3 and CH items
are independent fill-in work; CH-1 is a five-minute cleanup that can go
first in any PR. Wave 4 (dual-screen) starts after P1-1 lands and goes
through the Spec Kit pipeline as `012-presenter-view`; the WYSIWYG editor
follows its own plan (`2026-07-19-wysiwyg-editor-plan.md`) as
`013-authoring-editor`. Remaining fresh ideas go to the user for a scope
decision before any speccing.

Cross-plan sequencing (decided, rev 2): this plan's `render/` fixes
(P1-6, P2-1) land **before** the editor plan's E0 `SlideView` refactor —
they are small and snapshot-bound, and the refactor then carries them.
The editor plan says the same; neither plan may interleave with the
other inside `fireside-tui/src/render/`.
