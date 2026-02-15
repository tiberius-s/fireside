//! iTerm2 color scheme (.itermcolors) parser.
//!
//! Reads XML plist files from <https://iterm2colorschemes.com/> and maps
//! them to Slideways design tokens.
//!
//! ## iTerm2 Plist Structure
//!
//! Each `.itermcolors` file is an XML plist `<dict>` containing keys like:
//!
//! - `"Ansi 0 Color"` through `"Ansi 15 Color"` — the 16 ANSI colors
//! - `"Background Color"`, `"Foreground Color"`
//! - `"Bold Color"`, `"Cursor Color"`, `"Cursor Text Color"`
//! - `"Selection Color"`, `"Selected Text Color"`
//!
//! Each color entry is a `<dict>` with:
//! - `"Red Component"` (f64, 0.0–1.0)
//! - `"Green Component"` (f64, 0.0–1.0)
//! - `"Blue Component"` (f64, 0.0–1.0)
//! - `"Color Space"` (usually `"sRGB"` or `"P3"`)
//!
//! ## Mapping to Design Tokens
//!
//! | iTerm2 Key            | Design Token              |
//! |-----------------------|---------------------------|
//! | Background Color      | `background`              |
//! | Foreground Color      | `on_background`, `body`   |
//! | Bold Color            | `heading_h1`, `primary`   |
//! | Ansi 1 (Red)          | `error`                   |
//! | Ansi 2 (Green)        | `success`, `heading_h2`   |
//! | Ansi 3 (Yellow)       | `heading_h3`              |
//! | Ansi 4 (Blue)         | `accent`                  |
//! | Ansi 5 (Magenta)      | `accent` (fallback)       |
//! | Ansi 8 (Bright Black) | `muted`, `border_inactive`|
//! | Selection Color       | `surface`, `code_bg`      |
//! | Cursor Color          | `border_active`           |

use std::collections::HashMap;
use std::path::Path;

use ratatui::style::Color;

use super::tokens::DesignTokens;

/// Errors that can occur when parsing an iTerm2 color scheme.
#[derive(Debug, thiserror::Error)]
pub enum Iterm2Error {
    /// Could not read the plist file.
    #[error("failed to read itermcolors file: {0}")]
    Io(#[from] std::io::Error),

    /// Could not parse the plist file.
    #[error("failed to parse plist: {0}")]
    Plist(#[from] plist::Error),

    /// The plist structure is not the expected dictionary.
    #[error("expected a dictionary at the root of the plist")]
    NotADict,
}

/// Parsed color entries from an iTerm2 color scheme.
#[derive(Debug, Clone)]
pub struct Iterm2Scheme {
    /// Mapping from iTerm2 key names to RGB colors.
    pub colors: HashMap<String, Color>,
    /// Original file name (for display).
    pub name: String,
}

impl Iterm2Scheme {
    /// Load an iTerm2 color scheme from a `.itermcolors` file.
    ///
    /// # Errors
    ///
    /// Returns `Iterm2Error` if the file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, Iterm2Error> {
        let value = plist::Value::from_file(path)?;
        let dict = value.as_dictionary().ok_or(Iterm2Error::NotADict)?;

        let mut colors = HashMap::new();
        for (key, val) in dict {
            if let Some(color) = parse_iterm2_color(val) {
                colors.insert(key.clone(), color);
            }
        }

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_owned();

        Ok(Self { colors, name })
    }

    /// Get a color by its iTerm2 key name.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<Color> {
        self.colors.get(key).copied()
    }

    /// Convert this color scheme to design tokens.
    ///
    /// Falls back to the default token values for any missing colors.
    #[must_use]
    pub fn to_tokens(&self) -> DesignTokens {
        let defaults = DesignTokens::default();

        let bg = self.get("Background Color").unwrap_or(defaults.background);
        let fg = self
            .get("Foreground Color")
            .unwrap_or(defaults.on_background);
        let bold = self.get("Bold Color").unwrap_or(defaults.primary);
        let selection = self.get("Selection Color").unwrap_or(defaults.surface);
        let cursor = self.get("Cursor Color").unwrap_or(defaults.border_active);

        // ANSI colors
        let ansi = |n: u8| -> Option<Color> { self.get(&format!("Ansi {n} Color")) };

        let red = ansi(1).unwrap_or(defaults.error);
        let green = ansi(2).unwrap_or(defaults.success);
        let yellow = ansi(3).unwrap_or(defaults.heading_h3);
        let blue = ansi(4).unwrap_or(defaults.accent);
        let _magenta = ansi(5).unwrap_or(defaults.accent);
        let bright_black = ansi(8).unwrap_or(defaults.muted);

        DesignTokens {
            background: bg,
            surface: selection,
            primary: bold,
            accent: blue,
            muted: bright_black,
            error: red,
            success: green,

            on_background: fg,
            on_surface: fg,
            on_primary: bg,

            heading_h1: bold,
            heading_h2: green,
            heading_h3: yellow,
            body: fg,
            code_fg: fg,
            code_bg: selection,
            quote: bright_black,

            footer: bright_black,
            border_active: cursor,
            border_inactive: bright_black,
            toolbar_bg: bg,
            toolbar_fg: fg,

            syntax_theme: defaults.syntax_theme,
        }
    }
}

/// Parse a single iTerm2 color dictionary into an RGB `Color`.
///
/// Expects a plist `<dict>` with `"Red Component"`, `"Green Component"`,
/// and `"Blue Component"` keys containing float values 0.0–1.0.
fn parse_iterm2_color(value: &plist::Value) -> Option<Color> {
    let dict = value.as_dictionary()?;

    let r = dict.get("Red Component").and_then(plist::Value::as_real)?;
    let g = dict
        .get("Green Component")
        .and_then(plist::Value::as_real)?;
    let b = dict.get("Blue Component").and_then(plist::Value::as_real)?;

    let to_u8 = |v: f64| -> u8 { (v.clamp(0.0, 1.0) * 255.0).round() as u8 };

    Some(Color::Rgb(to_u8(r), to_u8(g), to_u8(b)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_iterm2_color_dict() {
        let mut dict = plist::Dictionary::new();
        dict.insert("Red Component".into(), plist::Value::Real(1.0));
        dict.insert("Green Component".into(), plist::Value::Real(0.5));
        dict.insert("Blue Component".into(), plist::Value::Real(0.0));
        dict.insert("Color Space".into(), plist::Value::String("sRGB".into()));

        let color = parse_iterm2_color(&plist::Value::Dictionary(dict));
        assert_eq!(color, Some(Color::Rgb(255, 128, 0)));
    }

    #[test]
    fn parse_iterm2_color_missing_component() {
        let mut dict = plist::Dictionary::new();
        dict.insert("Red Component".into(), plist::Value::Real(1.0));
        // Missing Green and Blue

        let color = parse_iterm2_color(&plist::Value::Dictionary(dict));
        assert_eq!(color, None);
    }

    #[test]
    fn parse_iterm2_color_not_a_dict() {
        let value = plist::Value::String("not a color".into());
        assert_eq!(parse_iterm2_color(&value), None);
    }
}
