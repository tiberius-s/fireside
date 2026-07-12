//! Drawing the presenter.
//!
//! The renderer is pure: it reads [`App`] state and paints a frame. All
//! content renders through [`blocks`] into a flat line flow, so the page
//! can be vertically centered when it fits and scrolled when it does not.
//! The footer always shows exactly the keys that are valid right now —
//! that contract is what makes the presenter learnable without a manual.

pub mod blocks;
mod map;
pub mod markdown;
pub mod syntax;

use fireside_core::ViewMode;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

use crate::app::{App, EditableField, EditableKind, FlashKind, Screen};
use crate::theme::Tokens;

/// The widest comfortable reading measure for slide content, in columns.
const MEASURE: u16 = 76;
/// Columns of padding between the card border and the content.
const PAD_X: u16 = 3;
/// Rows of padding between the card border and the content.
const PAD_Y: u16 = 1;

/// Paint one frame.
pub fn draw(frame: &mut Frame, app: &App) {
    let tokens = Tokens::default();
    let area = frame.area();
    if area.width < 10 || area.height < 4 {
        frame.render_widget(Paragraph::new("Too small"), area);
        return;
    }

    let (header, mut content, footer) = areas(app.view_mode(), area);

    if let Some(header) = header {
        draw_header(frame, header, app, &tokens);
    }

    if let Some(notes) = notes_panel(app, content) {
        content.height = content.height.saturating_sub(notes.height);
        draw_notes(frame, notes, app, &tokens);
    }

    draw_content(frame, content, app, &tokens);
    draw_footer(frame, footer, app, &tokens);

    match app.screen() {
        Screen::Present => {}
        Screen::Help => draw_help(frame, area, &tokens),
        Screen::Map { selected } => map::draw(frame, area, app, *selected, &tokens),
        Screen::Edit { fields, focused } => draw_edit(frame, area, fields, *focused, &tokens),
    }
}

/// The largest useful scroll offset at the given terminal size. Shared with
/// `App::update` so scrolling clamps to real geometry.
#[must_use]
pub fn max_scroll(app: &App, width: u16, height: u16) -> u16 {
    let (_, mut body, _) = areas(app.view_mode(), Rect::new(0, 0, width, height));
    if let Some(notes) = notes_panel(app, body) {
        body.height = body.height.saturating_sub(notes.height);
    }
    let surf = surface(app.view_mode(), body);
    let total = node_lines(app, surf.width, &Tokens::default()).len() as u16;
    total.saturating_sub(surf.height)
}

/// Split the frame into header / body / footer for the view mode.
fn areas(view: ViewMode, area: Rect) -> (Option<Rect>, Rect, Rect) {
    match view {
        ViewMode::Default => {
            let [header, body, footer] = Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .areas(area);
            (Some(header), body, footer)
        }
        ViewMode::Fullscreen => {
            let [body, footer] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
            (None, body, footer)
        }
    }
}

/// The slide surface: the columns and rows content lines get, and whether a
/// bordered card frames them. Fullscreen and too-small terminals get a bare
/// flow at (almost) full width; the default view gets a centered card capped
/// at a readable measure.
struct Surface {
    width: u16,
    height: u16,
    card: bool,
}

/// Rows of air between the card and the header rule / footer, so the card
/// reads as a stage rather than a fence around the whole screen.
const CARD_GAP: u16 = 2;

fn surface(view: ViewMode, body: Rect) -> Surface {
    let chrome_w = 2 + 2 * PAD_X;
    let chrome_h = 2 + 2 * PAD_Y;
    let card = view == ViewMode::Default
        && body.width >= chrome_w + 16
        && body.height >= chrome_h + CARD_GAP + 3;
    if card {
        let card_width = body.width.min(MEASURE + chrome_w);
        Surface {
            width: card_width - chrome_w,
            height: body.height - chrome_h - CARD_GAP,
            card: true,
        }
    } else {
        Surface {
            width: body.width.saturating_sub(2),
            height: body.height,
            card: false,
        }
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App, tokens: &Tokens) {
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

/// The node's full line flow: content blocks, then the branch menu or the
/// end-of-path marker.
fn node_lines(app: &App, width: u16, tokens: &Tokens) -> Vec<Line<'static>> {
    let node = app.session().current();
    let mut lines = blocks::render_blocks(&node.content, width, tokens);

    if let Some(bp) = app.session().branch_point() {
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
            let selected = i == app.branch_selected();
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
    } else if node.is_terminal() {
        if !lines.is_empty() {
            lines.push(Line::default());
        }
        lines.extend(end_marker(app, width, tokens));
    }
    lines
}

/// The close of a path. The deck should land, not shrug: a centered rule
/// with the end mark, a quiet word underneath — and the route actually
/// travelled, so the ending shows which story this audience got.
fn end_marker(app: &App, width: u16, tokens: &Tokens) -> Vec<Line<'static>> {
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

    let graph = app.session().graph();
    let mut stations: Vec<&str> = app
        .session()
        .history()
        .iter()
        .filter_map(|id| graph.node(id))
        .chain([app.session().current()])
        .map(|n| n.title.as_deref().unwrap_or(&n.id))
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

fn draw_content(frame: &mut Frame, body: Rect, app: &App, tokens: &Tokens) {
    let surf = surface(app.view_mode(), body);
    let lines = node_lines(app, surf.width, tokens);
    let total = lines.len() as u16;
    // During a fade-in the whole slide starts dim and brightens.
    let base = if app.fading() {
        Style::new().add_modifier(Modifier::DIM)
    } else {
        Style::new()
    };

    let inner = if surf.card {
        // The slide card: one constant stage for the whole deck — the same
        // frame on every slide — with the content flow centered inside it.
        let card_width = surf.width + 2 + 2 * PAD_X;
        let card_height = surf.height + 2 + 2 * PAD_Y;
        let card_area = Rect {
            x: body.x + (body.width - card_width) / 2,
            y: body.y + (body.height - card_height) / 2,
            width: card_width,
            height: card_height,
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(tokens.border.patch(base));
        let full = block.inner(card_area).inner(Margin {
            horizontal: PAD_X,
            vertical: PAD_Y,
        });
        frame.render_widget(block, card_area);
        if total < full.height {
            Rect {
                y: full.y + (full.height - total) / 2,
                height: total,
                ..full
            }
        } else {
            full
        }
    } else {
        // Bare flow (fullscreen, tiny terminals): centered when it fits.
        let width = surf.width.min(body.width);
        let height = total.min(body.height);
        Rect {
            x: body.x + (body.width - width) / 2,
            y: body.y + (body.height - height) / 2,
            width,
            height,
        }
    };

    let max = total.saturating_sub(inner.height);
    let scroll = app.scroll().min(max);
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

fn indicator(frame: &mut Frame, area: Rect, row: u16, text: &str, tokens: &Tokens) {
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
fn notes_panel(app: &App, content: Rect) -> Option<Rect> {
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

fn draw_notes(frame: &mut Frame, area: Rect, app: &App, tokens: &Tokens) {
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

fn draw_footer(frame: &mut Frame, area: Rect, app: &App, tokens: &Tokens) {
    if let Some(flash) = app.flash() {
        let style = match flash.kind {
            FlashKind::Info => tokens.accent,
            FlashKind::Error => tokens.error,
        };
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(" {}", flash.text),
                style.add_modifier(Modifier::BOLD),
            )),
            area,
        );
        draw_timer(frame, area, app, tokens);
        return;
    }

    let session = app.session();
    let hints: &[(&str, &str)] = if session.branch_point().is_some() {
        &[
            ("↑↓", "choose"),
            ("Enter", "go"),
            ("←", "back"),
            ("m", "map"),
            ("e", "edit"),
            ("?", "help"),
            ("q", "quit"),
        ]
    } else if session.current().is_terminal() {
        &[
            ("←", "back"),
            ("m", "map"),
            ("e", "edit"),
            ("?", "help"),
            ("q", "quit"),
        ]
    } else {
        &[
            ("Space", "next"),
            ("←", "back"),
            ("m", "map"),
            ("e", "edit"),
            ("?", "help"),
            ("q", "quit"),
        ]
    };

    let mut spans = vec![Span::raw(" ")];
    for (i, (key, action)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ·  ".to_owned(), tokens.border));
        }
        spans.push(Span::styled(
            (*key).to_owned(),
            tokens.text.add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(format!(" {action}"), tokens.muted));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
    draw_timer(frame, area, app, tokens);
}

/// The elapsed timer, right-aligned in the footer when switched on.
fn draw_timer(frame: &mut Frame, area: Rect, app: &App, tokens: &Tokens) {
    if !app.show_timer() {
        return;
    }
    let secs = app.elapsed().as_secs();
    let text = if secs >= 3600 {
        format!(
            "{}:{:02}:{:02} ",
            secs / 3600,
            (secs % 3600) / 60,
            secs % 60
        )
    } else {
        format!("{}:{:02} ", secs / 60, secs % 60)
    };
    frame.render_widget(
        Paragraph::new(Span::styled(text, tokens.muted)).alignment(Alignment::Right),
        area,
    );
}

/// A centered overlay rect.
fn overlay_rect(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width.saturating_sub(2));
    let h = height.min(area.height.saturating_sub(2));
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

/// The quick-edit modal: one editable field per heading/text block found on
/// the current node, each shown with its buffer and a visible cursor on the
/// focused field. Content-only per ADR-005 — no structural edits.
fn draw_edit(
    frame: &mut Frame,
    area: Rect,
    fields: &[EditableField],
    focused: usize,
    tokens: &Tokens,
) {
    let content_lines: u16 = fields
        .iter()
        .map(|f| 1 + f.buffer.len() as u16 + 1)
        .sum::<u16>()
        + 1;
    let rect = overlay_rect(area, MEASURE, content_lines + 4);
    frame.render_widget(Clear, rect);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(tokens.border)
        .title(Span::styled(
            " Quick edit ".to_owned(),
            tokens.accent.add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let mut lines: Vec<Line<'static>> = Vec::new();
    for (i, field) in fields.iter().enumerate() {
        let label = match field.kind {
            EditableKind::Heading(level) => format!("Heading (level {level})"),
            EditableKind::Text => "Text".to_owned(),
        };
        let label_style = if i == focused {
            tokens.selected.add_modifier(Modifier::BOLD)
        } else {
            tokens.muted
        };
        lines.push(Line::styled(format!(" {label}"), label_style));
        for (row, text) in field.buffer.iter().enumerate() {
            lines.push(edit_line(text, i == focused && row == field.cursor.0, field.cursor.1, tokens));
        }
        lines.push(Line::default());
    }
    lines.push(Line::styled(
        " Ctrl+S save  ·  Esc cancel".to_owned(),
        tokens.muted,
    ));
    frame.render_widget(Paragraph::new(Text::from(lines)), inner);
}

/// One line of quick-edit buffer text, with a reversed-block cursor cell
/// when this is the focused line.
fn edit_line(text: &str, cursor_here: bool, col: usize, tokens: &Tokens) -> Line<'static> {
    if !cursor_here {
        return Line::styled(format!("  {text}"), tokens.text);
    }
    let chars: Vec<char> = text.chars().collect();
    let before: String = chars[..col.min(chars.len())].iter().collect();
    let at = chars.get(col).copied().unwrap_or(' ');
    let after: String = chars.get(col + 1..).map_or(String::new(), |s| s.iter().collect());
    Line::from(vec![
        Span::raw("  "),
        Span::styled(before, tokens.text),
        Span::styled(at.to_string(), tokens.text.add_modifier(Modifier::REVERSED)),
        Span::styled(after, tokens.text),
    ])
}

fn draw_help(frame: &mut Frame, area: Rect, tokens: &Tokens) {
    const KEYS: &[(&str, &str)] = &[
        ("Space / → / Enter", "next slide"),
        ("← / Backspace", "previous slide"),
        ("↑ / ↓", "pick a choice · scroll"),
        ("1–9 or a letter", "take a choice directly"),
        ("m", "map — see and jump anywhere"),
        ("f", "fullscreen on/off"),
        ("s", "speaker notes"),
        ("e", "quick-edit heading/text on this slide"),
        ("t", "elapsed timer"),
        ("q", "quit"),
    ];
    let rect = overlay_rect(area, 50, KEYS.len() as u16 + 4);
    frame.render_widget(Clear, rect);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(tokens.border)
        .title(Span::styled(
            " Keys ".to_owned(),
            tokens.accent.add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let mut lines: Vec<Line<'static>> = KEYS
        .iter()
        .map(|(key, what)| {
            Line::from(vec![
                Span::styled(
                    format!(" {key:<18}"),
                    tokens.text.add_modifier(Modifier::BOLD),
                ),
                Span::styled((*what).to_owned(), tokens.muted),
            ])
        })
        .collect();
    lines.push(Line::default());
    lines.push(Line::styled(
        " press any key to close".to_owned(),
        tokens.muted,
    ));
    frame.render_widget(Paragraph::new(Text::from(lines)), inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Msg;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use fireside_core::{ContentBlock, Graph};
    use fireside_engine::Session;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    /// A node with only non-editable content — a `code` block, plus a
    /// container whose children are `image`/`divider` (no heading/text
    /// anywhere, including nested).
    const NOTHING_TO_EDIT: &str = r#"{
        "fireside-version": "0.1.0",
        "title": "fixture",
        "nodes": [
            {
                "id": "only",
                "content": [
                    { "kind": "code", "language": "text", "source": "no text here" },
                    { "kind": "container", "children": [
                        { "kind": "image", "src": "diagram.png" },
                        { "kind": "divider" }
                    ]}
                ]
            }
        ]
    }"#;

    fn press_with(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
        app.update(Msg::Terminal(Event::Key(KeyEvent::new(code, modifiers))));
    }

    const HELLO: &str = include_str!("../../../../docs/examples/hello.json");

    fn app() -> App {
        let graph = Graph::from_json(HELLO).expect("hello parses");
        App::new(Session::new(graph).expect("non-empty"))
    }

    fn press(app: &mut App, code: KeyCode) {
        app.update(Msg::Terminal(Event::Key(KeyEvent::from(code))));
    }

    /// Render the app to a plain-text screen, lines joined by '\n'.
    fn screen(app: &App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("backend");
        terminal.draw(|f| draw(f, app)).expect("draw");
        let buffer = terminal.backend().buffer().clone();
        let mut out = String::new();
        for y in 0..height {
            for x in 0..width {
                out.push_str(buffer[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn first_slide_shows_title_content_and_footer_contract() {
        let app = app();
        let s = screen(&app, 80, 24);
        assert!(s.contains("Hello, Fireside"), "deck content visible");
        assert!(s.contains("1/6 seen"), "progress visible");
        assert!(s.contains("Space next"), "footer teaches the basics");
        assert!(s.contains("? help"));
    }

    #[test]
    fn branch_point_renders_as_a_menu_with_selection() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' ')); // features
        press(&mut app, KeyCode::Char(' ')); // choose
        let s = screen(&app, 80, 24);
        assert!(s.contains("What would you like to explore?"));
        assert!(s.contains("▸"), "selection marker visible");
        assert!(s.contains("1.  Code demo "));
        assert!(s.contains("[a]"), "author hotkey visible");
        assert!(s.contains("Enter go"), "footer switches to branch keys");
    }

    #[test]
    fn space_at_branch_flashes_guidance_instead_of_moving() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char(' ')); // blocked
        assert_eq!(app.session().current().id, "choose");
        let s = screen(&app, 80, 24);
        assert!(s.contains("asks for a choice"), "got: {s}");
    }

    #[test]
    fn arrows_and_enter_choose_an_option() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Down); // -> Layout demo
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.session().current().id, "layout-demo");
        let s = screen(&app, 100, 30);
        assert!(s.contains("Left column"));
        assert!(s.contains("Right column"));
        let left = s.find("Left column").expect("left");
        let right = s.find("Right column").expect("right");
        let row_of = |pos: usize| s[..pos].matches('\n').count();
        assert_eq!(row_of(left), row_of(right), "columns share a row");
    }

    #[test]
    fn author_hotkey_jumps_straight_to_target() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('c')); // Finish -> thanks
        assert_eq!(app.session().current().id, "thanks");
    }

    #[test]
    fn terminal_node_shows_end_marker_and_next_flashes() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('3')); // quick-pick Finish
        let s = screen(&app, 80, 24);
        assert!(s.contains("■"), "end mark visible");
        assert!(s.contains("End of this path"));
        press(&mut app, KeyCode::Char(' '));
        let s = screen(&app, 80, 24);
        assert!(s.contains("End of this path — ← goes back"));
        assert_eq!(app.session().current().id, "thanks");
    }

    #[test]
    fn the_ending_is_centered_not_left_aligned() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('c')); // thanks (terminal)
        let s = screen(&app, 80, 24);
        // The header mini-rail also carries a ■; the end marker is the one
        // set off with spaces inside the card.
        let line = s.lines().find(|l| l.contains(" ■ ")).expect("end mark row");
        let lead = line.chars().take_while(|c| *c == ' ' || *c == '│').count();
        assert!(lead > 20, "end mark sits centered, lead was {lead}");
        let text = s
            .lines()
            .find(|l| l.contains("End of this path"))
            .expect("closing text row");
        assert!(
            text.trim_start_matches(['│', ' '])
                .starts_with("End of this path")
        );
    }

    #[test]
    fn t_toggles_the_elapsed_timer() {
        let mut app = app();
        let s = screen(&app, 80, 24);
        assert!(!s.contains("0:00"), "timer hidden by default");
        press(&mut app, KeyCode::Char('t'));
        let s = screen(&app, 80, 24);
        assert!(s.contains("0:00"), "timer visible after t: {s}");
        press(&mut app, KeyCode::Char('t'));
        let s = screen(&app, 80, 24);
        assert!(!s.contains("0:00"), "t hides it again");
    }

    #[test]
    fn timer_survives_fullscreen_and_flash() {
        let mut app = app();
        press(&mut app, KeyCode::Char('t'));
        press(&mut app, KeyCode::Backspace); // flashes "Already at the first slide"
        let s = screen(&app, 80, 24);
        assert!(s.contains("Already at the first slide"), "flash shows");
        assert!(s.contains("0:00"), "timer keeps its corner during a flash");
        press(&mut app, KeyCode::Char('f'));
        let s = screen(&app, 80, 24);
        assert!(s.contains("0:00"), "timer visible in fullscreen");
    }

    #[test]
    fn every_scene_renders_at_60x18() {
        // Walk the whole deck at a small size: no panics, key content visible.
        let mut app = app();
        let s = screen(&app, 60, 18);
        assert!(s.contains("Hello, Fireside"));
        press(&mut app, KeyCode::Char(' ')); // features
        let s = screen(&app, 60, 18);
        assert!(s.contains("Core Features"));
        press(&mut app, KeyCode::Char(' ')); // choose
        let s = screen(&app, 60, 18);
        assert!(s.contains("▸"), "branch menu renders");
        press(&mut app, KeyCode::Char('b')); // layout-demo (columns)
        let s = screen(&app, 60, 18);
        assert!(s.contains("Left column"), "columns content present: {s}");
        press(&mut app, KeyCode::Char('m'));
        let s = screen(&app, 60, 18);
        assert!(s.contains("Map — Enter jumps"), "map overlay fits");
        press(&mut app, KeyCode::Esc);
        press(&mut app, KeyCode::Char('?'));
        let s = screen(&app, 60, 18);
        assert!(s.contains(" Keys "), "help overlay fits");
    }

    #[test]
    fn reload_swaps_the_deck_and_stays_on_the_current_slide() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' ')); // features
        let edited = HELLO.replace("Core Features", "Fresh Features");
        let graph = Graph::from_json(&edited).expect("edited deck parses");
        app.update(Msg::Reload(Ok(graph)));
        assert_eq!(
            app.session().current().id,
            "features",
            "still on the same slide"
        );
        let s = screen(&app, 80, 24);
        assert!(s.contains("Fresh Features"), "new content visible: {s}");
        assert!(s.contains("Reloaded"), "footer confirms the reload");
    }

    #[test]
    fn reload_with_a_broken_save_keeps_the_working_deck() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        app.update(Msg::Reload(Err(
            "Reload failed — hello.json:3:7 — expected `,`".into(),
        )));
        let s = screen(&app, 80, 24);
        assert!(s.contains("Core Features"), "old deck still presented");
        assert!(
            s.contains("Reload failed — hello.json:3:7"),
            "footer explains"
        );
    }

    #[test]
    fn reload_with_validation_errors_keeps_the_working_deck() {
        let mut app = app();
        let broken = HELLO.replace(
            "\"traversal\": \"features\"",
            "\"traversal\": \"missing-slide\"",
        );
        let graph = Graph::from_json(&broken).expect("broken deck still parses");
        app.update(Msg::Reload(Ok(graph)));
        let s = screen(&app, 80, 24);
        assert!(s.contains("Hello, Fireside"), "old deck still presented");
        assert!(s.contains("Reload skipped"), "footer explains: {s}");
    }

    #[test]
    fn reload_that_removed_the_current_slide_returns_to_start() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' ')); // features
        let edited = HELLO
            .replace("\"id\": \"features\"", "\"id\": \"renamed\"")
            .replace("\"traversal\": \"features\"", "\"traversal\": \"renamed\"");
        let graph = Graph::from_json(&edited).expect("edited deck parses");
        app.update(Msg::Reload(Ok(graph)));
        assert_eq!(app.session().current().id, "intro", "back at the entry");
        let s = screen(&app, 80, 24);
        assert!(
            s.contains("is gone, back at the start"),
            "footer explains: {s}"
        );
    }

    #[test]
    fn resize_event_updates_scroll_geometry() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('a')); // code-demo
        app.update(Msg::Terminal(Event::Resize(60, 12)));
        // Scrolling clamps against the new, smaller viewport without panics.
        for _ in 0..50 {
            press(&mut app, KeyCode::Down);
        }
        let s = screen(&app, 60, 12);
        assert!(s.contains("│"), "code box still on screen");
    }

    #[test]
    fn back_walks_the_real_path_and_start_flashes() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Backspace);
        assert_eq!(app.session().current().id, "intro");
        press(&mut app, KeyCode::Backspace);
        let s = screen(&app, 80, 24);
        assert!(s.contains("Already at the first slide"));
    }

    #[test]
    fn fullscreen_node_hides_header_and_f_toggles_back() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('a')); // code-demo, view-mode fullscreen
        let s = screen(&app, 80, 24);
        assert!(!s.contains("1/6 seen"), "fullscreen hides the header");
        assert!(s.contains("fn main()"), "code visible");
        press(&mut app, KeyCode::Char('f')); // back to standard
        let s = screen(&app, 80, 24);
        assert!(s.contains("seen"), "header is back");
    }

    #[test]
    fn map_lists_slides_marks_progress_and_jumps() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' ')); // features
        press(&mut app, KeyCode::Char('m'));
        let s = screen(&app, 80, 24);
        assert!(s.contains("Map — Enter jumps"));
        assert!(s.contains("●"), "visited station");
        assert!(s.contains("◉"), "current station");
        assert!(s.contains("○"), "unvisited station");
        // Jump to the last slide.
        for _ in 0..5 {
            press(&mut app, KeyCode::Down);
        }
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.session().current().id, "thanks");
        // Back returns to where the jump came from (history, not order).
        press(&mut app, KeyCode::Backspace);
        assert_eq!(app.session().current().id, "features");
    }

    #[test]
    fn map_draws_the_fork_with_its_option_keys() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('m'));
        let s = screen(&app, 80, 24);
        assert!(s.contains("├"), "fork junction drawn: {s}");
        assert!(s.contains("╮"), "branch lane opens: {s}");
        assert!(s.contains("╯"), "branch lane rejoins: {s}");
        assert!(s.contains("[a]"), "option key legend: {s}");
        assert!(s.contains("[c]"), "all option keys shown: {s}");
        assert!(s.contains("you are here"), "glyph legend shown: {s}");
    }

    #[test]
    fn header_rule_carries_the_mini_rail() {
        let mut app = app();
        let s = screen(&app, 80, 24);
        let rail = s.lines().nth(1).expect("rule row");
        assert!(rail.contains("◉"), "current station on the rule: {rail}");
        press(&mut app, KeyCode::Char(' '));
        let s = screen(&app, 80, 24);
        let rail = s.lines().nth(1).expect("rule row");
        assert!(rail.contains("●───◉"), "travelled track then you: {rail}");
    }

    #[test]
    fn the_ending_lists_the_route_travelled() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('c')); // straight to thanks
        let s = screen(&app, 80, 24);
        assert!(s.contains("→"), "path trace shown on the ending: {s}");
    }

    #[test]
    fn help_overlay_opens_and_any_key_closes() {
        let mut app = app();
        press(&mut app, KeyCode::Char('?'));
        let s = screen(&app, 80, 24);
        assert!(s.contains(" Keys "));
        assert!(s.contains("map — see and jump anywhere"));
        press(&mut app, KeyCode::Char('x'));
        assert_eq!(*app.screen(), Screen::Present);
        assert_eq!(
            app.session().current().id,
            "intro",
            "closing help moved nothing"
        );
    }

    #[test]
    fn speaker_notes_toggle_and_absence_flashes() {
        let mut app = app();
        press(&mut app, KeyCode::Char('s')); // intro has no notes
        let s = screen(&app, 80, 24);
        assert!(s.contains("no speaker notes"));
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('a')); // code-demo has notes
        press(&mut app, KeyCode::Char('f')); // standard frame for the panel
        press(&mut app, KeyCode::Char('s'));
        let s = screen(&app, 80, 24);
        assert!(s.contains("Notes — s hides"));
        assert!(s.contains("Demonstrate fullscreen view mode"));
    }

    #[test]
    fn q_quits() {
        let mut app = app();
        press(&mut app, KeyCode::Char('q'));
        assert!(app.should_quit());
    }

    #[test]
    fn tiny_terminal_degrades_gracefully() {
        let app = app();
        let s = screen(&app, 9, 3);
        assert!(s.contains("Too small"));
    }

    /// Render and return the raw buffer for style-level assertions.
    fn buffer(app: &App, width: u16, height: u16) -> ratatui::buffer::Buffer {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("backend");
        terminal.draw(|f| draw(f, app)).expect("draw");
        terminal.backend().buffer().clone()
    }

    /// The (x, y) of the first cell where `needle` starts on screen.
    fn locate(buf: &ratatui::buffer::Buffer, width: u16, height: u16, needle: &str) -> (u16, u16) {
        for y in 0..height {
            let row: String = (0..width).map(|x| buf[(x, y)].symbol()).collect();
            if let Some(col) = row.find(needle) {
                let x = row[..col].chars().count() as u16;
                return (x, y);
            }
        }
        panic!("{needle:?} not on screen");
    }

    #[test]
    fn default_view_frames_the_slide_in_a_rounded_card() {
        let app = app();
        let s = screen(&app, 80, 24);
        assert!(s.contains('╭') && s.contains('╰'), "card corners visible");
        assert!(s.contains("─────"), "header rule visible");
    }

    #[test]
    fn the_card_is_the_same_stage_on_every_slide() {
        let mut app = app();
        let frame = |app: &App| {
            let buf = buffer(app, 80, 24);
            let top = locate(&buf, 80, 24, "╭");
            let bottom = locate(&buf, 80, 24, "╰");
            (top, bottom)
        };
        let first = frame(&app);
        press(&mut app, KeyCode::Char(' ')); // a slide with more content
        let second = frame(&app);
        assert_eq!(
            first, second,
            "the card frame must not resize between slides"
        );
    }

    #[test]
    fn wide_terminals_keep_a_readable_measure() {
        let app = app();
        let buf = buffer(&app, 200, 40);
        let (x, _) = locate(&buf, 200, 40, "╭");
        // Card is capped at MEASURE + chrome (84), centered: left edge at 58.
        assert_eq!(x, 58, "card centered at the measure cap, not full width");
    }

    #[test]
    fn fullscreen_uses_the_full_width_not_the_measure() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('a')); // code-demo is fullscreen
        let s = screen(&app, 120, 30);
        assert!(!s.contains('╭'), "no card in fullscreen");
        let rule_row = s
            .lines()
            .find(|l| l.contains("─ rust "))
            .expect("code header rule");
        assert!(
            rule_row.trim_end().chars().count() > 100,
            "code box spans the width"
        );
    }

    #[test]
    fn code_gets_syntax_colors_from_the_theme() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('a')); // code-demo
        let (w, h) = (100, 30);
        let buf = buffer(&app, w, h);
        let (x, y) = locate(&buf, w, h, "fn main");
        assert_eq!(
            buf[(x, y)].style().fg,
            Some(ratatui::style::Color::Magenta),
            "keywords use the keyword token"
        );
    }

    #[test]
    fn highlight_lines_dim_the_rest_and_keep_focus_bright() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('a')); // code-demo highlights lines 2-3
        let (w, h) = (100, 30);
        let buf = buffer(&app, w, h);
        let (x1, y1) = locate(&buf, w, h, "fn main");
        assert!(
            buf[(x1, y1)].style().add_modifier.contains(Modifier::DIM),
            "unhighlighted line is dimmed"
        );
        let (x2, y2) = locate(&buf, w, h, "let graph");
        assert!(
            !buf[(x2, y2)].style().add_modifier.contains(Modifier::DIM),
            "highlighted line keeps full brightness"
        );
    }

    #[test]
    fn fade_transition_starts_dim_and_is_only_for_fade_nodes() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' ')); // features: transition none
        assert!(!app.fading(), "no fade on transition: none");
        press(&mut app, KeyCode::Char(' '));
        press(&mut app, KeyCode::Char('c')); // thanks: transition fade
        assert!(app.fading(), "fade node enters its fade window");
        let (w, h) = (80, 24);
        let buf = buffer(&app, w, h);
        let (x, y) = locate(&buf, w, h, "Thanks!");
        assert!(
            buf[(x, y)].style().add_modifier.contains(Modifier::DIM),
            "slide starts dim during the fade"
        );
    }

    #[test]
    fn quick_edit_open_edit_save_updates_the_heading_and_leaves_other_blocks_alone() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' ')); // -> features
        assert_eq!(app.session().current().id, "features");

        press(&mut app, KeyCode::Char('e'));
        assert!(
            matches!(app.screen(), Screen::Edit { .. }),
            "e opens the modal: {:?}",
            app.screen()
        );

        // Cursor starts at (0, 0) on the first field (the heading) —
        // inserting a char prepends it.
        press(&mut app, KeyCode::Char('X'));
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);

        assert!(
            matches!(app.screen(), Screen::Edit { .. }),
            "the modal stays open until the write-back sink's result arrives"
        );
        let saved = app
            .take_pending_save()
            .expect("a save produces a pending graph");
        // The event loop hands the sink's outcome back via `Msg::SaveResult`;
        // here we simulate a successful write.
        app.update(Msg::SaveResult(Ok(())));
        assert_eq!(*app.screen(), Screen::Present, "a successful save closes the modal");

        let node = saved.node("features").expect("features node still exists");
        match &node.content[0] {
            ContentBlock::Heading { text, .. } => {
                assert_eq!(text, "XCore Features");
            }
            other => panic!("expected the heading block, got {other:?}"),
        }
        // The other editable block (the trailing text) is untouched.
        match &node.content[3] {
            ContentBlock::Text { body } => {
                assert_eq!(
                    body,
                    "Every edge is explicit. No implicit sequential fallback."
                );
            }
            other => panic!("expected the text block, got {other:?}"),
        }
        // Non-editable siblings on the same node are untouched too.
        assert!(matches!(node.content[1], ContentBlock::List { .. }));
        assert!(matches!(node.content[2], ContentBlock::Divider));
    }

    #[test]
    fn quick_edit_cancel_leaves_the_session_and_pending_save_untouched() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' ')); // -> features
        press(&mut app, KeyCode::Char('e'));
        press(&mut app, KeyCode::Char('X'));
        press(&mut app, KeyCode::Esc);

        assert_eq!(*app.screen(), Screen::Present, "esc closes the modal");
        assert!(
            app.take_pending_save().is_none(),
            "cancel must not produce a save"
        );
        assert_eq!(
            app.session().current().content[0],
            ContentBlock::Heading {
                level: 2,
                text: "Core Features".to_owned(),
            },
            "cancel must not mutate the live session"
        );
    }

    #[test]
    fn quick_edit_save_failure_keeps_the_modal_open_for_retry_or_cancel() {
        let mut app = app();
        press(&mut app, KeyCode::Char(' ')); // -> features
        press(&mut app, KeyCode::Char('e'));
        press(&mut app, KeyCode::Char('X'));
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        app.take_pending_save().expect("save produced a graph");

        // Simulate the write-back sink refusing the save (conflict, I/O
        // error, or no backing file) — the presenter's edit must not be
        // silently discarded (FR-013): the modal stays open so they can
        // retry (Ctrl+S again) or abandon (Esc).
        app.update(Msg::SaveResult(Err("Save skipped — the file changed on disk; Ctrl+S again to overwrite, Esc to discard your edit".to_owned())));
        assert!(
            matches!(app.screen(), Screen::Edit { .. }),
            "a failed save must not close the modal or discard the edit"
        );
        let s = screen(&app, 80, 24);
        assert!(s.contains("changed on disk"), "the failure is shown: {s}");

        // Retry: the presenter presses save again and it succeeds.
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        let saved = app
            .take_pending_save()
            .expect("retry produces a pending save with the same edit");
        app.update(Msg::SaveResult(Ok(())));
        assert_eq!(*app.screen(), Screen::Present, "a successful retry closes the modal");
        match &saved.node("features").expect("features node").content[0] {
            ContentBlock::Heading { text, .. } => assert_eq!(text, "XCore Features"),
            other => panic!("expected the heading block, got {other:?}"),
        }
    }

    #[test]
    fn quick_edit_save_never_touches_other_nodes_or_branch_structure() {
        let original = Graph::from_json(HELLO).expect("hello parses");

        let mut app = app();
        press(&mut app, KeyCode::Char(' ')); // -> features
        press(&mut app, KeyCode::Char('e'));
        press(&mut app, KeyCode::Char('X'));
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        let saved = app.take_pending_save().expect("save produced a graph");

        for node in &original.nodes {
            if node.id == "features" {
                continue;
            }
            let edited = saved.node(&node.id).unwrap_or_else(|| {
                panic!("node {} must still exist after an unrelated save", node.id)
            });
            assert_eq!(
                edited, node,
                "node {} must be untouched by a save on a different node",
                node.id
            );
        }
    }

    #[test]
    fn quick_edit_on_a_node_with_nothing_editable_flashes_instead_of_opening() {
        let graph = Graph::from_json(NOTHING_TO_EDIT).expect("fixture parses");
        let mut app = App::new(Session::new(graph).expect("non-empty"));

        press(&mut app, KeyCode::Char('e'));

        assert_eq!(*app.screen(), Screen::Present, "no modal opens");
        let s = screen(&app, 80, 24);
        assert!(
            s.contains("no editable text"),
            "expected a clear message: {s}"
        );
    }

    #[test]
    fn present_watching_refuses_saves_with_unavailable() {
        // `present`/`present_watching` (used by `fireside demo`, which has
        // no backing file) resolve internally to a sink that always
        // returns `Unavailable` — exercised directly here without a live
        // terminal, per quickstart.md scenario 4.
        let sink: crate::WriteBackSink<'_> = &mut |_| Err(crate::WriteBackError::Unavailable);
        let graph = Graph::from_json(HELLO).expect("hello parses");
        let err = sink(&graph).expect_err("the stub sink always refuses");
        assert_eq!(err, crate::WriteBackError::Unavailable);
    }

    #[test]
    fn save_result_flashes_a_distinct_message_for_every_write_back_error() {
        for (error, expect_contains) in [
            (crate::WriteBackError::Unavailable, "no file to save to"),
            (crate::WriteBackError::Conflict, "changed on disk"),
            (crate::WriteBackError::Io("disk full".to_owned()), "disk full"),
        ] {
            let mut app = app();
            app.update(Msg::SaveResult(Err(error.to_string())));
            let s = screen(&app, 80, 24);
            assert!(
                s.contains(expect_contains),
                "expected a message containing {expect_contains:?}: {s}"
            );
        }

        let mut app = app();
        app.update(Msg::SaveResult(Ok(())));
        let s = screen(&app, 80, 24);
        assert!(s.contains("Saved"), "{s}");
    }
}
