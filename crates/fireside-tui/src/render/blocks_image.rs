//! Image block rendering helpers for the content renderer.

use std::path::{Component, Path, PathBuf};

use image::ImageReader;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::design::tokens::DesignTokens;
use crate::error::RenderError;

#[must_use]
pub(super) fn render_image_placeholder<'a>(
    alt: &'a str,
    src: &'a str,
    caption: Option<&'a str>,
    tokens: &DesignTokens,
    width: u16,
    base_dir: Option<&Path>,
) -> Vec<Line<'a>> {
    let border_style = Style::default().fg(tokens.border_inactive);
    let label_style = Style::default()
        .fg(tokens.heading_h3)
        .add_modifier(Modifier::BOLD);
    let text_style = Style::default().fg(tokens.body);

    let inner_width = width.saturating_sub(2).max(24) as usize;
    let src_display = truncate_text(src, inner_width.saturating_sub(8));

    let mut lines = vec![Line::from(Span::styled(
        format!(
            "┌─ 🖼 {} {}",
            src_display,
            "─".repeat(inner_width.saturating_sub(src_display.chars().count() + 5))
        ),
        border_style,
    ))];

    if !alt.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("│ ", border_style),
            Span::styled(format!("alt: {alt}"), text_style),
        ]));
    }

    if let Some(cap) = caption {
        lines.push(Line::from(vec![
            Span::styled("│ ", border_style),
            Span::styled(
                cap.to_owned(),
                Style::default()
                    .fg(tokens.body)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]));
    }

    if let Some(path) = local_image_path(src, base_dir) {
        match read_image_dimensions(&path) {
            Ok((img_width, img_height)) => {
                lines.push(Line::from(vec![
                    Span::styled("│ ", border_style),
                    Span::styled(
                        format!("size: {img_width}×{img_height}"),
                        Style::default().fg(tokens.muted),
                    ),
                ]));
            }
            Err(err) => {
                lines.push(Line::from(vec![
                    Span::styled("│ ", border_style),
                    Span::styled(
                        truncate_text(&format!("fallback: {err}"), inner_width.saturating_sub(4)),
                        Style::default().fg(tokens.error),
                    ),
                ]));
            }
        }
    }

    if alt.is_empty() && caption.is_none() {
        lines.push(Line::from(vec![
            Span::styled("│ ", border_style),
            Span::styled("image block", label_style),
        ]));
    }

    lines.push(Line::from(Span::styled(
        format!("└{}", "─".repeat(inner_width + 1)),
        border_style,
    )));

    lines
}

#[must_use]
pub(super) fn local_image_path(src: &str, base_dir: Option<&Path>) -> Option<PathBuf> {
    if src.starts_with("http://") || src.starts_with("https://") {
        return None;
    }

    let path = if let Some(rest) = src.strip_prefix("file://") {
        PathBuf::from(rest)
    } else {
        PathBuf::from(src)
    };

    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        tracing::warn!(src = %src, "rejecting image path with parent traversal");
        return None;
    }

    if let Some(base_dir) = base_dir {
        let base_canonical = base_dir.canonicalize().ok()?;
        if path.is_absolute() {
            let resolved_for_check = path.canonicalize().unwrap_or_else(|_| path.clone());
            if !resolved_for_check.starts_with(&base_canonical) {
                tracing::warn!(src = %src, base_dir = %base_dir.display(), "rejecting image path outside base directory");
                return None;
            }
            return Some(resolved_for_check);
        }

        let resolved_for_check = base_canonical.join(path);
        if !resolved_for_check.starts_with(&base_canonical) {
            tracing::warn!(src = %src, base_dir = %base_dir.display(), "rejecting image path outside base directory");
            return None;
        }
        return Some(resolved_for_check);
    }

    Some(path)
}

fn read_image_dimensions(path: &Path) -> Result<(u32, u32), RenderError> {
    let reader = ImageReader::open(path).map_err(|source| RenderError::ImageLoad {
        path: path.to_path_buf(),
        source,
    })?;

    let reader = reader
        .with_guessed_format()
        .map_err(|source| RenderError::ImageLoad {
            path: path.to_path_buf(),
            source,
        })?;

    reader
        .into_dimensions()
        .map_err(|err| RenderError::ImageLoad {
            path: path.to_path_buf(),
            source: std::io::Error::other(err.to_string()),
        })
}

#[must_use]
fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let short: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{short}…")
}
