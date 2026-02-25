//! Branch point overlay UI.
//!
//! Renders an interactive chooser dialog when the current node contains a
//! branch point. The overlay dims the presenter content behind it and shows
//! a centred dialog with labelled key‑badge rows for each option.
//!
//! Visual design matches the Penpot "05 — Branch Point Overlay" frame:
//! - Full‑screen toolbar‑bg dim layer
//! - Centred surface dialog with `border_active` border
//! - Title bar in `heading_h1`, prompt in `on_surface`, options as rows
//! - Footer hint line listing all keys + `[Esc] cancel`

use fireside_core::model::content::ContentBlock;
use fireside_core::model::graph::Graph;
use fireside_core::model::node::Node;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::theme::Theme;

/// Compute the popup (dialog) area for branch point selection.
#[must_use]
pub fn branch_overlay_rect(area: Rect, option_count: u16) -> Rect {
    // title(1) + blank(1) + prompt(1) + blank(1) + separator(1)
    // + options + blank(1) + footer(1) + borders(2)
    let content_rows = option_count.saturating_add(8);
    let popup_height = content_rows.max(10).min(area.height.saturating_sub(4));

    // Width: prefer 60 % of screen, min 50, max 90
    let popup_width_pct: u16 = if area.width >= 120 { 55 } else { 75 };

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
            Constraint::Percentage((100 - popup_width_pct) / 2),
            Constraint::Percentage(popup_width_pct),
            Constraint::Min(0),
        ])
        .split(vert[1]);

    horiz[1]
}

/// Render branch point chooser overlay for the current node.
///
/// The overlay covers the full `area` with a dim background block, then
/// places a centred dialog on top.
pub fn render_branch_overlay(
    frame: &mut Frame,
    area: Rect,
    node: &Node,
    graph: &Graph,
    theme: &Theme,
    focused_option: usize,
) {
    let Some(branch_point) = node.branch_point() else {
        return;
    };

    // ── Dim the full background ───────────────────────────────────────────
    let dim_block = Block::default().style(Style::default().bg(theme.toolbar_bg));
    frame.render_widget(dim_block, area);

    // ── Popup dialog ──────────────────────────────────────────────────────
    let popup = branch_overlay_rect(area, branch_point.options.len() as u16);
    frame.render_widget(Clear, popup);

    // Dialog container: surface bg + active blue border
    let dialog_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_active))
        .style(Style::default().bg(theme.surface));
    let inner = dialog_block.inner(popup);
    frame.render_widget(dialog_block, popup);

    // ── Dialog title bar ──────────────────────────────────────────────────
    // Use the node's first heading content as context, or fall back to "Branch Point".
    let title_context = node_heading_text(node).unwrap_or_else(|| "Branch Point".to_owned());
    let title_str = format!(" Branch Point: {title_context} ");

    // Split inner area vertically into: title(1) | body(rest) | footer(1)
    let [title_area, body_area, footer_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .areas(inner);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            &title_str,
            Style::default()
                .fg(theme.heading_h1)
                .add_modifier(Modifier::BOLD),
        )))
        .style(Style::default().bg(theme.toolbar_bg)),
        title_area,
    );

    // ── Option rows ───────────────────────────────────────────────────────
    let mut lines: Vec<Line> = Vec::new();

    // Prompt (bold, on_surface)
    if let Some(prompt) = &branch_point.prompt {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            format!("  {prompt}"),
            Style::default()
                .fg(theme.on_surface)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  Press the key for your choice, or [Esc] to cancel.",
            Style::default().fg(theme.footer),
        )));
        lines.push(Line::default());
    }

    // Render each option
    for (idx, option) in branch_point.options.iter().enumerate() {
        let is_focused = idx == focused_option.min(branch_point.options.len().saturating_sub(1));
        let description = option_preview(graph, &option.target);

        let (badge_fg, badge_bg) = if is_focused {
            (theme.toolbar_bg, theme.heading_h1)
        } else {
            // Unfocused keys need to remain legible — use foreground on surface
            // instead of the dim footer colour so options are scannable.
            (theme.foreground, theme.surface)
        };

        let label_style = Style::default()
            .fg(theme.on_surface)
            .add_modifier(Modifier::BOLD);

        lines.push(Line::from(vec![
            Span::styled(
                if is_focused { "▌ " } else { "  " },
                Style::default().fg(theme.border_active).bg(theme.surface),
            ),
            Span::styled(
                format!(" {} ", option.key),
                Style::default().fg(badge_fg).bg(badge_bg),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                option.label.as_str(),
                if is_focused {
                    label_style
                        .fg(theme.heading_h2)
                        .add_modifier(Modifier::UNDERLINED)
                } else {
                    label_style
                },
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!("       {}", truncate(&description, 70)),
            Style::default().fg(theme.footer),
        )));
        if idx + 1 < branch_point.options.len() {
            lines.push(Line::from(Span::styled(
                "  ─────────────────────────────────────────",
                Style::default().fg(theme.border_inactive),
            )));
        }
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), body_area);

    // ── Footer: all key hints ─────────────────────────────────────────────
    let mut footer_parts: Vec<String> = branch_point
        .options
        .iter()
        .map(|o| format!("[{}] {}", o.key, short_label(&o.label)))
        .collect();
    footer_parts.push("[Esc] cancel".to_owned());

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("  {}", footer_parts.join("  ")),
            Style::default().fg(theme.footer),
        ))),
        footer_area,
    );
}

/// Produce a short label (first word, max 8 chars) for use in the footer.
fn short_label(label: &str) -> String {
    let first_word = label.split_whitespace().next().unwrap_or(label);
    if first_word.chars().count() <= 10 {
        first_word.to_owned()
    } else {
        let s: String = first_word.chars().take(9).collect();
        format!("{s}…")
    }
}

/// Returns the text of the first heading content block in a node, if any.
fn node_heading_text(node: &Node) -> Option<String> {
    node.content.iter().find_map(|block| {
        if let ContentBlock::Heading { text, .. } = block {
            Some(text.clone())
        } else {
            None
        }
    })
}

/// Truncate a string to at most `max_chars` characters, appending `…` if cut.
fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let s: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{s}…")
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
