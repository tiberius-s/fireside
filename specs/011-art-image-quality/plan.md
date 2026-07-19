# Implementation Plan: ASCII Art Image Quality

**Branch**: `011-art-image-quality` | **Date**: 2026-07-18 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/011-art-image-quality/spec.md`

## Summary

`fireside art image` currently hands `rascii_art` the raw decoded image with
no preprocessing; `rascii_art`'s own grayscale→char mapping normalizes only
by the single brightest pixel in the (possibly outlier-skewed) image, so an
ordinary low-contrast photo converts to a wall of near-identical dark
glyphs. The fix computes a 2nd/98th-percentile luminance range from the
source image, applies a linear levels stretch across that range to all
three color channels before handing the result to `rascii_art`, and skips
the stretch entirely as a no-op when the image has no meaningful range (a
solid fill). Two new flags (`--charset`, `--invert`) surface `rascii_art`'s
existing capability that the CLI never exposed; a third (`--no-normalize`)
opts out of the new stretch to reproduce today's exact behavior. A stderr
warning fires when the pre-stretch range is unusually narrow, using the
same percentile values already computed for the stretch. Documentation and
the demo asset are refreshed last, once the improved output exists to show.

## Technical Context

**Language/Version**: Rust 1.88 (workspace MSRV, 2024 edition, `resolver = "3"`)

**Primary Dependencies**: `rascii_art = "0.4"` (already a `fireside-cli`
dependency, ADR-011) for character rendering; `image = "0.24"` — **new
direct dependency**, pinned to the same 0.24.x line `rascii_art` 0.4.5
already vendors (`Cargo.lock` already resolves `image 0.24.9` transitively,
so a direct `^0.24` requirement unifies onto the existing resolved version
rather than creating a second `image` major-version subtree); `clap`
(already present) for the new `ValueEnum` charset flag.

**Storage**: N/A — no persisted state; all conversion is in-memory,
stdout/stderr only.

**Testing**: `cargo test` unit tests in `fireside-cli/src/art.rs` for the
percentile/stretch math (deterministic small pixel buffers with hand-computed
expected output — plain assertions, no snapshot crate needed for a handful
of numeric values); `fireside-cli/tests/cli_e2e.rs` end-to-end tests for the
new flags and the warning path, following the existing `art_image_*`
test conventions in that file.

**Target Platform**: Same as the rest of `fireside-cli` — macOS and Linux,
terminal/CLI.

**Project Type**: Single Rust workspace, CLI crate (`fireside-cli`) —
no new crate.

**Performance Goals**: Not a hot path — `fireside art image` is a one-shot,
human-in-the-loop authoring command; correctness and clarity of output
matter, not throughput. A single extra full-image pass (histogram +
per-pixel remap) alongside the existing decode/thumbnail work is
negligible for the small images this command targets.

**Constraints**: Must not change the text-banner path (`art text`,
`new --banner`) at all (FR-008); must not change output for images that
already use the full brightness range (spec User Story 1, Acceptance
Scenario 2); must not panic or divide by zero on a solid-fill image (Edge
Cases).

**Scale/Scope**: One CLI subcommand's implementation
(`crates/fireside-cli/src/art.rs`), its clap flag surface
(`crates/fireside-cli/src/main.rs`), its Cargo manifest, one constitution
allowlist line, one ADR, two doc pages, and one demo asset + GIF.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Spec Is the Source of Truth** — N/A to this feature; no protocol
  wire format, no `ContentBlock` kind, and no `docs/examples/hello.json`
  change. `fireside art image` is a CLI authoring convenience that never
  touches a deck file (per `art.rs`'s existing module doc). **Pass.**
- **II. Presenter-First Experience** — N/A directly (this is an authoring
  command, not the presenter runtime), but it serves the same audience
  indirectly: clearer art means decks presenters show are legible. No
  scope creep into `present`/`validate`/`new`'s core behavior. **Pass.**
- **III. Crate Boundary Discipline** — **Violation, justified below.**
  `fireside-cli`'s allowlist does not currently include `image`; this
  feature needs it directly (not just transitively through `rascii_art`)
  to run the percentile/stretch computation before handing a
  `DynamicImage` to `rascii_art::render_image_to`. `rascii_art`'s public
  API (`RenderOptions` + `render_image_to`) has no preprocessing hook, so
  there is no way to do a levels stretch without constructing/mutating
  image data directly. See Complexity Tracking. All other crates
  (`fireside-core`, `fireside-engine`, `fireside-tui`) are untouched by
  this feature — the boundary the feature actually cares about
  (`fireside-tui` never gaining image/render dependencies) holds exactly
  as ADR-011 established.
- **IV. Mandatory Code Idioms** — new code lives in `fireside-cli`
  (`main()`-adjacent boundary), so `unwrap()`/`expect()` restrictions for
  library crates don't apply, but the plan still avoids them in the new
  `art.rs` functions (percentile/stretch is pure `Vec`/slice arithmetic
  with no fallible unwraps needed); `#[must_use]` and `///` docs apply to
  new `pub(crate)` functions per existing `art.rs` style. **Pass.**
- **V. Stratified Error Handling** — new code stays inside the CLI
  boundary layer (`anyhow::Result`), consistent with the rest of `art.rs`.
  No new library-crate error types needed. **Pass.**
- **VI. MSRV 1.88** — `image 0.24.9` is already resolved in `Cargo.lock`
  and was already exercised end-to-end (decode → grayscale → charset
  shading) under `cargo +1.88 run` during ADR-011's spike; the additional
  API surface this feature needs (`GenericImageView`/`ImageBuffer` pixel
  iteration, already used internally by `rascii_art` itself at the same
  version) needs no new MSRV verification beyond a real `cargo +1.88
  build -p fireside-cli` before this feature ships. **Pass, pending that
  build check in Phase 0.**
- **VII. Test Discipline** — unit tests for the stretch/percentile math in
  `fireside-cli/src/art.rs`, CLI e2e tests in `cli_e2e.rs` for the new
  flags and warning — matches the "CLI behavior is covered end-to-end in
  `fireside-cli/tests/cli_e2e.rs`" requirement. No TUI-visible surface, so
  no scenario test or tmux smoke is required by the constitution itself;
  the plan still smoke-tests manually during verification since this is a
  visible-output change a human should eyeball. **Pass.**

**Overall**: one justified violation (Principle III, `image` as a direct
`fireside-cli` dependency) — see Complexity Tracking. No other gate fails.

## Project Structure

### Documentation (this feature)

```text
specs/011-art-image-quality/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/
├── fireside-cli/
│   ├── Cargo.toml               # add `image = "0.24"` direct dependency
│   ├── src/
│   │   ├── main.rs              # ArtMode::Image gains --charset/--invert/--no-normalize
│   │   └── art.rs               # new: percentile/stretch fns, charset mapping,
│   │                             #      low-range warning; render_image_ascii/art_image
│   │                             #      updated to call them
│   └── tests/
│       └── cli_e2e.rs           # new e2e cases for flags + warning
├── (fireside-core, fireside-engine, fireside-tui — untouched)

.github/
├── demo-art.png                 # replaced with a high-contrast CC0 subject
└── art-image.gif                # re-recorded via scripts/demos.sh

docs/src/content/docs/
├── reference/cli.md             # flag table + before/after image update
└── guides/authoring-markdown.md # pointer text, if it names old flags/behavior

.claude/adrs/
└── adr-013-*.md                 # new: image crate direct-dependency amendment

.specify/memory/constitution.md  # Principle III allowlist: fireside-cli + `image`
```

**Structure Decision**: Single Rust workspace, existing structure — this
feature is scoped entirely inside `fireside-cli` (one new direct
dependency, one module's worth of new functions, one CLI flag surface) plus
docs/demo assets. No new crate, no changes to `fireside-core`,
`fireside-engine`, or `fireside-tui`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|---------------------------------------|
| `fireside-cli` gains a direct `image` dependency (Principle III allowlist currently omits it — ADR-011 explicitly recorded `fireside-cli` as never touching `image` types directly, "file paths in, String ASCII out") | The percentile-based contrast stretch (FR-001/FR-002) must run on decoded pixel data *before* `rascii_art` sees it. `rascii_art`'s only entry points are `render`/`render_to` (path-based) and `render_image`/`render_image_to` (`&DynamicImage`-based) — none accept a preprocessing callback or a pre-normalized-range hint. The only way to intervene between decode and character-mapping is to decode the image ourselves, stretch it, and pass the resulting `DynamicImage` to `render_image_to`. | Doing the stretch inside `rascii_art` itself (a fork/patch) was rejected — forking a 3rd-party crate for one feature is a heavier, harder-to-maintain change than depending on a crate already fully resolved and MSRV-proven in the tree. Doing it without the `image` crate's pixel types (e.g. hand-rolling a PNG/JPEG decoder) was rejected as reinventing a well-tested wheel for no benefit — the crate is already present transitively at the exact version needed, so a direct dependency adds no new supply-chain surface, only a `Cargo.toml` line and a constitution allowlist edit. |
