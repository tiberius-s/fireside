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
    Branch,
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
            Self::Presenting => "■ PRESENT",
            Self::Editing => "✎ EDITING",
            Self::GotoNode => "⊞ GOTO",
            Self::Branch => "⎇ BRANCH",
        }
    }

    fn color(self, theme: &Theme) -> ratatui::style::Color {
        match self {
            Self::Presenting => theme.heading_h2,
            Self::Editing => theme.accent,
            Self::GotoNode => theme.heading_h3,
            Self::Branch => theme.error,
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
        .style(Style::default().bg(theme.border_inactive));
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

/// Build undo/redo status chips for the editor footer.
#[must_use]
pub fn render_undo_redo_chips(can_undo: bool, can_redo: bool, theme: &Theme) -> Vec<Span<'static>> {
    let undo_style = if can_undo {
        Style::default()
            .fg(theme.heading_h1)
            .bg(theme.toolbar_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.footer)
            .bg(theme.border_inactive)
            .add_modifier(Modifier::DIM)
    };

    let redo_style = if can_redo {
        Style::default()
            .fg(theme.accent)
            .bg(theme.toolbar_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.footer)
            .bg(theme.border_inactive)
            .add_modifier(Modifier::DIM)
    };

    vec![
        Span::styled("[Z undo]", undo_style),
        Span::raw(" "),
        Span::styled("[Y redo]", redo_style),
    ]
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

    // [y]/[n]/Enter = discard and exit; [s] = save first; [Esc] = stay.
    // "Save and quit?" was misleading because [y] discards — use clear framing.
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " Unsaved changes! ",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("[y] discard+exit  ", Style::default().fg(theme.heading_h2)),
            Span::styled("[s] save+exit  ", Style::default().fg(theme.success)),
            Span::styled("[Esc] stay", Style::default().fg(theme.footer)),
        ]))
        .block(Block::default().style(Style::default().bg(theme.toolbar_bg))),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::{ModeBadgeKind, mode_badge_width, render_undo_redo_chips};
    use crate::theme::Theme;
    use ratatui::style::Modifier;

    #[test]
    fn mode_badge_width_tracks_exact_label_lengths() {
        assert_eq!(mode_badge_width(ModeBadgeKind::Presenting), 13);
        assert_eq!(mode_badge_width(ModeBadgeKind::Editing), 13);
        assert_eq!(mode_badge_width(ModeBadgeKind::GotoNode), 10);
        assert_eq!(mode_badge_width(ModeBadgeKind::Branch), 12);
    }

    #[test]
    fn undo_redo_chip_styles_reflect_enabled_state() {
        let theme = Theme::default();
        let active = render_undo_redo_chips(true, true, &theme);
        let disabled = render_undo_redo_chips(false, false, &theme);

        assert_eq!(active[0].style.fg, Some(theme.heading_h1));
        assert!(active[0].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(active[2].style.fg, Some(theme.accent));
        assert!(active[2].style.add_modifier.contains(Modifier::BOLD));

        assert_eq!(disabled[0].style.fg, Some(theme.footer));
        assert_eq!(disabled[0].style.bg, Some(theme.border_inactive));
        assert!(disabled[0].style.add_modifier.contains(Modifier::DIM));
        assert_eq!(disabled[2].style.fg, Some(theme.footer));
        assert_eq!(disabled[2].style.bg, Some(theme.border_inactive));
        assert!(disabled[2].style.add_modifier.contains(Modifier::DIM));
    }
}
