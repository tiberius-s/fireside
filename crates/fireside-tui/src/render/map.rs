//! The map screen: a vertical rail diagram of the deck.
//!
//! Slides are stations on a subway-style line, listed in document order.
//! The gutter carries the tracks: the spine runs down the left edge, a
//! branch point forks extra lanes out to the right (with each option's key
//! in a legend beside the fork), lanes ride past intermediate stations and
//! bend back in at the row of their target. Track the presenter actually
//! travelled is bright; unexplored track is dim. Backward edges (cycles)
//! are noted at their station instead of drawn, so the diagram always
//! flows top to bottom. Selection stays one-row-per-slide, and the list
//! scrolls when a deck outgrows the overlay.

use std::collections::{HashMap, HashSet};

use fireside_core::{Graph, Node};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph};

use crate::app::App;
use crate::theme::Tokens;

/// Horizontal cells from one rail slot to the next.
const PITCH: usize = 3;

/// A parallel rail in flight: one forward edge riding beside the spine.
#[derive(Debug, Clone, PartialEq)]
struct Lane {
    /// Row index (into `graph.nodes`) where this lane bends back in.
    to: usize,
    /// Palette index for [`Tokens::rail`].
    color: usize,
    /// Whether the presenter has ridden this edge.
    travelled: bool,
}

/// One glyph of track: its lane color (`None` = the spine) and whether the
/// presenter has ridden that piece.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Track {
    glyph: char,
    color: Option<usize>,
    travelled: bool,
}

/// A fork legend entry: the key that takes the colored line.
#[derive(Debug, PartialEq)]
struct LegendEntry {
    key: String,
    title: String,
    color: Option<usize>,
    travelled: bool,
}

/// What a diagram row is, beyond its gutter glyphs.
#[derive(Debug, PartialEq)]
enum Kind {
    /// A slide row; `node` indexes `graph.nodes`.
    Station {
        node: usize,
        /// Title of a backward edge's target, when this node loops back.
        backward: Option<String>,
    },
    /// A fork's legend beside its connector glyphs.
    Legend(Vec<LegendEntry>),
    /// Plain track between stations.
    Rail,
}

/// One row of the diagram.
#[derive(Debug)]
struct RailRow {
    track: Vec<Track>,
    kind: Kind,
}

/// The spine's state across the gap between two consecutive rows.
#[derive(Debug, Clone, Copy, Default)]
struct Gap {
    live: bool,
    travelled: bool,
}

/// A node's outgoing edges: `(option key if branching, target id)`.
fn outgoing(node: &Node) -> Vec<(Option<String>, &str)> {
    if let Some(bp) = node.branch_point() {
        bp.options
            .iter()
            .enumerate()
            .map(|(j, o)| {
                let key = o.key.clone().unwrap_or_else(|| (j + 1).to_string());
                (Some(key), o.target.as_str())
            })
            .collect()
    } else {
        node.next_target()
            .map(|t| vec![(None, t)])
            .unwrap_or_default()
    }
}

fn title_of(node: &Node) -> &str {
    node.title.as_deref().unwrap_or(&node.id)
}

/// The gutter column of rail slot `s` (the spine sits at column 0).
fn slot_col(s: usize) -> usize {
    PITCH * (s + 1)
}

/// Lay the deck out as rail rows. `history` is the travelled path (oldest
/// first), `current` the presenter's position.
fn layout(
    graph: &Graph,
    history: &[String],
    current: &str,
    visited: &HashSet<String>,
) -> Vec<RailRow> {
    let nodes = &graph.nodes;
    let index: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();
    let path: Vec<usize> = history
        .iter()
        .map(String::as_str)
        .chain([current])
        .filter_map(|id| index.get(id).copied())
        .collect();
    let travelled = |a: usize, b: usize| path.windows(2).any(|w| w[0] == a && w[1] == b);

    let mut rows: Vec<RailRow> = Vec::new();
    let mut lanes: Vec<Option<Lane>> = Vec::new();
    let mut spine = Gap::default();
    // Whether the gap above the row being placed already has a connector.
    let mut gap_connected = true;

    for (i, node) in nodes.iter().enumerate() {
        // Lanes ending at this station bend back into the spine.
        let closing: Vec<usize> = lanes
            .iter()
            .enumerate()
            .filter(|(_, l)| l.as_ref().is_some_and(|l| l.to == i))
            .map(|(s, _)| s)
            .collect();
        if !closing.is_empty() {
            rows.push(merge_row(&lanes, &closing, spine));
            for &s in &closing {
                lanes[s] = None;
            }
            while lanes.last().is_some_and(Option::is_none) {
                lanes.pop();
            }
            gap_connected = true;
        }
        if i > 0 && !gap_connected {
            rows.push(rail_row(&lanes, spine));
        }

        // Sort this node's edges before drawing its station row, because a
        // backward edge annotates the station itself.
        let outs = outgoing(node);
        let is_branch = node.branch_point().is_some();
        let mut backward = None;
        for (_, target) in &outs {
            if let Some(&t) = index.get(target)
                && t <= i
                && backward.is_none()
            {
                backward = Some(title_of(&nodes[t]).to_owned());
            }
        }

        rows.push(station_row(i, node, current, visited, &lanes, backward));

        // Plan the gap below: the first edge to the very next row rides the
        // spine; other forward edges fork out into lanes.
        spine = Gap::default();
        let mut opened: Vec<usize> = Vec::new();
        let mut legend: Vec<LegendEntry> = Vec::new();
        for (key, target) in outs {
            let Some(&t) = index.get(target) else {
                continue;
            };
            let trav = travelled(i, t);
            if t <= i {
                if let Some(key) = key {
                    legend.push(LegendEntry {
                        key,
                        title: format!("↺ {}", title_of(&nodes[t])),
                        color: None,
                        travelled: trav,
                    });
                }
                continue;
            }
            if t == i + 1 && !spine.live {
                spine = Gap {
                    live: true,
                    travelled: trav,
                };
                if let Some(key) = key {
                    legend.push(LegendEntry {
                        key,
                        title: title_of(&nodes[t]).to_owned(),
                        color: None,
                        travelled: trav,
                    });
                }
            } else {
                let slot = lanes
                    .iter()
                    .position(Option::is_none)
                    .unwrap_or(lanes.len());
                if slot == lanes.len() {
                    lanes.push(None);
                }
                lanes[slot] = Some(Lane {
                    to: t,
                    color: slot,
                    travelled: trav,
                });
                opened.push(slot);
                if let Some(key) = key {
                    legend.push(LegendEntry {
                        key,
                        title: title_of(&nodes[t]).to_owned(),
                        color: Some(slot),
                        travelled: trav,
                    });
                }
            }
        }

        if !opened.is_empty() {
            rows.push(fork_row(&lanes, &opened, spine, legend));
            gap_connected = true;
        } else if is_branch && !legend.is_empty() {
            // A branch whose only routes ride the spine (or loop back):
            // show the legend on a plain track row.
            let mut row = rail_row(&lanes, spine);
            row.kind = Kind::Legend(legend);
            rows.push(row);
            gap_connected = true;
        } else {
            gap_connected = false;
        }
    }
    rows
}

fn station_row(
    i: usize,
    node: &Node,
    current: &str,
    visited: &HashSet<String>,
    lanes: &[Option<Lane>],
    backward: Option<String>,
) -> RailRow {
    let (glyph, travelled) = if node.id == current {
        ('◉', true)
    } else if visited.contains(&node.id) {
        ('●', true)
    } else {
        ('○', false)
    };
    let mut track = base_track(lanes);
    track[0] = Track {
        glyph,
        color: None,
        travelled,
    };
    RailRow {
        track,
        kind: Kind::Station { node: i, backward },
    }
}

/// A row of plain vertical track: the spine (when live) plus passing lanes.
fn rail_row(lanes: &[Option<Lane>], spine: Gap) -> RailRow {
    let mut track = base_track(lanes);
    track[0] = Track {
        glyph: if spine.live { '│' } else { ' ' },
        color: None,
        travelled: spine.travelled,
    };
    RailRow {
        track,
        kind: Kind::Rail,
    }
}

/// The gutter with passing lanes drawn and everything else blank.
fn base_track(lanes: &[Option<Lane>]) -> Vec<Track> {
    let cols = 1 + lanes.len() * PITCH;
    let mut track = vec![
        Track {
            glyph: ' ',
            color: None,
            travelled: false,
        };
        cols
    ];
    for (s, lane) in lanes.iter().enumerate() {
        if let Some(lane) = lane {
            track[slot_col(s)] = Track {
                glyph: '╎',
                color: Some(lane.color),
                travelled: lane.travelled,
            };
        }
    }
    track
}

/// The connector under a station whose edges fork out: `├──╮`, with `┬` for
/// intermediate new lanes and `┼` where an older lane is crossed.
fn fork_row(
    lanes: &[Option<Lane>],
    opened: &[usize],
    spine: Gap,
    legend: Vec<LegendEntry>,
) -> RailRow {
    let mut track = base_track(lanes);
    let rightmost = opened.iter().copied().max().unwrap_or(0);
    let any_travelled = opened
        .iter()
        .any(|&s| lanes[s].as_ref().is_some_and(|l| l.travelled));
    track[0] = Track {
        glyph: if spine.live { '├' } else { '╰' },
        color: None,
        travelled: spine.travelled || any_travelled,
    };
    // Right to left, each fill cell belongs to the nearest opening lane at
    // or beyond it — so every branch line is colored back to the junction.
    let mut owner: Option<&Lane> = None;
    for col in (1..=slot_col(rightmost)).rev() {
        let slot = (col >= PITCH && col % PITCH == 0).then(|| col / PITCH - 1);
        let cell = &mut track[col];
        if let Some(s) = slot
            && opened.contains(&s)
        {
            let lane = lanes[s].as_ref().unwrap_or_else(|| unreachable!());
            cell.glyph = if s == rightmost { '╮' } else { '┬' };
            cell.color = Some(lane.color);
            cell.travelled = lane.travelled;
            owner = Some(lane);
            continue;
        }
        let (color, travelled) = owner
            .map(|l| (Some(l.color), l.travelled))
            .unwrap_or((None, false));
        if cell.glyph == '╎' {
            cell.glyph = '┼';
        } else {
            *cell = Track {
                glyph: '─',
                color,
                travelled,
            };
        }
    }
    RailRow {
        track,
        kind: Kind::Legend(legend),
    }
}

/// The connector above a station that lanes bend into: `├──╯`, with `┴` for
/// intermediate closing lanes, `╭` when no spine arrives from above.
fn merge_row(lanes: &[Option<Lane>], closing: &[usize], spine: Gap) -> RailRow {
    let mut track = base_track(lanes);
    let rightmost = closing.iter().copied().max().unwrap_or(0);
    let any_travelled = closing
        .iter()
        .any(|&s| lanes[s].as_ref().is_some_and(|l| l.travelled));
    track[0] = Track {
        glyph: if spine.live { '├' } else { '╭' },
        color: None,
        travelled: spine.travelled || any_travelled,
    };
    let mut owner: Option<&Lane> = None;
    for col in (1..=slot_col(rightmost)).rev() {
        let slot = (col >= PITCH && col % PITCH == 0).then(|| col / PITCH - 1);
        let cell = &mut track[col];
        if let Some(s) = slot
            && closing.contains(&s)
        {
            let lane = lanes[s].as_ref().unwrap_or_else(|| unreachable!());
            cell.glyph = if s == rightmost { '╯' } else { '┴' };
            cell.color = Some(lane.color);
            cell.travelled = lane.travelled;
            owner = Some(lane);
            continue;
        }
        let (color, travelled) = owner
            .map(|l| (Some(l.color), l.travelled))
            .unwrap_or((None, false));
        if cell.glyph == '╎' {
            cell.glyph = '┼';
        } else {
            *cell = Track {
                glyph: '─',
                color,
                travelled,
            };
        }
    }
    RailRow {
        track,
        kind: Kind::Rail,
    }
}

/// Track cell → styled span. The spine and stations wear the accent; lanes
/// wear their line color; anything unexplored is dimmed.
fn track_span(cell: Track, tokens: &Tokens) -> Span<'static> {
    let mut style = match (cell.glyph, cell.color) {
        ('○', None) => tokens.muted,
        ('◉', None) => tokens.accent.add_modifier(Modifier::BOLD),
        (_, None) => tokens.accent,
        (_, Some(c)) => tokens.rail(c),
    };
    if !cell.travelled && !matches!(cell.glyph, '○' | ' ') {
        style = style.add_modifier(Modifier::DIM);
    }
    Span::styled(cell.glyph.to_string(), style)
}

/// Paint the map overlay.
pub fn draw(frame: &mut Frame, area: Rect, app: &App, selected: usize, tokens: &Tokens) {
    let session = app.session();
    let graph = session.graph();
    let visited: HashSet<String> = session.visited().iter().cloned().collect();
    let rows = layout(graph, session.history(), &session.current().id, &visited);
    let gutter = rows.iter().map(|r| r.track.len()).max().unwrap_or(1);

    // Build every line first; the overlay is then sized to fit them.
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut station_lines: Vec<usize> = Vec::new();
    for row in &rows {
        let mut spans: Vec<Span<'static>> = vec![Span::raw(" ")];
        for col in 0..gutter {
            match row.track.get(col) {
                Some(cell) => spans.push(track_span(*cell, tokens)),
                None => spans.push(Span::raw(" ")),
            }
        }
        spans.push(Span::raw("  "));
        match &row.kind {
            Kind::Station { node, backward } => {
                station_lines.push(lines.len());
                let n = &graph.nodes[*node];
                let style = if *node == selected {
                    tokens.selected
                } else if n.id == session.current().id {
                    tokens.accent.add_modifier(Modifier::BOLD)
                } else if visited.contains(&n.id) {
                    tokens.text
                } else {
                    tokens.muted
                };
                spans.push(Span::styled(format!(" {} ", title_of(n)), style));
                if n.is_terminal() {
                    spans.push(Span::styled(" ■".to_owned(), tokens.muted));
                }
                if let Some(back) = backward {
                    spans.push(Span::styled(
                        format!("  ↺ returns to {back}"),
                        tokens.muted.add_modifier(Modifier::ITALIC),
                    ));
                }
            }
            Kind::Legend(entries) => {
                for (j, e) in entries.iter().enumerate() {
                    if j > 0 {
                        spans.push(Span::styled(" · ".to_owned(), tokens.muted));
                    }
                    let mut style = match e.color {
                        Some(c) => tokens.rail(c),
                        None => tokens.accent,
                    };
                    if !e.travelled {
                        style = style.add_modifier(Modifier::DIM);
                    }
                    spans.push(Span::styled(format!("[{}] {}", e.key, e.title), style));
                }
            }
            Kind::Rail => {}
        }
        lines.push(Line::from(spans));
    }

    let footer = [
        Line::default(),
        Line::from(vec![
            Span::styled(" ◉".to_owned(), tokens.accent.add_modifier(Modifier::BOLD)),
            Span::styled(" you are here".to_owned(), tokens.muted),
            Span::styled("  ●".to_owned(), tokens.accent),
            Span::styled(" seen".to_owned(), tokens.muted),
            Span::styled("  ○ not yet".to_owned(), tokens.muted),
            Span::styled("  ■ end".to_owned(), tokens.muted),
        ]),
        Line::styled(" ↑↓ move · Enter jump · Esc close".to_owned(), tokens.muted),
    ];

    let content_w = lines
        .iter()
        .map(Line::width)
        .chain(footer.iter().map(Line::width))
        .max()
        .unwrap_or(0) as u16;
    let rect = super::overlay_rect(
        area,
        (content_w + 4).max(46),
        lines.len() as u16 + footer.len() as u16 + 2,
    );
    frame.render_widget(Clear, rect);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(tokens.border)
        .title(Span::styled(
            " Map — Enter jumps ".to_owned(),
            tokens.accent.add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    if inner.height <= footer.len() as u16 {
        return;
    }

    // The rail viewport scrolls to keep the selected station in view.
    let view_h = (inner.height - footer.len() as u16) as usize;
    let target = station_lines.get(selected).copied().unwrap_or(0);
    let max_skip = lines.len().saturating_sub(view_h);
    let skip = target.saturating_sub(view_h / 2).min(max_skip);
    let visible: Vec<Line<'static>> = lines.iter().skip(skip).take(view_h).cloned().collect();
    let rail_area = Rect {
        height: view_h as u16,
        ..inner
    };
    frame.render_widget(Paragraph::new(Text::from(visible)), rail_area);
    if skip > 0 {
        super::indicator(frame, rail_area, 0, "▲", tokens);
    }
    if skip < max_skip {
        super::indicator(frame, rail_area, rail_area.height - 1, "▼", tokens);
    }
    let footer_area = Rect {
        y: inner.y + view_h as u16,
        height: footer.len() as u16,
        ..inner
    };
    frame.render_widget(Paragraph::new(Text::from(footer.to_vec())), footer_area);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(json: &str) -> Graph {
        Graph::from_json(json).expect("test graph parses")
    }

    /// intro → fork(a: mid adjacent, b: end skipping mid) → both at end.
    const FORKED: &str = r#"{
        "fireside-version": "0.1.0",
        "nodes": [
            {"id": "intro", "title": "Intro", "content": [],
             "traversal": {"branch-point": {"options": [
                {"label": "Mid", "key": "a", "target": "mid"},
                {"label": "End", "key": "b", "target": "end"}
             ]}}},
            {"id": "mid", "title": "Mid", "content": [], "traversal": "end"},
            {"id": "end", "title": "End", "content": []}
        ]
    }"#;

    fn glyphs(rows: &[RailRow]) -> Vec<String> {
        rows.iter()
            .map(|r| r.track.iter().map(|c| c.glyph).collect())
            .collect()
    }

    #[test]
    fn a_fork_opens_a_lane_that_passes_and_merges() {
        let g = graph(FORKED);
        let rows = layout(&g, &[], "intro", &HashSet::from(["intro".to_owned()]));
        let track = glyphs(&rows);
        assert_eq!(
            track,
            vec![
                "◉",    // Intro
                "├──╮", // fork: a rides the spine, b swings out
                "○  ╎", // Mid, with b's lane passing
                "├──╯", // b bends back in
                "○",    // End
            ],
            "rows: {rows:?}"
        );
        // The fork row carries the legend with both keys.
        let Kind::Legend(entries) = &rows[1].kind else {
            panic!("fork row has a legend: {rows:?}");
        };
        assert_eq!(entries[0].key, "a");
        assert_eq!(entries[0].color, None, "adjacent option rides the spine");
        assert_eq!(entries[1].key, "b");
        assert_eq!(entries[1].color, Some(0), "skip option gets a rail color");
    }

    #[test]
    fn travelled_track_is_marked_and_untravelled_is_not() {
        let g = graph(FORKED);
        let history = ["intro".to_owned()];
        let visited = HashSet::from(["intro".to_owned(), "end".to_owned()]);
        let rows = layout(&g, &history, "end", &visited);
        // The b edge (intro → end) was ridden: its bend and its passing
        // track must be bright, and the junction was ridden through.
        let fork = &rows[1];
        let bend = fork.track[slot_col(0)];
        assert_eq!(bend.glyph, '╮');
        assert!(bend.travelled, "ridden lane is bright");
        assert!(fork.track[0].travelled, "the junction was ridden through");
        let mid = &rows[2];
        assert!(mid.track[slot_col(0)].travelled, "passing track is bright");
        // Mid itself was never visited: its station stays unlit.
        assert_eq!(mid.track[0].glyph, '○');
        assert!(!mid.track[0].travelled, "unvisited station stays dim");
    }

    #[test]
    fn a_backward_edge_becomes_a_station_note_not_a_rail() {
        let looped = r#"{
            "fireside-version": "0.1.0",
            "nodes": [
                {"id": "one", "title": "One", "content": [], "traversal": "two"},
                {"id": "two", "title": "Two", "content": [], "traversal": "one"}
            ]
        }"#;
        let g = graph(looped);
        let rows = layout(&g, &[], "one", &HashSet::from(["one".to_owned()]));
        let Kind::Station { backward, .. } = &rows[2].kind else {
            panic!("last row is the Two station: {rows:?}");
        };
        assert_eq!(backward.as_deref(), Some("One"));
        assert!(
            glyphs(&rows).iter().all(|r| !r.contains('╮')),
            "no forward lane for a backward edge"
        );
    }

    #[test]
    fn linear_decks_are_one_straight_line() {
        let linear = r#"{
            "fireside-version": "0.1.0",
            "nodes": [
                {"id": "a", "title": "A", "content": [], "traversal": "b"},
                {"id": "b", "title": "B", "content": [], "traversal": "c"},
                {"id": "c", "title": "C", "content": []}
            ]
        }"#;
        let g = graph(linear);
        let rows = layout(&g, &[], "a", &HashSet::from(["a".to_owned()]));
        assert_eq!(glyphs(&rows), vec!["◉", "│", "○", "│", "○"]);
    }
}
