//! Graph view overlay for editor mode.

use fireside_core::model::node::Node;
use fireside_engine::PresentationSession;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy)]
pub struct GraphOverlayViewState {
    pub selected_index: usize,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct GraphOverlayWindow {
    pub start: usize,
    pub end: usize,
}

pub fn render_graph_overlay(
    frame: &mut Frame,
    area: Rect,
    session: &PresentationSession,
    theme: &Theme,
    view_state: GraphOverlayViewState,
) {
    let popup = centered_popup(area, 74, 76);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Graph View ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.heading_h2))
        .style(Style::default().bg(theme.background));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(inner);

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(78), Constraint::Percentage(22)])
        .split(body[0]);

    let total = session.graph.nodes.len();
    if total == 0 {
        frame.render_widget(
            Paragraph::new("No nodes to render")
                .style(Style::default().fg(theme.footer))
                .wrap(Wrap { trim: true }),
            body[0],
        );
        return;
    }

    let selected = view_state.selected_index.min(total.saturating_sub(1));
    let window = graph_overlay_window(area, session, selected, view_state.scroll_offset);
    let start = window.start;
    let end = window.end;

    let items = (start..end)
        .map(|idx| ListItem::new(graph_item_lines(session, idx)))
        .collect::<Vec<_>>();

    let mut state = ListState::default();
    state.select(Some(selected.saturating_sub(start)));

    let list = List::new(items)
        .highlight_symbol("▶ ")
        .highlight_style(
            Style::default()
                .fg(theme.heading_h1)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .title(" Topology ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.code_border)),
        );

    frame.render_stateful_widget(list, panes[0], &mut state);

    let minimap = Paragraph::new(minimap_lines(
        total,
        start,
        end,
        selected,
        session.current_node_index(),
    ))
    .block(
        Block::default()
            .title(" Mini-map ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.code_border)),
    )
    .style(Style::default().fg(theme.footer))
    .wrap(Wrap { trim: true });

    frame.render_widget(minimap, panes[1]);

    let legend = Paragraph::new(Line::from(vec![
        Span::styled("j/k/↑/↓", Style::default().fg(theme.heading_h2)),
        Span::styled(" move  ", Style::default().fg(theme.foreground)),
        Span::styled("PgUp/PgDn", Style::default().fg(theme.heading_h2)),
        Span::styled(" page  ", Style::default().fg(theme.foreground)),
        Span::styled("Home/End", Style::default().fg(theme.heading_h2)),
        Span::styled(" bounds  ", Style::default().fg(theme.foreground)),
        Span::styled("Enter", Style::default().fg(theme.heading_h2)),
        Span::styled(" jump  ", Style::default().fg(theme.foreground)),
        Span::styled("P", Style::default().fg(theme.heading_h2)),
        Span::styled(" present  ", Style::default().fg(theme.foreground)),
        Span::styled("Esc/v", Style::default().fg(theme.heading_h2)),
        Span::styled(" close", Style::default().fg(theme.foreground)),
    ]));

    frame.render_widget(legend, body[1]);
}

fn graph_item_lines(session: &PresentationSession, index: usize) -> Vec<Line<'static>> {
    let current = session.current_node_index();
    let Some(node) = session.graph.nodes.get(index) else {
        return vec![Line::from("(missing node)")];
    };

    let marker = if index == current { "*" } else { " " };
    let id = node.id.as_deref().unwrap_or("(no-id)");
    let rail = if index + 1 < session.graph.nodes.len() {
        "│"
    } else {
        "└"
    };
    let mut lines = vec![Line::from(format!(
        "{marker} {rail} ┌[{:>2}] {id}┐",
        index + 1
    ))];

    let edge_lines = summarize_edge_lines(session, node, index);
    if !edge_lines.is_empty() {
        let max_kind_len = edge_lines
            .iter()
            .map(|edge| edge.kind.len())
            .max()
            .unwrap_or(0);
        let max_target_len = edge_lines
            .iter()
            .map(|edge| edge.target_label.len())
            .max()
            .unwrap_or(0);

        for (edge_idx, edge) in edge_lines.iter().enumerate() {
            let branch_connector = if edge_idx + 1 == edge_lines.len() {
                "└"
            } else {
                "├"
            };

            let kind = format!("{:width$}", edge.kind, width = max_kind_len);
            let target = format!("{:width$}", edge.target_label, width = max_target_len);
            lines.push(Line::from(format!(
                "  {rail}  {branch_connector}╼ {kind} → {target}"
            )));
        }
    }

    lines
}

#[derive(Debug, Clone)]
struct EdgeLine {
    kind: String,
    target_label: String,
}

fn summarize_edge_lines(session: &PresentationSession, node: &Node, index: usize) -> Vec<EdgeLine> {
    let mut edges = Vec::new();

    if let Some(target) = node.next_override() {
        if let Some(idx) = session.graph.index_of(target) {
            edges.push(EdgeLine {
                kind: "next".to_string(),
                target_label: format!("#{}", idx + 1),
            });
        }
    } else if index + 1 < session.graph.nodes.len() {
        edges.push(EdgeLine {
            kind: "next".to_string(),
            target_label: format!("#{}", index + 2),
        });
    }

    if let Some(target) = node.after_target()
        && let Some(idx) = session.graph.index_of(target)
    {
        edges.push(EdgeLine {
            kind: "after".to_string(),
            target_label: format!("#{}", idx + 1),
        });
    }

    if let Some(branch) = node.branch_point() {
        for option in &branch.options {
            if let Some(idx) = session.graph.index_of(&option.target) {
                edges.push(EdgeLine {
                    kind: format!("branch [{}]", option.key),
                    target_label: format!("#{}", idx + 1),
                });
            }
        }
    }

    edges
}

pub fn graph_overlay_rect(area: Rect) -> Rect {
    centered_popup(area, 74, 76)
}

pub fn graph_overlay_list_panel_rect(area: Rect) -> Rect {
    let popup = graph_overlay_rect(area);
    let inner = Rect {
        x: popup.x.saturating_add(1),
        y: popup.y.saturating_add(1),
        width: popup.width.saturating_sub(2),
        height: popup.height.saturating_sub(2),
    };

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(inner);

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(78), Constraint::Percentage(22)])
        .split(body[0]);

    panes[0]
}

pub fn graph_overlay_window(
    area: Rect,
    session: &PresentationSession,
    selected: usize,
    scroll_offset: usize,
) -> GraphOverlayWindow {
    let total = session.graph.nodes.len();
    if total == 0 {
        return GraphOverlayWindow { start: 0, end: 0 };
    }

    let list_panel = graph_overlay_list_panel_rect(area);
    let row_capacity = list_panel.height.saturating_sub(2).max(1);

    let mut start = scroll_offset.min(total.saturating_sub(1));
    if selected < start {
        start = selected;
    }

    let mut end = compute_window_end(session, start, row_capacity);
    if selected >= end {
        start = selected;
        end = compute_window_end(session, start, row_capacity);
    }

    GraphOverlayWindow { start, end }
}

pub fn graph_overlay_page_span(
    area: Rect,
    session: &PresentationSession,
    selected: usize,
    scroll_offset: usize,
) -> usize {
    let window = graph_overlay_window(area, session, selected, scroll_offset);
    window.end.saturating_sub(window.start).max(1)
}

pub fn graph_overlay_row_to_node(
    area: Rect,
    session: &PresentationSession,
    selected: usize,
    scroll_offset: usize,
    row: u16,
) -> Option<usize> {
    let total = session.graph.nodes.len();
    if total == 0 {
        return None;
    }

    let list_panel = graph_overlay_list_panel_rect(area);
    let content_y = list_panel.y.saturating_add(1);
    let content_bottom = list_panel
        .y
        .saturating_add(list_panel.height.saturating_sub(1));
    if row < content_y || row >= content_bottom {
        return None;
    }

    let window = graph_overlay_window(area, session, selected, scroll_offset);
    let mut cursor = content_y;
    for idx in window.start..window.end {
        let height = graph_item_height(session, idx);
        let end = cursor.saturating_add(height);
        if row >= cursor && row < end {
            return Some(idx);
        }
        cursor = end;
    }

    None
}

fn compute_window_end(session: &PresentationSession, start: usize, row_capacity: u16) -> usize {
    let mut used = 0u16;
    let mut end = start;

    while end < session.graph.nodes.len() {
        let height = graph_item_height(session, end);
        if end > start && used.saturating_add(height) > row_capacity {
            break;
        }
        used = used.saturating_add(height);
        end += 1;
    }

    end.max(start + 1).min(session.graph.nodes.len())
}

fn graph_item_height(session: &PresentationSession, index: usize) -> u16 {
    let Some(node) = session.graph.nodes.get(index) else {
        return 1;
    };

    let edge_count = summarize_edge_lines(session, node, index).len() as u16;
    1 + edge_count
}

fn minimap_lines(
    total: usize,
    start: usize,
    end: usize,
    selected: usize,
    current: usize,
) -> Vec<Line<'static>> {
    if total == 0 {
        return vec![Line::from("No nodes")];
    }

    let mut lines = vec![
        Line::from(format!("total: {total}")),
        Line::from(format!("view: {}-{}", start + 1, end.max(start + 1))),
        Line::from(format!("sel:  #{}", selected + 1)),
        Line::from(format!("cur:  #{}", current + 1)),
        Line::from(""),
    ];

    let bar_rows = 8usize;
    for row in 0..bar_rows {
        let idx = ((row * total) / bar_rows).min(total.saturating_sub(1));
        let in_window = idx >= start && idx < end;
        let marker = if idx == selected {
            "●"
        } else if idx == current {
            "◆"
        } else if in_window {
            "█"
        } else {
            "│"
        };
        lines.push(Line::from(format!("{marker} #{:>2}", idx + 1)));
    }

    lines
}

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
