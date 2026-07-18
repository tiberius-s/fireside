//! `fireside art`: authoring-time ASCII art generation. Both verbs print
//! ready-to-paste art to stdout; neither edits a deck file
//! (`specs/009-ascii-art/contracts/cli-art-command.md`).

use std::path::Path;

use anyhow::{Context, Result};
use figlet_rs::FIGlet;
use rascii_art::RenderOptions;

/// Turn `phrase` into a stylized text banner via the built-in FIGlet
/// standard font and print it to stdout. Characters the font has no
/// letterform for are skipped, not fatal — this only fails when *no*
/// character in `phrase` is recognized (FR-013).
pub(crate) fn art_text(phrase: &str) -> Result<()> {
    let font = FIGlet::standard()
        .map_err(anyhow::Error::msg)
        .context("could not load the built-in banner font")?;
    let figure = font
        .convert(phrase)
        .with_context(|| format!("no recognized characters in \"{phrase}\" — nothing to render"))?;
    println!("{figure}");
    Ok(())
}

/// The width used when `--width` is omitted — matches the 76-column
/// threshold `ascii-art-too-wide` validates against
/// (`crates/fireside-engine/src/validation.rs`), so the default output
/// already fits the presentation card.
const DEFAULT_ART_WIDTH: u32 = 76;

/// Convert the image at `path` to ASCII shading via `rascii_art` and
/// print it to stdout. Reports a clear, actionable error — never a
/// panic — when `path` doesn't exist or isn't a readable image (FR-014).
pub(crate) fn art_image(path: &Path, width: Option<u32>) -> Result<()> {
    let path_str = path
        .to_str()
        .with_context(|| format!("{} is not valid UTF-8", path.display()))?;
    let options = RenderOptions::new().width(width.unwrap_or(DEFAULT_ART_WIDTH));
    let mut out = String::new();
    rascii_art::render_to(path_str, &mut out, &options)
        .with_context(|| format!("could not read {} as an image", path.display()))?;
    println!("{out}");
    Ok(())
}
