//! Content block rendering to ratatui primitives.
//!
//! Each [`ContentBlock`] variant is converted into one or more ratatui `Line`s
//! for composition by the layout engine.
//!
//! Individual renderers live in focused sibling modules:
//!
//! | Module             | Variants handled                            |
//! |--------------------|---------------------------------------------|
//! | [`blocks_heading`] | `Heading`                                   |
//! | [`blocks_text`]    | `Text`                                      |
//! | [`blocks_code`]    | `Code`                                      |
//! | [`blocks_list`]    | `List`                                      |
//! | [`blocks_divider`] | `Divider`                                   |
//! | [`blocks_image`]   | `Image`                                     |
//! | [`blocks_extension`] | `Extension`                               |
//! | (this module)      | `Container`, dispatch, plain-text utilities |

use ratatui::style::Style;
use ratatui::text::{Line, Span};
use serde_json::Value;
use std::path::Path;

use fireside_core::model::content::ContentBlock;

use super::blocks_code::render_code as render_code_block;
use super::blocks_divider::render_divider;
use super::blocks_extension::render_known_extension;
use super::blocks_heading::render_heading;
use super::blocks_image::render_image_placeholder;
use super::blocks_list::render_list;
use super::blocks_text::{fit_to_width, line_to_plain_text, render_text};
use crate::design::tokens::DesignTokens;
use crate::theme::Theme;

/// Render a single content block into a list of styled `Line`s.
#[must_use]
pub fn render_block<'a>(block: &'a ContentBlock, theme: &Theme, width: u16) -> Vec<Line<'a>> {
    let tokens = DesignTokens::from_theme(theme);
    render_block_with_tokens(block, &tokens, width, None)
}

pub(super) fn render_block_with_tokens<'a>(
    block: &'a ContentBlock,
    tokens: &DesignTokens,
    width: u16,
    base_dir: Option<&Path>,
) -> Vec<Line<'a>> {
    match block {
        ContentBlock::Heading { level, text } => render_heading(*level, text, tokens, width),
        ContentBlock::Text { body } => render_text(body, tokens, width),
        ContentBlock::Code {
            language,
            source,
            highlight_lines,
            show_line_numbers,
        } => render_code_block(
            language.as_deref(),
            source,
            highlight_lines,
            *show_line_numbers,
            tokens,
            width,
        ),
        ContentBlock::List { ordered, items } => render_list(*ordered, items, tokens, 0),
        ContentBlock::Image { alt, src, caption } => {
            render_image_placeholder(alt, src, caption.as_deref(), tokens, width, base_dir)
        }
        ContentBlock::Divider => render_divider(width, tokens),
        ContentBlock::Container { layout, children } => {
            render_container(layout.as_deref(), children, tokens, width, base_dir)
        }
        ContentBlock::Extension {
            extension_type,
            fallback,
            payload,
        } => render_extension(
            extension_type,
            payload,
            fallback.as_deref(),
            tokens,
            width,
            base_dir,
        ),
    }
}

/// Render all content blocks for a node into a flat list of lines.
#[must_use]
pub fn render_node_content<'a>(
    blocks: &'a [ContentBlock],
    theme: &Theme,
    width: u16,
) -> Vec<Line<'a>> {
    render_node_content_with_base(blocks, theme, width, None)
}

/// Render all content blocks with an optional base directory for image paths.
#[must_use]
pub fn render_node_content_with_base<'a>(
    blocks: &'a [ContentBlock],
    theme: &Theme,
    width: u16,
    base_dir: Option<&Path>,
) -> Vec<Line<'a>> {
    let tokens = DesignTokens::from_theme(theme);
    render_node_content_with_tokens(blocks, &tokens, width, base_dir)
}

pub(super) fn render_node_content_with_tokens<'a>(
    blocks: &'a [ContentBlock],
    tokens: &DesignTokens,
    width: u16,
    base_dir: Option<&Path>,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            lines.push(Line::default());
        }
        lines.extend(render_block_with_tokens(block, tokens, width, base_dir));
    }
    lines
}

// ── Container variants ──────────────────────────────────────────────────────

fn render_container<'a>(
    layout: Option<&str>,
    children: &'a [ContentBlock],
    tokens: &DesignTokens,
    width: u16,
    base_dir: Option<&Path>,
) -> Vec<Line<'a>> {
    let layout_hint = layout.unwrap_or("").to_ascii_lowercase();
    match layout_hint.as_str() {
        "split-horizontal" => render_container_split_horizontal(children, tokens, width, base_dir),
        "split-vertical" => render_container_split_vertical(children, tokens, width, base_dir),
        _ => {
            let mut lines = Vec::new();
            if let Some(layout_hint) = layout {
                lines.push(Line::from(Span::styled(
                    format!("[container: {layout_hint}]"),
                    Style::default()
                        .fg(tokens.muted)
                        .add_modifier(ratatui::style::Modifier::DIM),
                )));
            }
            lines.extend(render_node_content_with_tokens(
                children, tokens, width, base_dir,
            ));
            lines
        }
    }
}

fn render_extension<'a>(
    extension_type: &'a str,
    payload: &Value,
    fallback: Option<&'a ContentBlock>,
    tokens: &DesignTokens,
    width: u16,
    base_dir: Option<&Path>,
) -> Vec<Line<'a>> {
    let mut lines = vec![Line::from(vec![Span::styled(
        format!("[extension: {extension_type}]"),
        Style::default()
            .fg(tokens.heading_h3)
            .add_modifier(ratatui::style::Modifier::DIM),
    )])];

    if let Some(mut known_lines) = render_known_extension(extension_type, payload, tokens, width) {
        lines.push(Line::default());
        lines.append(&mut known_lines);
    } else if let Some(payload_keys) = payload
        .as_object()
        .map(|obj| obj.keys().take(5).cloned().collect::<Vec<_>>().join(", "))
        && !payload_keys.is_empty()
    {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            format!("payload keys: {payload_keys}"),
            Style::default().fg(tokens.muted),
        )));
    }

    if let Some(fallback_block) = fallback {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            "fallback:",
            Style::default()
                .fg(tokens.muted)
                .add_modifier(ratatui::style::Modifier::ITALIC),
        )));
        lines.extend(render_block_with_tokens(
            fallback_block,
            tokens,
            width,
            base_dir,
        ));
    }

    lines
}

fn render_container_split_horizontal<'a>(
    children: &'a [ContentBlock],
    tokens: &DesignTokens,
    width: u16,
    base_dir: Option<&Path>,
) -> Vec<Line<'a>> {
    if children.len() <= 1 {
        return render_node_content_with_tokens(children, tokens, width, base_dir);
    }

    let mid = children.len().div_ceil(2);
    let col_width = width.saturating_sub(3).saturating_div(2).max(10);
    let left_lines = render_node_content_with_tokens(&children[..mid], tokens, col_width, base_dir);
    let right_lines =
        render_node_content_with_tokens(&children[mid..], tokens, col_width, base_dir);
    compose_side_by_side(left_lines, right_lines, col_width as usize, tokens)
}

fn render_container_split_vertical<'a>(
    children: &'a [ContentBlock],
    tokens: &DesignTokens,
    width: u16,
    base_dir: Option<&Path>,
) -> Vec<Line<'a>> {
    if children.len() <= 1 {
        return render_node_content_with_tokens(children, tokens, width, base_dir);
    }

    let mid = children.len().div_ceil(2);
    let mut lines = render_node_content_with_tokens(&children[..mid], tokens, width, base_dir);
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "─".repeat(width.max(10) as usize),
        Style::default().fg(tokens.border_inactive),
    )));
    lines.push(Line::default());
    lines.extend(render_node_content_with_tokens(
        &children[mid..],
        tokens,
        width,
        base_dir,
    ));
    lines
}

fn compose_side_by_side<'a>(
    left: Vec<Line<'a>>,
    right: Vec<Line<'a>>,
    col_width: usize,
    tokens: &DesignTokens,
) -> Vec<Line<'a>> {
    let rows = left.len().max(right.len());
    let mut lines = Vec::with_capacity(rows);

    for row in 0..rows {
        let left_text = left.get(row).map_or_else(String::new, line_to_plain_text);
        let right_text = right.get(row).map_or_else(String::new, line_to_plain_text);
        let merged = format!(
            "{} │ {}",
            fit_to_width(&left_text, col_width),
            fit_to_width(&right_text, col_width)
        );
        lines.push(Line::from(Span::styled(
            merged,
            Style::default().fg(tokens.body),
        )));
    }

    lines
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::blocks_image::local_image_path;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("fireside-{prefix}-{unique}"));
        std::fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }

    fn lines_to_text(lines: &[Line<'_>]) -> Vec<String> {
        lines
            .iter()
            .map(super::super::blocks_text::line_to_plain_text)
            .collect()
    }

    #[test]
    fn container_split_horizontal_renders_columns() {
        let block = ContentBlock::Container {
            layout: Some("split-horizontal".to_string()),
            children: vec![
                ContentBlock::Text {
                    body: "left side".to_string(),
                },
                ContentBlock::Text {
                    body: "right side".to_string(),
                },
            ],
        };

        let tokens = DesignTokens::default();
        let lines = render_block_with_tokens(&block, &tokens, 60, None);
        let text = lines_to_text(&lines).join("\n");

        assert!(text.contains(" │ "));
        assert!(text.contains("left side"));
        assert!(text.contains("right side"));
    }

    #[test]
    fn extension_mermaid_renders_preview() {
        let block = ContentBlock::Extension {
            extension_type: "mermaid".to_string(),
            fallback: None,
            payload: serde_json::json!({"code": "graph TD; A-->B;"}),
        };

        let tokens = DesignTokens::default();
        let lines = render_block_with_tokens(&block, &tokens, 80, None);
        let text = lines_to_text(&lines).join("\n");

        assert!(text.contains("mermaid diagram preview"));
        assert!(text.contains("graph TD; A-->B;"));
    }

    #[test]
    fn extension_mermaid_normalizes_fenced_code() {
        let block = ContentBlock::Extension {
            extension_type: "mermaid".to_string(),
            fallback: None,
            payload: serde_json::json!({"code": "```mermaid\nflowchart TD\nA-->B\n```"}),
        };

        let tokens = DesignTokens::default();
        let lines = render_block_with_tokens(&block, &tokens, 80, None);
        let text = lines_to_text(&lines).join("\n");

        assert!(text.contains("flowchart TD"));
        assert!(!text.contains("```mermaid"));
    }

    #[test]
    fn extension_mermaid_reports_truncation_for_large_payload() {
        let long = "graph TD; ".repeat(600);
        let block = ContentBlock::Extension {
            extension_type: "acme.mermaid".to_string(),
            fallback: None,
            payload: serde_json::json!({"source": long}),
        };

        let tokens = DesignTokens::default();
        let lines = render_block_with_tokens(&block, &tokens, 60, None);
        let text = lines_to_text(&lines).join("\n");

        assert!(text.contains("preview truncated for performance"));
    }

    #[test]
    fn local_image_path_rejects_parent_traversal() {
        let base = temp_dir("image-base");
        let resolved = local_image_path("../../../etc/passwd", Some(&base));
        assert!(resolved.is_none());
    }

    #[test]
    fn local_image_path_rejects_absolute_path_outside_base_dir() {
        let base = temp_dir("image-base-abs");
        let resolved = local_image_path("/etc/passwd", Some(&base));
        assert!(resolved.is_none());
    }

    #[test]
    fn local_image_path_allows_relative_path_within_base_dir() {
        let base = temp_dir("image-base-valid");
        let image = base.join("valid-image.png");
        std::fs::write(&image, b"not-an-image").expect("test file should be written");

        let resolved = local_image_path("valid-image.png", Some(&base));
        let resolved = resolved
            .and_then(|path| path.canonicalize().ok())
            .expect("resolved path should canonicalize");
        let expected = image
            .canonicalize()
            .expect("expected image path should canonicalize");

        assert_eq!(resolved, expected);
    }
}
