//! Presenter view — composes the node content with chrome.
//!
//! This is the main rendering component that draws the current node
//! within the layout areas, overlaying the progress bar and optional help.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph, Wrap};

use fireside_core::model::layout::Layout;
use fireside_core::model::node::Node;
use fireside_engine::PresentationSession;

use crate::render::layout::compute_areas;
use crate::render::markdown::render_node_content;
use crate::theme::Theme;

use super::help::render_help_overlay;
use super::progress::render_progress_bar;

/// Render the full presenter view for the current node.
///
/// Draws the node content in the main area, the progress bar in the footer,
/// and optionally the help overlay on top.
pub fn render_presenter(
    frame: &mut Frame,
    session: &PresentationSession,
    theme: &Theme,
    show_help: bool,
    elapsed_secs: u64,
) {
    let area = frame.area();

    // Determine layout for current node
    let current = session.current_node_index();
    let node = &session.graph.nodes[current];
    let layout = node.layout.unwrap_or(Layout::Default);

    let areas = compute_areas(area, layout);

    // Clear background
    let bg_style = Style::default().bg(theme.background);
    let bg_block = Block::default().style(bg_style);
    frame.render_widget(bg_block, area);

    // Render node content
    render_node(frame, node, theme, areas.content);

    // Render progress bar
    render_progress_bar(
        frame,
        areas.footer,
        current,
        session.graph.nodes.len(),
        elapsed_secs,
        theme,
    );

    // Render help overlay if active
    if show_help {
        render_help_overlay(frame, area, theme);
    }
}

/// Render a single node's content into the given area.
fn render_node(frame: &mut Frame, node: &Node, theme: &Theme, area: Rect) {
    let lines = render_node_content(&node.content, theme, area.width);

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(theme.foreground));

    frame.render_widget(paragraph, area);
}
