---
title: 'ADR-008: ratatui-image MSRV spike — no-go for now'
status: 'accepted'
date: '2026-07-12'
deciders: ['@tiberius']
---

# ADR-008: ratatui-image MSRV spike — no-go for now

## Status

Accepted (records a NO-GO decision, not an adoption)

## Context

The strategic plan (`.claude/plans/2026-07-12-strategic-improvement-plan.md`,
§3 Week 2) names terminal image rendering via `ratatui-image` as a P1 item,
with an explicit gate: "Spike `ratatui-image`: MSRV check, terminal coverage
matrix, fallback behavior. Go/no-go by mid-week." The constitution
(`.specify/memory/constitution.md`, Principle III) requires an ADR before
any crate boundary/dependency change, and `fireside-tui`'s permitted
dependency list does not currently include `ratatui-image` or `image`.
MSRV is pinned at 1.88 (`Cargo.toml: rust-version = "1.88"`), and the
1.88 toolchain is installed locally, so the spike could verify against a
real compiler rather than trusting declared `rust-version` metadata alone.

The spike (throwaway `cargo init` scratch project, not committed) added
`ratatui-image` and inspected the resulting dependency graph:

- `ratatui-image` latest (v11.0.6) resolves against `ratatui = "0.30.2"`,
  matching Fireside's current `ratatui` version — good.
- `cargo tree` showed 176 unique transitive packages pulled in by
  `ratatui-image` alone (default features: `chafa-dyn`, `crossterm`,
  `image-defaults`).
- `cargo metadata` flagged three packages in that tree whose declared
  `rust-version` exceeds 1.88: `quantette@0.5.1` (1.90), `wide@0.8.3`
  (1.89), `safe_arch@0.9.3` (1.89).
- Ran `cargo +1.88 build` for real (not just metadata inspection) — it
  failed outright:
  ```
  error: rustc 1.88.0 is not supported by the following packages:
    quantette@0.5.1 requires rustc 1.90
    safe_arch@0.9.3 requires rustc 1.89
    wide@0.8.3 requires rustc 1.89
  ```
- Traced the chain: `ratatui-image` → `icy_sixel` (sixel encoder) →
  `quantette` (color quantization) → `wide`/`safe_arch` (SIMD). Checked
  `icy_sixel`'s position in `ratatui-image`'s own `Cargo.toml` — it is a
  **plain, non-optional dependency**, not gated behind any of
  `ratatui-image`'s own Cargo features. Disabling `chafa-dyn` or
  `image-defaults` does not remove it; there is no feature combination
  that avoids this chain on any `ratatui-image` version compatible with
  `ratatui` 0.30.
- Tried pinning the offending transitive crates to older, MSRV-compatible
  versions directly (`cargo update -p quantette --precise 0.5.0`, etc.) —
  `icy_sixel@0.5.0` hard-pins `quantette = "^0.5.1"`, which is the only
  published 0.5.x release, so there is no compatible version to pin to.
- Tried older `ratatui-image` majors as an escape hatch: v10.0.8 has the
  identical `icy_sixel`/`quantette`/`wide`/`safe_arch` chain and the same
  build failure. v8.1.1 (the last major before this chain was introduced)
  *does* build cleanly on 1.88, but drags `ratatui` back to `0.29.0` —
  incompatible with the `ratatui 0.30` already used throughout
  `fireside-tui`. Adopting it would mean downgrading ratatui workspace-wide
  and re-verifying every existing render function against an older ratatui
  API, which is a far bigger and riskier change than "add image support."
- Secondary finding, not the deciding factor but worth recording: the
  default feature `chafa-dyn` links against system `libchafa` via
  `pkg-config` at build time. This machine does not have `libchafa`
  installed (`pkg-config --exists chafa` fails), so even setting the MSRV
  issue aside, the default feature set introduces a non-Rust system
  dependency that Fireside has never needed before. `chafa-static` and
  disabling chafa entirely (falling back to `ratatui-image`'s built-in
  halfblock renderer) are both possible mitigations for this one, unlike
  the MSRV chain which has none.

## Decision

**No-go on `ratatui-image` at this time.** Do not add it to
`fireside-tui`'s permitted dependencies. The MSRV violation is
unconditional (not feature-gated, not resolvable by pinning, confirmed by
an actual `cargo +1.88 build` failure, not just declared-metadata
inspection) on every `ratatui-image` release compatible with the
`ratatui` version Fireside already uses. The only workaround (downgrading
to `ratatui-image` v8.x / `ratatui` 0.29) trades one blocker for a much
larger one and is rejected as out of proportion to the feature.

Terminal image rendering remains a named, deliberate gap (as it already
was per ADR-004's original scope note) rather than being closed this
phase. The placeholder text-box renderer in `blocks.rs::image()` is
unchanged and remains the only image rendering Fireside does.

## Consequences

### Positive

- No MSRV promise broken; no new native (non-Rust) build dependency
  introduced; no unplanned `ratatui` downgrade forced onto the rest of
  `fireside-tui`.
- The spike was cheap (a throwaway scratch project, ~30 minutes of cargo
  commands) and produced a decisive, empirically-verified answer instead
  of a guess — consistent with this project's practice of verifying
  claims against real toolchains/terminals rather than trusting declared
  metadata (see also the tmux smoke-testing discipline used elsewhere in
  this codebase).
- Frees Week 2 to proceed directly to incremental reveal (`reveal` field),
  which has no such dependency blocker.

### Negative or Trade-offs

- The strategic plan's P1 "real images" item does not land this phase.
  Fireside continues to show a placeholder box for `image` blocks.
- This blocker is upstream and time-dependent: `icy_sixel`/`quantette`
  may lower their MSRV in a future release, or Fireside's own MSRV may
  rise past 1.90 eventually, either of which would reopen this decision.
  Nothing here is permanent.

### Neutral / Follow-up

- Re-spike `ratatui-image` (or alternatives — e.g. driving kitty/iTerm2/
  sixel protocols directly without `icy_sixel`'s quantization path, or a
  lighter sixel-only crate) if either (a) Fireside's MSRV is deliberately
  raised past 1.90 in a future ADR, or (b) `icy_sixel`/`quantette` ship a
  release with a lower MSRV. Check `cargo tree -i quantette` again before
  re-attempting — this ADR's finding is specific to `icy_sixel@0.5.0`.
  Note also that even a future MSRV-clean adoption should default to
  `chafa-static` or no-chafa (halfblock-only) to avoid the `pkg-config`/
  system-library dependency found here.
- Ambiguity #6 from ADR-007 (image width/height MUST clamp to the content
  area) remains forward-looking guidance for whenever real image sizing
  does land — it was written knowing the placeholder renderer doesn't yet
  interpret those fields, and that remains true after this ADR.
- Update `.claude/plans/2026-07-12-strategic-improvement-plan.md`'s
  Progress Log and Week 2 plan to reflect the no-go, and proceed to the
  other P1 item (incremental reveal) next.
