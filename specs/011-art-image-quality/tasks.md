# Tasks: ASCII Art Image Quality

**Input**: Design documents from `/specs/011-art-image-quality/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/art-image-cli.md, quickstart.md, constitution 1.2.1 (amended to 1.3.0 in this feature's Foundational phase)

**Tests**: Included — Test Discipline (constitution VII) requires unit
tests for the new percentile/stretch math in `fireside-cli` and CLI
end-to-end tests in `fireside-cli/tests/cli_e2e.rs` for every new flag and
the warning path. No `fireside-tui`-visible surface exists in this feature
(it's a CLI stdout/stderr-only change), so no scenario test or tmux smoke
is constitutionally required — quickstart.md's manual eyeball pass covers
the "is it actually recognizable" judgment no automated test can make.

**Organization**: Tasks are grouped by user story per spec.md priorities
(US1 P1 default contrast stretch + `--no-normalize`, US2 P2 `--charset`/
`--invert`, US3 P3 low-range warning, US4 P4 docs/demo-asset refresh),
preceded by a Foundational phase. The `image` crate direct dependency, the
constitution amendment, and the decode-ourselves refactor of
`render_image_ascii` are shared prerequisites every later story's diff
builds on top of (all three stories touch that same function) — they carry
no user-visible behavior change themselves, which is why they're
Foundational rather than folded into US1.

## Phase 1: Setup

- [X] T001 Re-read the current state of every file this feature touches
      immediately before editing (line numbers may have shifted since
      planning): `crates/fireside-cli/src/art.rs`,
      `crates/fireside-cli/src/main.rs`, `crates/fireside-cli/Cargo.toml`,
      `crates/fireside-cli/tests/cli_e2e.rs`,
      `.specify/memory/constitution.md`,
      `docs/src/content/docs/reference/cli.md`,
      `docs/src/content/docs/guides/authoring-markdown.md`

---

## Phase 2: Foundational (blocking prerequisite for all user stories)

**Goal**: `fireside-cli` can decode an image itself, compute a percentile
brightness range, and apply a levels stretch — with zero change to today's
`fireside art image` output. No story is independently testable until this
lands, since US1/US2/US3 all extend the same refactored function.

- [X] T002 In `crates/fireside-cli/Cargo.toml`, add `image = "0.24"` to
      `[dependencies]` (pins to the `image 0.24.9` already resolved
      transitively via `rascii_art` in `Cargo.lock` — confirm
      `cargo tree -i image` still shows one unified version after this
      change, not two subtrees)
- [X] T003 Run `cargo +1.88 build -p fireside-cli --all-targets` and
      confirm it succeeds — a real MSRV build of the actual crate (not just
      the throwaway research.md spike) with the new direct dependency in
      place
- [X] T004 Write `.claude/adrs/adr-013-image-crate-direct-dependency.md`
      recording the decision to add `image` as a direct `fireside-cli`
      dependency, per `research.md` §1/§8 and `plan.md`'s Complexity
      Tracking entry — cite ADR-011's prior "never touches `image` types
      directly" framing and why this feature is the narrow exception
      (`rascii_art`'s public API has no preprocessing hook)
- [X] T005 Amend `.specify/memory/constitution.md`: Principle III's
      `fireside-cli` allowlist row gains `image`; add a Sync Impact Report
      entry at the top of the file (version 1.2.1 → 1.3.0, MINOR per
      Governance's "materially expanded guidance" rule, same class as the
      ADR-006/ADR-011 amendments); update the `**Version**`/`**Last
      Amended**` footer line
- [X] T006 [P] In `crates/fireside-cli/src/art.rs`, add private
      `fn luma(pixel: &image::Rgba<u8>) -> u8` (the same `0.299R + 0.587G +
      0.114B` weighting `rascii_art` itself uses, per research.md §2) and
      `fn percentile_bounds(img: &DynamicImage, lo_pct: f64, hi_pct: f64) ->
      (u8, u8)` (256-bucket luma histogram, cumulative-count lookup for the
      2nd/98th percentile values); add `#[cfg(test)] mod tests` unit tests
      using small in-code synthetic `RgbaImage` buffers (no file I/O) with
      hand-computed expected `(lo, hi)` — include a solid-fill buffer
      asserting `lo == hi`
- [X] T007 [P] In `crates/fireside-cli/src/art.rs`, add private
      `fn stretch(img: &DynamicImage, lo: u8, hi: u8) -> DynamicImage`
      implementing `clamp((channel - lo) * 255 / (hi - lo), 0, 255)` per
      channel per pixel, returning the input unchanged (not merely
      clamped-to-itself, but a true no-op / clone) when `hi <= lo`; unit
      tests asserting exact output pixel values for a small synthetic
      gradient and confirming the solid-fill case returns pixels identical
      to the input
- [X] T008 In `crates/fireside-cli/src/art.rs`, change `render_image_ascii`
      to call `image::open(path)` itself and pass the resulting
      `&DynamicImage` to `rascii_art::render_image_to` (instead of
      delegating decode-and-render to `rascii_art::render_to`'s
      path-based entry point) — pure refactor, **no behavior change**;
      verify by re-running the existing `art_image_converts_a_readable_file`
      and `art_image_reports_a_clear_error_for_a_missing_file` e2e tests
      unmodified before proceeding (depends on T006, T007 existing in the
      file, even though not yet called)

**Checkpoint**: Foundation ready — `fireside art image` behaves exactly as
before, but the module now has the decode-ourselves plumbing and the pure
percentile/stretch functions every user story below wires in.

---

## Phase 3: User Story 1 - Get a recognizable ASCII conversion from an ordinary photo (Priority: P1) 🎯 MVP

**Goal**: `fireside art image` applies the percentile-based contrast
stretch by default; `--no-normalize` reproduces the exact pre-feature
output.

**Independent Test**: Convert a known low-contrast source image with
default flags and confirm the output is recognizable; confirm
`--no-normalize` on the same image reproduces the old (muddy) output
exactly; confirm an already-high-contrast image looks the same with or
without the stretch.

### Tests for User Story 1

- [X] T009 [P] [US1] Add a new fixture
      `crates/fireside-cli/tests/fixtures/low-contrast.png`: a small (e.g.
      16×16) PNG whose luma values cluster in a narrow band (e.g. roughly
      100–140, well under the 40% low-range threshold), for the e2e tests
      below — generate it with a short one-off script (e.g. via the
      `image` crate in a scratch binary) rather than hand-editing bytes
- [X] T010 [P] [US1] Unit test in `crates/fireside-cli/src/art.rs`: given
      an in-code synthetic low-contrast `RgbaImage` buffer, stretching via
      `percentile_bounds` + `stretch` widens the pixel value spread
      compared to the unstretched buffer (assert min/max post-stretch are
      further apart than pre-stretch)
- [X] T011 [P] [US1] Unit test in `crates/fireside-cli/src/art.rs`: given
      an in-code synthetic full-range buffer (luma already spanning close
      to 0–255), stretching produces output identical (or negligibly
      different) to the input — confirms no regression on already-good
      images

### Implementation for User Story 1

- [X] T012 [US1] In `crates/fireside-cli/src/main.rs`'s `ArtMode::Image`
      variant, add `#[arg(long)] no_normalize: bool`
- [X] T013 [US1] In `crates/fireside-cli/src/art.rs`, extend
      `render_image_ascii`'s signature with `no_normalize: bool`; when
      `false` (default), call `percentile_bounds` then `stretch` on the
      decoded image before handing it to `rascii_art::render_image_to`;
      when `true`, skip both and pass the decoded image through unchanged
      (depends on T006, T007, T008)
- [X] T014 [US1] Update `art_image` in `art.rs` and the `ArtMode::Image`
      dispatch arm in `main.rs` to thread `no_normalize` through (depends
      on T012, T013)
- [X] T015 [P] [US1] e2e test in `crates/fireside-cli/tests/cli_e2e.rs`:
      `art_image_stretches_low_contrast_image_by_default` — convert the
      T009 fixture with default flags and again with `--no-normalize`;
      assert the default run's output uses a strictly wider variety of
      distinct characters than the `--no-normalize` run's output
- [X] T016 [P] [US1] e2e test in `cli_e2e.rs`:
      `art_image_no_normalize_reproduces_prior_behavior` — convert
      `tests/fixtures/tiny.png` (the existing fixture) with
      `--no-normalize` and assert it still succeeds and produces output
      with more than one line (same assertion shape as the existing
      `art_image_converts_a_readable_file`, confirming the opt-out path
      works end to end)

**Checkpoint**: User Story 1 is fully functional and independently
testable — this is the MVP. `fireside art image` now produces recognizable
output on low-contrast photos by default.

---

## Phase 4: User Story 2 - Choose a different look for the conversion (Priority: P2)

**Goal**: `--charset <default|block|slight>` and `--invert` are surfaced on
`fireside art image`.

**Independent Test**: Convert the same image with each charset choice and
once with `--invert`; confirm each produces valid, visibly different
output; confirm `--charset default` (or no flag) matches today's default
exactly.

### Tests for User Story 2

- [X] T017 [P] [US2] Unit test in `crates/fireside-cli/src/art.rs`: each
      `ArtCharset` variant maps to the correct `rascii_art::charsets`
      constant (`DEFAULT`/`BLOCK`/`SLIGHT`)

### Implementation for User Story 2

- [X] T018 [US2] In `crates/fireside-cli/src/main.rs`, add
      `#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
      #[value(rename_all = "kebab-case")] enum ArtCharset { Default, Block,
      Slight }`, matching the existing `Template` enum's pattern
- [X] T019 [US2] Add `#[arg(long, value_enum, default_value_t =
      ArtCharset::Default)] charset: ArtCharset` and `#[arg(long)] invert:
      bool` to `ArtMode::Image` in `main.rs` (depends on T018)
- [X] T020 [US2] In `crates/fireside-cli/src/art.rs`, extend
      `render_image_ascii`'s signature with `charset: ArtCharset, invert:
      bool`; map `ArtCharset` to the corresponding `rascii_art::charsets`
      slice and pass both into `RenderOptions` via `.charset(...)` and
      `.invert(invert)` (depends on T017, T018)
- [X] T021 [US2] Update `art_image` in `art.rs` and the dispatch arm in
      `main.rs` to thread `charset`/`invert` through (depends on T020)
- [X] T022 [P] [US2] e2e test in `cli_e2e.rs`:
      `art_image_charset_flag_changes_output_characters` — convert
      `tests/fixtures/tiny.png` with `--charset block` and with no flag;
      assert the two outputs use different character sets (e.g. the block
      run's output is a subset of `" ░▒▓█"`, the default run's is not)
- [X] T023 [P] [US2] e2e test in `cli_e2e.rs`:
      `art_image_invert_flag_flips_shading` — convert `tiny.png` with and
      without `--invert`; assert the two outputs differ
- [X] T024 [P] [US2] e2e test in `cli_e2e.rs`:
      `art_image_default_charset_matches_unflagged_output` — assert
      `--charset default` output is byte-identical to no-flag output for
      the same file

**Checkpoint**: User Stories 1 and 2 both work independently.
`fireside art image` now supports charset/invert selection on top of the
default stretch.

---

## Phase 5: User Story 3 - Find out why a conversion still looks muddy (Priority: P3)

**Goal**: A low-range source image triggers a stderr warning naming the
condition and suggesting `--invert` or a higher-contrast image, without
altering or blocking stdout.

**Independent Test**: Convert a deliberately flat/featureless test image
and confirm a warning appears on stderr while the image still converts and
prints to stdout; confirm a normal-contrast image produces no such warning.

### Tests for User Story 3

- [X] T025 [P] [US3] Add a new fixture
      `crates/fireside-cli/tests/fixtures/flat.png`: a small PNG whose luma
      range spans well under 40% of 0–255 (narrower than T009's
      `low-contrast.png`, deliberately in the warning-triggering zone)
- [X] T026 [P] [US3] Unit test in `crates/fireside-cli/src/art.rs`: a small
      parametrized check (no image I/O) that the "low range" condition
      (`hi - lo < 102`) is true/false at representative `(lo, hi)` pairs
      straddling the threshold

### Implementation for User Story 3

- [X] T027 [US3] In `crates/fireside-cli/src/art.rs`'s `render_image_ascii`,
      after computing `percentile_bounds` (already done for the stretch in
      T013), check `hi - lo < 102`; if true, `eprintln!` a note naming the
      approximate percentage of the range used and suggesting `--invert` or
      a higher-contrast source image — this check and print run
      **regardless** of `no_normalize` (depends on T013)
- [X] T028 [P] [US3] e2e test in `cli_e2e.rs`:
      `art_image_warns_on_stderr_for_low_contrast_source` — convert the
      T025 `flat.png` fixture; assert stderr contains a note and stdout
      still contains more than one line of output
- [X] T029 [P] [US3] e2e test in `cli_e2e.rs`:
      `art_image_silent_on_stderr_for_normal_contrast_source` — convert
      `tests/fixtures/tiny.png`; assert stderr is empty
- [X] T030 [P] [US3] e2e test in `cli_e2e.rs`:
      `art_image_warning_fires_even_with_no_normalize` — convert the T025
      fixture with `--no-normalize`; assert the stderr note still appears
      (confirms the warning is independent of the stretch opt-out, per
      research.md §5)

**Checkpoint**: All three CLI-behavior stories (US1–US3) are independently
functional. Every flag from `contracts/art-image-cli.md` now exists and
behaves as specified.

---

## Phase 6: User Story 4 - Trust the documentation's example (Priority: P4)

**Goal**: The published CLI reference documents every new flag; the
before/after example image is a high-contrast, recognizable subject.

**Independent Test**: Follow the documented example (source image plus
command) and confirm the output resembles both the documented output and
the depicted subject.

### Implementation for User Story 4

- [X] T031 [US4] Source a new CC0, high-contrast, simple-silhouette
      replacement for `.github/demo-art.png` (a single well-lit subject
      against a plain background), following the same sourcing/attribution
      approach as the current photo (Wikimedia Commons CC0 search, credited
      inline)
- [X] T032 [US4] Update
      `docs/src/content/docs/reference/cli.md`'s `fireside art image` flag
      table: add rows for `--charset`, `--invert`, `--no-normalize` (name,
      values/default, effect — per `contracts/art-image-cli.md`); update
      the source-photo caption and attribution credit for the T031 image
- [X] T033 [P] [US4] Review
      `docs/src/content/docs/guides/authoring-markdown.md`'s
      `fireside art image` pointer (~L135-137) for staleness against the
      new flags — per research.md §7 this pointer is generic and likely
      needs no edit; confirm rather than assume, and edit only if it names
      specific old behavior
- [X] T034 [US4] Regenerate `.github/art-image.gif` via `scripts/demos.sh`
      against the new demo image; visually confirm the recording shows
      recognizable output (depends on T031, and on US1–US3 being
      implemented so the recorded behavior reflects the finished feature)
- [X] T035 [US4] Run `cd docs && npm run check && npm run build`; confirm
      clean with the updated flag table and image reference

**Checkpoint**: Documentation and demo assets reflect the finished feature.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T036 Update `.claude/plans/2026-07-18-ux-polish-plan.md`'s Progress
      Log: mark `W3 (spec 011) art image quality` done with a one-line
      summary, per that file's own "update whenever an item lands"
      instruction
- [X] T037 Run the full verification gate: `cargo +1.88 build -p
      fireside-cli --all-targets`, `cargo test --workspace`, `cargo clippy
      --workspace --all-targets -- -D warnings`, `cargo fmt --check`,
      `scripts/verify.sh`
- [X] T038 Run `graphify update .` to refresh the knowledge graph
- [X] T039 Manually run `quickstart.md`'s five scenarios end to end,
      eyeballing the regenerated GIF and the updated `reference/cli.md`
      page — no automated test substitutes for judging "is this
      recognizable"

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup. **Blocks all user
  stories** — T008's decode-ourselves refactor is what US1/US2/US3 all
  extend.
- **User Story 1 (Phase 3)**: Depends on Foundational. No dependency on
  US2/US3.
- **User Story 2 (Phase 4)**: Depends on Foundational. Shares
  `render_image_ascii`'s signature with US1 — sequenced after US1 in this
  task list to avoid two stories editing the same function signature
  concurrently, not because US2's *behavior* logically requires the
  stretch to exist.
- **User Story 3 (Phase 5)**: Depends on Foundational and specifically on
  T013 (US1) having already added the `percentile_bounds` call site it
  reuses — a real logical dependency, not just file-sequencing.
- **User Story 4 (Phase 6)**: Depends on US1–US3 being implemented (T034's
  GIF recording must show the finished behavior) — this is the one story
  that genuinely cannot start earlier.
- **Polish (Phase 7)**: Depends on all prior phases.

### Parallel Opportunities

- T006 and T007 (Phase 2) touch the same file but different, independent
  functions — safe to implement together, though as a solo implementer
  sequential is simpler than true parallelism.
- T009/T010/T011 (US1 tests) can be done in parallel — different files
  (fixture PNG vs. unit tests) with no shared state.
- T015/T016 (US1 e2e) can be done in parallel with each other, after T014.
- T022/T023/T024 (US2 e2e) are independent of each other.
- T025/T026 (US3 tests) are independent of each other.
- T028/T029/T030 (US3 e2e) are independent of each other.
- T033 (docs review) can happen any time after Setup — it's read-mostly
  and touches a file nothing else in this feature edits.

---

## Parallel Example: User Story 1

```bash
# Once Phase 2 (Foundational) is complete, these can run together:
Task: "Add fixture crates/fireside-cli/tests/fixtures/low-contrast.png"
Task: "Unit test: stretch widens spread on a synthetic low-contrast buffer"
Task: "Unit test: stretch is a no-op on a synthetic full-range buffer"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational (image dependency, ADR-013, constitution
   amendment, percentile/stretch functions, decode-ourselves refactor).
3. Complete Phase 3: User Story 1.
4. **STOP and VALIDATE**: run quickstart.md scenarios 1–2 against a real
   low-contrast photo; confirm recognizable output.
5. This alone resolves the reported "undecipherable output" complaint.

### Incremental Delivery

1. Setup + Foundational → foundation ready, no behavior change yet.
2. US1 → default output fixed (MVP) → validate independently.
3. US2 → manual override flags available → validate independently.
4. US3 → diagnostic warning for the remaining hard cases → validate
   independently.
5. US4 → documentation and demo asset catch up to the now-finished
   behavior.
6. Polish → full verification gate, plan progress log update.

---

## Notes

- [P] tasks touch different files (or, within `art.rs`, different
  independent functions) with no shared in-progress edits.
- Every unit test added in Phases 2–5 uses in-code synthetic pixel buffers,
  not file I/O — keeps `cargo test` fast and keeps the two checked-in PNG
  fixtures (T009, T025) reserved for what only an e2e test can exercise
  (the full CLI invocation, stdout/stderr shape).
- Commit after each phase's checkpoint, not after every individual task —
  matches this project's existing per-feature commit granularity
  (`git log` shows one commit per completed Spec Kit feature).
- Avoid: adding a fourth PNG fixture when an existing one already covers
  the case (`tiny.png` already serves every "normal contrast" assertion
  needed across US1–US3).
