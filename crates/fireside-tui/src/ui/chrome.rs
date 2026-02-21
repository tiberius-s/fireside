//! Shared UI chrome widgets.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeBadgeKind {
    Presenting,
    Editing,
    GotoNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashKind {
    Info,
    Success,
    Warning,
    Error,
}

impl ModeBadgeKind {
    fn label(self) -> &'static str {
        match self {
            Self::Presenting => "PRESENTING",
            Self::Editing => "EDITING",
            Self::GotoNode => "GOTO",
        }
    }

    fn color(self, theme: &Theme) -> ratatui::style::Color {
        match self {
            Self::Presenting => theme.heading_h1,
            Self::Editing => theme.accent,
            Self::GotoNode => theme.heading_h3,
        }
    }
}

#[must_use]
pub fn mode_badge_width(kind: ModeBadgeKind) -> u16 {
    kind.label().chars().count() as u16 + 4
}

pub fn render_mode_badge(frame: &mut Frame, area: Rect, kind: ModeBadgeKind, theme: &Theme) {
    let badge_height = 3;
    let badge_width = mode_badge_width(kind).min(area.width);

    if area.width < 3 || area.height < badge_height || badge_width < 3 {
        return;
    }

    let badge = Rect {
        x: area.x + area.width - badge_width,
        y: area.y,
        width: badge_width,
        height: badge_height,
    };

    let color = kind.color(theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .style(Style::default().bg(theme.surface));
    let inner = block.inner(badge);

    frame.render_widget(block, badge);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            kind.label(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center),
        inner,
    );
}

pub fn render_flash_message(
    frame: &mut Frame,
    area: Rect,
    text: &str,
    kind: FlashKind,
    theme: &Theme,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let color = match kind {
        FlashKind::Info => theme.heading_h1,
        FlashKind::Success => theme.success,
        FlashKind::Warning => theme.heading_h3,
        FlashKind::Error => theme.error,
    };

    let content = format!(" {}", text);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            content,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )))
        .block(Block::default().style(Style::default().bg(theme.toolbar_bg))),
        area,
    );
}

pub fn render_quit_confirmation_banner(frame: &mut Frame, area: Rect, theme: &Theme) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " Save and quit? ",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("[y]es  ", Style::default().fg(theme.heading_h2)),
            Span::styled("[n]o  ", Style::default().fg(theme.heading_h2)),
            Span::styled("[s]ave first  ", Style::default().fg(theme.heading_h2)),
            Span::styled("[Esc] cancel", Style::default().fg(theme.footer)),
        ]))
        .block(Block::default().style(Style::default().bg(theme.toolbar_bg))),
        area,
    );
}
