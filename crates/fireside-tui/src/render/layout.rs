//! Layout engine — calculates render areas for node content.
//!
//! Divides the terminal area into zones (content, footer) and applies
//! layout variant logic (center, fullscreen, split, etc.).

use fireside_core::Layout;
use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

/// The calculated areas for rendering a node.
#[derive(Debug, Clone, Copy)]
pub struct NodeAreas {
    /// Area for the main node content.
    pub content: Rect,
    /// Area for the footer / progress bar.
    pub footer: Rect,
}

/// Compute the node areas from the full terminal area and layout variant.
///
/// Reserves the bottom row for the footer/progress bar, and applies
/// padding and centering based on the layout variant.
#[must_use]
pub fn compute_areas(area: Rect, layout: Layout) -> NodeAreas {
    // Split into main content and footer
    let chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let raw_content = chunks[0];
    let footer = chunks[1];

    // Apply layout-specific content area adjustments
    let content = apply_layout(raw_content, layout);

    NodeAreas { content, footer }
}

/// Apply layout-specific adjustments to the content area.
fn apply_layout(area: Rect, layout: Layout) -> Rect {
    match layout {
        Layout::Center => center_rect(area, 80, 80),
        Layout::Top | Layout::Default => pad_rect(area, 2, 1),
        Layout::Title => center_rect(area, 70, 60),
        Layout::CodeFocus | Layout::Fullscreen => pad_rect(area, 1, 0),
        Layout::Blank => area,
        Layout::SplitHorizontal => pad_rect(area, 2, 1),
        Layout::SplitVertical => pad_rect(area, 2, 1),
        Layout::AlignLeft => pad_rect(area, 2, 1),
        Layout::AlignRight => pad_rect(area, 2, 1),
    }
}

/// Compute the two column split for a split-horizontal layout.
///
/// Returns (left, right) content areas with a gutter between them.
#[must_use]
pub fn two_column_split(area: Rect) -> (Rect, Rect) {
    let chunks = RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(48),
            Constraint::Percentage(4), // gutter
            Constraint::Percentage(48),
        ])
        .split(area);

    (chunks[0], chunks[2])
}

/// Center a rect within its parent, constraining to a percentage of width/height.
fn center_rect(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let vert = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_pct) / 2),
            Constraint::Percentage(height_pct),
            Constraint::Percentage((100 - height_pct) / 2),
        ])
        .split(area);

    let horiz = RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_pct) / 2),
            Constraint::Percentage(width_pct),
            Constraint::Percentage((100 - width_pct) / 2),
        ])
        .split(vert[1]);

    horiz[1]
}

/// Add padding to a rect.
fn pad_rect(area: Rect, horizontal: u16, vertical: u16) -> Rect {
    Rect {
        x: area.x + horizontal,
        y: area.y + vertical,
        width: area.width.saturating_sub(horizontal * 2),
        height: area.height.saturating_sub(vertical * 2),
    }
}
