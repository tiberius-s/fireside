# Data Model: ASCII Art Image Quality

No persistent storage or protocol/wire-format changes — every entity below
is an in-memory value used only inside one `fireside art image` invocation,
or an existing CLI signature that grows new parameters. Shapes decided in
`research.md`.

## `ImageConversionOptions` (new, conceptual — CLI-facing)

The full set of knobs `ArtMode::Image` now carries, threaded into
`render_image_ascii`/`art_image`:

| Field          | Type                | Source / default                                             |
| -------------- | ------------------- | -------------------------------------------------------------- |
| `path`         | `&Path`              | existing positional argument, unchanged                       |
| `width`        | `Option<u32>`        | existing `--width`, unchanged                                  |
| `charset`      | `ArtCharset`         | new `--charset`; defaults to `ArtCharset::Default`             |
| `invert`       | `bool`               | new `--invert`; defaults to `false`                             |
| `no_normalize` | `bool`               | new `--no-normalize`; defaults to `false` (stretch runs by default) |

Not a literal Rust struct requirement — may be passed as individual
function parameters if that reads more clearly in `art.rs`; listed here as
one table because they're all part of the same command's contract.

## `ArtCharset` (new)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum ArtCharset {
    Default,
    Block,
    Slight,
}
```

Maps to `rascii_art::charsets::{DEFAULT, BLOCK, SLIGHT}`. Follows the exact
pattern of the existing `Template` enum in `main.rs` (`#[derive(ValueEnum)]`
+ kebab-case rename).

## Percentile bounds (new, internal)

A `(u8, u8)` pair — `(lo, hi)` — the 2nd and 98th percentile luma values of
the source image, computed once per invocation and consumed by both the
stretch step and the low-range warning check (`research.md` §2/§3). Not a
named struct; a plain tuple return from a private `percentile_bounds`
function is sufficient for two call sites in the same function.

| Value | Meaning                                                             |
| ----- | ---------------------------------------------------------------------- |
| `lo`  | Luma value below which ~2% of the image's pixels fall                |
| `hi`  | Luma value above which ~2% of the image's pixels fall (i.e. the 98th percentile) |

Degenerate case: `hi <= lo` (a solid-fill or otherwise zero-range image) —
the stretch step becomes a no-op (image passed through unchanged); the
low-range warning still fires in this case, since a zero-width range is the
extreme end of "concentrated in less than 40% of the full range."

## Stretched image (new, internal, transient)

A new `image::DynamicImage` produced by applying the affine remap
`clamp((channel - lo) * 255 / (hi - lo), 0, 255)` to every color channel of
every pixel of the source image. Exists only long enough to be passed to
`rascii_art::render_image_to`; never written to disk, never returned to the
caller.

## Low-range warning (no new type)

A single `eprintln!`, gated on `(hi - lo) < 102` (`research.md` §5), naming
the measured percentile span as a rough percentage of the full 0–255 range
and suggesting `--invert` or a higher-contrast source image. Fires
independent of `--no-normalize` (research.md §5); stdout output is
unaffected either way.

## `render_image_ascii` (changed signature)

| Before                                                              | After                                                                                       |
| ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `fn render_image_ascii(path: &Path, width: Option<u32>) -> Result<String>` | `fn render_image_ascii(path: &Path, width: Option<u32>, charset: ArtCharset, invert: bool, no_normalize: bool) -> Result<String>` |

Internals change from delegating decode-and-render to `rascii_art::render_to`
(path-based) to: `image::open(path)` itself, compute percentile bounds,
conditionally stretch, print the low-range warning if applicable, then call
`rascii_art::render_image_to` with the (possibly stretched) `DynamicImage`
and a `RenderOptions` built from `width`/`charset`/`invert`.

## `art_image` (changed signature)

Grows the same new parameters as `render_image_ascii`, passing them
straight through; still just prints the returned `String` to stdout — no
behavior change to the print/error-handling shape itself.

## Text-banner path (explicitly unchanged)

`render_text_banner`, `art_text`, and `new.rs`'s `--banner` handling are not
touched by any entity above — confirmed out of scope per FR-008.
