//! The design tokens — every color and text style in the presenter.
//!
//! One polished default theme. It deliberately uses ANSI palette colors and
//! leaves the background untouched (`Color::Reset`), so it sits well on any
//! terminal the presenter already likes. No render code may construct a
//! `Style` from raw colors; everything goes through [`Tokens`].

use ratatui::style::{Color, Modifier, Style};

/// Semantic styles for the presenter UI.
#[derive(Debug, Clone)]
pub struct Tokens {
    /// Body text.
    pub text: Style,
    /// De-emphasized text: hints, captions, separators, metadata.
    pub muted: Style,
    /// Brand accent: deck title, prompts, selection markers.
    pub accent: Style,
    /// Code block text (plain / unrecognized tokens).
    pub code: Style,
    /// Emphasized (highlighted) code lines when no syntax colors apply.
    pub code_highlight: Style,
    /// Code: keywords and storage words (`fn`, `let`, `if`, `return`).
    pub code_keyword: Style,
    /// Code: string literals.
    pub code_string: Style,
    /// Code: comments.
    pub code_comment: Style,
    /// Code: function names at definition and call sites.
    pub code_function: Style,
    /// Code: type, class, and other entity names.
    pub code_type: Style,
    /// Code: numeric and language constants.
    pub code_constant: Style,
    /// The currently selected item in menus and pickers.
    pub selected: Style,
    /// Positive feedback.
    pub success: Style,
    /// Cautionary feedback.
    pub warning: Style,
    /// Failure feedback.
    pub error: Style,
    /// Borders and rules.
    pub border: Style,
}

impl Default for Tokens {
    fn default() -> Self {
        Self {
            text: Style::new(),
            muted: Style::new().fg(Color::DarkGray),
            accent: Style::new().fg(Color::Cyan),
            code: Style::new().fg(Color::Gray),
            code_highlight: Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            code_keyword: Style::new().fg(Color::Magenta),
            code_string: Style::new().fg(Color::Green),
            code_comment: Style::new()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
            code_function: Style::new().fg(Color::Blue),
            code_type: Style::new().fg(Color::Cyan),
            code_constant: Style::new().fg(Color::Yellow),
            selected: Style::new().add_modifier(Modifier::REVERSED | Modifier::BOLD),
            success: Style::new().fg(Color::Green),
            warning: Style::new().fg(Color::Yellow),
            error: Style::new().fg(Color::Red),
            border: Style::new().fg(Color::DarkGray),
        }
    }
}

impl Tokens {
    /// Style for a heading of the given level (1–6).
    #[must_use]
    pub fn heading(&self, level: u8) -> Style {
        match level {
            1 => self.accent.add_modifier(Modifier::BOLD),
            2 => self.text.add_modifier(Modifier::BOLD),
            _ => self.text.add_modifier(Modifier::BOLD | Modifier::DIM),
        }
    }
}
