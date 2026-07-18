# Tasks: ASCII art content block

**Input**: Design documents from `/specs/009-ascii-art/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md, ADR-011, ADR-012, constitution 1.2.0

**Tests**: Included — Test Discipline (constitution VII) requires unit
tests at the model/engine/validator layers, scenario tests for the new
user-visible TUI render path, CLI end-to-end tests for the two new
subcommands, and a tmux smoke test for the presenter-facing rendering
change.

**Organization**: Tasks are grouped by user story per spec.md priorities
(US1 P1 banner, US2 P1 image, US3 P2 composition, US4 P3 validation),
preceded by a Foundational phase since the wire field, core model, and
render path are shared prerequisites every story depends on — no
`ascii-art` block can be authored or demonstrated at all until it exists
on the wire and renders.

## Phase 1: Setup

- [X] T001 Re-read the current state of every file this feature touches
      immediately before editing (line numbers may have shifted since
      planning): `protocol/main.tsp`, `protocol/validate.mjs`,
      `crates/fireside-core/src/model/mod.rs`,
      `crates/fireside-engine/src/validation.rs`,
      `crates/fireside-tui/src/render/blocks.rs`,
      `crates/fireside-tui/src/render/tests.rs`,
      `crates/fireside-cli/src/main.rs`,
      `crates/fireside-cli/Cargo.toml`

## Phase 2: Foundational (blocking prerequisite for all user stories)

**Goal**: The `ascii-art` block kind exists on the wire, in the Rust
model, and renders centered/sized-to-content in the TUI. No story is
independently testable until this phase is done — every user story
either generates or presents this block kind.

- [X] T002 In `protocol/main.tsp`, add `model AsciiArtBlock { ...Revealable; kind: "ascii-art"; art: string; alt?: string; }` per `contracts/ascii-art-block.md`, near the other block models
- [X] T003 In `protocol/main.tsp`, add `AsciiArtBlock` as an eighth member of `union ContentBlock`; update the "Conforming engines MUST support all 7 block kinds" doc comment to "8 block kinds"
- [X] T004 In `protocol/main.tsp`, add `v0_1_3: "0.1.3"` to the `Versions` enum and rewrite the "## Protocol Version" doc banner per `data-model.md` (state the compatibility-break caveat, citing ADR-012)
- [X] T005 Run `cd protocol && npm run build`; commit the regenerated `protocol/tsp-output/` output (constitution Operational Constraints)
- [X] T006 In `crates/fireside-core/src/model/mod.rs`, add the `ContentBlock::AsciiArt { reveal: Option<u32>, art: String, alt: Option<String> }` variant per `data-model.md`, doc-commented like the existing variants
- [X] T007 In `crates/fireside-core/src/model/mod.rs`, add an `AsciiArt` arm to `ContentBlock::reveal()`'s match (the existing `children()` catch-all `_ => &[]` already covers it — no change needed there, `AsciiArt` is a leaf)
- [X] T008 In `crates/fireside-core/src/model/mod.rs`'s `proptest_support` module, add an `AsciiArt`-generating arm to `arbitrary_leaf_block()` (arbitrary `reveal`, `art` via `arbitrary_string()`, optional `alt` via `arbitrary_string()`), so `graph_round_trips_through_json` and `reveal_levels_are_sorted_deduped_and_positive` cover the new variant automatically
- [X] T009 [P] Add unit test `ascii_art_block_round_trips_with_kebab_case_wire_format` in `crates/fireside-core/src/model/mod.rs`'s test module: parse `{"kind":"ascii-art","art":"x","alt":"y","reveal":1}`, assert fields, re-serialize, assert `"kind":"ascii-art"` and both other keys present; a block with no `alt` omits the key on serialize (matching `round_trip_preserves_absent_fields`'s existing style)
- [X] T010 [P] Add a standalone unit test `unknown_kind_produces_clear_parse_error` in `crates/fireside-core/src/model/mod.rs` asserting `Graph::from_json` on a document with an unrecognized `"kind"` value fails with a message containing `"unknown variant"` (locks in the FR-011 compatibility behavior verified in `research.md` §2, so a future refactor can't silently change it)
- [X] T011 In `crates/fireside-tui/src/render/blocks.rs`, extract the box-width/centering computation currently inlined in `code()`'s `is_ascii_art(language)` branch into a shared private helper (per `data-model.md`'s `centered_box_width` sketch); update `code()` to call it — **no behavior change**, verify by re-running the 5 existing scenario/insta tests covering the language-less code-block ASCII-art path before proceeding
- [X] T012 In `crates/fireside-tui/src/render/blocks.rs`, add `fn ascii_art(art: &str, alt: Option<&str>, width: u16, tokens: &Tokens) -> Vec<Line<'static>>` using the shared helper from T011: bordered `─ ascii-art ─...` header (same border token `code()` uses), centered, plain (unstyled) monospace lines, no line numbers or syntax highlighting; `alt` is not rendered as visible text
- [X] T013 In `crates/fireside-tui/src/render/blocks.rs`'s `render_block()` match, add `ContentBlock::AsciiArt { art, alt, .. } => ascii_art(art, alt.as_deref(), width, tokens),`
- [X] T014 [P] Add scenario test `ascii_art_block_renders_centered_and_sized_to_content` in `crates/fireside-tui/src/render/tests.rs`: a node with a narrow multi-line `ascii-art` block; assert the rendered screen shows it centered and boxed, not stretched full-width (mirrors spec 005's existing centering assertions for the language-less code-block path)
- [X] T015 Run `cargo test -p fireside-core -p fireside-tui` and `node protocol/validate.mjs docs/examples/hello.json` (must stay 0 errors — `hello.json` uses no `ascii-art` block)

**Checkpoint**: `ascii-art` blocks parse, round-trip, and render
centered/sized-to-content in the TUI; `cargo test -p fireside-core -p
fireside-tui` passes; existing ASCII-art-via-code-block behavior
(spec 005) is unchanged. No CLI generation or validator warnings yet —
authors could hand-type an `ascii-art` block today and it would already
present correctly.

---

## Phase 3: User Story 1 - Author turns a title into a stylized banner (Priority: P1)

**Goal**: `fireside art text <PHRASE>` produces ready-to-paste banner text.

**Independent Test**: Per spec.md US1 — run the command against a short
phrase, confirm multi-line stylized output; paste into a deck and confirm
it presents centered.

- [X] T016 [US1] In `crates/fireside-cli/Cargo.toml`, add `figlet-rs = "1"` as a direct dependency
- [X] T017 [US1] Create `crates/fireside-cli/src/art.rs` with `pub(crate) fn art_text(phrase: &str) -> anyhow::Result<()>` per `contracts/cli-art-command.md`: load `figlet_rs::FIGlet::standard()`, convert `phrase`, print to stdout; `anyhow::bail!` with a clear message if `convert()` returns `None` (no recognized character) — see FR-013
- [X] T018 [US1] In `crates/fireside-cli/src/main.rs`, add `mod art;`, a `Command::Art { #[command(subcommand)] mode: ArtMode }` variant, and an `ArtMode` enum with a `Text { phrase: String }` variant (nested `Subcommand` derive per `data-model.md`); wire dispatch in `main()`'s match to call `art::art_text(&phrase)`
- [X] T019 [P] [US1] Add CLI end-to-end test `art_text_prints_a_multiline_banner` in `crates/fireside-cli/tests/cli_e2e.rs`: run `fireside art text "Fireside"`, assert exit code 0 and stdout has more than one line
- [X] T020 [P] [US1] Add CLI end-to-end test `art_text_partial_recognition_still_produces_output` in `crates/fireside-cli/tests/cli_e2e.rs`: run against a phrase mixing ordinary letters with an unsupported character (e.g. an emoji); assert exit code 0 and non-empty stdout (FR-013)
- [X] T021 [P] [US1] Add CLI end-to-end test `art_text_with_no_recognized_characters_errors_clearly` in `crates/fireside-cli/tests/cli_e2e.rs`: run against a phrase of only unsupported characters; assert non-zero exit and a non-empty, readable stderr message

**Checkpoint**: `fireside art text` is fully functional and
independently demonstrable — an author can generate a banner and drop it
into a deck.

---

## Phase 4: User Story 2 - Author converts an existing image into ASCII art (Priority: P1)

**Goal**: `fireside art image <PATH>` produces ready-to-paste ASCII shading.

**Independent Test**: Per spec.md US2 — run the command against a small
local image, confirm multi-line text output; run against a bad path,
confirm a clear error, not a crash.

- [X] T022 [US2] In `crates/fireside-cli/Cargo.toml`, add `rascii_art = "0.4"` as a direct dependency
- [X] T023 [US2] In `crates/fireside-cli/src/art.rs`, add `pub(crate) fn art_image(path: &Path, width: Option<u32>) -> anyhow::Result<()>` per `contracts/cli-art-command.md`: build `rascii_art::RenderOptions` (width defaulted per data-model.md if `None`, colored/invert left at defaults so output stays plain text), call `rascii_art::render_to` against `path`, print to stdout; wrap the error with `.with_context(...)` naming the path on failure (FR-014) — no panics
- [X] T024 [US2] In `crates/fireside-cli/src/main.rs`, add `ArtMode::Image { path: PathBuf, #[arg(long)] width: Option<u32> }`; wire dispatch to `art::art_image(&path, width)`
- [X] T025 [US2] Add a tiny fixture image at `crates/fireside-cli/tests/fixtures/tiny.png` (small, checked-in, e.g. a 16×16 PNG) for the end-to-end tests below
- [X] T026 [P] [US2] Add CLI end-to-end test `art_image_converts_a_readable_file` in `crates/fireside-cli/tests/cli_e2e.rs`: run `fireside art image tests/fixtures/tiny.png`, assert exit code 0 and multi-line stdout
- [X] T027 [P] [US2] Add CLI end-to-end test `art_image_reports_a_clear_error_for_a_missing_file` in `crates/fireside-cli/tests/cli_e2e.rs`: run against a nonexistent path, assert non-zero exit, readable stderr, and (implicitly, since the process exits normally) no panic

**Checkpoint**: `fireside art image` is fully functional and
independently demonstrable — an author can convert a local image and
drop the result into a deck.

---

## Phase 5: User Story 3 - Ascii-art appears alongside other content, on its own or hand-authored (Priority: P2)

**Goal**: Confirm the block behaves as ordinary content regardless of
origin, and composes correctly with progressive reveal — using the
generic reveal machinery already wired in Phase 2/Foundational (no new
engine code needed; `Session::next()` and `blocks.rs`'s reveal filter
already operate on any `ContentBlock` via `.reveal()`/leaf semantics).

**Independent Test**: Per spec.md US3 — a hand-typed `ascii-art` block
presents identically to generated art; a reveal-marked one stays fully
hidden until its step, then appears whole.

- [X] T028 [P] [US3] Add scenario test `ascii_art_reveal_gated_block_appears_as_one_unit` in `crates/fireside-tui/src/render/tests.rs`: a node with an always-visible block and an `ascii-art` block at `reveal: 1`; assert the art is fully absent (no reserved space) before the reveal step and every line appears together after one Space press (mirrors spec 006's `reveal_hides_content_until_next_is_pressed_enough_times` pattern)
- [X] T029 [US3] Smoke-test in tmux: launch `./target/debug/fireside present <fixture with a reveal-gated ascii-art block>`, press Space, and visually confirm the art appears centered and all at once — matching this project's established practice of not trusting `TestBackend` alone for interactive keypress-driven, presenter-facing rendering (constitution Principle VII; see `feedback_tmux_smoke_catches_timing_bugs` precedent). Confirmed: before Space, footer read "0/1 revealed" and no art was visible (zero reserved space); after Space, all three lines of the cat art appeared together, boxed and centered, footer badge gone.

**Checkpoint**: US3 is fully functional and independently demonstrable —
ascii-art composes with reveal and with hand-authoring exactly like every
other block kind, verified in a real terminal.

---

## Phase 6: User Story 4 - Author is warned about an oversized or empty art block before presenting (Priority: P3)

**Goal**: Two new symmetric (Rust + Node) validator warnings.

**Independent Test**: Per spec.md US4 — a too-wide block and an empty
block each produce a distinct, correctly-named warning; a normal block
produces neither.

- [X] T030 [US4] In `crates/fireside-engine/src/validation.rs`, add `fn check_ascii_art_too_wide(graph: &Graph, diags: &mut Vec<Diagnostic>)` (WARNING, rule `"ascii-art-too-wide"`, threshold 76 columns per `data-model.md`/`research.md` §4, walks `content` recursively through `Container` children like `walk_reveal_masking`/`walk_link_urls`); call it from `validate()` — width measured as `chars().count()`, not `unicode-width` (fireside-engine's crate boundary forbids it; see updated data-model.md/research.md §4)
- [X] T031 [US4] In `crates/fireside-engine/src/validation.rs`, add `fn check_ascii_art_empty(graph: &Graph, diags: &mut Vec<Diagnostic>)` (WARNING, rule `"ascii-art-empty"`, fires when `art.trim().is_empty()`); call it from `validate()`
- [X] T032 [US4] In `protocol/validate.mjs`, add `checkAsciiArtTooWide(graph)` mirroring T030 exactly (same rule name, same 76-column threshold, same message shape); call it from the main diagnostic-collection function
- [X] T033 [US4] In `protocol/validate.mjs`, add `checkAsciiArtEmpty(graph)` mirroring T031; call it from the main diagnostic-collection function
- [X] T034 [P] [US4] Add fixture `protocol/fixtures/valid/ascii-art-too-wide.json`: a node with an `ascii-art` block whose widest line exceeds 76 columns
- [X] T035 [P] [US4] Add fixture `protocol/fixtures/valid/ascii-art-empty.json`: a node with an `ascii-art` block whose `art` is `""`
- [X] T036 [P] [US4] Add fixture `protocol/fixtures/valid/ascii-art-clean.json`: a node with a normal, non-empty, within-width `ascii-art` block, expecting zero diagnostics
- [X] T037 [US4] Add all three fixtures' expected rule-name arrays to `protocol/fixtures.expected.json` (`["ascii-art-too-wide"]`, `["ascii-art-empty"]`, `[]`)
- [X] T038 [P] [US4] Add unit test `ascii_art_too_wide_warns_on_oversized_art` in `crates/fireside-engine/src/validation.rs`'s test module
- [X] T039 [P] [US4] Add unit test `ascii_art_empty_warns_on_blank_art` in `crates/fireside-engine/src/validation.rs`'s test module
- [X] T040 [P] [US4] Add unit test `ascii_art_within_limits_produces_no_warning` in `crates/fireside-engine/src/validation.rs`'s test module
- [X] T041 [US4] Run `node protocol/run-fixtures.mjs` and `cargo test -p fireside-engine`; confirm Rust/Node parity on all three new fixtures (same rule ids fire in both)

**Checkpoint**: US4 is fully functional and independently demonstrable —
an author is warned about an oversized or empty `ascii-art` block by
checking their deck, before presenting, with matching Rust/Node
diagnostics.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T042 Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check`; fix anything flagged
- [X] T043 Run the full `scripts/verify.sh` and confirm green — every step except the `tsp-output/` vs. `git diff` check passed directly; that one step fails only because nothing in this session is committed yet (matches the rest of this plan's Progress Log convention of staying uncommitted), not a real defect — `npm run build` itself produces the exact regenerated files already staged in the working tree
- [X] T044 Run `graphify update .` to refresh the knowledge graph (constitution Operational Constraints)
- [X] T045 Grep `docs/src/content/docs/` for any place that enumerates "7 block kinds" or lists all `ContentBlock` kinds by name; update to include `ascii-art` if found (the protocol spec pages mirror `main.tsp`, per constitution Principle I) — found and fixed two: `spec/introduction.md` ("seven" → "eight") and `spec/data-model.md` (kind table + new `### AsciiArtBlock` subsection, mirroring the existing `### ContainerBlock` subsection's format)
- [X] T046 Final regression pass: `cargo test --workspace` full green (210/210); `node protocol/validate.mjs docs/examples/hello.json` still 0 errors/0 warnings (1 info, unchanged); tmux capture of `fireside demo` confirms zero visual change (demo deck intentionally does not gain an `ascii-art` block per ADR-012's Consequences — `demo_deck_shows_every_block_kind` still asserts exactly the original 7 kinds, untouched by this feature, and still passes)

## Dependencies & Execution Order

- **Phase 1 (Setup)** → **Phase 2 (Foundational)**: strictly sequential, blocks everything else.
- **Phase 2 (Foundational)** blocks all of Phase 3–6 — no user story can be demonstrated until the block kind exists on the wire and renders.
- **Phase 3 (US1)** and **Phase 4 (US2)** are independent of each other (different files: `art.rs`'s two functions, `ArtMode`'s two variants) — may be done in either order or in parallel by different contributors.
- **Phase 5 (US3)** depends only on Phase 2 (it tests existing reveal machinery against the new block kind) — does not depend on Phase 3 or 4.
- **Phase 6 (US4)** depends only on Phase 2 (it validates the new block kind's fields) — does not depend on Phase 3, 4, or 5.
- **Phase 7 (Polish)** runs last, after all stories land.

## Parallel Execution Examples

- Within Phase 2: T009 and T010 (independent unit tests) can run in parallel once T006–T008 land; T014 depends on T011–T013 completing first (same file, sequential).
- Across phases: once Phase 2's checkpoint is green, Phase 3, Phase 4, Phase 5, and Phase 6 can all proceed in parallel (touch disjoint files: `art.rs`/`main.rs` CLI dispatch for US1/US2 — same two files, so US1 and US2 should still land as two small sequential edits to `main.rs` even if `art.rs`'s two functions are written in parallel; `render/tests.rs` for US3; `validation.rs`/`validate.mjs`/fixtures for US4 — fully disjoint from the others).

## Implementation Strategy

**MVP scope**: Phase 1 + Phase 2 (Foundational) alone already delivers a
complete, if manually-authored-only, version of the feature — an author
can hand-type an `ascii-art` block and present it correctly. This is a
reasonable stopping point for an early demo.

**Incremental delivery**: Phase 2 → Phase 3 (banner generation, the
single most-requested use case per spec.md's Why-this-priority) → Phase 4
(image conversion) → Phase 5 (reveal/composition confidence) → Phase 6
(authoring-safety warnings) → Phase 7 (polish). Each phase after
Foundational is independently shippable and independently testable per
its own Independent Test criterion in spec.md.
