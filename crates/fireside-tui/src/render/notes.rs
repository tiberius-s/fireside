//! Rendering the `fireside notes` follower screen (spec 012).
//!
//! Deliberately its own small layout, not a variant of the presenter's
//! [`crate::render::draw`] — the follower has no card, no branch menu, no
//! scrolling content; it is a plain status readout. Every color still
//! flows through [`Tokens`] (Constitution Principle IV).

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};

use crate::follower::{Follower, FollowerView, NextView};
use crate::theme::Tokens;

/// Paint one follower frame.
pub(crate) fn draw(frame: &mut Frame, follower: &Follower) {
    let tokens = Tokens::default();
    let area = frame.area();
    if area.width < 10 || area.height < 4 {
        frame.render_widget(Paragraph::new("Too small"), area);
        return;
    }
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    draw_header(frame, header, &tokens);
    match follower.view() {
        FollowerView::NotRunning => draw_centered(
            frame,
            body,
            "Presenter not running — start \"fireside <deck>\" in another window",
            tokens.muted,
        ),
        FollowerView::Waiting => {
            draw_centered(frame, body, "waiting for presenter…", tokens.muted);
        }
        FollowerView::Tracking {
            title,
            notes,
            next,
            reveal,
            elapsed_secs,
        } => draw_tracking(
            frame,
            body,
            &tokens,
            title,
            notes,
            &next,
            reveal,
            elapsed_secs,
        ),
    }
    draw_footer(frame, footer, &tokens);
}

fn draw_header(frame: &mut Frame, area: Rect, tokens: &Tokens) {
    frame.render_widget(
        Paragraph::new(Line::styled(
            "Notes",
            tokens.accent.add_modifier(Modifier::BOLD),
        )),
        area,
    );
}

fn draw_centered(frame: &mut Frame, area: Rect, message: &str, style: ratatui::style::Style) {
    frame.render_widget(
        Paragraph::new(Span::styled(message, style))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        center_rows(area, 1),
    );
}

/// Vertically centers a block `rows` tall within `area` — the follower's
/// status messages are short and read better mid-screen than pinned to the
/// top, unlike the presenter's own content, which scrolls.
fn center_rows(area: Rect, rows: u16) -> Rect {
    let top = area.height.saturating_sub(rows) / 2;
    Rect {
        x: area.x,
        y: area.y + top,
        width: area.width,
        height: rows.min(area.height),
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_tracking(
    frame: &mut Frame,
    area: Rect,
    tokens: &Tokens,
    title: Option<&str>,
    notes: Option<&str>,
    next: &NextView<'_>,
    reveal: Option<(usize, usize)>,
    elapsed_secs: u64,
) {
    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::styled(
        title.unwrap_or("(untitled slide)").to_owned(),
        tokens.text.add_modifier(Modifier::BOLD),
    ));
    lines.push(Line::default());
    match notes {
        Some(text) => {
            for line in text.split('\n') {
                lines.push(Line::styled(line.to_owned(), tokens.text));
            }
        }
        None => lines.push(Line::styled("No notes for this slide", tokens.muted)),
    }
    lines.push(Line::default());
    match next {
        NextView::Title(title) => {
            lines.push(Line::from(vec![
                Span::styled("Next: ", tokens.muted),
                Span::styled((*title).to_owned(), tokens.text),
            ]));
        }
        NextView::Branch(options) => {
            lines.push(Line::styled("Choice:", tokens.muted));
            for (label, key) in options {
                let key_hint = key.map(|k| format!(" ({k})")).unwrap_or_default();
                lines.push(Line::from(vec![
                    Span::styled(format!("  {label}"), tokens.text),
                    Span::styled(key_hint, tokens.muted),
                ]));
            }
        }
        NextView::LastSlide => lines.push(Line::styled("This is the last slide", tokens.muted)),
    }
    if let Some((revealed, total)) = reveal {
        lines.push(Line::default());
        lines.push(Line::styled(
            format!("{revealed}/{total} revealed"),
            tokens.accent,
        ));
    }
    let secs = elapsed_secs;
    lines.push(Line::default());
    lines.push(Line::styled(
        format!("{}:{:02} elapsed", secs / 60, secs % 60),
        tokens.muted,
    ));

    frame.render_widget(
        Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_footer(frame: &mut Frame, area: Rect, tokens: &Tokens) {
    let spans = vec![
        Span::raw(" "),
        Span::styled("q", tokens.text.add_modifier(Modifier::BOLD)),
        Span::styled(" quit", tokens.muted),
    ];
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::follower::FollowerMsg;
    use crate::{SessionSnapshot, SessionStatus};
    use fireside_core::Graph;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use std::time::Duration;

    /// A small fixture covering every follower state this suite exercises:
    /// a branch point, a node with notes, a node without notes, and a
    /// terminal node with no next edge.
    const FIXTURE: &str = r#"{
        "nodes": [
            { "id": "intro", "title": "Welcome", "speaker-notes": "Say hi warmly",
              "traversal": "branch", "content": [] },
            { "id": "branch", "title": "Choose a path",
              "traversal": { "branch-point": { "options": [
                  { "label": "Path A", "key": "a", "target": "no-notes" },
                  { "label": "Path B", "target": "end" }
              ] } }, "content": [] },
            { "id": "no-notes", "title": "No Notes Slide", "traversal": "end", "content": [] },
            { "id": "end", "title": "The End", "content": [] }
        ]
    }"#;

    fn follower() -> Follower {
        Follower::new(Graph::from_json(FIXTURE).expect("fixture parses"))
    }

    fn running_at(node_id: &str, reveal: (usize, usize), elapsed: Duration) -> SessionStatus {
        SessionStatus::Running(SessionSnapshot {
            node_id: node_id.to_owned(),
            reveal_step: reveal.0,
            reveal_total: reveal.1,
            elapsed,
        })
    }

    fn screen(follower: &Follower, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("backend");
        terminal.draw(|f| draw(f, follower)).expect("draw");
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
    fn shows_current_title_and_notes() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running_at(
            "intro",
            (0, 0),
            Duration::from_secs(5),
        )));
        let s = screen(&f, 80, 24);
        assert!(s.contains("Welcome"), "title visible: {s}");
        assert!(s.contains("Say hi warmly"), "notes visible: {s}");
    }

    #[test]
    fn a_node_with_no_notes_says_so_plainly() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running_at(
            "no-notes",
            (0, 0),
            Duration::ZERO,
        )));
        let s = screen(&f, 80, 24);
        assert!(s.contains("No notes for this slide"), "{s}");
    }

    #[test]
    fn next_title_is_shown_for_a_plain_edge() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running_at(
            "no-notes",
            (0, 0),
            Duration::ZERO,
        )));
        let s = screen(&f, 80, 24);
        assert!(s.contains("Next:"), "{s}");
        assert!(s.contains("The End"), "{s}");
    }

    #[test]
    fn the_final_slide_says_so_instead_of_an_empty_next_field() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running_at(
            "end",
            (0, 0),
            Duration::ZERO,
        )));
        let s = screen(&f, 80, 24);
        assert!(s.contains("This is the last slide"), "{s}");
    }

    #[test]
    fn a_branch_point_lists_its_options_instead_of_a_single_next_title() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running_at(
            "branch",
            (0, 0),
            Duration::ZERO,
        )));
        let s = screen(&f, 80, 24);
        assert!(s.contains("Path A"), "{s}");
        assert!(s.contains("Path B"), "{s}");
        assert!(s.contains('a'), "the option's key hint should render: {s}");
    }

    #[test]
    fn reveal_progress_renders_when_present_and_is_omitted_when_not() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running_at(
            "intro",
            (2, 5),
            Duration::ZERO,
        )));
        let with_reveal = screen(&f, 80, 24);
        assert!(with_reveal.contains("2/5 revealed"), "{with_reveal}");

        f.update(FollowerMsg::SessionUpdate(running_at(
            "intro",
            (0, 0),
            Duration::ZERO,
        )));
        let without_reveal = screen(&f, 80, 24);
        assert!(!without_reveal.contains("revealed"), "{without_reveal}");
    }

    #[test]
    fn elapsed_time_renders_as_mm_ss() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running_at(
            "intro",
            (0, 0),
            Duration::from_secs(90),
        )));
        let s = screen(&f, 80, 24);
        assert!(s.contains("1:30 elapsed"), "{s}");
    }

    #[test]
    fn no_presenter_ever_seen_shows_the_not_running_message() {
        let f = follower();
        let s = screen(&f, 80, 24);
        assert!(s.contains("Presenter not running"), "{s}");
        assert!(s.contains("fireside <deck>"), "{s}");
    }

    #[test]
    fn a_presenter_that_stops_flips_the_view_to_not_running() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running_at(
            "intro",
            (0, 0),
            Duration::ZERO,
        )));
        assert!(screen(&f, 80, 24).contains("Welcome"));

        f.update(FollowerMsg::SessionUpdate(SessionStatus::NotRunning));
        let s = screen(&f, 80, 24);
        assert!(s.contains("Presenter not running"), "{s}");
        assert!(!s.contains("Welcome"), "no leftover slide content: {s}");
    }

    #[test]
    fn an_unresolvable_node_id_renders_a_benign_waiting_state_not_a_crash() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running_at(
            "node-from-a-newer-deck",
            (0, 0),
            Duration::ZERO,
        )));
        let s = screen(&f, 80, 24);
        assert!(s.contains("waiting for presenter"), "{s}");
    }

    #[test]
    fn a_live_edit_reload_updates_the_rendered_notes() {
        let mut f = follower();
        f.update(FollowerMsg::SessionUpdate(running_at(
            "intro",
            (0, 0),
            Duration::ZERO,
        )));
        let edited = {
            let mut graph = Graph::from_json(FIXTURE).expect("fixture parses");
            let node = graph
                .nodes
                .iter_mut()
                .find(|n| n.id == "intro")
                .expect("intro node");
            node.speaker_notes = Some("Edited live during rehearsal".to_owned());
            graph
        };
        f.update(FollowerMsg::Reload(Ok(edited)));
        let s = screen(&f, 80, 24);
        assert!(s.contains("Edited live during rehearsal"), "{s}");
    }
}
