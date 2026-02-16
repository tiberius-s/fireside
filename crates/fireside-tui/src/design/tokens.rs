//! Design tokens — semantic color system for Fireside.
//!
//! Maps abstract roles (background, surface, primary, accent, etc.) to
//! concrete `ratatui::style::Color` values. Tokens can be populated from
//! iTerm2 color schemes, TOML theme files, or built-in defaults.
//!
//! ## Token Hierarchy
//!
//! ```text
//! Background ─── base layer (terminal background)
//! Surface    ─── elevated panels, cards, code blocks
//! Primary    ─── headings, selected items, active borders
//! Accent     ─── links, highlights, interactive elements
//! Muted      ─── dimmed text, placeholders, borders
//! Error      ─── destructive actions, warnings
//! Success    ─── confirmations, positive indicators
//! OnBackground ─ text on Background
//! OnSurface    ─ text on Surface
//! OnPrimary    ─ text on Primary-colored elements
//! ```

use ratatui::style::Color;

use crate::theme::Theme;

/// Complete set of semantic design tokens for a Fireside theme.
///
/// These tokens provide a consistent color language across all UI components.
/// Each token maps to a specific role, not a raw color — enabling theme
/// switching without updating individual widget styles.
#[derive(Debug, Clone, PartialEq)]
pub struct DesignTokens {
    // ─── Base palette ───────────────────────────────────────────────
    /// Primary background color (terminal bg).
    pub background: Color,
    /// Elevated surface color for panels, cards, code blocks.
    pub surface: Color,
    /// Primary brand/accent color: headings, active borders.
    pub primary: Color,
    /// Secondary accent for links, interactive elements.
    pub accent: Color,
    /// Muted color for borders, dimmed text, separators.
    pub muted: Color,
    /// Error / destructive action color.
    pub error: Color,
    /// Success / positive indicator color.
    pub success: Color,

    // ─── On-colors (text on specific backgrounds) ────────────────
    /// Text on `background`.
    pub on_background: Color,
    /// Text on `surface`.
    pub on_surface: Color,
    /// Text on `primary` backgrounds (e.g., selected item label).
    pub on_primary: Color,

    // ─── Typography tokens ──────────────────────────────────────
    /// H1 heading color.
    pub heading_h1: Color,
    /// H2 heading color.
    pub heading_h2: Color,
    /// H3+ heading color.
    pub heading_h3: Color,
    /// Body text color.
    pub body: Color,
    /// Code text foreground.
    pub code_fg: Color,
    /// Code block background.
    pub code_bg: Color,
    /// Block quote border and accent.
    pub quote: Color,

    // ─── Chrome tokens ──────────────────────────────────────────
    /// Footer / status bar color.
    pub footer: Color,
    /// Active border color for focused panels.
    pub border_active: Color,
    /// Inactive border color.
    pub border_inactive: Color,
    /// Toolbar background.
    pub toolbar_bg: Color,
    /// Toolbar text.
    pub toolbar_fg: Color,

    // ─── Syntax highlighting ────────────────────────────────────
    /// Name of the syntect theme to use for code blocks.
    pub syntax_theme: String,
}

impl Default for DesignTokens {
    fn default() -> Self {
        Self {
            // Base palette — dark terminal defaults
            background: Color::Reset,
            surface: Color::Rgb(40, 44, 52),
            primary: Color::Rgb(97, 175, 239),
            accent: Color::Rgb(198, 120, 221),
            muted: Color::Rgb(92, 99, 112),
            error: Color::Rgb(224, 108, 117),
            success: Color::Rgb(152, 195, 121),

            // On-colors
            on_background: Color::Rgb(171, 178, 191),
            on_surface: Color::Rgb(220, 223, 228),
            on_primary: Color::Rgb(40, 44, 52),

            // Typography
            heading_h1: Color::Rgb(97, 175, 239),
            heading_h2: Color::Rgb(152, 195, 121),
            heading_h3: Color::Rgb(229, 192, 123),
            body: Color::Rgb(171, 178, 191),
            code_fg: Color::Rgb(220, 223, 228),
            code_bg: Color::Rgb(40, 44, 52),
            quote: Color::Rgb(92, 99, 112),

            // Chrome
            footer: Color::Rgb(92, 99, 112),
            border_active: Color::Rgb(97, 175, 239),
            border_inactive: Color::Rgb(62, 68, 81),
            toolbar_bg: Color::Rgb(33, 37, 43),
            toolbar_fg: Color::Rgb(171, 178, 191),

            // Syntax
            syntax_theme: String::from("base16-ocean.dark"),
        }
    }
}

impl DesignTokens {
    /// Convert tokens to the `Theme` struct used by the renderer.
    #[must_use]
    pub fn to_theme(&self) -> Theme {
        Theme {
            background: self.background,
            foreground: self.on_background,
            heading_h1: self.heading_h1,
            heading_h2: self.heading_h2,
            heading_h3: self.heading_h3,
            code_background: self.code_bg,
            code_foreground: self.code_fg,
            code_border: self.border_inactive,
            block_quote: self.quote,
            footer: self.footer,
            syntax_theme: self.syntax_theme.clone(),
        }
    }

    /// Create tokens from a `Theme`.
    #[must_use]
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.background,
            on_background: theme.foreground,
            on_surface: theme.foreground,
            heading_h1: theme.heading_h1,
            heading_h2: theme.heading_h2,
            heading_h3: theme.heading_h3,
            body: theme.foreground,
            code_fg: theme.code_foreground,
            code_bg: theme.code_background,
            quote: theme.block_quote,
            footer: theme.footer,
            border_inactive: theme.code_border,
            syntax_theme: theme.syntax_theme.clone(),
            ..Self::default()
        }
    }
}

/// Spacing scale for consistent margins and padding.
///
/// Values are in terminal cells (columns for horizontal, rows for vertical).
pub struct Spacing;

impl Spacing {
    /// Extra-small: 1 cell.
    pub const XS: u16 = 1;
    /// Small: 2 cells.
    pub const SM: u16 = 2;
    /// Medium: 3 cells.
    pub const MD: u16 = 3;
    /// Large: 4 cells.
    pub const LG: u16 = 4;
    /// Extra-large: 6 cells.
    pub const XL: u16 = 6;
}

/// Terminal size breakpoints for responsive layout adjustments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    /// Compact: 80x24 or smaller.
    Compact,
    /// Standard: up to 120x40.
    Standard,
    /// Wide: 121+ columns.
    Wide,
}

impl Breakpoint {
    /// Determine the breakpoint from terminal dimensions.
    #[must_use]
    pub fn from_size(width: u16, height: u16) -> Self {
        if width <= 80 || height <= 24 {
            Self::Compact
        } else if width <= 120 || height <= 40 {
            Self::Standard
        } else {
            Self::Wide
        }
    }

    /// Recommended content width percentage for this breakpoint.
    #[must_use]
    pub fn content_width_pct(&self) -> u16 {
        match self {
            Self::Compact => 96,
            Self::Standard => 85,
            Self::Wide => 75,
        }
    }

    /// Recommended horizontal padding for this breakpoint.
    #[must_use]
    pub fn h_padding(&self) -> u16 {
        match self {
            Self::Compact => Spacing::XS,
            Self::Standard => Spacing::SM,
            Self::Wide => Spacing::LG,
        }
    }
}

/// WCAG contrast ratio check for two RGB colors.
///
/// Returns `true` if the contrast ratio meets the AA threshold (>= 4.5:1).
#[must_use]
pub fn meets_contrast_aa(fg: Color, bg: Color) -> bool {
    let ratio = contrast_ratio(fg, bg);
    ratio >= 4.5
}

/// Calculate the WCAG 2.1 contrast ratio between two colors.
///
/// Returns 1.0 for identical colors, up to 21.0 for black-on-white.
/// Non-RGB colors return 1.0 (unknown).
#[must_use]
pub fn contrast_ratio(c1: Color, c2: Color) -> f64 {
    let l1 = relative_luminance(c1);
    let l2 = relative_luminance(c2);

    if l1 < 0.0 || l2 < 0.0 {
        return 1.0; // Can't compute for non-RGB
    }

    let lighter = l1.max(l2);
    let darker = l1.min(l2);

    (lighter + 0.05) / (darker + 0.05)
}

/// Compute relative luminance per WCAG 2.1.
///
/// Returns -1.0 for non-RGB colors.
fn relative_luminance(color: Color) -> f64 {
    let Color::Rgb(r, g, b) = color else {
        return -1.0;
    };

    let linearize = |c: u8| -> f64 {
        let s = f64::from(c) / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };

    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tokens_body_on_background_contrast() {
        let tokens = DesignTokens::default();
        // One Dark body (#abb2bf) on dark bg (#282c34) should pass AA
        let ratio = contrast_ratio(tokens.body, tokens.surface);
        assert!(ratio >= 4.5, "Body on surface contrast {ratio:.1} < 4.5");
    }

    #[test]
    fn black_on_white_max_contrast() {
        let ratio = contrast_ratio(Color::Rgb(0, 0, 0), Color::Rgb(255, 255, 255));
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn same_color_contrast_is_one() {
        let ratio = contrast_ratio(Color::Rgb(128, 128, 128), Color::Rgb(128, 128, 128));
        assert!((ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn breakpoint_compact() {
        assert_eq!(Breakpoint::from_size(80, 24), Breakpoint::Compact);
        assert_eq!(Breakpoint::from_size(60, 20), Breakpoint::Compact);
    }

    #[test]
    fn breakpoint_standard() {
        assert_eq!(Breakpoint::from_size(120, 40), Breakpoint::Standard);
    }

    #[test]
    fn breakpoint_wide() {
        assert_eq!(Breakpoint::from_size(160, 50), Breakpoint::Wide);
    }

    #[test]
    fn theme_roundtrip() {
        let tokens = DesignTokens::default();
        let theme = tokens.to_theme();
        let back = DesignTokens::from_theme(&theme);
        assert_eq!(tokens.heading_h1, back.heading_h1);
        assert_eq!(tokens.code_bg, back.code_bg);
        assert_eq!(tokens.footer, back.footer);
    }
}
