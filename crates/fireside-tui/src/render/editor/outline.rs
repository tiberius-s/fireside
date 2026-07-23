//! The outline pane: every slide in the deck's depth-first display order
//! (`fireside_engine::authoring::outline_order`), a divider before any
//! slide not yet reachable from the start, and the permanent
//! "+ new slide" row (spec 013).

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::editor::hit::OutlineLine;
use crate::editor::{EditorApp, Selection, hit};
use crate::theme::Tokens;

pub(super) fn draw(frame: &mut Frame, area: Rect, app: &EditorApp, tokens: &Tokens) {
    let scroll = hit::outline_scroll_offset(app, area);
    let lines: Vec<Line<'static>> = hit::outline_lines(app.working_graph())
        .iter()
        .skip(scroll)
        .take(area.height as usize)
        .map(|item| render_line(item, app, tokens))
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_line(item: &OutlineLine, app: &EditorApp, tokens: &Tokens) -> Line<'static> {
    match item {
        OutlineLine::Divider => Line::from(Span::styled(
            " \u{2500}\u{2500} not linked yet \u{2500}\u{2500}",
            tokens.muted,
        )),
        OutlineLine::NewSlide => Line::from(Span::styled(" + new slide", tokens.affordance)),
        OutlineLine::Row(row) => {
            let node = app.working_graph().node(&row.node_id);
            let title = node
                .and_then(|n| n.title.clone())
                .unwrap_or_else(|| row.node_id.clone());
            let marker = node.map_or(' ', |n| {
                if n.branch_point().is_some() {
                    '\u{2442}'
                } else if n.is_terminal() {
                    '\u{25a0}'
                } else {
                    ' '
                }
            });
            let selected = matches!(
                app.selection(),
                Selection::Slide(id) | Selection::Block(id, _) if *id == row.node_id
            );
            let hovered = app.hover() == Some(&hit::Target::OutlineRow(row.node_id.clone()));
            let style = if selected {
                tokens.selection
            } else if hovered {
                tokens.affordance
            } else {
                tokens.text
            };
            Line::from(vec![
                Span::styled(format!(" {:>2} ", row.display_number), tokens.muted),
                Span::styled(format!("{title} "), style),
                Span::styled(marker.to_string(), tokens.muted),
            ])
        }
    }
}
