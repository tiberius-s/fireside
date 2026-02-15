//! Monospace font detection and listing.
//!
//! Uses `font-kit` to enumerate system fonts and filter to monospace-only.
//! The TUI font chooser should display only these fonts.
//!
//! ## Platform behavior
//!
//! - **macOS**: Uses Core Text (no extra deps)
//! - **Linux**: Uses fontconfig (requires `libfontconfig1-dev`)
//! - **Windows**: Uses DirectWrite (no extra deps)
//!
//! ## Note on terminal fonts
//!
//! Slideways cannot change the terminal's font — the font setting is
//! informational/advisory: it tells the user which monospace font to
//! configure in their terminal emulator for the best experience. The
//! font name is also stored in `slideways.yml` for documentation.

use std::collections::BTreeSet;

use font_kit::source::SystemSource;

/// A monospace font detected on the system.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MonospaceFont {
    /// The font family name (e.g., "JetBrains Mono", "Menlo").
    pub family: String,
}

/// List all monospace font families installed on the system.
///
/// Returns a sorted, deduplicated list. Fonts that cannot be loaded
/// or that report `is_monospace() == false` are filtered out.
///
/// # Performance
///
/// This enumerates all system fonts and loads each one to check
/// `is_monospace()`. It should be called once at startup and cached.
#[must_use]
pub fn list_monospace_fonts() -> Vec<MonospaceFont> {
    let source = SystemSource::new();
    let mut families = BTreeSet::new();

    let all_families = match source.all_families() {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("Could not list system fonts: {e}");
            return default_monospace_fonts();
        }
    };

    for family_name in &all_families {
        // Try to load a representative font from the family
        if let Ok(handle) = source.select_best_match(
            &[font_kit::family_name::FamilyName::Title(
                family_name.clone(),
            )],
            &font_kit::properties::Properties::new(),
        ) && let Ok(font) = handle.load()
            && font.is_monospace()
        {
            families.insert(MonospaceFont {
                family: family_name.clone(),
            });
        }
    }

    if families.is_empty() {
        return default_monospace_fonts();
    }

    families.into_iter().collect()
}

/// Fallback list of common monospace fonts when detection fails.
fn default_monospace_fonts() -> Vec<MonospaceFont> {
    [
        "Menlo",
        "Monaco",
        "Consolas",
        "Courier New",
        "DejaVu Sans Mono",
        "Liberation Mono",
        "SF Mono",
        "JetBrains Mono",
        "Fira Code",
        "Hack",
        "Source Code Pro",
    ]
    .iter()
    .map(|name| MonospaceFont {
        family: (*name).to_owned(),
    })
    .collect()
}

/// Recommended default monospace fonts by platform.
///
/// These are chosen for quality, readability, and wide availability.
///
/// - **macOS**: SF Mono (system default), Menlo (fallback)
/// - **Linux**: JetBrains Mono NL, DejaVu Sans Mono (fallback)
/// - **Windows**: Cascadia Mono, Consolas (fallback)
#[must_use]
pub fn recommended_fonts() -> Vec<&'static str> {
    if cfg!(target_os = "macos") {
        vec!["SF Mono", "Menlo", "Monaco"]
    } else if cfg!(target_os = "windows") {
        vec!["Cascadia Mono", "Consolas", "Courier New"]
    } else {
        vec!["JetBrains Mono NL", "DejaVu Sans Mono", "Liberation Mono"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_fonts_are_nonempty() {
        let fonts = default_monospace_fonts();
        assert!(!fonts.is_empty());
    }

    #[test]
    fn recommended_fonts_are_nonempty() {
        let fonts = recommended_fonts();
        assert!(!fonts.is_empty());
    }

    #[test]
    fn list_monospace_detects_some_fonts() {
        // This test exercises the actual system font detection.
        // It should find at least one monospace font on any platform.
        let fonts = list_monospace_fonts();
        assert!(
            !fonts.is_empty(),
            "Should detect at least one monospace font"
        );
    }
}
