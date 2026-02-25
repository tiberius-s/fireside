//! Presenter view — composes the node content with chrome.
//!
//! This is the main rendering component that draws the current node
//! within the layout areas, overlaying the progress bar and optional help.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use fireside_core::model::layout::Layout;
use fireside_core::model::node::Node;
use fireside_core::model::transition::Transition;
use fireside_engine::PresentationSession;

use crate::design::templates::NodeTemplate;
use crate::design::tokens::Breakpoint;
use crate::render::blocks::render_node_content_with_base;
use crate::render::layout::two_column_split;
use crate::theme::Theme;
use std::path::Path;

use super::branch::render_branch_overlay;
use super::breadcrumb::render_breadcrumb;
use super::chrome::{
    FlashKind, ModeBadgeKind, render_flash_message, render_mode_badge,
    render_quit_confirmation_banner,
};
use super::help::{HelpMode, render_help_overlay};
use super::progress::render_progress_bar;
use super::timeline::render_timeline;
use super::transitions::transition_lines;

#[derive(Debug, Clone, Copy)]
pub struct PresenterTransition {
    pub kind: Transition,
    pub progress: f32,
    pub from_index: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct PresenterViewState<'a> {
    pub show_help: bool,
    pub help_scroll_offset: usize,
    pub show_speaker_notes: bool,
    pub show_progress_bar: bool,
    pub show_elapsed_timer: bool,
    pub show_chrome: bool,
    pub show_timeline: bool,
    pub target_duration_secs: Option<u64>,
    pub visited_nodes: &'a [usize],
    pub nav_path: &'a [(usize, bool)],
    pub content_base_dir: Option<&'a Path>,
    pub transition: Option<PresenterTransition>,
    pub elapsed_secs: u64,
    /// When in `GotoNode` mode, the digits typed so far (shown as a badge).
    pub goto_buffer: Option<&'a str>,
    /// Focused branch option index when branch overlay is visible.
    pub branch_focused_option: usize,
    /// Optional transient flash message shown above the footer.
    pub flash_message: Option<(&'a str, FlashKind)>,
    /// Whether quit confirmation is currently pending.
    pub pending_exit_confirmation: bool,
}

/// Render the full presenter view for the current node.
///
/// Draws the node content in the main area, the progress bar in the footer,
/// and optionally the help overlay on top.
pub fn render_presenter(
    frame: &mut Frame,
    session: &PresentationSession,
    theme: &Theme,
    view_state: PresenterViewState<'_>,
) {
    let area = frame.area();

    // Determine layout for current node
    let current = session.current_node_index();
    let node = &session.graph.nodes[current];
    let layout = node.layout.unwrap_or(Layout::Default);
    let breakpoint = Breakpoint::from_size(area.width, area.height);
    let template = if view_state.show_speaker_notes && node.speaker_notes.is_some() {
        NodeTemplate::SpeakerNotes
    } else {
        NodeTemplate::from_layout(layout)
    };
    let areas = template.compute_areas(area, breakpoint);

    // Clear background
    let bg_style = Style::default().bg(theme.background);
    let bg_block = Block::default().style(bg_style);
    frame.render_widget(bg_block, area);

    let content_area = if breakpoint == Breakpoint::Compact {
        Rect {
            x: area.x,
            y: areas.main.y,
            width: area.width,
            height: areas.main.height,
        }
    } else {
        areas.main
    };
    let notes_area = if template == NodeTemplate::SpeakerNotes {
        areas.secondary
    } else {
        None
    };

    // Render node content
    if let Some(transition) = view_state.transition {
        render_transition_node(
            frame,
            session,
            theme,
            content_area,
            layout,
            view_state.content_base_dir,
            transition,
        );
    } else {
        render_node(
            frame,
            node,
            theme,
            content_area,
            layout,
            view_state.content_base_dir,
        );
    }

    if let Some(notes_area) = notes_area {
        render_speaker_notes(frame, notes_area, node, theme);
    }

    // Render branch overlay for branch nodes
    if view_state.transition.is_none() && node.branch_point().is_some() {
        render_branch_overlay(
            frame,
            area,
            node,
            &session.graph,
            theme,
            view_state.branch_focused_option,
        );
    }

    // Render progress bar
    if view_state.show_progress_bar {
        render_progress_bar(
            frame,
            areas.footer,
            session,
            view_state.elapsed_secs,
            view_state.show_elapsed_timer,
            view_state.target_duration_secs,
            theme,
        );
    }

    if view_state.show_chrome {
        let breadcrumb_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        render_breadcrumb(
            frame,
            breadcrumb_area,
            session,
            view_state.nav_path,
            current,
            theme,
        );
    }

    let mut timeline_rendered = false;
    if view_state.show_chrome && view_state.show_timeline {
        let timeline_y = if view_state.show_progress_bar {
            areas.footer.y.saturating_sub(1)
        } else {
            area.y.saturating_add(area.height.saturating_sub(2))
        };
        if timeline_y > area.y {
            let timeline_area = Rect {
                x: area.x,
                y: timeline_y,
                width: area.width,
                height: 1,
            };
            render_timeline(
                frame,
                timeline_area,
                session,
                view_state.visited_nodes,
                current,
                theme,
            );
            timeline_rendered = true;
        }
    }

    let mut banner_y = if view_state.show_progress_bar {
        if timeline_rendered {
            areas.footer.y.saturating_sub(2)
        } else {
            areas.footer.y.saturating_sub(1)
        }
    } else {
        let mut y = area.y.saturating_add(area.height.saturating_sub(1));
        if timeline_rendered {
            y = y.saturating_sub(1);
        }
        y
    };

    if view_state.pending_exit_confirmation {
        let banner = Rect {
            x: area.x,
            y: banner_y,
            width: area.width,
            height: 1,
        };
        render_quit_confirmation_banner(frame, banner, theme);
        banner_y = banner_y.saturating_sub(1);
    }

    if let Some((text, kind)) = view_state.flash_message {
        let banner = Rect {
            x: area.x,
            y: banner_y,
            width: area.width,
            height: 1,
        };
        render_flash_message(frame, banner, text, kind, theme);
    }

    // Render help overlay if active
    if view_state.show_help {
        render_help_overlay(
            frame,
            area,
            theme,
            HelpMode::Presenting,
            view_state.help_scroll_offset,
        );
    }

    if view_state.show_chrome {
        render_mode_badge(
            frame,
            area,
            if view_state.goto_buffer.is_some() {
                ModeBadgeKind::GotoNode
            } else if node.branch_point().is_some() {
                ModeBadgeKind::Branch
            } else {
                ModeBadgeKind::Presenting
            },
            theme,
        );

        // Render GOTO mode badge (gold border, top-right corner)
        if let Some(buffer) = view_state.goto_buffer {
            render_goto_badge(frame, area, buffer, theme);
            render_goto_autocomplete(frame, area, areas.footer, buffer, session, theme);
        }
    }
}

fn render_goto_badge(frame: &mut Frame, area: Rect, buffer: &str, theme: &Theme) {
    // Small badge in the top-right: `GOTO: <buffer>_`
    // Gold border (heading_h3), surface bg.
    // Shows "type number or ID prefix" hint when the buffer is empty.
    let hint = if buffer.is_empty() {
        " type number or ID prefix ".to_string()
    } else {
        format!(" GOTO: {buffer}_ ")
    };
    let badge_width = (hint.chars().count() as u16 + 2).min(area.width);
    let badge_height = 3u16;
    if area.width < badge_width || area.height < badge_height {
        return;
    }
    let badge = Rect {
        x: area.x + area.width - badge_width,
        y: area.y.saturating_add(3),
        width: badge_width,
        height: badge_height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.heading_h3))
        .style(Style::default().bg(theme.surface));
    let inner = block.inner(badge);
    frame.render_widget(block, badge);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            &hint,
            Style::default()
                .fg(theme.heading_h3)
                .add_modifier(Modifier::BOLD),
        ))),
        inner,
    );
}

fn render_goto_autocomplete(
    frame: &mut Frame,
    area: Rect,
    footer_area: Rect,
    buffer: &str,
    session: &PresentationSession,
    theme: &Theme,
) {
    if buffer.is_empty() {
        return;
    }

    let matches = goto_matches(session, buffer, 5);
    if matches.is_empty() {
        return;
    }

    let max_rows: usize = if area.height <= 24 { 3 } else { 6 };
    let visible = matches.len().min(max_rows.saturating_sub(1));
    if visible == 0 {
        return;
    }

    let height = (visible as u16).saturating_add(1);
    let y = footer_area.y.saturating_sub(height);
    if y <= area.y {
        return;
    }

    let panel = Rect {
        x: area.x,
        y,
        width: area.width,
        height,
    };

    let block = Block::default()
        .title(" GOTO MATCHES ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.heading_h3))
        .style(Style::default().bg(theme.surface));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let lines = matches
        .iter()
        .take(visible)
        .enumerate()
        .map(|(row, (idx, id, title))| {
            let style = if row == 0 {
                Style::default()
                    .fg(theme.heading_h1)
                    .bg(theme.background)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };
            let title_text = title.as_deref().unwrap_or("(untitled)");
            Line::from(Span::styled(
                format!(
                    " {:>2} │ {:<16} │ {}",
                    idx + 1,
                    id,
                    truncate_line(title_text, 40)
                ),
                style,
            ))
        })
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

/// Collect up to `limit` nodes whose IDs start with `buffer` (case-insensitive),
/// OR — when `buffer` is all-digits — the node at the corresponding 1-based index.
fn goto_matches<'a>(
    session: &'a PresentationSession,
    buffer: &str,
    limit: usize,
) -> Vec<(usize, &'a str, Option<String>)> {
    // Numeric: return the node at the 1-based index as the sole result.
    if let Ok(num) = buffer.parse::<usize>() {
        let idx = num.saturating_sub(1);
        if let Some(node) = session.graph.nodes.get(idx) {
            let id = node.id.as_deref().unwrap_or("(no id)");
            let title = node.title.clone();
            return vec![(idx, id, title)];
        }
        return vec![];
    }

    // Text prefix: match node IDs.
    let prefix = buffer.to_ascii_lowercase();
    session
        .graph
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(idx, node)| {
            let id = node.id.as_deref()?;
            if !id.to_ascii_lowercase().starts_with(&prefix) {
                return None;
            }
            let title = node
                .title
                .clone()
                .or_else(|| {
                    node.content.iter().find_map(|block| {
                        if let fireside_core::model::content::ContentBlock::Heading {
                            text, ..
                        } = block
                        {
                            Some(text.clone())
                        } else {
                            None
                        }
                    })
                })
                .or_else(|| Some(id.to_string()));
            Some((idx, id, title))
        })
        .take(limit)
        .collect()
}

fn truncate_line(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let clipped: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{clipped}…")
}

fn render_transition_node(
    frame: &mut Frame,
    session: &PresentationSession,
    theme: &Theme,
    area: Rect,
    layout: Layout,
    content_base_dir: Option<&Path>,
    transition: PresenterTransition,
) {
    let to_index = session.current_node_index();
    let Some(from_node) = session.graph.nodes.get(transition.from_index) else {
        let to_node = &session.graph.nodes[to_index];
        render_node(frame, to_node, theme, area, layout, content_base_dir);
        return;
    };
    let to_node = &session.graph.nodes[to_index];

    if transition.from_index == to_index {
        render_node(frame, to_node, theme, area, layout, content_base_dir);
        return;
    }

    if matches!(layout, Layout::SplitHorizontal | Layout::SplitVertical) {
        render_node(frame, to_node, theme, area, layout, content_base_dir);
        return;
    }

    let width = area.width.max(1) as usize;
    let from_lines = render_node_content_with_base(
        &from_node.content,
        theme,
        area.width.max(1),
        content_base_dir,
    );
    let to_lines =
        render_node_content_with_base(&to_node.content, theme, area.width.max(1), content_base_dir);
    let lines = transition_lines(
        &from_lines,
        &to_lines,
        width,
        transition.kind,
        transition.progress.clamp(0.0, 1.0),
        theme,
    );

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(theme.foreground));

    frame.render_widget(paragraph, area);
}

/// Render a single node's content into the given area.
fn render_node(
    frame: &mut Frame,
    node: &Node,
    theme: &Theme,
    area: Rect,
    layout: Layout,
    content_base_dir: Option<&Path>,
) {
    match layout {
        Layout::SplitHorizontal => {
            render_split_horizontal_node(frame, node, theme, area, content_base_dir)
        }
        Layout::SplitVertical => {
            render_split_vertical_node(frame, node, theme, area, content_base_dir)
        }
        _ => render_single_node(frame, node, theme, area, content_base_dir),
    }
}

fn render_single_node(
    frame: &mut Frame,
    node: &Node,
    theme: &Theme,
    area: Rect,
    content_base_dir: Option<&Path>,
) {
    let lines =
        render_node_content_with_base(&node.content, theme, area.width.max(1), content_base_dir);

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(theme.foreground));

    frame.render_widget(paragraph, area);
}

fn render_split_horizontal_node(
    frame: &mut Frame,
    node: &Node,
    theme: &Theme,
    area: Rect,
    content_base_dir: Option<&Path>,
) {
    let (left, right) = two_column_split(area);
    let mid = node.content.len().div_ceil(2);
    let left_lines = render_node_content_with_base(
        &node.content[..mid],
        theme,
        left.width.max(1),
        content_base_dir,
    );
    let right_lines = render_node_content_with_base(
        &node.content[mid..],
        theme,
        right.width.max(1),
        content_base_dir,
    );

    let left_paragraph = Paragraph::new(left_lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(theme.foreground));
    let right_paragraph = Paragraph::new(right_lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(theme.foreground));

    frame.render_widget(left_paragraph, left);
    frame.render_widget(right_paragraph, right);
}

fn render_split_vertical_node(
    frame: &mut Frame,
    node: &Node,
    theme: &Theme,
    area: Rect,
    content_base_dir: Option<&Path>,
) {
    let chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let mid = node.content.len().div_ceil(2);
    let top_lines = render_node_content_with_base(
        &node.content[..mid],
        theme,
        chunks[0].width.max(1),
        content_base_dir,
    );
    let bottom_lines = render_node_content_with_base(
        &node.content[mid..],
        theme,
        chunks[1].width.max(1),
        content_base_dir,
    );

    let top_paragraph = Paragraph::new(top_lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(theme.foreground));
    let bottom_paragraph = Paragraph::new(bottom_lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(theme.foreground));

    frame.render_widget(top_paragraph, chunks[0]);
    frame.render_widget(bottom_paragraph, chunks[1]);
}

fn render_speaker_notes(frame: &mut Frame, area: Rect, node: &Node, theme: &Theme) {
    let Some(notes) = &node.speaker_notes else {
        return;
    };

    let block = Block::default()
        .title(" Speaker notes ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.heading_h3));

    let paragraph = Paragraph::new(notes.as_str())
        .block(block)
        .style(Style::default().fg(theme.foreground))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}
