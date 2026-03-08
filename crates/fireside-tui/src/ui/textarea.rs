//! Minimal multi-line text-area widget for the block editor popup.
//!
//! Handles cursor movement, text insertion/deletion, and renders itself as a
//! bordered [`Paragraph`] with the cursor position marked by an inverted-video
//! span.  No external crate dependency required.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::theme::Theme;

/// A simple multi-line text editor with a visible cursor.
///
/// Store one of these in `App` state; pass it to `render_textarea_popup` to
/// draw it.  Route [`crossterm::event::KeyEvent`]s through `input()`.
#[derive(Debug, Clone)]
pub struct TextArea {
    /// Lines of text (always at least one element).
    lines: Vec<String>,
    /// Zero-based row of the cursor.
    cursor_row: usize,
    /// Zero-based byte offset of the cursor within the current line.
    cursor_col: usize,
    /// Whether Enter inserts a newline (`true`) or should be handled by the
    /// caller as a commit signal (`false`).
    pub multiline: bool,
    /// Label shown in the popup title bar.
    pub label: String,
}

impl TextArea {
    /// Create a new `TextArea` seeded with `content`.
    ///
    /// If `multiline` is `false` only the first line of `content` is kept.
    #[must_use]
    pub fn new(content: &str, multiline: bool, label: impl Into<String>) -> Self {
        let raw_lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|l| l.to_string()).collect()
        };
        let lines = if multiline {
            raw_lines
        } else {
            vec![raw_lines.into_iter().next().unwrap_or_default()]
        };
        let last_row = lines.len().saturating_sub(1);
        let last_col = lines.last().map(|l| l.len()).unwrap_or(0);
        Self {
            lines,
            cursor_row: last_row,
            cursor_col: last_col,
            multiline,
            label: label.into(),
        }
    }

    /// Return the full text content (lines joined by `\n`).
    #[must_use]
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Return just the first line (useful for single-line metadata fields).
    #[must_use]
    pub fn first_line(&self) -> &str {
        self.lines.first().map(|s| s.as_str()).unwrap_or("")
    }

    /// Process a key event.  Returns `true` if the event was consumed.
    ///
    /// The caller must intercept **Esc** and **Ctrl+C** before calling this
    /// so those keys trigger commit / cancel.  For single-line areas **Enter**
    /// is also handled by the caller as a commit signal.
    pub fn input(&mut self, key: KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        match key.code {
            // ── Cursor movement ───────────────────────────────────────────
            KeyCode::Left => {
                self.move_left();
                true
            }
            KeyCode::Right => {
                self.move_right();
                true
            }
            KeyCode::Up => {
                self.move_up();
                true
            }
            KeyCode::Down => {
                self.move_down();
                true
            }
            KeyCode::Home | KeyCode::Char('a') if ctrl => {
                self.cursor_col = 0;
                true
            }
            KeyCode::End | KeyCode::Char('e') if ctrl => {
                self.cursor_col = self.current_line().len();
                true
            }
            // ── Newline (multiline only) ───────────────────────────────────
            KeyCode::Enter if self.multiline => {
                let tail = self.current_line()[self.cursor_col..].to_string();
                self.lines[self.cursor_row].truncate(self.cursor_col);
                self.cursor_row += 1;
                self.lines.insert(self.cursor_row, tail);
                self.cursor_col = 0;
                true
            }
            // ── Deletion ──────────────────────────────────────────────────
            KeyCode::Backspace => {
                self.delete_before_cursor();
                true
            }
            KeyCode::Delete => {
                self.delete_at_cursor();
                true
            }
            // Kill line after cursor (Ctrl+K)
            KeyCode::Char('k') if ctrl => {
                let line = &mut self.lines[self.cursor_row];
                line.truncate(self.cursor_col);
                true
            }
            // ── Character input ───────────────────────────────────────────
            KeyCode::Char(ch) if !ctrl && !alt => {
                let pos = self.cursor_col;
                self.lines[self.cursor_row].insert(pos, ch);
                self.cursor_col += ch.len_utf8();
                true
            }
            _ => false,
        }
    }

    // ── Private cursor helpers ────────────────────────────────────────────────

    fn current_line(&self) -> &str {
        self.lines
            .get(self.cursor_row)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            // Step back one char (not just one byte).
            let prev = self.current_line()[..self.cursor_col]
                .chars()
                .next_back()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            self.cursor_col -= prev;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    fn move_right(&mut self) {
        let line_len = self.current_line().len();
        if self.cursor_col < line_len {
            let next = self.current_line()[self.cursor_col..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            self.cursor_col += next;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let new_len = self.lines[self.cursor_row].len();
            self.cursor_col = self.cursor_col.min(new_len);
        }
    }

    fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            let new_len = self.lines[self.cursor_row].len();
            self.cursor_col = self.cursor_col.min(new_len);
        }
    }

    fn delete_before_cursor(&mut self) {
        if self.cursor_col > 0 {
            let prev_char_len = self.lines[self.cursor_row][..self.cursor_col]
                .chars()
                .next_back()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            let col = self.cursor_col - prev_char_len;
            self.lines[self.cursor_row].remove(col);
            self.cursor_col = col;
        } else if self.cursor_row > 0 {
            // Merge with previous line.
            let tail = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&tail);
        }
    }

    fn delete_at_cursor(&mut self) {
        let line_len = self.current_line().len();
        if self.cursor_col < line_len {
            self.lines[self.cursor_row].remove(self.cursor_col);
        } else if self.cursor_row + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next);
        }
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

/// Render a [`TextArea`] popup over `area`, clearing the background first.
pub fn render_textarea_popup(
    frame: &mut ratatui::Frame,
    textarea: &TextArea,
    theme: &Theme,
    area: Rect,
) {
    use ratatui::widgets::Clear;
    frame.render_widget(Clear, area);

    let keys_hint = if textarea.multiline {
        " Esc=save  Ctrl+C=cancel  Enter=newline  ↑↓←→ move "
    } else {
        " Enter/Esc=save  Ctrl+C=cancel  ←→ move "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.heading_h2))
        .title(Span::styled(
            format!(" ✎ {} ", textarea.label),
            Style::default()
                .fg(theme.heading_h2)
                .add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Span::styled(keys_hint, Style::default().fg(theme.footer)));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build styled lines with cursor highlight.
    let body_style = Style::default().fg(theme.foreground).bg(theme.surface);
    let cursor_style = Style::default()
        .fg(theme.surface)
        .bg(theme.heading_h3)
        .add_modifier(Modifier::BOLD);
    let cursor_line_bg = Style::default().bg(theme.toolbar_bg);

    let lines: Vec<Line<'_>> = textarea
        .lines
        .iter()
        .enumerate()
        .map(|(row, line)| {
            if row == textarea.cursor_row {
                // Split around the cursor character for the highlight span.
                let before = &line[..textarea.cursor_col];
                let rest = &line[textarea.cursor_col..];
                let (cursor_char, after) = if rest.is_empty() {
                    (" ", "")
                } else {
                    let ch_len = rest.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                    (&rest[..ch_len], &rest[ch_len..])
                };

                // Pad before-cursor so the whole cursor line fills the area.
                let before_width = before.width();
                let cursor_width = cursor_char.width();
                let after_width = after.width();
                let used = before_width + cursor_width + after_width;
                let pad = (inner.width as usize).saturating_sub(used);

                Line::from(vec![
                    Span::styled(before.to_string(), cursor_line_bg),
                    Span::styled(cursor_char.to_string(), cursor_style),
                    Span::styled(after.to_string(), cursor_line_bg),
                    Span::styled(" ".repeat(pad), cursor_line_bg),
                ])
            } else {
                Line::from(Span::styled(line.clone(), body_style))
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .style(body_style)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}
