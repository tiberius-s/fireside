//! Reusable helper functions for the editor shell view.

use fireside_core::model::content::ContentBlock;
use fireside_core::model::layout::Layout as NodeLayout;
use fireside_core::model::node::Node;
use fireside_core::model::transition::Transition;
use fireside_engine::PresentationSession;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::EditorPickerOverlay;
use crate::theme::Theme;

#[must_use]
pub(crate) fn node_label(session: &PresentationSession, index: usize) -> String {
    let prefix = format!("{:>2}. ", index + 1);
    let id = session
        .graph
        .nodes
        .get(index)
        .and_then(|node| node.id.as_deref())
        .unwrap_or("(no-id)");
    format!("{prefix}{id}")
}

#[must_use]
pub(crate) fn section_header(theme: &Theme, title: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            title,
            Style::default()
                .fg(theme.heading_h3)
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

#[must_use]
pub(crate) fn metadata_chip_row_layout(
    theme: &Theme,
    prev: NodeLayout,
    current: NodeLayout,
    next: NodeLayout,
) -> Line<'static> {
    Line::from(vec![
        Span::styled("  layout      ", Style::default().fg(theme.footer)),
        Span::styled(
            format!("◀ {}  ", layout_name(prev)),
            Style::default().fg(theme.footer),
        ),
        Span::styled(
            format!(" {} ", layout_name(current)),
            Style::default()
                .fg(theme.on_surface)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {} ▶", layout_name(next)),
            Style::default().fg(theme.footer),
        ),
        Span::styled("   L/l", Style::default().fg(theme.heading_h2)),
    ])
}

#[must_use]
pub(crate) fn metadata_chip_row_transition(
    theme: &Theme,
    prev: Transition,
    current: Transition,
    next: Transition,
) -> Line<'static> {
    Line::from(vec![
        Span::styled("  transition  ", Style::default().fg(theme.footer)),
        Span::styled(
            format!("◀ {}  ", transition_name(prev)),
            Style::default().fg(theme.footer),
        ),
        Span::styled(
            format!(" {} ", transition_name(current)),
            Style::default()
                .fg(theme.on_surface)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {} ▶", transition_name(next)),
            Style::default().fg(theme.footer),
        ),
        Span::styled("   T/t", Style::default().fg(theme.heading_h2)),
    ])
}

#[must_use]
pub(crate) fn block_type_glyph(block: &ContentBlock) -> char {
    match block {
        ContentBlock::Heading { .. } => '▸',
        ContentBlock::Text { .. } => '¶',
        ContentBlock::Code { .. } => '⌥',
        ContentBlock::List { .. } => '•',
        ContentBlock::Image { .. } => '⬛',
        ContentBlock::Divider => '─',
        ContentBlock::Container { .. } => '□',
        ContentBlock::Extension { .. } => '⎇',
    }
}

#[must_use]
pub(crate) fn block_summary(block: &ContentBlock) -> String {
    match block {
        ContentBlock::Heading { level, text } => {
            format!("heading h{level}: {}", truncate(text, 52))
        }
        ContentBlock::Text { body } => format!("text: {}", truncate(body, 56)),
        ContentBlock::Code {
            language,
            highlight_lines,
            ..
        } => {
            let lang = language.as_deref().unwrap_or("plain");
            if highlight_lines.is_empty() {
                format!("code: {lang}")
            } else {
                format!(
                    "code: {lang}  highlights [{}]",
                    highlight_lines
                        .iter()
                        .map(|line| line.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
        }
        ContentBlock::List { ordered, items } => {
            let list_type = if *ordered { "ordered" } else { "unordered" };
            format!("list: {list_type}, {} item(s)", items.len())
        }
        ContentBlock::Image { src, alt, .. } => {
            let alt_preview = if alt.trim().is_empty() {
                "(no alt)".to_string()
            } else {
                truncate(alt, 24)
            };
            format!("image: {}  alt: {alt_preview}", truncate(src, 28))
        }
        ContentBlock::Divider => "divider".to_string(),
        ContentBlock::Container { layout, children } => {
            let layout_name = layout.as_deref().unwrap_or("default");
            format!(
                "container: {layout_name}, {} child block(s)",
                children.len()
            )
        }
        ContentBlock::Extension {
            extension_type,
            fallback,
            ..
        } => {
            let fallback_blocks = usize::from(fallback.is_some());
            format!("extension: {extension_type} ({fallback_blocks} fallback block(s))")
        }
    }
}

#[must_use]
pub(crate) fn traversal_summary_lines(node: &Node, theme: &Theme) -> Vec<Line<'static>> {
    let traversal = node.traversal.as_ref();
    let next = traversal
        .and_then(|value| value.next.as_deref())
        .unwrap_or("(sequential)");
    let after = traversal
        .and_then(|value| value.after.as_deref())
        .unwrap_or("(none)");
    let mut lines = vec![
        Line::from(vec![
            Span::styled("  next", Style::default().fg(theme.footer)),
            Span::styled(format!("  {next}"), Style::default().fg(theme.foreground)),
        ]),
        Line::from(vec![
            Span::styled("  after", Style::default().fg(theme.footer)),
            Span::styled(format!("  {after}"), Style::default().fg(theme.foreground)),
        ]),
    ];

    if let Some(branch) = traversal.and_then(|value| value.branch_point.as_ref()) {
        lines.push(Line::from(vec![
            Span::styled("  branch", Style::default().fg(theme.footer)),
            Span::styled(
                format!("  {} option(s)", branch.options.len()),
                Style::default().fg(theme.heading_h2),
            ),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("  branch", Style::default().fg(theme.footer)),
            Span::styled("  (none)", Style::default().fg(theme.footer)),
        ]));
    }

    lines
}

#[must_use]
pub(crate) fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let clipped: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{clipped}…")
}

#[must_use]
pub(crate) fn popup_area(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height,
    }
}

#[must_use]
pub(crate) fn adjacent_layouts(current: NodeLayout) -> (NodeLayout, NodeLayout) {
    let variants = [
        NodeLayout::Default,
        NodeLayout::Center,
        NodeLayout::Top,
        NodeLayout::SplitHorizontal,
        NodeLayout::SplitVertical,
        NodeLayout::Title,
        NodeLayout::CodeFocus,
        NodeLayout::Fullscreen,
        NodeLayout::AlignLeft,
        NodeLayout::AlignRight,
        NodeLayout::Blank,
    ];

    let idx = variants.iter().position(|v| *v == current).unwrap_or(0);
    let prev = variants[(idx + variants.len() - 1) % variants.len()];
    let next = variants[(idx + 1) % variants.len()];
    (prev, next)
}

#[must_use]
pub(crate) fn adjacent_transitions(current: Transition) -> (Transition, Transition) {
    let variants = [
        Transition::None,
        Transition::Fade,
        Transition::SlideLeft,
        Transition::SlideRight,
        Transition::Wipe,
        Transition::Dissolve,
        Transition::Matrix,
        Transition::Typewriter,
    ];

    let idx = variants.iter().position(|v| *v == current).unwrap_or(0);
    let prev = variants[(idx + variants.len() - 1) % variants.len()];
    let next = variants[(idx + 1) % variants.len()];
    (prev, next)
}

#[must_use]
fn layout_name(layout: NodeLayout) -> &'static str {
    match layout {
        NodeLayout::Default => "default",
        NodeLayout::Center => "center",
        NodeLayout::Top => "top",
        NodeLayout::SplitHorizontal => "split-h",
        NodeLayout::SplitVertical => "split-v",
        NodeLayout::Title => "title",
        NodeLayout::CodeFocus => "code-focus",
        NodeLayout::Fullscreen => "fullscreen",
        NodeLayout::AlignLeft => "align-left",
        NodeLayout::AlignRight => "align-right",
        NodeLayout::Blank => "blank",
    }
}

#[must_use]
fn transition_name(transition: Transition) -> &'static str {
    match transition {
        Transition::None => "none",
        Transition::Fade => "fade",
        Transition::SlideLeft => "slide-left",
        Transition::SlideRight => "slide-right",
        Transition::Wipe => "wipe",
        Transition::Dissolve => "dissolve",
        Transition::Matrix => "matrix",
        Transition::Typewriter => "typewriter",
    }
}

pub(crate) fn render_picker_overlay(
    frame: &mut Frame,
    overlay: EditorPickerOverlay,
    theme: &Theme,
    area: Rect,
) {
    use ratatui::widgets::Clear;

    let popup = centered_popup(area, 55, 65);
    frame.render_widget(Clear, popup);

    let (title, variants, selected): (&str, Vec<(String, Option<String>)>, usize) = match overlay {
        EditorPickerOverlay::Layout { selected } => (
            " Layout Picker ",
            vec![
                "default",
                "center",
                "top",
                "split-horizontal",
                "split-vertical",
                "title",
                "code-focus",
                "fullscreen",
                "align-left",
                "align-right",
                "blank",
            ]
            .into_iter()
            .map(|value| (value.to_string(), None))
            .collect(),
            selected,
        ),
        EditorPickerOverlay::Transition { selected } => (
            " Transition Picker ",
            vec![
                "none",
                "fade",
                "slide-left",
                "slide-right",
                "wipe",
                "dissolve",
                "matrix",
                "typewriter",
            ]
            .into_iter()
            .map(|value| (value.to_string(), None))
            .collect(),
            selected,
        ),
        EditorPickerOverlay::BlockType { selected } => (
            " Block Type Picker ",
            vec![
                ("heading".to_string(), Some("Large title block".to_string())),
                ("text".to_string(), Some("Paragraph body text".to_string())),
                (
                    "code".to_string(),
                    Some("Syntax-highlighted source".to_string()),
                ),
                (
                    "list".to_string(),
                    Some("Bullet or numbered items".to_string()),
                ),
                (
                    "image".to_string(),
                    Some("Image source + alt text".to_string()),
                ),
                (
                    "divider".to_string(),
                    Some("Horizontal separator".to_string()),
                ),
                (
                    "container".to_string(),
                    Some("Nested child blocks".to_string()),
                ),
                (
                    "extension".to_string(),
                    Some("Custom typed payload".to_string()),
                ),
            ],
            selected,
        ),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_active));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let rows = variants
        .iter()
        .enumerate()
        .flat_map(|(idx, (value, synopsis))| {
            let marker = if idx == selected { "›" } else { " " };
            let shortcut = if idx < 9 {
                (idx + 1).to_string()
            } else if idx == 9 {
                "0".to_string()
            } else {
                "-".to_string()
            };
            let mut lines = vec![Line::from(vec![
                Span::styled(
                    format!(" {marker} {shortcut:>2} "),
                    Style::default().fg(theme.heading_h2),
                ),
                Span::styled(value.clone(), Style::default().fg(theme.foreground)),
            ])];

            if let Some(summary) = synopsis {
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(summary.clone(), Style::default().fg(theme.footer)),
                ]));
            }

            lines
        })
        .collect::<Vec<_>>();

    let mut lines = rows;
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "Use ↑/↓ or j/k, 1-9/0, Enter to apply, Esc to cancel",
        Style::default().fg(theme.footer),
    )));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

#[must_use]
fn centered_popup(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_pct) / 2),
            Constraint::Percentage(height_pct),
            Constraint::Percentage((100 - height_pct) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_pct) / 2),
            Constraint::Percentage(width_pct),
            Constraint::Percentage((100 - width_pct) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
