//! Editor shell view for phase-2 foundations.

use fireside_core::model::content::ContentBlock;
use fireside_engine::PresentationSession;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::app::{EditorPaneFocus, EditorPickerOverlay};
use crate::theme::Theme;
use crate::ui::help::render_help_overlay;
use fireside_core::model::layout::Layout as NodeLayout;
use fireside_core::model::transition::Transition;

/// Transient editor view state provided by the app layer.
pub struct EditorViewState<'a> {
    pub selected_index: usize,
    pub list_scroll_offset: usize,
    pub focus: EditorPaneFocus,
    pub inline_text_input: Option<&'a str>,
    pub search_input: Option<&'a str>,
    pub index_jump_input: Option<&'a str>,
    pub status: Option<&'a str>,
    pub pending_exit_confirmation: bool,
    pub picker_overlay: Option<EditorPickerOverlay>,
}

/// Render the editing mode shell.
pub fn render_editor(
    frame: &mut Frame,
    session: &PresentationSession,
    theme: &Theme,
    show_help: bool,
    view_state: EditorViewState<'_>,
) {
    let area = frame.area();

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(sections[0]);

    let total = session.graph.nodes.len();
    let selected = view_state.selected_index.min(total.saturating_sub(1));
    let node = &session.graph.nodes[selected];
    let selected_node_label = node_label(session, selected);
    let can_undo = session.command_history.can_undo();
    let can_redo = session.command_history.can_redo();
    let dirty_marker = if session.dirty { "*" } else { "" };
    let current_layout = node.layout.unwrap_or(NodeLayout::Default);
    let current_transition = node.transition.unwrap_or(Transition::None);
    let (prev_layout, next_layout) = adjacent_layouts(current_layout);
    let (prev_transition, next_transition) = adjacent_transitions(current_transition);
    let notes_state = if node
        .speaker_notes
        .as_deref()
        .is_some_and(|notes| !notes.trim().is_empty())
    {
        "present"
    } else {
        "empty"
    };

    let list_border = if view_state.focus == EditorPaneFocus::NodeList {
        theme.heading_h1
    } else {
        theme.code_border
    };
    let detail_border = if view_state.focus == EditorPaneFocus::NodeDetail {
        theme.heading_h1
    } else {
        theme.code_border
    };

    let visible_rows = body[0].height.saturating_sub(2) as usize;
    let safe_visible_rows = visible_rows.max(1);
    let max_start = total.saturating_sub(safe_visible_rows);
    let mut list_start = view_state.list_scroll_offset.min(max_start);
    if selected < list_start {
        list_start = selected;
    } else if selected >= list_start + safe_visible_rows {
        list_start = selected + 1 - safe_visible_rows;
    }
    let list_end = (list_start + safe_visible_rows).min(total);
    let selected_pct = if total == 0 {
        0
    } else {
        ((selected + 1) * 100) / total
    };

    let list_items = (list_start..list_end)
        .map(|index| {
            let label = node_label(session, index);
            ListItem::new(Line::from(Span::styled(
                label,
                Style::default().fg(theme.foreground),
            )))
        })
        .collect::<Vec<_>>();

    let list = List::new(list_items)
        .block(
            Block::default()
                .title(" Nodes ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(list_border)),
        )
        .highlight_style(
            Style::default()
                .fg(theme.heading_h2)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(selected.saturating_sub(list_start)));
    frame.render_stateful_widget(list, body[0], &mut state);

    let mut detail_lines = vec![
        Line::from(Span::styled(
            format!("Selected: {selected_node_label} ({}/{total})", selected + 1),
            Style::default()
                .fg(theme.foreground)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!(
                "List window: {}-{} ({selected_pct}%)",
                list_start + 1,
                list_end
            ),
            Style::default().fg(theme.footer),
        )),
        Line::from(Span::styled(
            format!("Session: {dirty_marker} undo={can_undo} redo={can_redo}"),
            Style::default().fg(theme.footer),
        )),
        Line::default(),
        Line::from(Span::styled(
            "Metadata Selectors:",
            Style::default()
                .fg(theme.heading_h3)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("L", Style::default().fg(theme.heading_h2)),
            Span::styled(
                format!("◀ {}   ", layout_name(prev_layout)),
                Style::default().fg(theme.footer),
            ),
            Span::styled(
                format!("[{}]", layout_name(current_layout)),
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("   ", Style::default().fg(theme.foreground)),
            Span::styled(
                format!("{} ▶", layout_name(next_layout)),
                Style::default().fg(theme.footer),
            ),
            Span::styled("   l", Style::default().fg(theme.heading_h2)),
        ]),
        Line::from(vec![
            Span::styled("T", Style::default().fg(theme.heading_h2)),
            Span::styled(
                format!("◀ {}   ", transition_name(prev_transition)),
                Style::default().fg(theme.footer),
            ),
            Span::styled(
                format!("[{}]", transition_name(current_transition)),
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("   ", Style::default().fg(theme.foreground)),
            Span::styled(
                format!("{} ▶", transition_name(next_transition)),
                Style::default().fg(theme.footer),
            ),
            Span::styled("   t", Style::default().fg(theme.heading_h2)),
        ]),
        Line::from(vec![
            Span::styled("o", Style::default().fg(theme.heading_h2)),
            Span::styled(
                format!(" edit speaker notes ({notes_state})"),
                Style::default().fg(theme.foreground),
            ),
        ]),
        Line::default(),
        Line::from(Span::styled(
            format!("Blocks: {}", node.content.len()),
            Style::default().fg(theme.foreground),
        )),
        Line::default(),
        Line::from(Span::styled(
            "Preview:",
            Style::default()
                .fg(theme.heading_h3)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            preview_line(node.content.first()),
            Style::default().fg(theme.foreground),
        )),
    ];

    if let Some(buffer) = view_state.inline_text_input {
        detail_lines.push(Line::default());
        detail_lines.push(Line::from(Span::styled(
            "Inline Text Editor (Enter=commit, Esc=cancel)",
            Style::default().fg(theme.heading_h2),
        )));
        detail_lines.push(Line::from(Span::styled(
            truncate(buffer, 220),
            Style::default()
                .fg(theme.foreground)
                .add_modifier(Modifier::BOLD),
        )));
    }

    let detail = Paragraph::new(detail_lines)
        .block(
            Block::default()
                .title(" Node Detail ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(detail_border)),
        )
        .style(Style::default().bg(theme.background))
        .wrap(Wrap { trim: true });

    frame.render_widget(detail, body[1]);

    let mut control_spans = vec![
        Span::styled("Tab", Style::default().fg(theme.heading_h2)),
        Span::styled(" focus  ", Style::default().fg(theme.foreground)),
        Span::styled("j/k", Style::default().fg(theme.heading_h2)),
        Span::styled(" select  ", Style::default().fg(theme.foreground)),
        Span::styled("PgUp/PgDn", Style::default().fg(theme.heading_h2)),
        Span::styled(" page  ", Style::default().fg(theme.foreground)),
        Span::styled("Home/End", Style::default().fg(theme.heading_h2)),
        Span::styled(" top/bottom  ", Style::default().fg(theme.foreground)),
        Span::styled("/", Style::default().fg(theme.heading_h2)),
        Span::styled(" search  ", Style::default().fg(theme.foreground)),
        Span::styled("[/]", Style::default().fg(theme.heading_h2)),
        Span::styled(" prev/next hit  ", Style::default().fg(theme.foreground)),
        Span::styled("g", Style::default().fg(theme.heading_h2)),
        Span::styled(" jump  ", Style::default().fg(theme.foreground)),
        Span::styled("i", Style::default().fg(theme.heading_h2)),
        Span::styled(" inline edit  ", Style::default().fg(theme.foreground)),
        Span::styled("a", Style::default().fg(theme.heading_h2)),
        Span::styled(" append  ", Style::default().fg(theme.foreground)),
        Span::styled("n", Style::default().fg(theme.heading_h2)),
        Span::styled(" add node  ", Style::default().fg(theme.foreground)),
        Span::styled("d", Style::default().fg(theme.heading_h2)),
        Span::styled(" delete  ", Style::default().fg(theme.foreground)),
        Span::styled("u/r", Style::default().fg(theme.heading_h2)),
        Span::styled(" undo/redo  ", Style::default().fg(theme.foreground)),
        Span::styled("w", Style::default().fg(theme.heading_h2)),
        Span::styled(" save", Style::default().fg(theme.foreground)),
    ];

    if let Some(search) = view_state.search_input {
        control_spans.push(Span::styled("  |  ", Style::default().fg(theme.footer)));
        control_spans.push(Span::styled(
            format!("Search node id: {search}_"),
            Style::default().fg(theme.heading_h2),
        ));
    } else if let Some(index_jump) = view_state.index_jump_input {
        control_spans.push(Span::styled("  |  ", Style::default().fg(theme.footer)));
        control_spans.push(Span::styled(
            format!("Jump to index: {index_jump}_"),
            Style::default().fg(theme.heading_h2),
        ));
    }

    if let Some(message) = view_state.status {
        control_spans.push(Span::styled("  |  ", Style::default().fg(theme.footer)));
        control_spans.push(Span::styled(
            truncate(message, 60),
            Style::default().fg(theme.heading_h3),
        ));
    }

    if view_state.pending_exit_confirmation {
        control_spans.push(Span::styled("  |  ", Style::default().fg(theme.footer)));
        control_spans.push(Span::styled(
            "Unsaved changes: y=leave n=stay",
            Style::default().fg(theme.heading_h1),
        ));
    }

    let controls = Line::from(control_spans);

    let footer = Paragraph::new(controls).block(
        Block::default()
            .title(" Editor Mode ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.footer)),
    );

    frame.render_widget(footer, sections[1]);

    if show_help {
        render_help_overlay(frame, popup_area(sections[0]), theme);
    }

    if let Some(overlay) = view_state.picker_overlay {
        render_picker_overlay(frame, overlay, theme, sections[0]);
    }
}

fn node_label(session: &PresentationSession, index: usize) -> String {
    let prefix = format!("{:>2}. ", index + 1);
    let id = session
        .graph
        .nodes
        .get(index)
        .and_then(|node| node.id.as_deref())
        .unwrap_or("(no-id)");
    format!("{prefix}{id}")
}

fn preview_line(block: Option<&ContentBlock>) -> String {
    match block {
        Some(ContentBlock::Heading { text, .. }) => format!("heading: {text}"),
        Some(ContentBlock::Text { body }) => format!("text: {}", truncate(body, 70)),
        Some(ContentBlock::Code { language, .. }) => {
            format!("code: {}", language.as_deref().unwrap_or("plain"))
        }
        Some(ContentBlock::List { items, .. }) => format!("list: {} items", items.len()),
        Some(ContentBlock::Image { src, .. }) => format!("image: {}", truncate(src, 70)),
        Some(ContentBlock::Divider) => "divider".to_string(),
        Some(ContentBlock::Container { children, .. }) => {
            format!("container: {} children", children.len())
        }
        Some(ContentBlock::Extension { extension_type, .. }) => {
            format!("extension: {extension_type}")
        }
        None => "(empty node)".to_string(),
    }
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let clipped: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{clipped}…")
}

fn popup_area(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height,
    }
}

fn adjacent_layouts(current: NodeLayout) -> (NodeLayout, NodeLayout) {
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

fn adjacent_transitions(current: Transition) -> (Transition, Transition) {
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

fn render_picker_overlay(
    frame: &mut Frame,
    overlay: EditorPickerOverlay,
    theme: &Theme,
    area: Rect,
) {
    use ratatui::widgets::Clear;

    let popup = centered_popup(area, 55, 65);
    frame.render_widget(Clear, popup);

    let (title, variants, selected): (&str, Vec<String>, usize) = match overlay {
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
            .map(str::to_string)
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
            .map(str::to_string)
            .collect(),
            selected,
        ),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.heading_h1));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let rows = variants
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            let marker = if idx == selected { "›" } else { " " };
            let shortcut = if idx < 9 {
                (idx + 1).to_string()
            } else if idx == 9 {
                "0".to_string()
            } else {
                "-".to_string()
            };
            Line::from(vec![
                Span::styled(
                    format!(" {marker} {shortcut:>2} "),
                    Style::default().fg(theme.heading_h2),
                ),
                Span::styled(value.clone(), Style::default().fg(theme.foreground)),
            ])
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
