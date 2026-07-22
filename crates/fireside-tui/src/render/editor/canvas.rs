//! The canvas pane: the selected slide, rendered through the exact same
//! `content::draw_content` path the presenter uses (spec 013's WYSIWYG
//! guarantee, spec SC-008) — chrome-free until selection glow and hover
//! cues land (US1/US2). Every block always renders regardless of reveal
//! step: the editor shows the whole slide at once, badges (not omission)
//! are how later stories (US3, T053) will mark staged content.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::editor::EditorApp;
use crate::editor::hit;
use crate::render::content::{SlideView, draw_content};
use crate::theme::Tokens;

pub(super) fn draw(frame: &mut Frame, area: Rect, app: &EditorApp, tokens: &Tokens) {
    let Some(node) = hit::selected_node(app) else {
        frame.render_widget(
            Paragraph::new(Span::styled("This deck has no slides yet.", tokens.muted)),
            area,
        );
        return;
    };
    let view_mode = node.resolved_view_mode(app.working_graph().defaults.as_ref());
    let view = SlideView {
        node,
        reveal_level: u32::MAX,
        has_pending_reveal: false,
        branch_selected: 0,
        fading: false,
        scroll: app.scroll(),
        view_mode,
        history_titles: Vec::new(),
    };
    draw_content(frame, area, &view, tokens);
}
