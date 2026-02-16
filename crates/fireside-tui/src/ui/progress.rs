//! Progress bar widget showing node position and elapsed time.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::theme::Theme;

/// Render the progress bar in the footer area.
pub fn render_progress_bar(
    frame: &mut Frame,
    area: Rect,
    current: usize,
    total: usize,
    elapsed_secs: u64,
    theme: &Theme,
) {
    let style = Style::default().fg(theme.footer);
    let bold = style.add_modifier(Modifier::BOLD);

    let minutes = elapsed_secs / 60;
    let seconds = elapsed_secs % 60;

    let node_info = format!(" Node {} / {} ", current + 1, total);
    let time_info = format!(" {minutes:02}:{seconds:02} ");

    // Calculate padding to right-align the time
    let padding_len = (area.width as usize).saturating_sub(node_info.len() + time_info.len());
    let padding = " ".repeat(padding_len);

    let line = Line::from(vec![
        Span::styled(node_info, bold),
        Span::styled(padding, style),
        Span::styled(time_info, style),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}
