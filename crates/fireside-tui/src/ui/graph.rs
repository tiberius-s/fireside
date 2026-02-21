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
        .map(|idx| ListItem::new(graph_item_lines(session, idx, theme)))
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

    let mini_inner = Block::default().inner(panes[1]);
    let legend_area = Rect {
        x: mini_inner.x,
        y: mini_inner
            .y
            .saturating_add(mini_inner.height.saturating_sub(4)),
        width: mini_inner.width,
        height: 4,
    };
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                "── next",
                Style::default().fg(theme.heading_h1),
            )),
            Line::from(Span::styled(
                "── branch",
                Style::default().fg(theme.heading_h3),
            )),
            Line::from(Span::styled("── after", Style::default().fg(theme.success))),
            Line::from(Span::styled("── goto", Style::default().fg(theme.error))),
        ]),
        legend_area,
    );

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

fn graph_item_lines(
    session: &PresentationSession,
    index: usize,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let current = session.current_node_index();
    let Some(node) = session.graph.nodes.get(index) else {
        return vec![Line::from("(missing node)")];
    };

    let is_current = index == current;
    let is_branch = node.branch_point().is_some();
    let marker = if is_current { "*" } else { " " };
    let id = node.id.as_deref().unwrap_or("(no-id)");
    let rail = if index + 1 < session.graph.nodes.len() {
        "│"
    } else {
        "└"
    };
    let header_text = if is_branch {
        format!("{marker} {rail} ┌⎇[{:>2}] {id}┐", index + 1)
    } else {
        format!("{marker} {rail} ┌[{:>2}] {id}┐", index + 1)
    };
    let header_style = if is_current {
        Style::default()
            .fg(theme.border_active)
            .add_modifier(Modifier::BOLD)
    } else if is_branch {
        Style::default().fg(theme.heading_h3)
    } else {
        Style::default().fg(theme.foreground)
    };
    let mut lines = vec![Line::from(Span::styled(header_text, header_style))];

    let edge_lines = summarize_edge_lines(session, node, index);
    if !edge_lines.is_empty() {
        let max_kind_len = edge_lines
            .iter()
            .map(|edge| edge.kind_label.len())
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

            let kind = format!("{:width$}", edge.kind_label, width = max_kind_len);
            let target = format!("{:width$}", edge.target_label, width = max_target_len);
            lines.push(Line::from(Span::styled(
                format!("  {rail}  {branch_connector}╼ {kind} → {target}"),
                Style::default().fg(edge.kind.color(theme)),
            )));
        }
    }

    lines
}

#[derive(Debug, Clone)]
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
            Self::Next => theme.heading_h1,
            Self::Branch => theme.heading_h3,
            Self::After => theme.success,
            Self::Goto => theme.error,
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::{EdgeKind, graph_item_lines, summarize_edge_lines};
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

    fn lines_to_text(lines: &[ratatui::text::Line<'_>]) -> Vec<String> {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect()
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
        assert_eq!(edges[1].kind, EdgeKind::After);
        assert_eq!(edges[1].kind_label, "after");
        assert_eq!(edges[2].kind, EdgeKind::Branch);
        assert_eq!(edges[2].kind_label, "branch [a]");
    }

    #[test]
    fn graph_item_lines_marks_branch_headers() {
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

        let lines = graph_item_lines(&session, 0, &theme);
        let text = lines_to_text(&lines);
        assert!(text[0].contains("┌⎇[ 1] branch┐"));
    }
}
