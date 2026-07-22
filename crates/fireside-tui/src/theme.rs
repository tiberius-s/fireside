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
    /// Rail-line colors for the map: parallel branch tracks cycle through
    /// these, subway-style. Index with [`Tokens::rail`]. None of them repeat
    /// the accent, which the spine (main line) wears.
    pub rail_lines: [Style; 4],
    /// The authoring editor (spec 013): "you can interact with this" — the
    /// one accent every clickable chip, row, and hover cue wears (design
    /// brief principle 3).
    pub affordance: Style,
    /// The authoring editor: the currently selected block or outline row —
    /// distinct from [`Tokens::selected`], which is the presenter's
    /// highlighted branch option.
    pub selection: Style,
    /// The authoring editor: where a drag-in-progress would land if
    /// released now.
    pub drop_target: Style,
    /// The authoring editor: the dimmed block that follows the pointer
    /// while it is being dragged.
    pub ghost: Style,
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
            rail_lines: [
                Style::new().fg(Color::Magenta),
                Style::new().fg(Color::Yellow),
                Style::new().fg(Color::Green),
                Style::new().fg(Color::Blue),
            ],
            affordance: Style::new().fg(Color::Cyan),
            selection: Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            drop_target: Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ghost: Style::new().fg(Color::DarkGray).add_modifier(Modifier::DIM),
        }
    }
}

impl Tokens {
    /// The line style for the `i`-th parallel rail at a fork.
    #[must_use]
    pub fn rail(&self, i: usize) -> Style {
        self.rail_lines[i % self.rail_lines.len()]
    }

    /// Style for a heading of the given level (1–6).
    #[must_use]
    pub fn heading(&self, level: u8) -> Style {
        match level {
            1 => self.accent.add_modifier(Modifier::BOLD),
            2 => self.text.add_modifier(Modifier::BOLD),
            _ => self.text.add_modifier(Modifier::BOLD | Modifier::DIM),
        }
    }

    /// Style for the `index`-th hyperlink label: a real, always-reasonable
    /// accent-and-underline look on every terminal, plus the link's index
    /// smuggled into `underline_color`'s red channel (1-based, so `None`
    /// unambiguously means "not a link"). `render::apply_hyperlinks`
    /// recovers it after the frame draws, to wrap exactly those cells in
    /// an OSC 8 escape (see specs/007-modern-tui-leverage/research.md §4).
    /// No other style in this theme sets `underline_color`, so there is no
    /// collision to worry about.
    #[must_use]
    pub fn link(&self, index: usize) -> Style {
        let marker = (index % 255) as u8 + 1;
        self.accent
            .add_modifier(Modifier::UNDERLINED)
            .underline_color(Color::Rgb(marker, 0, 0))
    }

    /// Decodes a link index from a style produced by [`Tokens::link`], if
    /// any.
    #[must_use]
    pub fn link_index(style: Style) -> Option<usize> {
        match style.underline_color {
            Some(Color::Rgb(marker, 0, 0)) if marker > 0 => Some(usize::from(marker - 1)),
            _ => None,
        }
    }
}
