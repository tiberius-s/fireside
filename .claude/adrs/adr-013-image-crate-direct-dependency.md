---
title: 'ADR-013: `image` crate as a direct `fireside-cli` dependency'
status: 'accepted'
date: '2026-07-18'
deciders: ['@tiberius']
---

# ADR-013: `image` crate as a direct `fireside-cli` dependency

## Status

Accepted.

## Context

Spec `011-art-image-quality` (`.claude/plans/2026-07-18-ux-polish-plan.md`
Wave 3) fixes `fireside art image`'s undecipherable output on ordinary
low-contrast photos: `rascii_art`'s own grayscale→char mapping
(`image_renderer.rs::get_char_for_pixel`) normalizes only by the single
brightest pixel found anywhere in the image, so a photo with a few
near-white outlier pixels (a highlight, a blown sky corner) sets that
ceiling near 255 even though the actual subject sits in a much narrower
dark band — the existing normalization doesn't help because an outlier,
not the subject, defines the range. The fix is a percentile-based (2nd/98th
percentile) levels stretch applied to the image *before* handing it to
`rascii_art`.

`rascii_art 0.4.5`'s public API (`RenderOptions` + `render`/`render_to`/
`render_image`/`render_image_to`) has no preprocessing hook — no callback,
no pre-normalization flag, nothing that would let a caller intervene
between decode and character-mapping. `render_image`/`render_image_to` do
accept a `&image::DynamicImage` directly (rather than only a file path), so
the only way to run the stretch is to decode the image with the `image`
crate ourselves, mutate its pixel data, and hand the result to
`rascii_art::render_image_to`.

This directly narrows a property ADR-011 established and called out as a
headline design win: "`fireside-cli` never touches `image` types directly
(file paths in, `String` ASCII out)." That framing no longer holds once
this feature ships — hence this ADR, rather than a silent Cargo.toml edit.

## Decision

Add `image = "0.24"` as a direct `fireside-cli` dependency, pinned to the
same 0.24.x line `rascii_art` 0.4.5 already vendors. `Cargo.lock` already
resolves `image 0.24.9` transitively (via `rascii_art`); confirmed via
`cargo tree -p fireside-cli -i image` that adding the direct requirement
unifies onto that same resolved version rather than creating a second
`image` major-version subtree — this dependency adds no new supply-chain
surface, only a manifest line and a constitution allowlist entry.

MSRV re-verified with a real build, not just declared metadata: `cargo
+1.88 build -p fireside-cli --all-targets` succeeds with the new dependency
in place (following on from ADR-011's own spike, which already exercised
`image 0.24.6`'s decode→grayscale→charset pipeline end-to-end via
`rascii_art` under the same 1.88 toolchain — this feature's additional
surface, `GenericImageView`/`ImageBuffer` pixel iteration and construction,
is exercised by `rascii_art` itself internally at the same crate version,
so no new MSRV risk class is introduced).

Constitution Principle III's `fireside-cli` allowlist row gains `image`.
The `fireside-tui` row is unchanged — image decoding/manipulation stays
entirely inside the CLI's authoring-time conversion command, never reaching
the presenter runtime, which is the boundary this codebase actually cares
about protecting (per ADR-008/ADR-011 precedent).

## Consequences

### Positive

- Fixes the reported bug (undecipherable ASCII art from ordinary photos)
  with no new supply-chain dependency — `image` was already being compiled
  into every `fireside-cli` build via `rascii_art`, just not directly named.
- `cargo tree -i image` confirms a single unified `image` version — no
  duplicate-subtree bloat, the risk ADR-011 flagged as a "neutral/
  follow-up" item ("worth a glance in `cargo tree -i image` if it
  recurs") is checked and clear.
- Percentile/stretch logic is plain, dependency-free arithmetic over
  `image`'s existing pixel types — no additional crate needed beyond
  `image` itself.

### Negative or Trade-offs

- ADR-011's "never touches `image` types directly" property no longer
  holds. Accepted as a narrow, well-justified exception: no alternative
  exists within `rascii_art`'s actual public API (verified by reading its
  source, not assumed), and forking a third-party crate for one feature
  was rejected as heavier and harder to maintain long-term than a direct,
  already-resolved dependency.
- Any future `image`-major-version bump in `rascii_art` (or vice versa)
  now needs both to move together, or the direct dependency's version
  range widened — a coupling that didn't exist before. Low risk in
  practice: both crates are pinned to compatible `0.24.x` ranges today, and
  a future bump is a normal dependency-update PR, not an architectural one.

### Neutral / Follow-up

- Constitution Principle III allowlist amendment: `fireside-cli` row gains
  `image` (version bump 1.2.1 → 1.3.0, same amendment class as ADR-006 and
  ADR-011).
- Proceed with spec `011-art-image-quality`'s Foundational phase (T006–T008
  in `tasks.md`): percentile/stretch functions and the decode-ourselves
  refactor of `render_image_ascii`.
