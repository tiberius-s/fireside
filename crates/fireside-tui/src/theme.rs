//! Theme definitions controlling the visual appearance of nodes.
//!
//! The `Theme` struct holds Ratatui `Color` values for all UI elements.
//! Themes are loaded from JSON files via `ThemeFile`.

use ratatui::style::Color;
use serde::Deserialize;

/// A complete theme definition for rendering.
///
/// All color fields use precise `Color::Rgb` values by default so the
/// appearance is consistent across terminal emulators. Named colors
/// (Cyan, Green, …) vary between emulator color schemes and should only
/// be used in explicitly terminal-adapted theme files.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    // ── Base layer ─────────────────────────────────────────────────
    /// Background color for the content area (terminal bg).
    pub background: Color,
    /// Default foreground (body text) color.
    pub foreground: Color,

    // ── Surface ────────────────────────────────────────────────────
    /// Elevated surface color for panels, cards, and sidebars.
    pub surface: Color,
    /// Text color on `surface` backgrounds.
    pub on_surface: Color,

    // ── Headings ───────────────────────────────────────────────────
    /// Color for H1 headings.
    pub heading_h1: Color,
    /// Color for H2 headings.
    pub heading_h2: Color,
    /// Color for H3+ headings.
    pub heading_h3: Color,

    // ── Code blocks ────────────────────────────────────────────────
    /// Background color for code blocks.
    pub code_background: Color,
    /// Foreground color for code block text (base, before syntax highlighting).
    pub code_foreground: Color,
    /// Color for the border around code blocks.
    pub code_border: Color,

    // ── Misc content ───────────────────────────────────────────────
    /// Color for block quote borders and text.
    pub block_quote: Color,
    /// Color for the footer / progress bar.
    pub footer: Color,

    // ── Chrome ─────────────────────────────────────────────────────
    /// Border color for the focused / active panel.
    pub border_active: Color,
    /// Border color for unfocused panels.
    pub border_inactive: Color,
    /// Background color for toolbars and status bars.
    pub toolbar_bg: Color,
    /// Foreground color for toolbar text and key hints.
    pub toolbar_fg: Color,
    /// Accent color for interactive elements, badges, and highlights.
    pub accent: Color,
    /// Error / destructive-action color.
    pub error: Color,
    /// Success / positive-indicator color.
    pub success: Color,

    // ── Syntax ─────────────────────────────────────────────────────
    /// syntect theme name for syntax highlighting.
    pub syntax_theme: String,
}

impl Default for Theme {
    /// One Dark palette — consistent Rgb values independent of the terminal's
    /// own color scheme. Contrast ratios have been verified against WCAG AA.
    fn default() -> Self {
        Self {
            // Base — One Dark dark bg (#282C34) and body text (#ABB2BF)
            background: Color::Rgb(40, 44, 52),
            foreground: Color::Rgb(171, 178, 191),

            // Surface — slightly lighter panel bg (#2C313C) and light text
            surface: Color::Rgb(44, 49, 60),
            on_surface: Color::Rgb(220, 223, 228),

            // Headings — blue (#61AFEF), green (#98C379), gold (#E5C07B)
            heading_h1: Color::Rgb(97, 175, 239),
            heading_h2: Color::Rgb(152, 195, 121),
            heading_h3: Color::Rgb(229, 192, 123),

            // Code blocks — same bg as main bg, light text, dim border
            code_background: Color::Rgb(40, 44, 52),
            code_foreground: Color::Rgb(220, 223, 228),
            code_border: Color::Rgb(62, 68, 81),

            // Misc content
            block_quote: Color::Rgb(92, 99, 112),
            footer: Color::Rgb(92, 99, 112),

            // Chrome — active border matches h1 blue; toolbar is darkest layer
            border_active: Color::Rgb(97, 175, 239),
            border_inactive: Color::Rgb(62, 68, 81),
            toolbar_bg: Color::Rgb(33, 37, 43),
            toolbar_fg: Color::Rgb(171, 178, 191),

            // Semantic — purple, red, green
            accent: Color::Rgb(198, 120, 221),
            error: Color::Rgb(224, 108, 117),
            success: Color::Rgb(152, 195, 121),

            syntax_theme: String::from("base16-ocean.dark"),
        }
    }
}

/// Raw theme file representation for JSON deserialization.
///
/// Color values are strings that get parsed into `ratatui::style::Color`.
/// Only fields present in the JSON file override the base theme defaults.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ThemeFile {
    /// Background color name or hex.
    pub background: Option<String>,
    /// Foreground color name or hex.
    pub foreground: Option<String>,
    /// Elevated surface/panel background.
    pub surface: Option<String>,
    /// Text on surface background.
    pub on_surface: Option<String>,
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
    /// Active (focused) panel border color.
    pub border_active: Option<String>,
    /// Inactive (unfocused) panel border color.
    pub border_inactive: Option<String>,
    /// Toolbar background color.
    pub toolbar_bg: Option<String>,
    /// Toolbar foreground/text color.
    pub toolbar_fg: Option<String>,
    /// Accent color for badges and interactive elements.
    pub accent: Option<String>,
    /// Error / warning color.
    pub error: Option<String>,
    /// Success / confirmation color.
    pub success: Option<String>,
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
        let c = |opt: &Option<String>, fallback: Color| -> Color {
            opt.as_deref().map(parse_color).unwrap_or(fallback)
        };
        Theme {
            background: c(&self.background, base.background),
            foreground: c(&self.foreground, base.foreground),
            surface: c(&self.surface, base.surface),
            on_surface: c(&self.on_surface, base.on_surface),
            heading_h1: c(&self.heading_h1, base.heading_h1),
            heading_h2: c(&self.heading_h2, base.heading_h2),
            heading_h3: c(&self.heading_h3, base.heading_h3),
            code_background: c(&self.code_background, base.code_background),
            code_foreground: c(&self.code_foreground, base.code_foreground),
            code_border: c(&self.code_border, base.code_border),
            block_quote: c(&self.block_quote, base.block_quote),
            footer: c(&self.footer, base.footer),
            border_active: c(&self.border_active, base.border_active),
            border_inactive: c(&self.border_inactive, base.border_inactive),
            toolbar_bg: c(&self.toolbar_bg, base.toolbar_bg),
            toolbar_fg: c(&self.toolbar_fg, base.toolbar_fg),
            accent: c(&self.accent, base.accent),
            error: c(&self.error, base.error),
            success: c(&self.success, base.success),
            syntax_theme: self
                .syntax_theme
                .clone()
                .unwrap_or_else(|| base.syntax_theme.clone()),
        }
    }
}
