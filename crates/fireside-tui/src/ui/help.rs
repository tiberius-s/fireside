//! Help overlay showing keybindings.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpMode {
    Presenting,
    Editing,
}

#[derive(Debug, Clone)]
pub struct HelpNavigation {
    pub total_rows: usize,
    pub viewport_rows: usize,
    pub section_starts: Vec<usize>,
}

const MODE_PRESENTING: u8 = 0b01;
const MODE_EDITING: u8 = 0b10;
const MODE_BOTH: u8 = MODE_PRESENTING | MODE_EDITING;

struct HelpEntry {
    section: &'static str,
    key: &'static str,
    desc: &'static str,
    modes: u8,
}

#[derive(Debug, Clone, Copy)]
struct SectionLegend {
    index: usize,
    short: &'static str,
    active: bool,
}

const KEYBINDINGS: &[HelpEntry] = &[
    HelpEntry {
        section: "Navigation",
        key: "→ / l / Space / Enter",
        desc: "Next node",
        modes: MODE_PRESENTING,
    },
    HelpEntry {
        section: "Navigation",
        key: "← / h",
        desc: "Previous node",
        modes: MODE_PRESENTING,
    },
    HelpEntry {
        section: "Navigation",
        key: "g",
        desc: "Go to node (enter number)",
        modes: MODE_PRESENTING,
    },
    HelpEntry {
        section: "Branching",
        key: "a-f",
        desc: "Choose branch option",
        modes: MODE_PRESENTING,
    },
    HelpEntry {
        section: "Display",
        key: "s",
        desc: "Toggle speaker notes",
        modes: MODE_PRESENTING,
    },
    HelpEntry {
        section: "Editor",
        key: "e / Esc",
        desc: "Enter / exit editor mode",
        modes: MODE_BOTH,
    },
    HelpEntry {
        section: "Editor",
        key: "j / k, PgUp/PgDn",
        desc: "Select or page nodes",
        modes: MODE_EDITING,
    },
    HelpEntry {
        section: "Editor",
        key: "Home / End, g",
        desc: "Jump top/bottom or by index",
        modes: MODE_EDITING,
    },
    HelpEntry {
        section: "Editor",
        key: "/, [, ]",
        desc: "Search node ids and jump hits",
        modes: MODE_EDITING,
    },
    HelpEntry {
        section: "Editor",
        key: "i / o",
        desc: "Inline edit text / speaker notes",
        modes: MODE_EDITING,
    },
    HelpEntry {
        section: "Editor",
        key: "a / n / d",
        desc: "Append text, add node, remove node",
        modes: MODE_EDITING,
    },
    HelpEntry {
        section: "Editor",
        key: "l/t, L/T",
        desc: "Open/cycle layout + transition",
        modes: MODE_EDITING,
    },
    HelpEntry {
        section: "Editor",
        key: "w / Ctrl+s, u / r",
        desc: "Save graph, undo, redo",
        modes: MODE_EDITING,
    },
    HelpEntry {
        section: "Editor",
        key: "Tab",
        desc: "Toggle pane focus",
        modes: MODE_EDITING,
    },
    HelpEntry {
        section: "Graph View",
        key: "v",
        desc: "Toggle graph overlay",
        modes: MODE_EDITING,
    },
    HelpEntry {
        section: "Graph View",
        key: "j/k, PgUp/PgDn",
        desc: "Move graph selection / page",
        modes: MODE_EDITING,
    },
    HelpEntry {
        section: "Graph View",
        key: "Home / End",
        desc: "Jump to first / last graph node",
        modes: MODE_EDITING,
    },
    HelpEntry {
        section: "Graph View",
        key: "Enter / p",
        desc: "Jump node / jump + present",
        modes: MODE_EDITING,
    },
    HelpEntry {
        section: "System",
        key: "?",
        desc: "Toggle help overlay",
        modes: MODE_BOTH,
    },
    HelpEntry {
        section: "System",
        key: "q / Esc / Ctrl+c",
        desc: "Quit app or cancel prompt",
        modes: MODE_BOTH,
    },
    HelpEntry {
        section: "System",
        key: "Mouse",
        desc: "Presenter nav + editor interactions",
        modes: MODE_BOTH,
    },
];

/// Render the help overlay as a centered popup.
pub fn render_help_overlay(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    mode: HelpMode,
    scroll_offset: usize,
) {
    let popup = centered_popup(area, 66, 78);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Keybindings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.heading_h2))
        .style(Style::default().bg(theme.background));

    let inner = block.inner(popup);

    frame.render_widget(block, popup);

    let content = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let header_style = Style::default()
        .fg(theme.heading_h2)
        .add_modifier(Modifier::BOLD);
    let active_key_style = Style::default()
        .fg(theme.heading_h1)
        .add_modifier(Modifier::BOLD);
    let active_desc_style = Style::default().fg(theme.foreground);
    let inactive_key_style = Style::default()
        .fg(theme.footer)
        .add_modifier(Modifier::DIM);
    let inactive_desc_style = Style::default()
        .fg(theme.footer)
        .add_modifier(Modifier::DIM);

    let rows = build_help_rows(mode);
    let viewport_rows = content[0].height as usize;
    let max_scroll = rows.len().saturating_sub(viewport_rows);
    let start = scroll_offset.min(max_scroll);

    let lines = rows
        .iter()
        .skip(start)
        .take(viewport_rows)
        .map(|row| match row {
            HelpRow::Section { name } => Line::from(Span::styled(format!(" {name}"), header_style)),
            HelpRow::Entry { key, desc, active } => {
                let key_style = if *active {
                    active_key_style
                } else {
                    inactive_key_style
                };
                let desc_style = if *active {
                    active_desc_style
                } else {
                    inactive_desc_style
                };

                Line::from(vec![
                    Span::styled(format!("  {:<24}", key), key_style),
                    Span::styled(*desc, desc_style),
                ])
            }
            HelpRow::Spacer => Line::default(),
        })
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(lines), content[0]);

    let scroll_legend = format!("↑↓/Pg  {}/{}", start + 1, rows.len().max(1));
    let mut legend_spans = Vec::new();
    let sections = section_legends(mode);

    for (idx, section) in sections.iter().enumerate() {
        let style = if section.active {
            active_key_style
        } else {
            inactive_key_style
        };

        legend_spans.push(Span::styled(
            format!("{} {}", section.index, section.short),
            style,
        ));

        if idx + 1 < sections.len() {
            legend_spans.push(Span::styled("  ", active_desc_style));
        }
    }

    legend_spans.push(Span::styled("  •  ", active_desc_style));
    legend_spans.push(Span::styled(scroll_legend, active_desc_style));

    let legend = Line::from(legend_spans);

    frame.render_widget(Paragraph::new(legend), content[1]);
}

pub fn help_navigation(area: Rect, mode: HelpMode) -> HelpNavigation {
    let popup = centered_popup(area, 66, 78);
    let inner = Rect {
        x: popup.x.saturating_add(1),
        y: popup.y.saturating_add(1),
        width: popup.width.saturating_sub(2),
        height: popup.height.saturating_sub(2),
    };

    let rows = build_help_rows(mode);
    let viewport_rows = inner.height.saturating_sub(1) as usize;
    let mut section_starts = Vec::new();
    for (idx, row) in rows.iter().enumerate() {
        if matches!(row, HelpRow::Section { .. }) {
            section_starts.push(idx);
        }
    }

    HelpNavigation {
        total_rows: rows.len(),
        viewport_rows,
        section_starts,
    }
}

fn entry_active(mode: HelpMode, modes: u8) -> bool {
    match mode {
        HelpMode::Presenting => (modes & MODE_PRESENTING) != 0,
        HelpMode::Editing => (modes & MODE_EDITING) != 0,
    }
}

#[derive(Debug, Clone, Copy)]
enum HelpRow {
    Section {
        name: &'static str,
    },
    Entry {
        key: &'static str,
        desc: &'static str,
        active: bool,
    },
    Spacer,
}

fn build_help_rows(mode: HelpMode) -> Vec<HelpRow> {
    let mut rows = Vec::new();
    let mut last_section = "";

    for entry in KEYBINDINGS {
        if entry.section != last_section {
            if !last_section.is_empty() {
                rows.push(HelpRow::Spacer);
            }
            rows.push(HelpRow::Section {
                name: entry.section,
            });
            last_section = entry.section;
        }

        rows.push(HelpRow::Entry {
            key: entry.key,
            desc: entry.desc,
            active: entry_active(mode, entry.modes),
        });
    }

    rows
}

fn section_legends(mode: HelpMode) -> Vec<SectionLegend> {
    const SECTIONS: [(&str, &str); 6] = [
        ("Navigation", "Nav"),
        ("Branching", "Branch"),
        ("Display", "Display"),
        ("Editor", "Editor"),
        ("Graph View", "Graph"),
        ("System", "System"),
    ];

    SECTIONS
        .iter()
        .enumerate()
        .map(|(index, (name, short))| SectionLegend {
            index: index + 1,
            short,
            active: KEYBINDINGS
                .iter()
                .any(|entry| entry.section == *name && entry_active(mode, entry.modes)),
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::{HelpMode, section_legends};

    #[test]
    fn section_legend_activity_reflects_current_mode() {
        let presenting = section_legends(HelpMode::Presenting);
        let editing = section_legends(HelpMode::Editing);

        assert!(presenting[0].active);
        assert!(presenting[1].active);
        assert!(presenting[2].active);
        assert!(presenting[3].active);
        assert!(!presenting[4].active);
        assert!(presenting[5].active);

        assert!(!editing[0].active);
        assert!(!editing[1].active);
        assert!(!editing[2].active);
        assert!(editing[3].active);
        assert!(editing[4].active);
        assert!(editing[5].active);
    }
}
