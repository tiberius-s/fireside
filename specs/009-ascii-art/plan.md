# Implementation Plan: ASCII art content block

**Branch**: `009-ascii-art` | **Date**: 2026-07-18 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/009-ascii-art/spec.md`

## Summary

Add a new `ascii-art` `ContentBlock` variant (protocol 0.1.3) carrying
pre-rendered, plain-text art plus an optional `alt` description and the
standard `reveal` field. The TUI renders it centered and sized-to-content
by reusing the width/centering math the existing language-less-code-block
ASCII-art path (`spec 005`) already has, factored into a shared helper —
no new `fireside-tui` dependency. Two new `fireside-cli`-only subcommands,
`fireside art text` and `fireside art image`, generate ready-to-paste art
via `figlet-rs` and `rascii_art` respectively (GO decision, ADR-011); they
print art to stdout, they do not edit deck files. Two new symmetric
validator warnings (`ascii-art-too-wide`, `ascii-art-empty`) mirror the
existing Rust/Node parity pattern established by `reveal-masked-by-container`.
Because this is a new block *kind* (not an additive field like 0.1.2's
`reveal`), older engines cannot safely degrade — ADR-012 records this
compatibility break explicitly, unlike every prior additive protocol
change this project has shipped.

## Technical Context

**Language/Version**: Rust 1.88 (workspace MSRV), 2024 edition; Node.js
(ESM) for `protocol/validate.mjs`; TypeSpec for `protocol/main.tsp`.

**Primary Dependencies**: `figlet-rs` and `rascii_art`, both new,
`fireside-cli`-only (ADR-011 GO decision, constitution Principle III
amendment below). No new dependency in `fireside-core`, `fireside-engine`,
or `fireside-tui` — the headline property of the C-1 design (protocol
main.tsp — B-2/models.yml already runs symmetric fixture parity in CI, so
adds a data-only block kind; renderer reuses existing code).

**Storage**: N/A (art is plain text embedded in the deck JSON document,
same as every other content block).

**Testing**: `cargo test --workspace` (unit tests in `fireside-core` for
the new variant + serde round-trip, `fireside-engine` for the two new
validation rules, `fireside-tui` scenario/insta tests for rendering,
`fireside-cli` `tests/cli_e2e.rs` for the two new subcommands); `node
protocol/run-fixtures.mjs` + new fixtures for validator parity (B-2
already wires this into CI); a tmux real-terminal smoke of an ascii-art
slide (constitution Principle VII, this project's established discipline
for UI changes — `TestBackend` alone has missed real-terminal-only bugs
before, per project history).

**Target Platform**: Same as the rest of Fireside — macOS/Linux terminal,
truecolor, monospace font.

**Project Type**: CLI + TUI application (existing 4-crate workspace:
`fireside-core` → `fireside-engine` → `fireside-tui` → `fireside-cli`).

**Performance Goals**: N/A — art generation is a one-shot CLI invocation
against small local inputs (a phrase or a small image file), not a hot
path. No new goal beyond "completes promptly for a single small image."

**Constraints**: MSRV 1.88 (verified for both new dependencies by
ADR-011's real `cargo +1.88 build`/`run`, not just declared metadata).
Generated art MUST be plain text — no ANSI color/formatting codes
embedded — per constitution Principle IV ("all visual styling flows
through `theme.rs::Tokens`").

**Scale/Scope**: One new `ContentBlock` variant, one new TUI render
helper, two new validator rules (×2 languages), one new CLI subcommand
with two verbs, one constitution amendment, two ADRs (011 already
written, 012 for the protocol change).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Spec Is the Source of Truth)**: `main.tsp` changes first;
  `tsp-output/` regenerated and committed; `docs/examples/hello.json`
  stays parseable (it will not use `ascii-art` — no existing-content
  change required, it just must keep validating after the schema grows).
  **PASS**, by construction — this plan's ordering puts the TypeSpec
  change before any Rust code.
- **Principle II (Presenter-First Experience)**: adding `fireside art
  text`/`fireside art image` grows CLI scope beyond
  `present`/`validate`/`new` (ADR-004's baseline). The constitution
  permits scope growth only when "the user explicitly asks for it" — this
  plan traces directly to the user-approved 2026-07-17 strategic plan's
  Stream C (C-4 names `fireside art` by name) and the user's own
  2026-07-18 instruction to implement Stream C, which is the same bar
  `import` cleared via ADR-006. **PASS**, with that provenance recorded
  here rather than re-litigated.
- **Principle III (Crate Boundary Discipline)**: `fireside-cli` gains
  `figlet-rs`, `rascii_art`. Every other crate's row is unchanged — no
  proposal here touches `fireside-tui`'s dependency list, which is this
  design's headline property (C-1). **PASS, pending** the constitution
  amendment below (drafted as part of this plan, not deferred).
- **Principle IV (Mandatory Code Idioms)**: new `ContentBlock::AsciiArt`
  match arms needed in `reveal()`/`children()` (trivial, mirrors
  `Divider`/existing arms); no `unwrap()`/`expect()` in the new library
  code (CLI-boundary `anyhow::Result` only); `#[must_use]` on new public
  fns; kebab-case serde untouched (`kind: "ascii-art"` matches the
  existing discriminator pattern). **PASS** — no new idiom introduced,
  only extension of existing ones.
- **Principle V (Stratified Error Handling)**: the two CLI verbs are
  CLI-boundary code — `anyhow::Result` with context, exactly like
  `import_file`. No new error variant needed in `CoreError`/`EngineError`/
  `TuiError` (parsing/rendering a block with a plain `String` field
  introduces no new failure mode beyond what already exists for
  `TextBlock`/`CodeBlock`). **PASS**.
- **Principle VI (MSRV 1.88)**: verified for both new dependencies by
  ADR-011 via a real `cargo +1.88 build`/`run`, not declared metadata
  alone. **PASS**.
- **Principle VII (Test Discipline)**: new unit tests at every layer
  (core round-trip + `reveal()`/`children()` arm, engine validation rules,
  TUI scenario test + insta snapshot for the render path, CLI
  `cli_e2e.rs` for both new verbs), plus the mandatory tmux smoke test for
  the UI change. **PASS**, planned explicitly in tasks.md (next phase).

No unresolved violations. **Complexity Tracking section is empty below —
not needed.**

## Project Structure

### Documentation (this feature)

```text
specs/009-ascii-art/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md         # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   ├── ascii-art-block.md
│   └── cli-art-command.md
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
protocol/
├── main.tsp                      # + AsciiArtBlock model, + ContentBlock member, + v0_1_3
├── validate.mjs                  # + checkAsciiArtTooWide, + checkAsciiArtEmpty
├── fixtures/                     # + ascii-art fixtures (valid + both new warnings)
└── fixtures.expected.json        # + expected diagnostics for the above

crates/fireside-core/src/model/mod.rs
    # + ContentBlock::AsciiArt variant, + reveal()/children() match arms,
    # + proptest_support generator arm, + unit tests

crates/fireside-engine/src/validation.rs
    # + check_ascii_art_too_wide, + check_ascii_art_empty (called from validate())

crates/fireside-tui/src/render/blocks.rs
    # + ascii_art() render fn (factors the existing is_ascii_art()/
    #   box-width centering logic out of code() into a shared helper both
    #   call)
crates/fireside-tui/src/render/tests.rs   # + scenario test(s)
crates/fireside-tui/src/render/snapshots/ # + insta snapshot(s) if needed

crates/fireside-cli/
├── Cargo.toml            # + figlet-rs, + rascii_art
├── src/main.rs            # + Command::Art { .. } subcommand + dispatch
└── src/art.rs             # NEW — `art_text`, `art_image` (mirrors new.rs's
                            # shape: thin CLI-boundary fns, anyhow::Result)
crates/fireside-cli/tests/cli_e2e.rs  # + tests for `fireside art text`/`image`

.specify/memory/constitution.md   # Principle III amendment (fireside-cli row)
.claude/adrs/adr-012-ascii-art-protocol-change.md   # NEW (protocol/compat ADR)
docs/src/content/docs/...         # spec-site regeneration follows existing
                                    # docs build (no manual edit needed beyond
                                    # what `npm run build` regenerates)
```

**Structure Decision**: Existing 4-crate workspace, no new crate. New CLI
logic lives in a new sibling module `fireside-cli/src/art.rs`, following
the same post-A-2 pattern as `new.rs`/`import.rs`/`report.rs` (thin,
testable, `pub(crate)` fns called from `main.rs`'s dispatch match). New
protocol validation logic is added as new functions in the existing
`validation.rs`/`validate.mjs`, called from the existing `validate()`
dispatcher — no new file, consistent with how `reveal-masked-by-container`
and `container-nesting-depth-exceeded` were added in spec 006/008.

## Complexity Tracking

*No Constitution Check violations — this section is intentionally empty.*
