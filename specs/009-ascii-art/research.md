# Phase 0 Research: ASCII art content block

All items below were resolved before or during this planning pass; none
remain as `NEEDS CLARIFICATION` in `plan.md`'s Technical Context.

## 1. Conversion crate choice

**Decision**: `figlet-rs` (text → banner) and `rascii_art` (image →
ASCII), both `fireside-cli`-only.

**Rationale**: ADR-011 (`.claude/adrs/adr-011-ascii-art-crate-msrv-spike.md`)
ran a real `cargo +1.88 build`/`run` spike, not just metadata inspection.
Both crates build and run correctly on the pinned MSRV. `rascii_art` was
chosen over `artem` (the other image→ASCII candidate) because it pulls
72 total packages vs. artem's 200 for the same job — artem's default
features add an HTTP client and zip reader Fireside's local-file-only use
case doesn't need. `figlet-rs` embeds its fonts as crate resources, so
banner generation needs no filesystem/network access beyond the input
phrase itself.

**Alternatives considered**: `artem` (rejected, see above). `tui-big-text`
(rejected — a render-time widget, which the C-1 design deliberately avoids
so `fireside-tui`'s dependency list never grows for this feature; recorded
in ADR-011 for completeness only).

## 2. Whether a new block *kind* can safely degrade in older engines

**Decision**: No degrade attempt is needed or wanted — a pre-0.1.3 engine
already rejects a document containing an `ascii-art` block outright, with
a clear, actionable error, using the existing closed-enum parse machinery.
No new compatibility-detection code is required.

**Rationale**: Verified directly against the current `fireside-core`
(scratch project, `Graph::from_json` against a document containing
`{"kind":"ascii-art",...}`): the existing serde `#[serde(tag = "kind")]`
closed enum already produces
`not a valid Fireside document: unknown variant `ascii-art`, expected one
of `heading`, `text`, `code`, `list`, `image`, `divider`, `container` at
line 1 column 50` — a whole-document parse failure via `CoreError::Parse`,
surfaced through the same `report::parse_report` path every other parse
error already uses (`fireside-cli/src/main.rs::load`). This *is* FR-011's
"clearly rejected, with an explicit compatibility message" — it falls out
of the existing closed-union design for free. This is different from
0.1.2's `reveal` field, which is an *additive optional field* an old
engine can safely ignore (`serde` skips unknown fields by default); a new
enum variant is not ignorable the same way, which is exactly why ADR-012
must record this as a deliberate, named compatibility break rather than a
silent one.

**Alternatives considered**: A permissive/untagged fallback variant that
degrades an unknown `ascii-art` block to, say, a `TextBlock` showing the
raw JSON — rejected as strictly worse than a clear parse error: it would
silently corrupt a presentation rather than fail loudly before the
presenter ever goes on stage, which contradicts Principle II
(presenter-first) far more than a refusal-to-open does.

## 3. Rendering: reuse vs. duplicate the spec-005 centering logic

**Decision**: Factor the box-width/centering computation currently inlined
in `blocks.rs::code()` (guarded by `is_ascii_art(language)`) into a
shared private helper both `code()` and the new `ascii_art()` call, rather
than duplicating the math or routing `ascii-art` blocks through `code()`
with a synthetic language.

**Rationale**: `code()` draws syntax-highlighting/line-number/highlighted-line
machinery an `ascii-art` block has no concept of (it has no `language`,
no `highlight-lines`, no `show-line-numbers` fields in the protocol — see
`data-model.md`). Routing through `code()` would mean threading `None`s
through parameters that don't apply, which is more confusing than a small
shared width-centering helper plus a purpose-built `ascii_art()` renderer
that draws the same bordered/centered box chrome (satisfying spec 009's
FR-002, "same visual treatment") without the source-code-specific
machinery.

**Alternatives considered**: Full duplication of the width-centering
logic in a new function — rejected as the literal thing spec 005's
docstring already warns against ("sized to its own widest line", now
needed identically in two places); a shared helper is a small,
low-risk refactor of already-tested code (5 existing scenario tests in
`render/tests.rs` cover `code()`'s ASCII-art path already, per the
strategic plan's A-3 progress log).

## 4. Width threshold for `ascii-art-too-wide`

**Decision**: 76 columns, matching the value specified in the strategic
plan's C-2 and consistent with spec 005's existing "80-col terminal minus
card chrome" reasoning for the same class of content.

**Rationale**: Keeps the new validator rule's threshold documented the
same way as every other content-quality warning in `validation.rs` — a
concrete, explained constant (mirrors `MAX_CONTAINER_NESTING_DEPTH`'s
doc comment pattern), not TUI-coupled (the message doesn't reference pixel
or cell measurements the reader can't picture).

**Alternatives considered**: Deriving the threshold dynamically from the
TUI's actual card width — rejected because `fireside-engine` (where
validation lives) cannot depend on `fireside-tui` per the crate boundary
table (Principle III), and a fixed, documented constant is exactly the
pattern the codebase already uses for `MAX_CONTAINER_NESTING_DEPTH`.
Measuring "columns" via true Unicode display width (matching
`blocks.rs`'s `UnicodeWidthStr::width`) was the original intent but was
corrected during implementation: `fireside-engine` cannot depend on
`unicode-width` either, for the same Principle III reason. The shipped
measure is `chars().count()` (Rust) / `[...line].length` (Node) — a
documented approximation, exact for plain ASCII art.

## 5. CLI subcommand shape

**Decision**: `fireside art text <PHRASE>` and `fireside art image
<PATH>`, a new `Art` subcommand with two further verbs (clap's nested
`Subcommand` derive, same pattern `clap` already supports and this crate
already uses for the top-level `Command` enum). Both print generated art
to stdout; neither edits a deck file.

**Rationale**: Matches spec 009's Assumptions section exactly (generation
commands are authoring-time conveniences, not deck editors) and keeps the
new surface area small and consistent with the existing verb-per-subcommand
CLI shape (`present`/`validate`/`new`/`demo`/`import`). A nested
`art text`/`art image` groups the two generation modes under one verb
rather than inventing two unrelated top-level verbs, keeping `fireside
--help`'s top-level list short (Principle II: simplicity beats surface
area).

**Alternatives considered**: Two flat top-level verbs (`fireside
banner`/`fireside ascii`) — rejected, needlessly grows the top-level verb
list for two commands that are conceptually one feature with two input
modes. Writing generated art directly into a deck's JSON (`--in-place`
style) — rejected as out of scope per spec 009's Assumptions; authors
already have `quick-edit`/hand-editing/`import` for getting content into a
deck, and inventing deck-mutation logic for this feature alone would be
speculative scope beyond what the spec asks for.
