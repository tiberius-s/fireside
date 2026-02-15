//! Presenter view — composes the slide content with chrome.
//!
//! This is the main rendering component that draws the current slide
//! within the layout areas, overlaying the progress bar and optional help.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::model::layout::Layout;
use crate::model::theme::Theme;
use crate::model::{Slide, SlideDeck};
use crate::render::layout::compute_areas;
use crate::render::markdown::render_slide_content;

use super::help::render_help_overlay;
use super::progress::render_progress_bar;

/// Render the full presenter view for the current slide.
///
/// Draws the slide content in the main area, the progress bar in the footer,
/// and optionally the help overlay on top.
pub fn render_presenter(
    frame: &mut Frame,
    deck: &SlideDeck,
    current_slide: usize,
    theme: &Theme,
    show_help: bool,
    elapsed_secs: u64,
) {
    let area = frame.area();

    // Determine layout for current slide
    let slide = &deck.slides[current_slide];
    let layout = slide.layout.unwrap_or(Layout::Top);

    let areas = compute_areas(area, layout);

    // Clear background
    let bg_style = Style::default().bg(theme.background);
    let bg_block = Block::default().style(bg_style);
    frame.render_widget(bg_block, area);

    // Render slide content
    render_slide(frame, slide, theme, areas.content);

    // Render progress bar
    render_progress_bar(
        frame,
        areas.footer,
        current_slide,
        deck.slides.len(),
        elapsed_secs,
        theme,
    );

    // Render help overlay if active
    if show_help {
        render_help_overlay(frame, area, theme);
    }
}

/// Render a single slide's content into the given area.
fn render_slide(frame: &mut Frame, slide: &Slide, theme: &Theme, area: Rect) {
    let lines = render_slide_content(&slide.content, theme, area.width);

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(theme.foreground));

    frame.render_widget(paragraph, area);
}
