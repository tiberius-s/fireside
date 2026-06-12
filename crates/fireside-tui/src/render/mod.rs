//! Drawing the presenter.
//!
//! The renderer is pure: it reads [`App`] state and paints a frame. All
//! content renders through [`blocks`] into a flat line flow, so the page
//! can be vertically centered when it fits and scrolled when it does not.
//! The footer always shows exactly the keys that are valid right now —
//! that contract is what makes the presenter learnable without a manual.

pub mod blocks;
pub mod markdown;

use fireside_core::ViewMode;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::{App, FlashKind, Screen};
use crate::theme::Tokens;

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
        Screen::Map { selected } => draw_map(frame, area, app, selected, &tokens),
    }
}

/// The largest useful scroll offset at the given terminal size. Shared with
/// `App::update` so scrolling clamps to real geometry.
#[must_use]
pub fn max_scroll(app: &App, width: u16, height: u16) -> u16 {
    let (_, mut content, _) = areas(app.view_mode(), Rect::new(0, 0, width, height));
    if let Some(notes) = notes_panel(app, content) {
        content.height = content.height.saturating_sub(notes.height);
    }
    let total = node_lines(app, content.width, &Tokens::default()).len() as u16;
    total.saturating_sub(content.height)
}

/// Split the frame into header / content / footer for the view mode.
fn areas(view: ViewMode, area: Rect) -> (Option<Rect>, Rect, Rect) {
    match view {
        ViewMode::Default => {
            let [header, body, footer] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1), Constraint::Length(1)])
                    .areas(area);
            let content = body.inner(Margin { horizontal: 3, vertical: 1 });
            (Some(header), content, footer)
        }
        ViewMode::Fullscreen => {
            let [body, footer] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
            let content = body.inner(Margin { horizontal: 1, vertical: 0 });
            (None, content, footer)
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

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(deck.to_owned(), tokens.accent.add_modifier(Modifier::BOLD)),
        ])),
        area,
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(here.to_owned(), tokens.muted),
            Span::styled(format!("  ·  {seen}/{total} seen "), tokens.muted),
        ]))
        .alignment(Alignment::Right),
        area,
    );
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
            let label_style = if selected { tokens.selected } else { tokens.text };
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
        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::styled("■ End of this path".to_owned(), tokens.text.add_modifier(Modifier::BOLD)),
            Span::styled("  — ← goes back".to_owned(), tokens.muted),
        ]));
    }
    lines
}

fn draw_content(frame: &mut Frame, area: Rect, app: &App, tokens: &Tokens) {
    let lines = node_lines(app, area.width, tokens);
    let total = lines.len() as u16;

    if total <= area.height {
        // The page fits: center it vertically, like a slide.
        let pad = (area.height - total) / 2;
        let target = Rect { y: area.y + pad, height: total, ..area };
        frame.render_widget(Paragraph::new(Text::from(lines)), target);
        return;
    }

    let max = total - area.height;
    let scroll = app.scroll().min(max);
    let visible: Vec<Line<'static>> = lines
        .into_iter()
        .skip(scroll as usize)
        .take(area.height as usize)
        .collect();
    frame.render_widget(Paragraph::new(Text::from(visible)), area);

    if scroll > 0 {
        indicator(frame, area, 0, "▲", tokens);
    }
    if scroll < max {
        indicator(frame, area, area.height.saturating_sub(1), "▼ more (↓)", tokens);
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
    frame.render_widget(Paragraph::new(Span::styled(text.to_owned(), tokens.muted)), rect);
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
        return;
    }

    let session = app.session();
    let hints: &[(&str, &str)] = if session.branch_point().is_some() {
        &[("↑↓", "choose"), ("Enter", "go"), ("←", "back"), ("m", "map"), ("?", "help"), ("q", "quit")]
    } else if session.current().is_terminal() {
        &[("←", "back"), ("m", "map"), ("?", "help"), ("q", "quit")]
    } else {
        &[("Space", "next"), ("←", "back"), ("m", "map"), ("?", "help"), ("q", "quit")]
    };

    let mut spans = vec![Span::raw(" ")];
    for (i, (key, action)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ·  ".to_owned(), tokens.border));
        }
        spans.push(Span::styled((*key).to_owned(), tokens.text.add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(format!(" {action}"), tokens.muted));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
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

fn draw_help(frame: &mut Frame, area: Rect, tokens: &Tokens) {
    const KEYS: &[(&str, &str)] = &[
        ("Space / → / Enter", "next slide"),
        ("← / Backspace", "previous slide"),
        ("↑ / ↓", "pick a choice · scroll"),
        ("1–9 or a letter", "take a choice directly"),
        ("m", "map — see and jump anywhere"),
        ("f", "fullscreen on/off"),
        ("s", "speaker notes"),
        ("q", "quit"),
    ];
    let rect = overlay_rect(area, 50, KEYS.len() as u16 + 4);
    frame.render_widget(Clear, rect);
    let block = Block::bordered()
        .border_style(tokens.border)
        .title(Span::styled(" Keys ".to_owned(), tokens.accent.add_modifier(Modifier::BOLD)));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let mut lines: Vec<Line<'static>> = KEYS
        .iter()
        .map(|(key, what)| {
            Line::from(vec![
                Span::styled(format!(" {key:<18}"), tokens.text.add_modifier(Modifier::BOLD)),
                Span::styled((*what).to_owned(), tokens.muted),
            ])
        })
        .collect();
    lines.push(Line::default());
    lines.push(Line::styled(" press any key to close".to_owned(), tokens.muted));
    frame.render_widget(Paragraph::new(Text::from(lines)), inner);
}

fn draw_map(frame: &mut Frame, area: Rect, app: &App, selected: usize, tokens: &Tokens) {
    let session = app.session();
    let nodes = &session.graph().nodes;
    let rect = overlay_rect(area, 56, nodes.len() as u16 + 5);
    frame.render_widget(Clear, rect);
    let block = Block::bordered()
        .border_style(tokens.border)
        .title(Span::styled(
            " Map — Enter jumps ".to_owned(),
            tokens.accent.add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let current = &session.current().id;
    let mut lines: Vec<Line<'static>> = Vec::with_capacity(nodes.len() + 2);
    for (i, node) in nodes.iter().enumerate() {
        let marker = if node.id == *current {
            Span::styled(" ▶ ".to_owned(), tokens.accent.add_modifier(Modifier::BOLD))
        } else if session.visited().contains(&node.id) {
            Span::styled(" ✓ ".to_owned(), tokens.success)
        } else {
            Span::styled(" · ".to_owned(), tokens.muted)
        };
        let name = node.title.clone().unwrap_or_else(|| node.id.clone());
        let style = if i == selected { tokens.selected } else { tokens.text };
        let mut spans = vec![marker, Span::styled(format!(" {name} "), style)];
        if node.is_terminal() {
            spans.push(Span::styled("■".to_owned(), tokens.muted));
        }
        lines.push(Line::from(spans));
    }
    lines.push(Line::default());
    lines.push(Line::styled(" ↑↓ move · Enter jump · Esc close".to_owned(), tokens.muted));
    frame.render_widget(Paragraph::new(Text::from(lines)), inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{Event, KeyCode, KeyEvent};
    use fireside_core::Graph;
    use fireside_engine::Session;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    const HELLO: &str = include_str!("../../../../docs/examples/hello.json");

    fn app() -> App {
        let graph = Graph::from_json(HELLO).expect("hello parses");
        App::new(Session::new(graph).expect("non-empty"))
    }

    fn press(app: &mut App, code: KeyCode) {
        app.update(&Event::Key(KeyEvent::from(code)));
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
        assert!(s.contains("■ End of this path"));
        press(&mut app, KeyCode::Char(' '));
        let s = screen(&app, 80, 24);
        assert!(s.contains("End of this path — ← goes back"));
        assert_eq!(app.session().current().id, "thanks");
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
        assert!(s.contains("✓"), "visited marker");
        assert!(s.contains("▶"), "current marker");
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
    fn help_overlay_opens_and_any_key_closes() {
        let mut app = app();
        press(&mut app, KeyCode::Char('?'));
        let s = screen(&app, 80, 24);
        assert!(s.contains(" Keys "));
        assert!(s.contains("map — see and jump anywhere"));
        press(&mut app, KeyCode::Char('x'));
        assert_eq!(app.screen(), Screen::Present);
        assert_eq!(app.session().current().id, "intro", "closing help moved nothing");
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
}
