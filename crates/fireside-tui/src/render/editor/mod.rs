//! Drawing the authoring editor's read-only studio (spec 013): toolbar,
//! outline, canvas, status line, hint line — chrome only until later
//! stories add selection glow, hover cues, and the block/form overlays
//! (US1–US3). The canvas draws through `content::draw_content`, the exact
//! path the presenter uses, so nothing here can drift from what
//! `fireside <deck>` would show (the WYSIWYG guarantee, spec SC-008).

mod canvas;
mod outline;

use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Wrap};

use crate::editor::EditorApp;
use crate::editor::hit::{self, MIN_HEIGHT, MIN_WIDTH, TOOLBAR_CHIPS, editor_areas};
use crate::theme::Tokens;

/// Paint one frame of the authoring studio.
pub(crate) fn draw(frame: &mut Frame, app: &EditorApp) {
    let tokens = Tokens::default();
    let area = frame.area();
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        draw_size_guard(frame, area, &tokens);
        return;
    }
    let areas = editor_areas(area);
    draw_toolbar(frame, areas.toolbar, app, &tokens);
    outline::draw(frame, areas.outline, app, &tokens);
    canvas::draw(frame, areas.canvas, app, &tokens);
    draw_status(frame, areas.status, app, &tokens);
    draw_hint(frame, areas.hint, &tokens);
    if app.showing_help() {
        draw_help(frame, area, &tokens);
    }
}

/// Below the studio's minimum usable size (spec FR-029): a single centered
/// guard, word-wrapped so the message still reads whole even well under
/// 80 columns — the panes never draw, and never overlap, beneath this.
fn draw_size_guard(frame: &mut Frame, area: Rect, tokens: &Tokens) {
    let msg = "Fireside edit needs at least an 80\u{d7}24 window \u{2014} make this one bigger";
    let width = area.width.saturating_sub(4).clamp(1, 60);
    let height = 3.min(area.height);
    let rect = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };
    frame.render_widget(
        Paragraph::new(Span::styled(msg, tokens.muted))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center),
        rect,
    );
}

/// The toolbar: the deck title (with its dirty dot) on the left, the five
/// chips right-aligned — the same rects `hit::toolbar_chip_rects` resolves
/// clicks against, so drawing and hit-testing can never disagree.
fn draw_toolbar(frame: &mut Frame, area: Rect, app: &EditorApp, tokens: &Tokens) {
    let title = app
        .working_graph()
        .title
        .clone()
        .unwrap_or_else(|| "Untitled deck".to_owned());
    let dot = if app.dirty() { " \u{25cf}" } else { "" };
    let label = format!(" {title}{dot}");
    frame.render_widget(Paragraph::new(Span::styled(label, tokens.accent)), area);

    for (action, chip_area) in hit::toolbar_chip_rects(area) {
        let label = TOOLBAR_CHIPS
            .iter()
            .find(|(a, _)| *a == action)
            .map_or("", |(_, label)| label);
        // Add-slide/save/undo aren't wired until later stories (US1/US3) —
        // muted here says "not yet actionable" without hiding the chip
        // (principle 2: the five chips are always the whole top-level
        // surface).
        let actionable = matches!(
            action,
            hit::ToolbarAction::Present | hit::ToolbarAction::Help
        );
        let hovered = app.hover() == Some(&hit::Target::ToolbarChip(action));
        let style = match (actionable, hovered) {
            (true, true) => tokens.selection,
            (true, false) => tokens.affordance,
            (false, _) => tokens.muted,
        };
        frame.render_widget(Paragraph::new(Span::styled(label, style)), chip_area);
    }
}

fn draw_status(frame: &mut Frame, area: Rect, app: &EditorApp, tokens: &Tokens) {
    let errors = app
        .status()
        .iter()
        .filter(|d| d.severity == fireside_engine::Severity::Error)
        .count();
    let (text, style) = if errors == 0 {
        (
            format!(
                "\u{2713} ready to present \u{b7} {} slides",
                app.working_graph().nodes.len()
            ),
            tokens.success,
        )
    } else {
        let word = if errors == 1 { "problem" } else { "problems" };
        (
            format!("\u{2717} won't present yet: {errors} {word}"),
            tokens.error,
        )
    };
    frame.render_widget(Paragraph::new(Span::styled(text, style)), area);
}

fn draw_hint(frame: &mut Frame, area: Rect, tokens: &Tokens) {
    frame.render_widget(
        Paragraph::new(Span::styled(
            "Click a slide or block to select \u{b7} ? shows every key",
            tokens.muted,
        )),
        area,
    );
}

fn draw_help(frame: &mut Frame, area: Rect, tokens: &Tokens) {
    let lines = vec![
        Line::from(Span::styled(
            "Editor keys",
            tokens.accent.add_modifier(Modifier::BOLD),
        )),
        Line::default(),
        Line::from("click / Enter     select a slide or block"),
        Line::from("p                 present from the selected slide"),
        Line::from("\u{2191}/\u{2193}, wheel       scroll the canvas"),
        Line::from("Esc               deselect"),
        Line::from("q                 quit"),
        Line::from("?                 this screen"),
    ];
    let rect = super::overlay_rect(area, 44, lines.len() as u16 + 2);
    frame.render_widget(Clear, rect);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(tokens.border);
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_widget(Paragraph::new(lines), inner);
}
