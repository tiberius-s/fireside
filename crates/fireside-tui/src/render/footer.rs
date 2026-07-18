//! The footer: contextual key hints, flash messages, and the optional
//! elapsed-time display.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{App, FlashKind};
use crate::theme::Tokens;

pub(super) fn draw_footer(frame: &mut Frame, area: Rect, app: &App, tokens: &Tokens) {
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
    let pending_reveal = session.has_pending_reveal();
    let hints: &[(&str, &str)] = if pending_reveal {
        // Reveal always finishes before a branch menu or end-of-path
        // marker can appear, so while it's pending the only advance key
        // is "reveal", regardless of what the node would otherwise do.
        &[
            ("Space", "reveal"),
            ("←", "back"),
            ("m", "map"),
            ("e", "edit"),
            ("?", "help"),
            ("q", "quit"),
        ]
    } else if session.branch_point().is_some() {
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
    if pending_reveal && let Some((revealed, total)) = session.reveal_progress() {
        spans.push(Span::styled(
            format!("{revealed}/{total} revealed"),
            tokens.accent.add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("  ·  ".to_owned(), tokens.border));
    }
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
