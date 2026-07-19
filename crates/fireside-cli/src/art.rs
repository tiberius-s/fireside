//! `fireside art`: authoring-time ASCII art generation. [`art_text`]/
//! [`art_image`] print ready-to-paste art to stdout for the standalone
//! `fireside art` verb; [`render_text_banner`]/[`render_image_ascii`] are
//! the same conversions as plain functions, reused by `new.rs`'s
//! `--banner` flag and by nothing else — neither this module nor its
//! callers edit a deck file directly (`import.rs`'s `ascii-art` fence
//! handling is separate: it promotes art *already pasted* into Markdown,
//! rather than generating it).

use std::path::Path;

use anyhow::{Context, Result};
use figlet_rs::FIGlet;
use rascii_art::RenderOptions;

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

/// Convert the image at `path` to ASCII shading via `rascii_art`.
/// Reports a clear, actionable error — never a panic — when `path`
/// doesn't exist or isn't a readable image (FR-014).
pub(crate) fn render_image_ascii(path: &Path, width: Option<u32>) -> Result<String> {
    let path_str = path
        .to_str()
        .with_context(|| format!("{} is not valid UTF-8", path.display()))?;
    let options = RenderOptions::new().width(width.unwrap_or(DEFAULT_ART_WIDTH));
    let mut out = String::new();
    rascii_art::render_to(path_str, &mut out, &options)
        .with_context(|| format!("could not read {} as an image", path.display()))?;
    Ok(out)
}

/// Prints [`render_image_ascii`]'s output to stdout — the standalone
/// `fireside art image` verb.
pub(crate) fn art_image(path: &Path, width: Option<u32>) -> Result<()> {
    println!("{}", render_image_ascii(path, width)?);
    Ok(())
}
