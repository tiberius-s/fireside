//! Heading block renderer.
//!
//! Renders a [`ContentBlock::Heading`] into styled ratatui [`Line`]s.
//! H1 and H2 get a horizontal rule beneath them; deeper levels use
//! increasing indentation only.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::design::tokens::DesignTokens;

/// Render a heading into styled lines.
///
/// Level 1 → bold accent, double-rule under text.
/// Level 2 → bold accent, single-rule under text, 2-space indent.
/// Level 3+ → bold accent, increasing indent, no rule.
pub(super) fn render_heading<'a>(
    level: u8,
    text: &'a str,
    tokens: &DesignTokens,
    width: u16,
) -> Vec<Line<'a>> {
    let color = match level {
        1 => tokens.heading_h1,
        2 => tokens.heading_h2,
        _ => tokens.heading_h3,
    };

    let style = Style::default().fg(color).add_modifier(Modifier::BOLD);

    let prefix = match level {
        1 => "",
        2 => "  ",
        3 => "    ",
        _ => "      ",
    };

    let mut lines = vec![Line::from(vec![
        Span::raw(prefix),
        Span::styled(text.to_owned(), style),
    ])];

    if level <= 2 {
        let dash = if level == 1 { '═' } else { '─' };
        let rule_width = width.saturating_sub(prefix.len() as u16).max(10) as usize;
        lines.push(Line::from(vec![
            Span::raw(prefix),
            Span::styled(
                dash.to_string().repeat(rule_width),
                Style::default().fg(tokens.border_inactive),
            ),
        ]));
    }

    lines
}
