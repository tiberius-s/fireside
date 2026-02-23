//! Graph view overlay for editor mode.

use std::collections::HashSet;

#[cfg(test)]
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

#[derive(Debug, Clone)]
struct TreeRow {
    node_index: usize,
    depth: usize,
    incoming_kind: Option<EdgeKind>,
    incoming_label: Option<String>,
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

    let rows = graph_tree_rows(session);
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new("No topology to render")
                .style(Style::default().fg(theme.footer))
                .wrap(Wrap { trim: true }),
            body[0],
        );
        return;
    }

    let selected = view_state.selected_index.min(total.saturating_sub(1));
    let selected_row = rows
        .iter()
        .position(|row| row.node_index == selected)
        .unwrap_or(0);
    let window = graph_overlay_window(area, session, selected, view_state.scroll_offset);
    let start = window.start;
    let end = window.end;

    let items = (start..end)
        .map(|row_index| {
            let row = &rows[row_index];
            ListItem::new(Line::from(render_tree_row(session, row, theme)))
        })
        .collect::<Vec<_>>();

    let mut state = ListState::default();
    state.select(Some(selected_row.saturating_sub(start)));

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
        selected_row,
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
        Span::styled("next", Style::default().fg(theme.border_active)),
        Span::styled("/", Style::default().fg(theme.footer)),
        Span::styled("branch", Style::default().fg(theme.heading_h3)),
        Span::styled("/", Style::default().fg(theme.footer)),
        Span::styled("after", Style::default().fg(theme.accent)),
        Span::styled("/", Style::default().fg(theme.footer)),
        Span::styled("goto", Style::default().fg(theme.error)),
        Span::styled("  ·  ", Style::default().fg(theme.footer)),
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

fn render_tree_row(
    session: &PresentationSession,
    row: &TreeRow,
    theme: &Theme,
) -> Vec<Span<'static>> {
    let current = session.current_node_index();
    let Some(node) = session.graph.nodes.get(row.node_index) else {
        return vec![Span::raw("(missing node)")];
    };

    let is_current = row.node_index == current;
    let is_branch = node.branch_point().is_some();
    let marker = if is_current { "●" } else { "○" };
    let row_style = if is_current {
        Style::default()
            .fg(theme.border_active)
            .add_modifier(Modifier::BOLD)
    } else if is_branch {
        Style::default().fg(theme.heading_h3)
    } else {
        Style::default().fg(theme.foreground)
    };

    let mut spans = Vec::new();
    spans.push(Span::raw("  "));
    if row.depth > 0 {
        spans.push(Span::styled(
            format!("{}", "  ".repeat(row.depth.saturating_sub(1))),
            Style::default().fg(theme.footer),
        ));
        spans.push(Span::styled("└─", Style::default().fg(theme.footer)));
    }

    if let Some(kind) = row.incoming_kind {
        spans.push(Span::styled(
            match kind {
                EdgeKind::Next => "→ ",
                EdgeKind::After => "↷ ",
                EdgeKind::Branch => "⎇ ",
                EdgeKind::Goto => "↪ ",
            },
            Style::default().fg(kind.color(theme)),
        ));
    }

    if let Some(label) = row.incoming_label.as_deref() {
        spans.push(Span::styled(
            format!("[{label}] "),
            Style::default().fg(theme.footer),
        ));
    }

    spans.push(Span::styled(
        format!("{marker} {}", node_short_label(session, row.node_index, 42)),
        row_style,
    ));
    if is_branch {
        spans.push(Span::styled("  ⎇", Style::default().fg(theme.heading_h3)));
    }

    spans
}

#[derive(Debug, Clone)]
#[cfg(test)]
struct EdgeLine {
    kind: EdgeKind,
    kind_label: String,
    target_label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeKind {
    Next,
    Branch,
    After,
    Goto,
}

impl EdgeKind {
    fn color(self, theme: &Theme) -> ratatui::style::Color {
        match self {
            Self::Next => theme.border_active,
            Self::Branch => theme.heading_h3,
            Self::After => theme.accent,
            Self::Goto => theme.error,
        }
    }
}

#[cfg(test)]
fn summarize_edge_lines(session: &PresentationSession, node: &Node, index: usize) -> Vec<EdgeLine> {
    let mut edges = Vec::new();

    if let Some(target) = node.next_override() {
        if let Some(idx) = session.graph.index_of(target) {
            let is_goto = index + 1 != idx;
            edges.push(EdgeLine {
                kind: if is_goto {
                    EdgeKind::Goto
                } else {
                    EdgeKind::Next
                },
                kind_label: if is_goto {
                    "goto".to_string()
                } else {
                    "next".to_string()
                },
                target_label: format!("#{}", idx + 1),
            });
        }
    } else if index + 1 < session.graph.nodes.len() {
        edges.push(EdgeLine {
            kind: EdgeKind::Next,
            kind_label: "next".to_string(),
            target_label: format!("#{}", index + 2),
        });
    }

    if let Some(target) = node.after_target()
        && let Some(idx) = session.graph.index_of(target)
    {
        edges.push(EdgeLine {
            kind: EdgeKind::After,
            kind_label: "after".to_string(),
            target_label: format!("#{}", idx + 1),
        });
    }

    if let Some(branch) = node.branch_point() {
        for option in &branch.options {
            if let Some(idx) = session.graph.index_of(&option.target) {
                edges.push(EdgeLine {
                    kind: EdgeKind::Branch,
                    kind_label: format!("branch [{}]", option.key),
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
    let rows = graph_tree_rows(session);
    if rows.is_empty() {
        return GraphOverlayWindow { start: 0, end: 0 };
    }

    let total = rows.len();

    let selected_row = rows
        .iter()
        .position(|row| row.node_index == selected)
        .unwrap_or(0);

    let list_panel = graph_overlay_list_panel_rect(area);
    let row_capacity = list_panel.height.saturating_sub(2).max(1);

    let mut start = scroll_offset.min(total.saturating_sub(1));
    if selected_row < start {
        start = selected_row;
    }

    let mut end = (start + row_capacity as usize).min(total);
    if selected_row >= end {
        start = selected_row;
        end = (start + row_capacity as usize).min(total);
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
    let rows = graph_tree_rows(session);
    if rows.is_empty() {
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
    let row_index = window
        .start
        .saturating_add(row.saturating_sub(content_y) as usize);
    if row_index >= window.end {
        return None;
    }

    rows.get(row_index).map(|row| row.node_index)
}

fn graph_tree_rows(session: &PresentationSession) -> Vec<TreeRow> {
    let total = session.graph.nodes.len();
    if total == 0 {
        return Vec::new();
    }

    let mut rows = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = vec![(0usize, 0usize, None, None)];

    while let Some((index, depth, incoming_kind, incoming_label)) = stack.pop() {
        if !visited.insert(index) {
            continue;
        }

        rows.push(TreeRow {
            node_index: index,
            depth,
            incoming_kind,
            incoming_label,
        });

        let children = node_children(session, index);
        for (child_index, kind, label) in children.into_iter().rev() {
            if !visited.contains(&child_index) {
                stack.push((child_index, depth + 1, Some(kind), label));
            }
        }
    }

    for index in 0..total {
        if visited.insert(index) {
            rows.push(TreeRow {
                node_index: index,
                depth: 0,
                incoming_kind: None,
                incoming_label: Some("detached".to_string()),
            });
        }
    }

    rows
}

fn node_children(
    session: &PresentationSession,
    index: usize,
) -> Vec<(usize, EdgeKind, Option<String>)> {
    let mut children = Vec::new();
    let Some(node) = session.graph.nodes.get(index) else {
        return children;
    };

    if let Some(target) = node.next_override()
        && let Some(target_index) = session.graph.index_of(target)
    {
        let kind = if target_index == index + 1 {
            EdgeKind::Next
        } else {
            EdgeKind::Goto
        };
        children.push((target_index, kind, None));
    } else if index + 1 < session.graph.nodes.len() {
        children.push((index + 1, EdgeKind::Next, None));
    }

    if let Some(target) = node.after_target()
        && let Some(target_index) = session.graph.index_of(target)
    {
        children.push((target_index, EdgeKind::After, None));
    }

    if let Some(branch) = node.branch_point() {
        for option in &branch.options {
            if let Some(target_index) = session.graph.index_of(&option.target) {
                children.push((target_index, EdgeKind::Branch, Some(option.key.to_string())));
            }
        }
    }

    children
}

fn node_short_label(session: &PresentationSession, index: usize, max_chars: usize) -> String {
    let label = session
        .graph
        .nodes
        .get(index)
        .and_then(|node| node.id.as_deref().or(node.title.as_deref()))
        .map_or_else(|| format!("#{}", index + 1), ToOwned::to_owned);

    if label.chars().count() <= max_chars {
        label
    } else {
        let mut out = label
            .chars()
            .take(max_chars.saturating_sub(1))
            .collect::<String>();
        out.push('…');
        out
    }
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

#[cfg(test)]
mod tests {
    use super::{EdgeKind, TreeRow, render_tree_row, summarize_edge_lines};
    use crate::theme::Theme;
    use fireside_core::model::branch::{BranchOption, BranchPoint};
    use fireside_core::model::content::ContentBlock;
    use fireside_core::model::graph::{Graph, GraphFile};
    use fireside_core::model::node::Node;
    use fireside_core::model::traversal::Traversal;
    use fireside_engine::PresentationSession;

    fn node(id: &str) -> Node {
        Node {
            id: Some(id.to_string()),
            title: None,
            tags: vec![],
            duration: None,
            layout: None,
            transition: None,
            speaker_notes: None,
            traversal: None,
            content: vec![ContentBlock::Text {
                body: format!("node {id}"),
            }],
        }
    }

    fn graph_with_nodes(nodes: Vec<Node>) -> Graph {
        Graph::from_file(GraphFile {
            title: None,
            fireside_version: None,
            author: None,
            date: None,
            description: None,
            version: None,
            tags: vec![],
            theme: None,
            font: None,
            defaults: None,
            extensions: vec![],
            nodes,
        })
        .expect("graph should be valid")
    }

    fn spans_to_text(spans: &[ratatui::text::Span<'_>]) -> String {
        spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>()
            .join("")
    }

    #[test]
    fn summarize_edge_lines_classifies_next_after_branch_and_goto() {
        let mut start = node("start");
        start.traversal = Some(Traversal {
            next: Some("end".to_string()),
            after: Some("middle".to_string()),
            branch_point: Some(BranchPoint {
                id: Some("bp".to_string()),
                prompt: None,
                options: vec![BranchOption {
                    label: "to-middle".to_string(),
                    key: 'a',
                    target: "middle".to_string(),
                }],
            }),
        });

        let middle = node("middle");
        let end = node("end");

        let graph = graph_with_nodes(vec![start, middle, end]);
        let session = PresentationSession::new(graph, 0);
        let edges = summarize_edge_lines(&session, &session.graph.nodes[0], 0);

        assert_eq!(edges.len(), 3);
        assert_eq!(edges[0].kind, EdgeKind::Goto);
        assert_eq!(edges[0].kind_label, "goto");
        assert_eq!(edges[0].target_label, "#3");
        assert_eq!(edges[1].kind, EdgeKind::After);
        assert_eq!(edges[1].kind_label, "after");
        assert_eq!(edges[1].target_label, "#2");
        assert_eq!(edges[2].kind, EdgeKind::Branch);
        assert_eq!(edges[2].kind_label, "branch [a]");
        assert_eq!(edges[2].target_label, "#2");
    }

    #[test]
    fn render_tree_row_marks_branch_nodes() {
        let mut branch = node("branch");
        branch.traversal = Some(Traversal {
            next: None,
            after: None,
            branch_point: Some(BranchPoint {
                id: Some("branch-node".to_string()),
                prompt: None,
                options: vec![BranchOption {
                    label: "go".to_string(),
                    key: '1',
                    target: "end".to_string(),
                }],
            }),
        });

        let end = node("end");
        let graph = graph_with_nodes(vec![branch, end]);
        let session = PresentationSession::new(graph, 0);
        let theme = Theme::default();

        let row = TreeRow {
            node_index: 0,
            depth: 0,
            incoming_kind: None,
            incoming_label: None,
        };

        let spans = render_tree_row(&session, &row, &theme);
        let text = spans_to_text(&spans);
        assert!(text.contains("branch"));
        assert!(text.contains("⎇"));
    }
}
