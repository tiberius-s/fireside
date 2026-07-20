//! Drawing the presenter.
//!
//! The renderer is pure: it reads [`App`] state and paints a frame. All
//! content renders through [`blocks`] into a flat line flow, so the page
//! can be vertically centered when it fits and scrolled when it does not.
//! The footer always shows exactly the keys that are valid right now —
//! that contract is what makes the presenter learnable without a manual.

pub mod blocks;
mod content;
mod footer;
mod header;
mod hits;
mod map;
pub mod markdown;
mod overlays;
pub mod syntax;

/// Expands tab characters to the next 4-column tab stop (P1-3): ratatui
/// drops raw `\t` from spans, so an unexpanded tab in a code block or text
/// body renders with indentation silently deleted. Column position tracks
/// display width (`unicode_width`), not byte/char count, so a tab after a
/// wide glyph still lands on the right stop. Tab stops reset at the start
/// of each line — they don't carry across a `\n`.
pub(crate) fn expand_tabs(text: &str) -> String {
    const TAB_STOP: usize = 4;
    text.split('\n')
        .map(|line| {
            let mut out = String::with_capacity(line.len());
            let mut col = 0usize;
            for ch in line.chars() {
                if ch == '\t' {
                    let spaces = TAB_STOP - (col % TAB_STOP);
                    out.push_str(&" ".repeat(spaces));
                    col += spaces;
                } else {
                    out.push(ch);
                    col += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                }
            }
            out
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod expand_tabs_tests {
    use super::expand_tabs;

    #[test]
    fn tab_advances_to_the_next_four_column_stop() {
        assert_eq!(expand_tabs("a\tb"), "a   b");
        assert_eq!(expand_tabs("abc\td"), "abc d");
        assert_eq!(expand_tabs("abcd\te"), "abcd    e");
    }

    #[test]
    fn each_line_resets_its_own_tab_stops() {
        assert_eq!(expand_tabs("ab\tc\nabc\td"), "ab  c\nabc d");
    }

    #[test]
    fn no_tabs_is_unchanged() {
        assert_eq!(expand_tabs("plain text"), "plain text");
    }
}

use fireside_core::ViewMode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Paragraph;

use crate::app::{App, Screen};
use crate::theme::Tokens;

use content::indicator;
pub use hits::{branch_option_hit, map_row_hit};

/// The widest comfortable reading measure for slide content, in columns.
const MEASURE: u16 = 76;
/// Columns of padding between the card border and the content.
const PAD_X: u16 = 3;
/// Rows of padding between the card border and the content.
const PAD_Y: u16 = 1;

/// Paint one frame.
pub fn draw(frame: &mut Frame, app: &App) {
    let tokens = Tokens::default();
    // Every link fragment parsed this frame registers its URL under a
    // fresh index (`markdown::register_link`) — clearing first means a
    // link's index (and thus its `Tokens::link` marker style) never
    // accidentally survives from the previous frame's registry.
    markdown::reset_links();
    let area = frame.area();
    if area.width < 10 || area.height < 4 {
        frame.render_widget(Paragraph::new("Too small"), area);
        return;
    }

    let (header, content_area, footer) = areas(app.view_mode(), area);
    let (mut content_area, footer) = grow_footer_for_flash(app, content_area, footer);

    if let Some(header) = header {
        header::draw_header(frame, header, app, &tokens);
    }

    if let Some(notes) = content::notes_panel(app, content_area) {
        content_area.height = content_area.height.saturating_sub(notes.height);
        content::draw_notes(frame, notes, app, &tokens);
    }

    content::draw_content(frame, content_area, app, &tokens);
    footer::draw_footer(frame, footer, app, &tokens);

    match app.screen() {
        Screen::Present => {}
        Screen::Help => overlays::draw_help(frame, area, &tokens),
        Screen::Map { selected } => map::draw(frame, area, app, *selected, &tokens),
        Screen::Edit { fields, focused } => {
            overlays::draw_edit(frame, area, fields, *focused, &tokens);
        }
    }

    apply_hyperlinks(frame.buffer_mut());
}

/// Rewrites every contiguous run of [`Tokens::link`]-styled cells in the
/// frame's buffer into a real OSC 8 hyperlink: the run's first cell gets
/// the OSC 8 open sequence + the run's visible text + OSC 8 close, with
/// [`CellDiffOption::ForcedWidth`] set to the run's real column width —
/// ratatui's buffer-diff iterator (`BufferDiff::next`) advances past
/// exactly `width - 1` further cells whenever it sees `ForcedWidth`, the
/// same mechanism it uses internally for wide (CJK) characters, so those
/// trailing cells are never independently diffed/written regardless of
/// their own content (research.md §4 in specs/007-modern-tui-leverage/).
/// They are still blanked to a single space, matching the "no double-width
/// cell followed by non-blank content" well-formedness `Buffer::diff`'s own
/// docs assume — keeping the raw buffer sane for any direct reader (tests
/// included), not just the diffed/backend path. A terminal that doesn't
/// understand OSC 8 simply doesn't act on the inert escape bytes; the
/// label prints as plain, distinctly-styled text either way (FR-013/FR-014).
fn apply_hyperlinks(buffer: &mut ratatui::buffer::Buffer) {
    use ratatui::buffer::CellDiffOption;
    use std::num::NonZeroU16;

    let area = buffer.area;
    for y in area.top()..area.bottom() {
        let mut x = area.left();
        while x < area.right() {
            let Some(index) = Tokens::link_index(buffer[(x, y)].style()) else {
                x += 1;
                continue;
            };
            let Some(url) = markdown::link_url(index) else {
                x += 1;
                continue;
            };
            let start = x;
            let mut text = String::new();
            while x < area.right() && Tokens::link_index(buffer[(x, y)].style()) == Some(index) {
                text.push_str(buffer[(x, y)].symbol());
                x += 1;
            }
            let Some(width) = NonZeroU16::new(x - start) else {
                continue;
            };
            let wrapped = format!("\u{1b}]8;;{url}\u{1b}\\{text}\u{1b}]8;;\u{1b}\\");
            let cell = &mut buffer[(start, y)];
            cell.set_symbol(&wrapped);
            cell.set_diff_option(CellDiffOption::ForcedWidth(width));
            for skip_x in (start + 1)..x {
                buffer[(skip_x, y)].set_symbol(" ");
            }
        }
    }
}

/// The largest useful scroll offset at the given terminal size. Shared with
/// `App::update` so scrolling clamps to real geometry.
#[must_use]
pub fn max_scroll(app: &App, width: u16, height: u16) -> u16 {
    let (_, body, footer) = areas(app.view_mode(), Rect::new(0, 0, width, height));
    let (mut body, _) = grow_footer_for_flash(app, body, footer);
    if let Some(notes) = content::notes_panel(app, body) {
        body.height = body.height.saturating_sub(notes.height);
    }
    let surf = surface(app.view_mode(), body);
    let total = content::node_lines(app, surf.width, &Tokens::default())
        .lines
        .len() as u16;
    total.saturating_sub(surf.height)
}

/// Shrinks `content_area` and grows `footer` by however many extra rows
/// (P1-6) a currently-showing flash needs to word-wrap without truncation
/// — borrowed from the bottom of the content area, never from the header.
/// A no-op (returns the inputs unchanged) when there's no flash or it fits
/// on one row.
fn grow_footer_for_flash(app: &App, mut content_area: Rect, mut footer: Rect) -> (Rect, Rect) {
    let needed = footer::footer_rows(app, footer.width);
    let extra = needed
        .saturating_sub(footer.height)
        .min(content_area.height);
    content_area.height -= extra;
    footer.y -= extra;
    footer.height += extra;
    (content_area, footer)
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

#[cfg(test)]
mod tests;
