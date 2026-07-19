# Research: ASCII Art Image Quality

This feature has one real unknown (whether a direct `image` crate dependency
is safe to add and sufficient to implement the stretch) and several small
design decisions made while grounding the spec against the current
`fireside-cli` code and the `rascii_art` crate's actual API. The unknown was
resolved with a real, throwaway build/run spike (ADR-011's methodology),
not metadata inspection alone.

## 1. Can the contrast stretch be done without a direct `image` dependency?

**Decision**: No — a direct `image = "0.24"` dependency is required. Verified
by reading `rascii_art 0.4.5`'s vendored source
(`~/.cargo/registry/src/.../rascii_art-0.4.5/src/lib.rs`,
`image_renderer.rs`): its only entry points are `render`/`render_to`
(`Path`-based, decode happens inside the crate) and
`render_image`/`render_image_to` (take `&image::DynamicImage` directly).
`RenderOptions` exposes `width`, `height`, `colored`, `invert`, `charset` —
no preprocessing hook, no brightness-adjustment parameter. The only place to
intervene between decode and character-mapping is to decode the image
ourselves (requiring `image::DynamicImage`/pixel types directly), stretch
it, and pass the result to `render_image_to`.

Also discovered while reading the source: `rascii_art`'s own char-mapping
(`image_renderer.rs::get_char_for_pixel`) already divides each pixel's
grayscale value by the single brightest grayscale value found anywhere in
the (thumbnailed) image — i.e. it already does a same-image max-normalize,
but only at the bright end, and using the single darkest/brightest pixel
rather than a percentile. This explains why the reported bug persists even
though `rascii_art` does *some* normalization: a photo with a few
near-white outlier pixels (a highlight, a blown sky corner) sets `maximum`
near 255 even though the meaningful subject content sits in a much narrower
dark band — the existing per-image max-normalize doesn't help because the
outlier, not the subject, defines the ceiling. A percentile-based stretch
(ignoring the top/bottom 2% of pixels) fixes exactly this failure mode.

**Real spike performed** (throwaway `cargo new` scratch project, not
committed): added `image = "0.24"` and `rascii_art = "0.4"`, implemented a
percentile-bounds function and a per-channel linear stretch over a
synthetic low-contrast `RgbaImage`, and a solid-fill (zero-range) image.
`cargo +1.88 build` and `cargo +1.88 run` both succeeded: the stretch
correctly widened the synthetic gradient's output character spread, and the
solid-fill case computed `lo == hi` and returned an unmodified image without
panicking — confirming both the API surface and the edge-case guard work
under the workspace's real MSRV toolchain, not just against declared
`rust-version` metadata.

**Alternatives considered**:

- *Fork/patch `rascii_art` to add a preprocessing hook.* Rejected — forking
  a third-party crate for one feature is heavier and harder to maintain
  than depending on a crate already fully resolved (transitively) at the
  exact version needed; `image` adds no new supply-chain surface, only a
  direct manifest line.
- *Hand-roll image decoding to avoid the `image` crate entirely.*
  Rejected — reimplements a well-tested, already-present dependency for no
  benefit; `rascii_art` itself depends on it, so it's already being
  compiled into every `fireside-cli` build today.

## 2. Percentile/stretch algorithm

**Decision**: Compute a 256-bucket histogram of per-pixel luma (the same
`0.299R + 0.587G + 0.114B` weighting `rascii_art` itself uses, for
consistency between "what we measured" and "what rascii will grayscale
next"), find the luma values at the 2nd and 98th percentile by cumulative
pixel count (`lo`, `hi`), then apply a single linear affine remap
`clamp((channel - lo) * 255 / (hi - lo), 0, 255)` to **all three color
channels** of every pixel (not just luma) — a standard "auto-levels" stretch
that widens tonal range while preserving relative color balance, since
`rascii_art`'s `colored` option exists in `RenderOptions` even though
`fireside-cli` doesn't currently surface it. If `hi <= lo` (a solid-fill or
otherwise zero-range image), skip the stretch and pass the image through
unchanged — verified in the spike to require no special-case error, just an
early return.

**Rationale**: Doing the remap on all channels (not converting to grayscale
first) means a future `--color` flag would still see correct color, not a
desaturated image; it costs nothing extra since the transform is identical
per channel. Percentile (not absolute min/max) directly satisfies FR-002's
requirement that a few outlier pixels not defeat the adjustment — this is
the crux of the actual reported bug (§1 above).

**Alternatives considered**:

- *Stretch luma only, then reconstruct color from original chrominance.*
  Rejected as needless complexity (chrominance math, e.g. HSL round-trip)
  for a benefit (perfect color preservation under extreme stretches) the
  current feature scope doesn't need — `fireside-cli` doesn't even expose
  colored output today.
- *Use absolute min/max instead of percentiles.* Rejected — this is
  literally what `rascii_art`'s own existing per-image max-normalize
  already approximates and already fails to fix the reported bug (§1).

## 3. Where the percentile/stretch/warning logic lives

**Decision**: New private functions in `crates/fireside-cli/src/art.rs`
(same module as today's `render_image_ascii`): `luma(&Rgba<u8>) -> u8`,
`percentile_bounds(&DynamicImage, f64, f64) -> (u8, u8)`,
`stretch(&DynamicImage, u8, u8) -> DynamicImage`. `render_image_ascii` calls
`image::open` itself (rather than delegating that to `rascii_art::render_to`
as it does today), computes bounds once, applies the stretch (unless
disabled), checks the low-range condition against the *same* bounds for the
warning, and calls `rascii_art::render_image_to` with the (possibly
stretched) image.

**Rationale**: Reuses the module `render_image_ascii` already occupies and
its existing role as "the plain-function conversion `new.rs`'s `--banner`
and the standalone `art image` verb both call" (per its current doc
comment) — except this feature's stretch/warning behavior is specific to
the standalone `art image` conversion path, not the banner path (FR-008), so
`render_image_ascii`'s signature grows to take the new flags/options as
parameters with sensible defaults, keeping any future caller in control.
Computing `percentile_bounds` exactly once and reusing it for both the
stretch and the warning avoids scanning the image's pixels twice.

## 4. Charset and invert flags

**Decision**: Add a `#[derive(ValueEnum)] enum ArtCharset { Default, Block,
Slight }` (kebab-case rename, matching the existing `Template` enum's
pattern in `main.rs`) mapped to `rascii_art::charsets::{DEFAULT, BLOCK,
SLIGHT}` via the crate's own `charsets::from_str` (or a local `match`, since
the enum variants are already known statically — `match` avoids a
stringly-typed round-trip). `--invert` is a plain `bool` flag passed
straight to `RenderOptions::invert(bool)`. Default charset (no flag) stays
`Default` — `rascii_art::charsets::DEFAULT`, identical to today's unflagged
behavior.

**Rationale**: `rascii_art` already ships exactly these three (plus
`chinese`/`emoji`/`russian`, out of scope per the feature description's
"at least three... default/block/slight") as `&'static [&'static str]`
constants — no new charset data needs to be authored, only a flag that
selects among what already exists.

## 5. Low-range warning threshold

**Decision**: Reuse the `(lo, hi)` percentile bounds already computed for
the stretch (§2/§3); warn on stderr when `(hi - lo) < 102` (`0.40 * 255`,
rounded), regardless of whether `--no-normalize` was passed — the warning
describes the *source image's* inherent quality ("this image uses N% of its
brightness range"), which is true independent of whether this run chose to
compensate for it.

**Rationale**: The spec's FR-006 measures the condition "before any
adjustment," so computing it pre-stretch (which the implementation already
does, since the stretch bounds are exactly that measurement) and firing it
unconditionally is both the simplest reading of the requirement and avoids
computing the same histogram twice under different flag combinations.

## 6. `--no-normalize` interaction with charset/invert

**Decision**: `--no-normalize` only disables the stretch step (§2); charset
and invert selection are independent and still apply. Verified against the
spec's edge case ("A charset or invert choice is combined with the opt-out
flag ... the two are independent controls and must compose").

**Rationale**: The three flags control three orthogonal concerns (whether to
stretch, which characters to use, which direction to shade) — nothing in
their implementation couples them, so no extra logic is needed to keep them
independent; this is a design constraint to preserve while wiring the CLI
args, not a new mechanism to build.

## 7. Demo image replacement and doc updates

**Decision**: Replace `.github/demo-art.png` with a CC0, high-contrast,
simple-silhouette photo (a single well-lit subject against a plain
background — the opposite of the current busy, low-detail night photo),
sourced the same way the current photo was (Wikimedia Commons CC0 search),
credited in `reference/cli.md` following the exact existing attribution
format ("`<title>` by `<author>`, [CC0 1.0](<url>), Wikimedia Commons").
Re-record `art-image.gif` via the existing `scripts/demos.sh` (no script
changes needed — it already regenerates this tape). Update
`reference/cli.md`'s flag table (add `--charset`, `--invert`,
`--no-normalize` rows) and check `guides/authoring-markdown.md` for any
stale flag/behavior references to the image conversion path.

**Rationale**: Matches Wave 2 (Stream D)'s already-established pattern of
sourcing CC0 images and keeping the input-photo-next-to-output presentation
in `reference/cli.md`; no new documentation mechanism needed, only content
updates to existing pages.

## 8. Constitution amendment

**Decision**: Principle III's `fireside-cli` allowlist row gains `image`,
recorded via a new ADR (`adr-013-*`) that supersedes ADR-011's "never
touches `image` types directly" framing for this one, narrow reason (no
preprocessing hook exists in `rascii_art`'s public API — see §1), and notes
the version-pin rationale (§ Technical Context in `plan.md`: `image = "0.24"`
unifies with the already-resolved `0.24.9` rather than creating a second
`image` subtree).

**Rationale**: Same amendment pattern as ADR-006 (`pulldown-cmark`) and
ADR-011 itself (`figlet-rs`, `rascii_art`) — a small, well-justified
allowlist addition recorded as an ADR plus a constitution diff, not a
larger architectural change.
