//! Editor shell view for phase-2 foundations.

use fireside_core::model::content::ContentBlock;
use fireside_engine::PresentationSession;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::app::{EditorPaneFocus, EditorPickerOverlay};
use crate::design::tokens::Breakpoint;
use crate::theme::Theme;
use crate::ui::graph::{GraphOverlayViewState, render_graph_overlay};
use crate::ui::help::{HelpMode, render_help_overlay};
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
    pub graph_overlay: Option<GraphOverlayViewState>,
    pub help_scroll_offset: usize,
    pub node_list_visible: bool,
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

    // ── Top-level vertical split: body | slim status bar ─────────────────
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(area);

    let compact = Breakpoint::from_size(area.width, area.height) == Breakpoint::Compact;

    // ── Adaptive shell layout ─────────────────────────────────────────────
    let (left_panels, detail_area) = if compact {
        if view_state.node_list_visible {
            let v = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
                .split(sections[0]);
            (Some(v[0]), v[1])
        } else {
            (None, sections[0])
        }
    } else {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(sections[0]);
        (Some(body[0]), body[1])
    };

    // ── Computed state ────────────────────────────────────────────────────
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
        "has notes"
    } else {
        "empty"
    };

    // ── Border colors based on focus ──────────────────────────────────────
    let list_border = if view_state.focus == EditorPaneFocus::NodeList {
        theme.border_active
    } else {
        theme.border_inactive
    };
    let detail_border = if view_state.focus == EditorPaneFocus::NodeDetail {
        theme.border_active
    } else {
        theme.border_inactive
    };

    // ── Node list (left-top panel) ────────────────────────────────────────
    let visible_rows = left_panels
        .map(|lp| lp.height.saturating_sub(2) as usize)
        .unwrap_or(0);
    let safe_visible_rows = visible_rows.max(1);
    let max_start = total.saturating_sub(safe_visible_rows);
    let mut list_start = view_state.list_scroll_offset.min(max_start);
    if selected < list_start {
        list_start = selected;
    } else if selected >= list_start + safe_visible_rows {
        list_start = selected + 1 - safe_visible_rows;
    }
    let list_end = (list_start + safe_visible_rows).min(total);

    let list_items = (list_start..list_end)
        .map(|index| {
            let n = &session.graph.nodes[index];
            // Type prefix icon for quick visual scanning
            let icon = match n.content.first() {
                Some(ContentBlock::Heading { .. }) => "▸",
                Some(ContentBlock::Code { .. }) => "⌥",
                Some(ContentBlock::Image { .. }) => "⬛",
                _ if n
                    .traversal
                    .as_ref()
                    .and_then(|t| t.branch_point.as_ref())
                    .is_some() =>
                {
                    "⎇"
                }
                _ => "·",
            };
            let label = format!("{} {}", icon, node_label(session, index));
            ListItem::new(Line::from(Span::styled(
                label,
                Style::default().fg(theme.foreground),
            )))
        })
        .collect::<Vec<_>>();

    let list = List::new(list_items)
        .block(
            Block::default()
                .title(format!(" Nodes {dirty_marker}({}/{total}) ", selected + 1))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(list_border))
                .style(Style::default().bg(theme.surface)),
        )
        .highlight_style(
            Style::default()
                .fg(theme.heading_h2)
                .bg(theme.background)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");

    if let Some(left_area) = left_panels {
        let list_split = if compact {
            vec![left_area]
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(left_area)
                .to_vec()
        };

        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(selected.saturating_sub(list_start)));
        frame.render_stateful_widget(list, list_split[0], &mut state);

        if !compact {
            // ── Tools panel (left-bottom panel) ──────────────────────────────────
            let key = |k: &'static str| Span::styled(k, Style::default().fg(theme.heading_h2));
            let sep = || Span::styled("  ", Style::default().fg(theme.toolbar_fg));
            let hint = |h: &str| Span::styled(h.to_string(), Style::default().fg(theme.toolbar_fg));

            let tools_lines = vec![
                Line::from(vec![Span::styled(
                    "Navigation",
                    Style::default()
                        .fg(theme.heading_h3)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![
                    key("j/k"),
                    hint(" up/dn"),
                    sep(),
                    key("PgUpDn"),
                    hint(" page"),
                    sep(),
                    key("Home/End"),
                ]),
                Line::from(vec![
                    key("g"),
                    hint(" jump#"),
                    sep(),
                    key("/"),
                    hint(" search"),
                    sep(),
                    key("[/]"),
                    hint(" hits"),
                ]),
                Line::from(vec![]),
                Line::from(vec![Span::styled(
                    "Editing",
                    Style::default()
                        .fg(theme.heading_h3)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![
                    key("i"),
                    hint(" inline"),
                    sep(),
                    key("a"),
                    hint(" append"),
                    sep(),
                    key("d"),
                    hint(" delete"),
                ]),
                Line::from(vec![
                    key("n"),
                    hint(" add node"),
                    sep(),
                    key("u/r"),
                    hint(" undo/redo"),
                ]),
                Line::from(vec![
                    key("w"),
                    hint(" save"),
                    sep(),
                    key("e"),
                    hint(" → present"),
                ]),
                Line::from(vec![]),
                Line::from(vec![Span::styled(
                    "Metadata",
                    Style::default()
                        .fg(theme.heading_h3)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![
                    key("L/l"),
                    hint(" layout"),
                    sep(),
                    key("T/t"),
                    hint(" transition"),
                ]),
                Line::from(vec![
                    key("o"),
                    hint(format!(" notes ({notes_state})").as_str()),
                    sep(),
                    key("v"),
                    hint(" graph"),
                ]),
                Line::from(vec![
                    key("Tab"),
                    hint(" focus"),
                    sep(),
                    key("?"),
                    hint(" help"),
                ]),
            ];

            let tools_panel = Paragraph::new(tools_lines)
                .block(
                    Block::default()
                        .title(" Keybindings ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border_inactive))
                        .style(Style::default().bg(theme.toolbar_bg)),
                )
                .style(Style::default().fg(theme.toolbar_fg));

            frame.render_widget(tools_panel, list_split[1]);
        }
    }

    // ── Detail panel (right) ──────────────────────────────────────────────
    let mut detail_lines = vec![
        Line::from(Span::styled(
            format!("  Node {}/{total}: {selected_node_label}", selected + 1),
            Style::default()
                .fg(theme.on_surface)
                .add_modifier(Modifier::BOLD),
        )),
        Line::default(),
        section_header(theme, "METADATA"),
        metadata_chip_row_layout(theme, prev_layout, current_layout, next_layout),
        metadata_chip_row_transition(theme, prev_transition, current_transition, next_transition),
        Line::from(vec![
            Span::styled("  id", Style::default().fg(theme.footer)),
            Span::styled(
                format!(
                    "  {}",
                    truncate(node.id.as_deref().unwrap_or("(no-id)"), 72)
                ),
                Style::default().fg(theme.foreground),
            ),
        ]),
        Line::from(vec![
            Span::styled("  blocks", Style::default().fg(theme.footer)),
            Span::styled(
                format!("  {}", node.content.len()),
                Style::default().fg(theme.foreground),
            ),
        ]),
        Line::default(),
        section_header(theme, "CONTENT BLOCKS"),
    ];

    if node.content.is_empty() {
        detail_lines.push(Line::from(vec![
            Span::styled("  · ", Style::default().fg(theme.footer)),
            Span::styled("(no content blocks)", Style::default().fg(theme.footer)),
        ]));
    } else {
        for (idx, block) in node.content.iter().enumerate() {
            detail_lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} ", block_type_glyph(block)),
                    Style::default().fg(theme.heading_h2),
                ),
                Span::styled(
                    format!("{:>2}. ", idx + 1),
                    Style::default().fg(theme.footer),
                ),
                Span::styled(
                    truncate(&block_summary(block), 74),
                    Style::default().fg(theme.foreground),
                ),
            ]));
        }
    }

    detail_lines.push(Line::default());
    detail_lines.push(section_header(theme, "TRAVERSAL"));
    detail_lines.extend(traversal_summary_lines(node, theme));

    detail_lines.push(Line::default());
    detail_lines.push(section_header(theme, "SPEAKER NOTES"));
    detail_lines.push(Line::from(vec![
        Span::styled("  status", Style::default().fg(theme.footer)),
        Span::styled(
            format!("  {notes_state}"),
            Style::default().fg(if notes_state == "has notes" {
                theme.success
            } else {
                theme.footer
            }),
        ),
        Span::styled("   [o] edit", Style::default().fg(theme.heading_h2)),
    ]));

    if let Some(notes) = node.speaker_notes.as_deref() {
        let trimmed = notes.trim();
        if !trimmed.is_empty() {
            detail_lines.push(Line::from(vec![
                Span::styled("  preview", Style::default().fg(theme.footer)),
                Span::styled(
                    format!("  {}", truncate(trimmed, 70)),
                    Style::default().fg(theme.foreground),
                ),
            ]));
        }
    }

    if let Some(buffer) = view_state.inline_text_input {
        detail_lines.push(Line::default());
        detail_lines.push(Line::from(Span::styled(
            "Inline Text Editor  Enter=commit  Esc=cancel",
            Style::default().fg(theme.heading_h2),
        )));
        detail_lines.push(Line::from(Span::styled(
            truncate(buffer, 220),
            Style::default()
                .fg(theme.on_surface)
                .add_modifier(Modifier::BOLD),
        )));
    }

    let detail = Paragraph::new(detail_lines)
        .block(
            Block::default()
                .title(" Node Detail ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(detail_border))
                .style(Style::default().bg(theme.surface)),
        )
        .style(Style::default().fg(theme.foreground))
        .wrap(Wrap { trim: true });

    frame.render_widget(detail, detail_area);

    // ── Status bar ────────────────────────────────────────────────────────
    let mut status_spans = vec![
        Span::styled(
            " EDITING ",
            Style::default()
                .fg(theme.toolbar_bg)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default().bg(theme.toolbar_bg)),
    ];

    // Filename + dirty marker
    let filename = if session.dirty {
        format!("● {dirty_marker}unsaved")
    } else {
        "● saved".to_string()
    };
    status_spans.push(Span::styled(
        filename,
        Style::default()
            .fg(if session.dirty {
                theme.error
            } else {
                theme.success
            })
            .bg(theme.toolbar_bg),
    ));

    // Undo/redo chips
    status_spans.push(Span::styled("  ", Style::default().bg(theme.toolbar_bg)));
    let undo_style = if can_undo {
        Style::default().fg(theme.heading_h2).bg(theme.toolbar_bg)
    } else {
        Style::default()
            .fg(theme.footer)
            .bg(theme.toolbar_bg)
            .add_modifier(Modifier::DIM)
    };
    let redo_style = if can_redo {
        Style::default().fg(theme.heading_h2).bg(theme.toolbar_bg)
    } else {
        Style::default()
            .fg(theme.footer)
            .bg(theme.toolbar_bg)
            .add_modifier(Modifier::DIM)
    };
    status_spans.push(Span::styled("[Z undo]", undo_style));
    status_spans.push(Span::styled(" ", Style::default().bg(theme.toolbar_bg)));
    status_spans.push(Span::styled("[Y redo]", redo_style));

    // Active search/jump prompt
    if let Some(search) = view_state.search_input {
        status_spans.push(Span::styled(
            format!("  Search: {search}_"),
            Style::default().fg(theme.heading_h1).bg(theme.toolbar_bg),
        ));
    } else if let Some(index_jump) = view_state.index_jump_input {
        status_spans.push(Span::styled(
            format!("  Jump to: {index_jump}_"),
            Style::default().fg(theme.heading_h1).bg(theme.toolbar_bg),
        ));
    }

    // Status message
    if let Some(message) = view_state.status {
        status_spans.push(Span::styled(
            format!("  {}", truncate(message, 60)),
            Style::default().fg(theme.heading_h3).bg(theme.toolbar_bg),
        ));
    }

    // Pending exit confirmation
    if view_state.pending_exit_confirmation {
        status_spans.push(Span::styled(
            "  Save and quit? y=yes n=no s=save-first Esc=cancel",
            Style::default()
                .fg(theme.error)
                .bg(theme.toolbar_bg)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if compact {
        status_spans.push(Span::styled(
            if view_state.node_list_visible {
                "  [n] hide list"
            } else {
                "  [n] show list"
            },
            Style::default().fg(theme.heading_h2).bg(theme.toolbar_bg),
        ));
    }

    let status_bar = Paragraph::new(Line::from(status_spans))
        .block(Block::default().style(Style::default().bg(theme.toolbar_bg)));

    frame.render_widget(status_bar, sections[1]);

    // ── Overlays ──────────────────────────────────────────────────────────
    if show_help {
        render_help_overlay(
            frame,
            popup_area(sections[0]),
            theme,
            HelpMode::Editing,
            view_state.help_scroll_offset,
        );
    }

    if let Some(overlay) = view_state.picker_overlay {
        render_picker_overlay(frame, overlay, theme, sections[0]);
    }

    if let Some(graph_overlay) = view_state.graph_overlay {
        render_graph_overlay(frame, sections[0], session, theme, graph_overlay);
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

fn section_header(theme: &Theme, title: &'static str) -> Line<'static> {
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

fn metadata_chip_row_layout(
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

fn metadata_chip_row_transition(
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

fn block_type_glyph(block: &ContentBlock) -> char {
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

fn block_summary(block: &ContentBlock) -> String {
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

fn traversal_summary_lines(
    node: &fireside_core::model::node::Node,
    theme: &Theme,
) -> Vec<Line<'static>> {
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
        .border_style(Style::default().fg(theme.border_active));
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
