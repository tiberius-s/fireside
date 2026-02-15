//! Theme definitions controlling the visual appearance of slides.

use ratatui::style::Color;
use serde::Deserialize;

/// A complete theme definition for rendering slides.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Background color for the slide area.
    pub background: Color,
    /// Default foreground (text) color.
    pub foreground: Color,
    /// Color for H1 headings.
    pub heading_h1: Color,
    /// Color for H2 headings.
    pub heading_h2: Color,
    /// Color for H3+ headings.
    pub heading_h3: Color,
    /// Background color for code blocks.
    pub code_background: Color,
    /// Foreground color for code block text (base, before syntax highlighting).
    pub code_foreground: Color,
    /// Color for the border around code blocks.
    pub code_border: Color,
    /// Color for block quote borders and text.
    pub block_quote: Color,
    /// Color for the footer / progress bar.
    pub footer: Color,
    /// syntect theme name for syntax highlighting.
    pub syntax_theme: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::Reset,
            foreground: Color::Reset,
            heading_h1: Color::Cyan,
            heading_h2: Color::Green,
            heading_h3: Color::Yellow,
            code_background: Color::DarkGray,
            code_foreground: Color::White,
            code_border: Color::Gray,
            block_quote: Color::Gray,
            footer: Color::DarkGray,
            syntax_theme: String::from("base16-ocean.dark"),
        }
    }
}

/// Raw theme file representation for TOML deserialization.
///
/// Color values are strings that get parsed into `ratatui::style::Color`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ThemeFile {
    /// Background color name or hex.
    pub background: Option<String>,
    /// Foreground color name or hex.
    pub foreground: Option<String>,
    /// H1 heading color.
    pub heading_h1: Option<String>,
    /// H2 heading color.
    pub heading_h2: Option<String>,
    /// H3+ heading color.
    pub heading_h3: Option<String>,
    /// Code block background color.
    pub code_background: Option<String>,
    /// Code block foreground color.
    pub code_foreground: Option<String>,
    /// Code block border color.
    pub code_border: Option<String>,
    /// Block quote color.
    pub block_quote: Option<String>,
    /// Footer / progress bar color.
    pub footer: Option<String>,
    /// syntect theme name.
    pub syntax_theme: Option<String>,
}

/// Parse a color string into a `ratatui::style::Color`.
///
/// Supports named colors (`"red"`, `"blue"`, etc.), hex colors (`"#ff0000"`),
/// and `"reset"` for the terminal default.
#[must_use]
pub fn parse_color(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "reset" | "default" | "" => Color::Reset,
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" | "dark_gray" | "dark_grey" => Color::DarkGray,
        "lightred" | "light_red" => Color::LightRed,
        "lightgreen" | "light_green" => Color::LightGreen,
        "lightyellow" | "light_yellow" => Color::LightYellow,
        "lightblue" | "light_blue" => Color::LightBlue,
        "lightmagenta" | "light_magenta" => Color::LightMagenta,
        "lightcyan" | "light_cyan" => Color::LightCyan,
        "white" => Color::White,
        hex if hex.starts_with('#') && hex.len() == 7 => {
            let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0);
            Color::Rgb(r, g, b)
        }
        _ => Color::Reset,
    }
}

impl ThemeFile {
    /// Merge this theme file into a base `Theme`, overriding only specified fields.
    #[must_use]
    pub fn apply_to(&self, base: &Theme) -> Theme {
        Theme {
            background: self
                .background
                .as_deref()
                .map(parse_color)
                .unwrap_or(base.background),
            foreground: self
                .foreground
                .as_deref()
                .map(parse_color)
                .unwrap_or(base.foreground),
            heading_h1: self
                .heading_h1
                .as_deref()
                .map(parse_color)
                .unwrap_or(base.heading_h1),
            heading_h2: self
                .heading_h2
                .as_deref()
                .map(parse_color)
                .unwrap_or(base.heading_h2),
            heading_h3: self
                .heading_h3
                .as_deref()
                .map(parse_color)
                .unwrap_or(base.heading_h3),
            code_background: self
                .code_background
                .as_deref()
                .map(parse_color)
                .unwrap_or(base.code_background),
            code_foreground: self
                .code_foreground
                .as_deref()
                .map(parse_color)
                .unwrap_or(base.code_foreground),
            code_border: self
                .code_border
                .as_deref()
                .map(parse_color)
                .unwrap_or(base.code_border),
            block_quote: self
                .block_quote
                .as_deref()
                .map(parse_color)
                .unwrap_or(base.block_quote),
            footer: self
                .footer
                .as_deref()
                .map(parse_color)
                .unwrap_or(base.footer),
            syntax_theme: self
                .syntax_theme
                .clone()
                .unwrap_or_else(|| base.syntax_theme.clone()),
        }
    }
}
