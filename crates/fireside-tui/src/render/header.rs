//! The header: deck title, current node title, progress count, and the
//! mini "rail" rule that shows travelled/current/upcoming stations.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::theme::Tokens;

pub(super) fn draw_header(frame: &mut Frame, area: Rect, app: &App, tokens: &Tokens) {
    let graph = app.session().graph();
    let deck = graph.title.as_deref().unwrap_or("Fireside");
    let node = app.session().current();
    let here = node.title.as_deref().unwrap_or(&node.id);
    let seen = app.session().visited().len();
    let total = graph.nodes.len();

    let [text_row, rule_row] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(deck.to_owned(), tokens.accent.add_modifier(Modifier::BOLD)),
        ])),
        text_row,
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(here.to_owned(), tokens.muted),
            Span::styled(format!("  ·  {seen}/{total} seen "), tokens.muted),
        ]))
        .alignment(Alignment::Right),
        text_row,
    );
    frame.render_widget(
        Paragraph::new(header_rail(app, area.width, tokens)),
        rule_row,
    );
}

/// The header rule doubles as a rail strip: stations you have travelled,
/// the one you stand at, and the straight track ahead — the deck's shape,
/// always in the corner of your eye.
fn header_rail(app: &App, width: u16, tokens: &Tokens) -> Line<'static> {
    let w = usize::from(width);
    if w < 24 {
        return Line::styled("─".repeat(w), tokens.border);
    }
    let session = app.session();
    let graph = session.graph();

    // Stations: the travelled path, then the linear track ahead of the
    // cursor (a fork or the end of the line stops the lookahead).
    let mut ids: Vec<&str> = session.history().iter().map(String::as_str).collect();
    let behind = ids.len();
    ids.push(&session.current().id);
    let mut seen: std::collections::HashSet<&str> = ids.iter().copied().collect();
    let mut cursor = session.current();
    while let Some(next) = cursor.next_target().and_then(|id| graph.node(id)) {
        if !seen.insert(&next.id) || ids.len() >= 24 {
            break;
        }
        ids.push(&next.id);
        cursor = next;
    }

    // Each station takes 4 cells (glyph + track). Keep the tail when the
    // path outgrows the row: where you are matters more than where you began.
    const STEP: usize = 4;
    let max = (w.saturating_sub(6)) / STEP;
    let cut = ids.len().saturating_sub(max);
    let current_at = behind.saturating_sub(cut);
    let shown = &ids[cut..];

    let mut spans = vec![Span::styled(
        if cut > 0 { "┄─" } else { "──" }.to_owned(),
        tokens.border,
    )];
    let mut used = 2;
    for (k, id) in shown.iter().enumerate() {
        let terminal = graph.node(id).is_some_and(fireside_core::Node::is_terminal);
        let (glyph, style) = match k.cmp(&current_at) {
            std::cmp::Ordering::Less => ("●", tokens.accent),
            std::cmp::Ordering::Equal => ("◉", tokens.accent.add_modifier(Modifier::BOLD)),
            std::cmp::Ordering::Greater => ("○", tokens.muted),
        };
        spans.push(Span::styled((*glyph).to_owned(), style));
        used += 1;
        if terminal && k + 1 == shown.len() {
            spans.push(Span::styled("─■".to_owned(), style));
            used += 2;
            break;
        }
        if k + 1 < shown.len() {
            // Track between stations is bright once ridden.
            let track = if k < current_at {
                tokens.accent
            } else {
                tokens.border
            };
            spans.push(Span::styled("───".to_owned(), track));
            used += 3;
        }
    }
    spans.push(Span::styled(
        "─".repeat(w.saturating_sub(used)),
        tokens.border,
    ));
    Line::from(spans)
}
