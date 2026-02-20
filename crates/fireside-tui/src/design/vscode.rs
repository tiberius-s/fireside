//! VS Code terminal color scheme (`.json`) parser.
//!
//! Reads the JSON format exported by the
//! [iTerm2-Color-Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes/tree/master/vscode)
//! repository's `vscode/` directory. These files use VS Code's
//! `workbench.colorCustomizations` schema with `terminal.*` keys.
//!
//! ## File Format
//!
//! ```json
//! {
//!   "workbench.colorCustomizations": {
//!     "terminal.background":    "#21252b",
//!     "terminal.foreground":    "#abb2bf",
//!     "terminal.ansiBlack":     "#21252b",
//!     "terminal.ansiRed":       "#e06c75",
//!     "terminal.ansiGreen":     "#98c379",
//!     "terminal.ansiYellow":    "#e5c07b",
//!     "terminal.ansiBlue":      "#61afef",
//!     "terminal.ansiMagenta":   "#c678dd",
//!     "terminal.ansiCyan":      "#56b6c2",
//!     "terminal.ansiWhite":     "#abb2bf",
//!     "terminal.ansiBrightBlack":   "#767676",
//!     "terminal.selectionBackground": "#323844",
//!     "terminalCursor.foreground":  "#abb2bf",
//!     "terminalCursor.background":  "#21252b"
//!   }
//! }
//! ```
//!
//! ## Mapping to Design Tokens
//!
//! | VS Code Key                    | Design Token(s)                          |
//! |-------------------------------|------------------------------------------|
//! | `terminal.background`          | `background`                             |
//! | `terminal.foreground`          | `on_background`, `body`, `code_fg`       |
//! | `terminal.ansiCyan`            | `heading_h1`, `primary`                  |
//! | `terminal.ansiGreen`           | `heading_h2`, `success`                  |
//! | `terminal.ansiYellow`          | `heading_h3`                             |
//! | `terminal.ansiBlue`            | `accent`                                 |
//! | `terminal.ansiRed`             | `error`                                  |
//! | `terminal.ansiMagenta`         | `quote`                                  |
//! | `terminal.ansiBrightBlack`     | `border_inactive`, `muted`, `footer`     |
//! | `terminal.selectionBackground` | `surface`, `code_bg`                     |
//! | `terminalCursor.foreground`    | `border_active`                          |
//! | `terminal.ansiBlack`           | `toolbar_bg`                             |

use std::collections::HashMap;
use std::path::Path;

use ratatui::style::Color;

use super::tokens::DesignTokens;

const MAX_VSCODE_FILE_SIZE_BYTES: u64 = 512_000;

/// Errors that can occur when parsing a VS Code JSON color scheme.
#[derive(Debug, thiserror::Error)]
pub enum VscodeSchemeError {
    /// Could not read the JSON file.
    #[error("failed to read vscode theme file: {0}")]
    Io(#[from] std::io::Error),

    /// Could not parse the JSON file.
    #[error("failed to parse vscode theme json: {0}")]
    Json(#[from] serde_json::Error),

    /// The JSON structure is missing the expected `workbench.colorCustomizations` key.
    #[error("expected 'workbench.colorCustomizations' object in vscode theme")]
    MissingColorCustomizations,

    /// The file is too large to parse safely.
    #[error("vscode theme file too large: {path} ({size} bytes, max {max_size} bytes)")]
    FileTooLarge {
        /// Path to the oversize file.
        path: String,
        /// Actual file size.
        size: u64,
        /// Maximum accepted size.
        max_size: u64,
    },
}

/// Parsed color entries from a VS Code terminal color scheme JSON file.
#[derive(Debug, Clone)]
pub struct VscodeScheme {
    /// Mapping from VS Code terminal key names to RGB colors.
    pub colors: HashMap<String, Color>,
    /// Theme name derived from the file stem.
    pub name: String,
}

impl VscodeScheme {
    /// Load a VS Code color scheme from a `.json` file.
    ///
    /// The file must follow the `workbench.colorCustomizations` structure
    /// from the iTerm2-Color-Schemes vscode export format.
    ///
    /// # Errors
    ///
    /// Returns [`VscodeSchemeError`] if the file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, VscodeSchemeError> {
        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();
        if size > MAX_VSCODE_FILE_SIZE_BYTES {
            return Err(VscodeSchemeError::FileTooLarge {
                path: path.display().to_string(),
                size,
                max_size: MAX_VSCODE_FILE_SIZE_BYTES,
            });
        }

        let text = std::fs::read_to_string(path)?;
        let root: serde_json::Value = serde_json::from_str(&text)?;

        let customizations = root
            .get("workbench.colorCustomizations")
            .and_then(|v| v.as_object())
            .ok_or(VscodeSchemeError::MissingColorCustomizations)?;

        let mut colors = HashMap::new();
        for (key, val) in customizations {
            if let Some(hex) = val.as_str()
                && let Some(color) = parse_hex_color(hex)
            {
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

    /// Get a color by its VS Code terminal key name.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<Color> {
        self.colors.get(key).copied()
    }

    /// Convert this color scheme to Fireside design tokens.
    ///
    /// Falls back to the default token values for any missing colors.
    #[must_use]
    pub fn to_tokens(&self) -> DesignTokens {
        let defaults = DesignTokens::default();

        let get = |key: &str| -> Option<Color> { self.get(key) };

        let bg = get("terminal.background").unwrap_or(defaults.background);
        let fg = get("terminal.foreground").unwrap_or(defaults.on_background);

        // ANSI palette
        let black = get("terminal.ansiBlack").unwrap_or(bg);
        let red = get("terminal.ansiRed").unwrap_or(defaults.error);
        let green = get("terminal.ansiGreen").unwrap_or(defaults.success);
        let yellow = get("terminal.ansiYellow").unwrap_or(defaults.heading_h3);
        let blue = get("terminal.ansiBlue").unwrap_or(defaults.accent);
        let magenta = get("terminal.ansiMagenta").unwrap_or(defaults.quote);
        let cyan = get("terminal.ansiCyan").unwrap_or(defaults.heading_h1);
        let bright_black = get("terminal.ansiBrightBlack").unwrap_or(defaults.muted);

        // Semantic surface / cursor
        let selection = get("terminal.selectionBackground").unwrap_or(defaults.surface);
        let cursor_fg = get("terminalCursor.foreground").unwrap_or(defaults.border_active);

        DesignTokens {
            background: bg,
            surface: selection,
            primary: cyan,
            accent: blue,
            muted: bright_black,
            error: red,
            success: green,

            on_background: fg,
            on_surface: fg,
            on_primary: bg,

            heading_h1: cyan,
            heading_h2: green,
            heading_h3: yellow,
            body: fg,
            code_fg: fg,
            code_bg: selection,
            quote: magenta,

            footer: bright_black,
            border_active: cursor_fg,
            border_inactive: bright_black,
            toolbar_bg: black,
            toolbar_fg: fg,

            syntax_theme: defaults.syntax_theme,
        }
    }
}

/// Parse a hex color string (`#RRGGBB` or `#RGB`) into a Ratatui [`Color::Rgb`].
///
/// Returns `None` for malformed or non-hex values.
pub(crate) fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim().strip_prefix('#').unwrap_or(hex.trim());

    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        }
        3 => {
            // Short form: #RGB → #RRGGBB
            let expand = |c: &str| -> Option<u8> {
                let nib = u8::from_str_radix(c, 16).ok()?;
                Some(nib << 4 | nib)
            };
            let r = expand(&hex[0..1])?;
            let g = expand(&hex[1..2])?;
            let b = expand(&hex[2..3])?;
            Some(Color::Rgb(r, g, b))
        }
        8 => {
            // #RRGGBBAA — ignore alpha channel
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_6_digit() {
        assert_eq!(
            parse_hex_color("#abb2bf"),
            Some(Color::Rgb(0xab, 0xb2, 0xbf))
        );
        assert_eq!(
            parse_hex_color("#21252b"),
            Some(Color::Rgb(0x21, 0x25, 0x2b))
        );
    }

    #[test]
    fn parse_hex_without_hash() {
        assert_eq!(
            parse_hex_color("61afef"),
            Some(Color::Rgb(0x61, 0xaf, 0xef))
        );
    }

    #[test]
    fn parse_hex_3_digit() {
        // #f0f → #ff00ff
        assert_eq!(parse_hex_color("#f0f"), Some(Color::Rgb(0xff, 0x00, 0xff)));
    }

    #[test]
    fn parse_hex_8_digit_ignores_alpha() {
        assert_eq!(
            parse_hex_color("#abb2bfff"),
            Some(Color::Rgb(0xab, 0xb2, 0xbf))
        );
    }

    #[test]
    fn parse_hex_invalid_returns_none() {
        assert_eq!(parse_hex_color("#xyz"), None);
        assert_eq!(parse_hex_color(""), None);
        assert_eq!(parse_hex_color("#12"), None);
    }

    #[test]
    fn vscode_scheme_to_tokens_defaults_on_empty() {
        let scheme = VscodeScheme {
            colors: HashMap::new(),
            name: "test".to_owned(),
        };
        let tokens = scheme.to_tokens();
        let defaults = DesignTokens::default();
        assert_eq!(tokens.background, defaults.background);
        assert_eq!(tokens.accent, defaults.accent);
    }

    #[test]
    fn vscode_scheme_to_tokens_maps_colors() {
        let mut colors = HashMap::new();
        colors.insert(
            "terminal.background".to_owned(),
            Color::Rgb(0x21, 0x25, 0x2b),
        );
        colors.insert(
            "terminal.foreground".to_owned(),
            Color::Rgb(0xab, 0xb2, 0xbf),
        );
        colors.insert("terminal.ansiBlue".to_owned(), Color::Rgb(0x61, 0xaf, 0xef));
        colors.insert("terminal.ansiRed".to_owned(), Color::Rgb(0xe0, 0x6c, 0x75));

        let scheme = VscodeScheme {
            colors,
            name: "atom-one-dark".to_owned(),
        };
        let tokens = scheme.to_tokens();

        assert_eq!(tokens.background, Color::Rgb(0x21, 0x25, 0x2b));
        assert_eq!(tokens.on_background, Color::Rgb(0xab, 0xb2, 0xbf));
        assert_eq!(tokens.accent, Color::Rgb(0x61, 0xaf, 0xef));
        assert_eq!(tokens.error, Color::Rgb(0xe0, 0x6c, 0x75));
    }
}
