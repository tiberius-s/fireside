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
use crate::render::layout::two_column_split;
use crate::render::markdown::render_node_content_with_base;
use crate::theme::Theme;
use std::path::Path;

use super::branch::render_branch_overlay;
use super::help::{HelpMode, render_help_overlay};
use super::progress::render_progress_bar;

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
    pub content_base_dir: Option<&'a Path>,
    pub transition: Option<PresenterTransition>,
    pub elapsed_secs: u64,
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

    let content_area = areas.main;
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
        render_branch_overlay(frame, area, node, &session.graph, theme);
    }

    // Render progress bar
    if view_state.show_progress_bar {
        render_progress_bar(
            frame,
            areas.footer,
            session,
            view_state.elapsed_secs,
            view_state.show_elapsed_timer,
            theme,
        );
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

fn transition_lines(
    from_lines: &[Line<'_>],
    to_lines: &[Line<'_>],
    width: usize,
    kind: Transition,
    progress: f32,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let rows = from_lines.len().max(to_lines.len());
    let mut output = Vec::with_capacity(rows);
    let reveal = (progress * width as f32).floor() as usize;

    for row in 0..rows {
        let from_text = from_lines.get(row).map_or_else(String::new, line_to_text);
        let to_text = to_lines.get(row).map_or_else(String::new, line_to_text);
        let line = match kind {
            Transition::None => clip_pad(&to_text, width),
            Transition::Fade => {
                if progress < 0.5 {
                    clip_pad(&from_text, width)
                } else {
                    clip_pad(&to_text, width)
                }
            }
            Transition::SlideLeft => {
                let shift = ((1.0 - progress) * width as f32).floor() as usize;
                clip_pad(&format!("{}{}", " ".repeat(shift), to_text), width)
            }
            Transition::SlideRight => {
                let shift = ((1.0 - progress) * width as f32).floor() as usize;
                let padded = format!("{}{}", " ".repeat(width), to_text);
                let start = width.saturating_sub(shift).min(padded.chars().count());
                clip_pad(&padded.chars().skip(start).collect::<String>(), width)
            }
            Transition::Wipe => {
                let visible = take_chars(&to_text, reveal.min(width));
                clip_pad(&visible, width)
            }
            Transition::Dissolve => {
                let mut chars = Vec::with_capacity(width);
                let to_chars = to_text.chars().collect::<Vec<_>>();
                for col in 0..width {
                    let next = to_chars.get(col).copied().unwrap_or(' ');
                    let hash = pseudo_rand(row as u32, col as u32, 7) as f32 / u32::MAX as f32;
                    chars.push(if hash <= progress { next } else { ' ' });
                }
                chars.into_iter().collect::<String>()
            }
            Transition::Matrix => {
                let mut chars = Vec::with_capacity(width);
                let to_chars = to_text.chars().collect::<Vec<_>>();
                let matrix_chars = ['░', '▒', '▓'];
                for col in 0..width {
                    let next = to_chars.get(col).copied().unwrap_or(' ');
                    let hash = pseudo_rand(row as u32, col as u32, 31) as f32 / u32::MAX as f32;
                    if hash <= progress {
                        chars.push(next);
                    } else {
                        let idx = (pseudo_rand(row as u32, col as u32, 13) % 3) as usize;
                        chars.push(matrix_chars[idx]);
                    }
                }
                chars.into_iter().collect::<String>()
            }
            Transition::Typewriter => {
                let visible = take_chars(&to_text, reveal.min(width));
                clip_pad(&visible, width)
            }
        };

        let mut style = Style::default().fg(theme.foreground);
        if matches!(kind, Transition::Fade) && progress < 0.5 {
            style = style.add_modifier(Modifier::DIM);
        }
        if matches!(kind, Transition::Matrix) {
            style = Style::default().fg(theme.heading_h2);
        }

        output.push(Line::from(Span::styled(line, style)));
    }

    output
}

fn line_to_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<Vec<_>>()
        .join("")
}

fn clip_pad(text: &str, width: usize) -> String {
    let clipped = take_chars(text, width);
    let pad = width.saturating_sub(clipped.chars().count());
    format!("{clipped}{}", " ".repeat(pad))
}

fn take_chars(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

fn pseudo_rand(row: u32, col: u32, salt: u32) -> u32 {
    let mut value = row
        .wrapping_mul(374_761_393)
        .wrapping_add(col.wrapping_mul(668_265_263))
        .wrapping_add(salt.wrapping_mul(2_147_483_647));
    value ^= value >> 13;
    value = value.wrapping_mul(1_274_126_177);
    value ^ (value >> 16)
}
