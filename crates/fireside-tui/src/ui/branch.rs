//! Branch point overlay UI.
//!
//! Renders an interactive chooser panel when the current node contains a
//! branch point. The panel is keyboard-first (`a`-`z`) and mouse-compatible.

use fireside_core::model::content::ContentBlock;
use fireside_core::model::graph::Graph;
use fireside_core::model::node::Node;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::theme::Theme;

/// Compute the popup area used for branch point selection.
#[must_use]
pub fn branch_overlay_rect(area: Rect, option_count: u16) -> Rect {
    let min_height = 7u16;
    let content_height = option_count.saturating_add(4);
    let popup_height = content_height
        .max(min_height)
        .min(area.height.saturating_sub(2));
    let popup_width = if area.width <= 80 { 92 } else { 70 };

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(area.height.saturating_sub(popup_height) / 2),
            Constraint::Length(popup_height),
            Constraint::Min(0),
        ])
        .split(area);

    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - popup_width) / 2),
            Constraint::Percentage(popup_width),
            Constraint::Min(0),
        ])
        .split(vert[1]);

    horiz[1]
}

/// Render branch point chooser overlay for the current node.
pub fn render_branch_overlay(
    frame: &mut Frame,
    area: Rect,
    node: &Node,
    graph: &Graph,
    theme: &Theme,
) {
    let Some(branch_point) = node.branch_point() else {
        return;
    };

    let popup = branch_overlay_rect(area, branch_point.options.len() as u16);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Branch Point ")
        .borders(Borders::ALL)
        .style(Style::default().bg(theme.background))
        .border_style(Style::default().fg(theme.heading_h2));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let key_style = Style::default()
        .fg(theme.heading_h1)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(theme.foreground);
    let preview_style = Style::default()
        .fg(theme.footer)
        .add_modifier(Modifier::ITALIC);

    let mut lines = Vec::new();

    if let Some(prompt) = &branch_point.prompt {
        lines.push(Line::from(Span::styled(
            format!(" {prompt}"),
            Style::default()
                .fg(theme.heading_h3)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::default());
    }

    for option in &branch_point.options {
        let preview = option_preview(graph, &option.target);
        lines.push(Line::from(vec![
            Span::styled(format!(" [{}] ", option.key), key_style),
            Span::styled(option.label.as_str(), label_style),
            Span::styled(format!(" — {preview}"), preview_style),
        ]));
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

fn option_preview(graph: &Graph, target: &str) -> String {
    let Some(idx) = graph.index_of(target) else {
        return String::from("(missing target)");
    };

    let target_node = &graph.nodes[idx];
    if let Some(id) = &target_node.id {
        if let Some(text) = first_block_text(target_node) {
            return format!("{id}: {text}");
        }
        return id.clone();
    }

    first_block_text(target_node).unwrap_or_else(|| String::from("(empty node)"))
}

fn first_block_text(node: &Node) -> Option<String> {
    let block = node.content.first()?;
    let text = match block {
        ContentBlock::Heading { text, .. } => text.clone(),
        ContentBlock::Text { body } => body.clone(),
        ContentBlock::Code { source, .. } => source.lines().next().unwrap_or_default().to_string(),
        ContentBlock::List { items, .. } => items
            .first()
            .map(|item| item.text.clone())
            .unwrap_or_else(|| String::from("list")),
        ContentBlock::Image { caption, alt, src } => caption
            .clone()
            .or_else(|| (!alt.is_empty()).then(|| alt.clone()))
            .unwrap_or_else(|| src.clone()),
        ContentBlock::Divider => String::from("divider"),
        ContentBlock::Container { .. } => String::from("container"),
        ContentBlock::Extension { extension_type, .. } => extension_type.clone(),
    };

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Some(String::from("(empty)"));
    }

    let preview = if trimmed.chars().count() > 40 {
        format!("{}…", trimmed.chars().take(40).collect::<String>())
    } else {
        trimmed.to_string()
    };

    Some(preview)
}
