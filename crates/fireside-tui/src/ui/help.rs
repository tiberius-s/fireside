//! Help overlay showing keybindings.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::theme::Theme;

/// Keybinding entries displayed in the help overlay.
const KEYBINDINGS: &[(&str, &str)] = &[
    ("→ / l / Space / Enter", "Next node"),
    ("← / h", "Previous node"),
    ("g", "Go to node (enter number)"),
    ("a-f", "Choose branch option"),
    ("?", "Toggle this help"),
    ("q / Esc", "Quit"),
];

/// Render the help overlay as a centered popup.
pub fn render_help_overlay(frame: &mut Frame, area: Rect, theme: &Theme) {
    let popup = centered_popup(area, 50, 60);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Keybindings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.heading_h2))
        .style(Style::default().bg(theme.background));

    let inner = block.inner(popup);

    frame.render_widget(block, popup);

    let key_style = Style::default()
        .fg(theme.heading_h1)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(theme.foreground);

    let lines: Vec<Line<'_>> = KEYBINDINGS
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(format!("  {key:<28}"), key_style),
                Span::styled(*desc, desc_style),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

/// Create a centered rect for a popup overlay.
fn centered_popup(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_pct) / 2),
            Constraint::Percentage(height_pct),
            Constraint::Percentage((100 - height_pct) / 2),
        ])
        .split(area);

    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_pct) / 2),
            Constraint::Percentage(width_pct),
            Constraint::Percentage((100 - width_pct) / 2),
        ])
        .split(vert[1]);

    horiz[1]
}
