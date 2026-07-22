//! The content flow: a node's rendered lines (blocks, branch menu, or
//! end-of-path marker), the card/notes-panel geometry around them, and the
//! "▲/▼ more" scroll indicators.

use fireside_core::{Node, ViewMode};
use ratatui::Frame;
use ratatui::layout::{Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::App;
use crate::theme::Tokens;

use super::{PAD_X, PAD_Y, Surface, blocks, markdown, surface};

/// Everything the content-rendering path needs to draw one slide, decoupled
/// from `App`/`Session` — the seam that lets the authoring editor's canvas
/// (spec `013-authoring-editor`) call this exact rendering path instead of
/// a second implementation, which is what makes WYSIWYG fidelity (`SC-008`)
/// a structural guarantee rather than a discipline to maintain by hand.
pub(super) struct SlideView<'a> {
    pub(super) node: &'a Node,
    pub(super) reveal_level: u32,
    pub(super) has_pending_reveal: bool,
    pub(super) branch_selected: usize,
    pub(super) fading: bool,
    pub(super) scroll: u16,
    pub(super) view_mode: ViewMode,
    /// Titles (or ids) of nodes visited before `node`, oldest first — feeds
    /// the end-marker's route trace. Empty when there is no traversal
    /// history to show — always true for the editor's at-rest canvas,
    /// which has never "traveled" anywhere; `end_marker` already handles
    /// this the same way a fresh session landing immediately on an ending
    /// does.
    pub(super) history_titles: Vec<String>,
}

impl<'a> SlideView<'a> {
    pub(super) fn from_app(app: &'a App) -> Self {
        let session = app.session();
        let graph = session.graph();
        let history_titles = session
            .history()
            .iter()
            .filter_map(|id| graph.node(id))
            .map(|n| n.title.clone().unwrap_or_else(|| n.id.clone()))
            .collect();
        Self {
            node: session.current(),
            reveal_level: session.reveal_level(),
            has_pending_reveal: session.has_pending_reveal(),
            branch_selected: app.branch_selected(),
            fading: app.fading(),
            scroll: app.scroll(),
            view_mode: app.view_mode(),
            history_titles,
        }
    }
}

/// The node's full line flow plus, when the flow ends in a branch menu, the
/// line index of each option's label row — the row a mouse click must land
/// on to choose that option (`hit_test::branch_option_at`). Kept alongside
/// the lines themselves, computed once, so drawing and hit-testing can never
/// disagree about where an option actually is on screen.
pub(super) struct NodeLines {
    pub(super) lines: Vec<Line<'static>>,
    /// Line index of each branch option's label row, parallel to
    /// `branch_point().options`. Empty when there is no branch menu.
    pub(super) option_rows: Vec<usize>,
}

/// The node's full line flow: content blocks, then the branch menu or the
/// end-of-path marker.
pub(super) fn node_lines(view: &SlideView, width: u16, tokens: &Tokens) -> NodeLines {
    let node = view.node;
    let mut lines = blocks::render_blocks(&node.content, width, tokens, view.reveal_level);
    let mut option_rows = Vec::new();

    let pending_reveal = view.has_pending_reveal;
    if let Some(bp) = node.branch_point().filter(|_| !pending_reveal) {
        if !lines.is_empty() {
            lines.push(Line::default());
        }
        let prompt = bp.prompt.as_deref().unwrap_or("Choose a path:");
        lines.extend(markdown::wrap_styled(
            prompt,
            width,
            tokens.accent.add_modifier(Modifier::BOLD),
            tokens,
        ));
        lines.push(Line::default());
        for (i, opt) in bp.options.iter().enumerate() {
            let selected = i == view.branch_selected;
            let mut spans = vec![
                if selected {
                    Span::styled(" ▸ ".to_owned(), tokens.accent.add_modifier(Modifier::BOLD))
                } else {
                    Span::raw("   ".to_owned())
                },
                Span::styled(format!("{}. ", i + 1), tokens.muted),
            ];
            let label_style = if selected {
                tokens.selected
            } else {
                tokens.text
            };
            spans.push(Span::styled(format!(" {} ", opt.label), label_style));
            if let Some(key) = &opt.key {
                spans.push(Span::styled(format!("  [{key}]"), tokens.muted));
            }
            option_rows.push(lines.len());
            lines.push(Line::from(spans));
            if let Some(desc) = &opt.description {
                for d in markdown::wrap_styled(desc, width.saturating_sub(7), tokens.muted, tokens)
                {
                    let mut spans = vec![Span::raw("       ".to_owned())];
                    spans.extend(d.spans);
                    lines.push(Line::from(spans));
                }
            }
        }
    } else if node.is_terminal() && !pending_reveal {
        if !lines.is_empty() {
            lines.push(Line::default());
        }
        lines.extend(end_marker(view, width, tokens));
    }
    NodeLines { lines, option_rows }
}

/// The content card/flow's inner rect for a line flow of `total` lines —
/// pure geometry, no drawing. Shared by `draw_content` (which additionally
/// paints the card border) and mouse hit-testing (which only needs to know
/// where each line landed).
pub(super) fn content_inner(body: Rect, surf: &Surface, total: u16) -> (Option<Rect>, Rect) {
    if surf.card {
        let card_width = surf.width + 2 + 2 * PAD_X;
        let card_height = surf.height + 2 + 2 * PAD_Y;
        let card_area = Rect {
            x: body.x + (body.width - card_width) / 2,
            y: body.y + (body.height - card_height) / 2,
            width: card_width,
            height: card_height,
        };
        let block = Block::bordered().border_type(BorderType::Rounded);
        let full = block.inner(card_area).inner(Margin {
            horizontal: PAD_X,
            vertical: PAD_Y,
        });
        let inner = if total < full.height {
            Rect {
                y: full.y + (full.height - total) / 2,
                height: total,
                ..full
            }
        } else {
            full
        };
        (Some(card_area), inner)
    } else {
        let width = surf.width.min(body.width);
        let height = total.min(body.height);
        let inner = Rect {
            x: body.x + (body.width - width) / 2,
            y: body.y + (body.height - height) / 2,
            width,
            height,
        };
        (None, inner)
    }
}

/// The close of a path. The deck should land, not shrug: a centered rule
/// with the end mark, a quiet word underneath — and the route actually
/// travelled, so the ending shows which story this audience got.
fn end_marker(view: &SlideView, width: u16, tokens: &Tokens) -> Vec<Line<'static>> {
    let w = usize::from(width);
    let rule = (w / 4).clamp(2, 12);
    let rule_pad = w.saturating_sub(rule * 2 + 3) / 2;
    let text = "End of this path";
    let text_pad = w.saturating_sub(text.chars().count()) / 2;
    let mut lines = vec![
        Line::from(vec![
            Span::raw(" ".repeat(rule_pad)),
            Span::styled("─".repeat(rule), tokens.border),
            Span::styled(" ■ ".to_owned(), tokens.accent),
            Span::styled("─".repeat(rule), tokens.border),
        ]),
        Line::from(vec![
            Span::raw(" ".repeat(text_pad)),
            Span::styled(text.to_owned(), tokens.muted),
        ]),
    ];

    let current_title = view.node.title.as_deref().unwrap_or(&view.node.id);
    let mut stations: Vec<&str> = view
        .history_titles
        .iter()
        .map(String::as_str)
        .chain([current_title])
        .collect();
    if stations.len() > 1 {
        // Long journeys keep their tail: the recent stops tell the story.
        let overflow = stations.len() > 8;
        if overflow {
            stations.drain(..stations.len() - 8);
        }
        let mut trace = stations.join(" → ");
        if overflow {
            trace = format!("… → {trace}");
        }
        lines.push(Line::default());
        for row in markdown::wrap_styled(
            &trace,
            width,
            tokens.muted.add_modifier(Modifier::DIM),
            tokens,
        ) {
            let pad = w.saturating_sub(row.width()) / 2;
            let mut spans = vec![Span::raw(" ".repeat(pad))];
            spans.extend(row.spans);
            lines.push(Line::from(spans));
        }
    }
    lines
}

pub(super) fn draw_content(frame: &mut Frame, body: Rect, view: &SlideView, tokens: &Tokens) {
    let surf = surface(view.view_mode, body);
    let NodeLines { lines, .. } = node_lines(view, surf.width, tokens);
    let total = lines.len() as u16;
    // During a fade-in the whole slide starts dim and brightens.
    let base = if view.fading {
        Style::new().add_modifier(Modifier::DIM)
    } else {
        Style::new()
    };

    let (card_area, inner) = content_inner(body, &surf, total);
    if let Some(card_area) = card_area {
        // The slide card: one constant stage for the whole deck — the same
        // frame on every slide — with the content flow centered inside it.
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(tokens.border.patch(base));
        frame.render_widget(block, card_area);
    }

    let max = total.saturating_sub(inner.height);
    let scroll = view.scroll.min(max);
    let visible: Vec<Line<'static>> = lines
        .into_iter()
        .skip(scroll as usize)
        .take(inner.height as usize)
        .collect();
    frame.render_widget(Paragraph::new(Text::from(visible)).style(base), inner);

    if scroll > 0 {
        indicator(frame, inner, 0, "▲", tokens);
    }
    if scroll < max {
        indicator(
            frame,
            inner,
            inner.height.saturating_sub(1),
            "▼ more (↓)",
            tokens,
        );
    }
}

pub(super) fn indicator(frame: &mut Frame, area: Rect, row: u16, text: &str, tokens: &Tokens) {
    let w = text.chars().count() as u16;
    let rect = Rect {
        x: area.right().saturating_sub(w),
        y: area.y + row,
        width: w.min(area.width),
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Span::styled(text.to_owned(), tokens.muted)),
        rect,
    );
}

/// Where the speaker-notes panel goes, if it is open and the node has notes.
pub(super) fn notes_panel(app: &App, content: Rect) -> Option<Rect> {
    if !app.show_notes() {
        return None;
    }
    app.session().current().speaker_notes.as_ref()?;
    let height = 6.min(content.height / 2);
    if height < 3 {
        return None;
    }
    Some(Rect {
        y: content.bottom() - height,
        height,
        ..content
    })
}

pub(super) fn draw_notes(frame: &mut Frame, area: Rect, app: &App, tokens: &Tokens) {
    let notes = app
        .session()
        .current()
        .speaker_notes
        .clone()
        .unwrap_or_default();
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(tokens.border)
        .title(Span::styled(" Notes — s hides ".to_owned(), tokens.muted));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let lines = markdown::wrap_styled(&notes, inner.width, tokens.muted, tokens);
    frame.render_widget(Paragraph::new(Text::from(lines)), inner);
}
