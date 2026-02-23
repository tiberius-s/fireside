use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

use fireside_core::model::content::{ContentBlock, ListItem};
use fireside_core::model::layout::Layout;
use fireside_core::model::transition::Transition;
use fireside_engine::PresentationSession;

use super::EditorPickerOverlay;

pub(super) fn search_tokens(query: &str) -> Vec<String> {
    let tokens = query
        .split_whitespace()
        .map(|token| token.to_ascii_lowercase())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    if tokens.is_empty() {
        vec![query.to_ascii_lowercase()]
    } else {
        tokens
    }
}

pub(super) fn update_block_from_inline_text(existing: ContentBlock, text: String) -> ContentBlock {
    match existing {
        ContentBlock::Heading { level, .. } => ContentBlock::Heading { level, text },
        ContentBlock::Text { .. } => ContentBlock::Text { body: text },
        ContentBlock::Code {
            language,
            highlight_lines,
            show_line_numbers,
            ..
        } => ContentBlock::Code {
            language,
            source: text,
            highlight_lines,
            show_line_numbers,
        },
        ContentBlock::List { ordered, mut items } => {
            if let Some(first) = items.first_mut() {
                first.text = text;
            } else {
                items.push(ListItem {
                    text,
                    children: Vec::new(),
                });
            }
            ContentBlock::List { ordered, items }
        }
        ContentBlock::Image { alt, caption, .. } => ContentBlock::Image {
            src: text,
            alt,
            caption,
        },
        ContentBlock::Divider => ContentBlock::Divider,
        ContentBlock::Container { children, .. } => {
            let layout = if text.trim().is_empty() {
                None
            } else {
                Some(text)
            };
            ContentBlock::Container { layout, children }
        }
        ContentBlock::Extension {
            fallback, payload, ..
        } => ContentBlock::Extension {
            extension_type: text,
            fallback,
            payload,
        },
    }
}

pub(super) fn score_node_id_match(candidate: &str, tokens: &[String]) -> Option<usize> {
    let mut total_score = 0usize;
    for token in tokens {
        let component = if candidate == token {
            0
        } else if candidate.starts_with(token) {
            1
        } else if candidate.contains(token) {
            2
        } else if is_subsequence(candidate, token) {
            3
        } else {
            return None;
        };

        total_score += component;
    }

    Some(total_score)
}

fn is_subsequence(candidate: &str, needle: &str) -> bool {
    let mut needle_chars = needle.chars();
    let mut current = needle_chars.next();

    for ch in candidate.chars() {
        if Some(ch) == current {
            current = needle_chars.next();
            if current.is_none() {
                return true;
            }
        }
    }

    false
}

pub(super) fn next_search_hit_from(
    session: &PresentationSession,
    tokens: &[String],
    current: usize,
) -> Option<usize> {
    let total = session.graph.nodes.len();
    if total == 0 {
        return None;
    }

    for step in 1..=total {
        let idx = (current + step) % total;
        let id = session.graph.nodes[idx].id.as_deref().unwrap_or("");
        if score_node_id_match(&id.to_ascii_lowercase(), tokens).is_some() {
            return Some(idx);
        }
    }

    None
}

pub(super) fn prev_search_hit_from(
    session: &PresentationSession,
    tokens: &[String],
    current: usize,
) -> Option<usize> {
    let total = session.graph.nodes.len();
    if total == 0 {
        return None;
    }

    for step in 1..=total {
        let idx = (current + total - (step % total)) % total;
        let id = session.graph.nodes[idx].id.as_deref().unwrap_or("");
        if score_node_id_match(&id.to_ascii_lowercase(), tokens).is_some() {
            return Some(idx);
        }
    }

    None
}

pub(super) fn layout_variants() -> &'static [Layout] {
    &[
        Layout::Default,
        Layout::Center,
        Layout::Top,
        Layout::SplitHorizontal,
        Layout::SplitVertical,
        Layout::Title,
        Layout::CodeFocus,
        Layout::Fullscreen,
        Layout::AlignLeft,
        Layout::AlignRight,
        Layout::Blank,
    ]
}

pub(super) fn transition_variants() -> &'static [Transition] {
    &[
        Transition::None,
        Transition::Fade,
        Transition::SlideLeft,
        Transition::SlideRight,
        Transition::Wipe,
        Transition::Dissolve,
        Transition::Matrix,
        Transition::Typewriter,
    ]
}

pub(super) fn block_type_variants() -> &'static [(&'static str, ContentBlock)] {
    static VARIANTS: std::sync::LazyLock<Vec<(&'static str, ContentBlock)>> =
        std::sync::LazyLock::new(|| {
            vec![
                (
                    "Heading",
                    ContentBlock::Heading {
                        level: 1,
                        text: "New heading".to_string(),
                    },
                ),
                (
                    "Text",
                    ContentBlock::Text {
                        body: "New text block".to_string(),
                    },
                ),
                (
                    "Code",
                    ContentBlock::Code {
                        language: Some("text".to_string()),
                        source: String::new(),
                        highlight_lines: vec![],
                        show_line_numbers: false,
                    },
                ),
                (
                    "List",
                    ContentBlock::List {
                        ordered: false,
                        items: vec![ListItem {
                            text: "New list item".to_string(),
                            children: vec![],
                        }],
                    },
                ),
                (
                    "Image",
                    ContentBlock::Image {
                        src: String::new(),
                        alt: String::new(),
                        caption: None,
                    },
                ),
                ("Divider", ContentBlock::Divider),
                (
                    "Container",
                    ContentBlock::Container {
                        layout: None,
                        children: vec![ContentBlock::Text {
                            body: "Container child".to_string(),
                        }],
                    },
                ),
                (
                    "Extension",
                    ContentBlock::Extension {
                        extension_type: "custom.unknown".to_string(),
                        fallback: Some(Box::new(ContentBlock::Text {
                            body: "Extension fallback".to_string(),
                        })),
                        payload: serde_json::Value::Object(serde_json::Map::new()),
                    },
                ),
            ]
        });
    VARIANTS.as_slice()
}

pub(super) fn bump_index(current: usize, max_index: usize, forward: bool) -> usize {
    if forward {
        (current + 1).min(max_index)
    } else {
        current.saturating_sub(1)
    }
}

pub(super) fn digit_to_index(code: KeyCode) -> Option<usize> {
    match code {
        KeyCode::Char(ch @ '1'..='9') => ch.to_digit(10).map(|d| d as usize - 1),
        KeyCode::Char('0') => Some(9),
        _ => None,
    }
}

pub(super) fn picker_row_span(overlay: EditorPickerOverlay) -> usize {
    match overlay {
        EditorPickerOverlay::BlockType { .. } => 2,
        EditorPickerOverlay::Layout { .. } | EditorPickerOverlay::Transition { .. } => 1,
    }
}

pub(super) fn is_editor_actionable_warning(message: &str) -> bool {
    [
        "heading text is empty",
        "text body is empty",
        "code source is empty",
        "list has no items",
        "list item #1 is empty",
        "image src is empty",
        "extension type is empty",
    ]
    .iter()
    .any(|pattern| message.contains(pattern))
}

pub(super) fn point_in_rect(column: u16, row: u16, rect: Rect) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

pub(super) fn centered_popup(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let vertical = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_pct) / 2),
            Constraint::Percentage(height_pct),
            Constraint::Percentage((100 - height_pct) / 2),
        ])
        .split(area);

    let horizontal = RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_pct) / 2),
            Constraint::Percentage(width_pct),
            Constraint::Percentage((100 - width_pct) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
