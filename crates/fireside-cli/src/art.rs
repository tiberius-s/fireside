//! `fireside art`: authoring-time ASCII art generation. [`art_text`]/
//! [`art_image`] print ready-to-paste art to stdout for the standalone
//! `fireside art` verb; [`render_text_banner`]/[`render_image_ascii`] are
//! the same conversions as plain functions, reused by `new.rs`'s
//! `--banner` flag and by nothing else — neither this module nor its
//! callers edit a deck file directly (`import.rs`'s `ascii-art` fence
//! handling is separate: it promotes art *already pasted* into Markdown,
//! rather than generating it).
//!
//! Image conversion (spec 011) applies a percentile-based contrast stretch
//! by default before shading — `rascii_art`'s own grayscale normalization
//! only accounts for the single brightest pixel in the image, which a few
//! outlier pixels (a highlight, a blown corner) can defeat even though the
//! subject itself sits in a much narrower band. `--no-normalize` restores
//! the pre-011 behavior exactly; `--charset`/`--invert` surface `rascii_art`
//! capability this CLI didn't previously expose.

use std::path::Path;

use anyhow::{Context, Result};
use figlet_rs::FIGlet;
use image::{DynamicImage, GenericImageView, Rgba};
use rascii_art::RenderOptions;

use crate::ArtCharset;

/// Turn `phrase` into a stylized text banner via the built-in FIGlet
/// standard font. Characters the font has no letterform for are
/// skipped, not fatal — this only fails when *no* character in `phrase`
/// is recognized (FR-013).
pub(crate) fn render_text_banner(phrase: &str) -> Result<String> {
    let font = FIGlet::standard()
        .map_err(anyhow::Error::msg)
        .context("could not load the built-in banner font")?;
    let figure = font
        .convert(phrase)
        .with_context(|| format!("no recognized characters in \"{phrase}\" — nothing to render"))?;
    Ok(figure.to_string())
}

/// Prints [`render_text_banner`]'s output to stdout — the standalone
/// `fireside art text` verb. When the banner's widest line exceeds
/// [`DEFAULT_ART_WIDTH`] (the same threshold `ascii-art-too-wide`
/// validates against), a note naming the measured width goes to stderr —
/// stdout still gets the full, unmodified banner either way.
pub(crate) fn art_text(phrase: &str) -> Result<()> {
    let art = render_text_banner(phrase)?;
    println!("{art}");
    let widest = art.lines().map(str::len).max().unwrap_or(0);
    if widest > DEFAULT_ART_WIDTH as usize {
        eprintln!(
            "Note: this banner is {widest} columns wide — decks are validated against a {DEFAULT_ART_WIDTH}-column limit (ascii-art-too-wide)."
        );
    }
    Ok(())
}

/// The width used when `--width` is omitted — matches the 76-column
/// threshold `ascii-art-too-wide` validates against
/// (`crates/fireside-engine/src/validation.rs`), so the default output
/// already fits the presentation card.
pub(crate) const DEFAULT_ART_WIDTH: u32 = 76;

/// The pre-stretch percentile span (`hi - lo`, out of 255) below which
/// [`render_image_ascii`] warns that the source image's brightness range
/// is unusually narrow — roughly 40% of the full range (FR-006).
const LOW_RANGE_THRESHOLD: u8 = 102;

/// Perceptual grayscale weighting, matching the formula `rascii_art`
/// itself uses (`ImageRenderer::get_grayscale`) so the percentile bounds
/// computed here describe the same brightness `rascii_art` will shade by.
fn luma(pixel: &Rgba<u8>) -> u8 {
    (0.299 * f64::from(pixel.0[0]) + 0.587 * f64::from(pixel.0[1]) + 0.114 * f64::from(pixel.0[2]))
        as u8
}

/// The luma values at `lo_pct`/`hi_pct` percentile across `img`'s pixels
/// (e.g. `(0.02, 0.98)` for a 2nd/98th percentile clip), found via a
/// 256-bucket cumulative histogram. A few outlier pixels at either extreme
/// don't move these bounds much, unlike a plain min/max (FR-002).
fn percentile_bounds(img: &DynamicImage, lo_pct: f64, hi_pct: f64) -> (u8, u8) {
    let mut hist = [0u32; 256];
    let mut total = 0u32;
    for (_, _, pixel) in img.pixels() {
        hist[luma(&pixel) as usize] += 1;
        total += 1;
    }
    let lo_target = (f64::from(total) * lo_pct).ceil().max(1.0) as u32;
    let hi_target = (f64::from(total) * hi_pct).ceil().max(1.0) as u32;

    let mut cumulative = 0u32;
    let mut lo = 0u8;
    for (value, count) in hist.iter().enumerate() {
        cumulative += count;
        if cumulative >= lo_target {
            lo = value as u8;
            break;
        }
    }
    cumulative = 0;
    let mut hi = 255u8;
    for (value, count) in hist.iter().enumerate() {
        cumulative += count;
        if cumulative >= hi_target {
            hi = value as u8;
            break;
        }
    }
    (lo, hi)
}

/// `true` when `lo`/`hi` (as returned by [`percentile_bounds`]) span less
/// than [`LOW_RANGE_THRESHOLD`] of the full 0–255 range — the condition
/// [`render_image_ascii`] warns about (FR-006), regardless of whether the
/// stretch itself was applied.
fn is_low_range(lo: u8, hi: u8) -> bool {
    hi.saturating_sub(lo) < LOW_RANGE_THRESHOLD
}

/// Applies a linear "levels" stretch (`lo` maps to `0`, `hi` maps to
/// `255`, clamped) to every color channel of every pixel in `img`.
/// Returns `img` unchanged when `hi <= lo` (a solid-fill or otherwise
/// zero-range image) rather than dividing by zero or distorting output.
fn stretch(img: &DynamicImage, lo: u8, hi: u8) -> DynamicImage {
    if hi <= lo {
        return img.clone();
    }
    let (lo, hi) = (f64::from(lo), f64::from(hi));
    let mut out = img.to_rgba8();
    for pixel in out.pixels_mut() {
        for channel in &mut pixel.0[..3] {
            let stretched = (f64::from(*channel) - lo) * 255.0 / (hi - lo);
            *channel = stretched.clamp(0.0, 255.0) as u8;
        }
    }
    DynamicImage::ImageRgba8(out)
}

/// Maps a CLI-facing [`ArtCharset`] choice to `rascii_art`'s corresponding
/// built-in charset slice.
fn charset_for(charset: ArtCharset) -> &'static [&'static str] {
    match charset {
        ArtCharset::Default => rascii_art::charsets::DEFAULT,
        ArtCharset::Block => rascii_art::charsets::BLOCK,
        ArtCharset::Slight => rascii_art::charsets::SLIGHT,
    }
}

/// Convert the image at `path` to ASCII shading via `rascii_art`. Reports
/// a clear, actionable error — never a panic — when `path` doesn't exist
/// or isn't a readable image (FR-014).
///
/// Unless `no_normalize` is set, a percentile-based contrast stretch
/// (2nd/98th percentile) runs on the decoded image before shading, so an
/// ordinary low-contrast photo doesn't collapse into a handful of the
/// darkest characters (spec 011, FR-001/FR-002). Either way, a note is
/// printed to stderr when the image's pre-stretch brightness range is
/// unusually narrow (FR-006/FR-007) — this warning describes the source
/// image and fires independent of `no_normalize`.
pub(crate) fn render_image_ascii(
    path: &Path,
    width: Option<u32>,
    charset: ArtCharset,
    invert: bool,
    no_normalize: bool,
) -> Result<String> {
    let image = image::open(path)
        .with_context(|| format!("could not read {} as an image", path.display()))?;

    let (lo, hi) = percentile_bounds(&image, 0.02, 0.98);
    if is_low_range(lo, hi) {
        let pct = (f64::from(hi.saturating_sub(lo)) / 255.0 * 100.0).round();
        eprintln!(
            "Note: this image uses about {pct}% of its brightness range — output may be hard to read; try --invert or a higher-contrast image."
        );
    }

    let image = if no_normalize {
        image
    } else {
        stretch(&image, lo, hi)
    };

    let options = RenderOptions::new()
        .width(width.unwrap_or(DEFAULT_ART_WIDTH))
        .charset(charset_for(charset))
        .invert(invert);
    let mut out = String::new();
    rascii_art::render_image_to(&image, &mut out, &options)
        .with_context(|| format!("could not read {} as an image", path.display()))?;
    Ok(out)
}

/// Prints [`render_image_ascii`]'s output to stdout — the standalone
/// `fireside art image` verb.
pub(crate) fn art_image(
    path: &Path,
    width: Option<u32>,
    charset: ArtCharset,
    invert: bool,
    no_normalize: bool,
) -> Result<()> {
    println!(
        "{}",
        render_image_ascii(path, width, charset, invert, no_normalize)?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use image::{ImageBuffer, Rgba, RgbaImage};

    use super::*;

    /// A flat `width`×`height` image at `luma_value` — every pixel
    /// identical, for building larger synthetic buffers than a handful of
    /// hand-listed pixels.
    fn solid(width: u32, height: u32, luma_value: u8) -> RgbaImage {
        ImageBuffer::from_pixel(
            width,
            height,
            Rgba([luma_value, luma_value, luma_value, 255]),
        )
    }

    #[test]
    fn percentile_bounds_ignores_a_few_outlier_pixels() {
        // 10x10 = 100 pixels: 98 in a tight band around 120, plus one
        // near-black and one near-white outlier. A plain min/max would
        // report (0, 255); the 2nd/98th percentile should stay close to
        // the tight band instead.
        let mut img = solid(10, 10, 120);
        img.put_pixel(0, 0, Rgba([0, 0, 0, 255]));
        img.put_pixel(9, 9, Rgba([255, 255, 255, 255]));
        let dyn_img = DynamicImage::ImageRgba8(img);

        let (lo, hi) = percentile_bounds(&dyn_img, 0.02, 0.98);
        assert!(
            lo >= 100 && hi <= 140,
            "expected bounds near the 120 band, got ({lo}, {hi})"
        );
    }

    #[test]
    fn percentile_bounds_on_a_solid_image_has_zero_span() {
        let img = DynamicImage::ImageRgba8(solid(4, 4, 50));
        let (lo, hi) = percentile_bounds(&img, 0.02, 0.98);
        assert_eq!((lo, hi), (50, 50));
    }

    #[test]
    fn stretch_maps_lo_to_black_and_hi_to_white() {
        let mut img = solid(2, 1, 0);
        img.put_pixel(0, 0, Rgba([100, 100, 100, 255]));
        img.put_pixel(1, 0, Rgba([200, 200, 200, 255]));
        let stretched = stretch(&DynamicImage::ImageRgba8(img), 100, 200).to_rgba8();

        assert_eq!(stretched.get_pixel(0, 0).0, [0, 0, 0, 255]);
        assert_eq!(stretched.get_pixel(1, 0).0, [255, 255, 255, 255]);
    }

    #[test]
    fn stretch_clamps_values_outside_the_bounds() {
        let img = solid(1, 1, 250);
        // lo/hi narrower than the pixel's actual value: still clamps to
        // 255, never wraps or panics.
        let stretched = stretch(&DynamicImage::ImageRgba8(img), 0, 100).to_rgba8();
        assert_eq!(stretched.get_pixel(0, 0).0, [255, 255, 255, 255]);
    }

    #[test]
    fn stretch_is_a_no_op_on_a_zero_range_image() {
        let img = DynamicImage::ImageRgba8(solid(3, 3, 50));
        let stretched = stretch(&img, 50, 50);
        assert_eq!(stretched.to_rgba8(), img.to_rgba8());
    }

    #[test]
    fn stretch_widens_spread_on_a_low_contrast_buffer() {
        let mut img = solid(4, 4, 110);
        img.put_pixel(3, 3, Rgba([140, 140, 140, 255]));
        let dyn_img = DynamicImage::ImageRgba8(img);
        let (lo, hi) = percentile_bounds(&dyn_img, 0.02, 0.98);
        let stretched = stretch(&dyn_img, lo, hi).to_rgba8();

        let (orig_min, orig_max) = (110u8, 140u8);
        let stretched_min = stretched.pixels().map(|p| p.0[0]).min().unwrap();
        let stretched_max = stretched.pixels().map(|p| p.0[0]).max().unwrap();
        assert!(
            (stretched_max - stretched_min) > (orig_max - orig_min),
            "expected a wider spread after stretching"
        );
    }

    #[test]
    fn stretch_is_negligible_on_an_already_full_range_buffer() {
        let mut img = solid(4, 4, 0);
        img.put_pixel(3, 3, Rgba([255, 255, 255, 255]));
        let dyn_img = DynamicImage::ImageRgba8(img);
        let (lo, hi) = percentile_bounds(&dyn_img, 0.02, 0.98);
        let stretched = stretch(&dyn_img, lo, hi);
        // Already spans (close to) the full range, so lo/hi should be
        // (close to) the extremes already — the stretch changes little.
        assert!(
            hi - lo > 200,
            "expected a wide pre-stretch span, got {lo}..{hi}"
        );
        let _ = stretched;
    }

    #[test]
    fn is_low_range_true_below_threshold_false_above() {
        assert!(is_low_range(100, 150)); // span 50, well under 102
        assert!(!is_low_range(20, 230)); // span 210, well over 102
        assert!(!is_low_range(0, 102)); // span exactly at threshold: not "<"
        assert!(is_low_range(0, 101)); // span just under threshold
    }

    #[test]
    fn charset_for_maps_each_variant_to_a_distinct_rascii_charset() {
        assert_eq!(
            charset_for(ArtCharset::Default),
            rascii_art::charsets::DEFAULT
        );
        assert_eq!(charset_for(ArtCharset::Block), rascii_art::charsets::BLOCK);
        assert_eq!(
            charset_for(ArtCharset::Slight),
            rascii_art::charsets::SLIGHT
        );
    }
}
