//! Heading block renderer.
//!
//! Renders a [`ContentBlock::Heading`] into styled ratatui [`Line`]s.
//! Visual hierarchy: H1 has a coloured accent bar + full-width rule;
//! H2 has a dimmed accent bar + rule; H3 uses a `❯` prefix; H4+ uses `‣`.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::design::tokens::DesignTokens;

/// Render a heading into styled lines.
///
/// | Level | Treatment                                              |
/// |-------|--------------------------------------------------------|
/// | 1     | `▐ ` accent + BOLD text + full-width `═` rule (h1 colour) |
/// | 2     | `▌ ` accent (dim) + BOLD text + `─` rule (h2 dim)     |
/// | 3     | `  ❯ ` prefix + BOLD text in h3 colour                 |
/// | 4+    | `    ‣ ` prefix (dim) + BOLD text (dim)               |
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

    let text_style = Style::default().fg(color).add_modifier(Modifier::BOLD);
    let w = width.max(10) as usize;

    match level {
        1 => {
            let rule_width = w.saturating_sub(0);
            vec![
                // Top accent bar: half-height block fills to create a "chapter card" cap.
                Line::from(Span::styled(
                    "▔".repeat(rule_width),
                    Style::default().fg(color).add_modifier(Modifier::DIM),
                )),
                Line::from(vec![
                    Span::styled("▐ ", Style::default().fg(color)),
                    Span::styled(text.to_owned(), text_style),
                ]),
                Line::from(Span::styled(
                    "═".repeat(rule_width),
                    Style::default().fg(color).add_modifier(Modifier::DIM),
                )),
            ]
        }
        2 => {
            let rule_width = w.saturating_sub(2);
            vec![
                Line::from(vec![
                    Span::styled("▌ ", Style::default().fg(color).add_modifier(Modifier::DIM)),
                    Span::styled(text.to_owned(), text_style),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        "─".repeat(rule_width),
                        Style::default().fg(color).add_modifier(Modifier::DIM),
                    ),
                ]),
            ]
        }
        3 => vec![Line::from(vec![
            Span::styled("  ❯ ", Style::default().fg(color)),
            Span::styled(text.to_owned(), text_style),
        ])],
        _ => vec![Line::from(vec![
            Span::styled(
                "    ‣ ",
                Style::default()
                    .fg(color)
                    .add_modifier(Modifier::DIM | Modifier::BOLD),
            ),
            Span::styled(text.to_owned(), text_style.add_modifier(Modifier::DIM)),
        ])],
    }
}
