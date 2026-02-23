//! Extension block rendering helpers for known extension previews.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use serde_json::Value;

use crate::design::tokens::DesignTokens;

#[must_use]
pub(super) fn render_known_extension<'a>(
    extension_type: &str,
    payload: &Value,
    tokens: &DesignTokens,
    width: u16,
) -> Option<Vec<Line<'a>>> {
    let normalized = extension_type.to_ascii_lowercase();
    if is_mermaid_extension(&normalized) {
        let (code, truncated) = extract_mermaid_code(payload);
        return Some(render_mermaid_preview(&code, tokens, width, truncated));
    }

    if normalized == "table" || normalized.ends_with(".table") || normalized.contains("table") {
        let headers = payload
            .get("headers")
            .and_then(Value::as_array)
            .map(|items| items.iter().map(payload_cell_text).collect::<Vec<_>>())
            .unwrap_or_default();

        let rows = payload
            .get("rows")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_array)
                    .map(|row| row.iter().map(payload_cell_text).collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        return Some(render_table_preview(&headers, &rows, tokens, width));
    }

    None
}

#[must_use]
fn is_mermaid_extension(normalized_type: &str) -> bool {
    normalized_type == "mermaid"
        || normalized_type.ends_with(".mermaid")
        || normalized_type.contains("mermaid")
}

#[must_use]
fn extract_mermaid_code(payload: &Value) -> (String, bool) {
    let raw = payload
        .get("code")
        .and_then(Value::as_str)
        .or_else(|| payload.get("diagram").and_then(Value::as_str))
        .or_else(|| payload.get("source").and_then(Value::as_str))
        .or_else(|| payload.as_str())
        .unwrap_or("");

    let normalized = normalize_mermaid_code(raw);
    if normalized.is_empty() {
        return ("(missing diagram code)".to_string(), false);
    }

    const MERMAID_MAX_CHARS: usize = 2_000;
    let mut preview = normalized;
    let mut truncated = false;
    if preview.chars().count() > MERMAID_MAX_CHARS {
        preview = truncate_text(&preview, MERMAID_MAX_CHARS);
        truncated = true;
    }

    (preview, truncated)
}

#[must_use]
fn normalize_mermaid_code(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Some(stripped) = trimmed.strip_prefix("```") {
        let mut lines = stripped.lines();
        let _fence_info = lines.next();
        let mut body = lines.collect::<Vec<_>>().join("\n");

        if let Some(without_trailing) = body.trim_end().strip_suffix("```") {
            body = without_trailing.trim_end().to_string();
        }

        return body.trim().to_string();
    }

    trimmed.to_string()
}

#[must_use]
fn render_mermaid_preview<'a>(
    code: &str,
    tokens: &DesignTokens,
    width: u16,
    truncated: bool,
) -> Vec<Line<'a>> {
    let mut lines = vec![Line::from(Span::styled(
        "mermaid diagram preview:",
        Style::default()
            .fg(tokens.heading_h2)
            .add_modifier(Modifier::BOLD),
    ))];

    const MERMAID_MAX_LINES: usize = 8;
    let wrapped = textwrap::wrap(code, width.saturating_sub(4).max(20) as usize);
    let total_wrapped = wrapped.len();

    for line in wrapped.into_iter().take(MERMAID_MAX_LINES) {
        lines.push(Line::from(Span::styled(
            format!("  {}", line.into_owned()),
            Style::default().fg(tokens.body),
        )));
    }

    if total_wrapped > MERMAID_MAX_LINES {
        lines.push(Line::from(Span::styled(
            format!(
                "  … {} more preview lines",
                total_wrapped - MERMAID_MAX_LINES
            ),
            Style::default().fg(tokens.muted),
        )));
    }

    if truncated {
        lines.push(Line::from(Span::styled(
            "  (preview truncated for performance)",
            Style::default()
                .fg(tokens.muted)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    lines
}

#[must_use]
fn render_table_preview<'a>(
    headers: &[String],
    rows: &[Vec<String>],
    tokens: &DesignTokens,
    width: u16,
) -> Vec<Line<'a>> {
    let mut lines = vec![Line::from(Span::styled(
        "table preview:",
        Style::default()
            .fg(tokens.heading_h2)
            .add_modifier(Modifier::BOLD),
    ))];

    if !headers.is_empty() {
        lines.push(Line::from(Span::styled(
            fit_to_width(&headers.join(" | "), width.saturating_sub(2) as usize),
            Style::default().fg(tokens.heading_h3),
        )));
        lines.push(Line::from(Span::styled(
            "-".repeat(width.saturating_sub(2).max(12) as usize),
            Style::default().fg(tokens.border_inactive),
        )));
    }

    for row in rows.iter().take(6) {
        lines.push(Line::from(Span::styled(
            fit_to_width(&row.join(" | "), width.saturating_sub(2) as usize),
            Style::default().fg(tokens.body),
        )));
    }

    if rows.len() > 6 {
        lines.push(Line::from(Span::styled(
            format!("… {} more rows", rows.len() - 6),
            Style::default().fg(tokens.muted),
        )));
    }

    lines
}

#[must_use]
fn payload_cell_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

#[must_use]
fn fit_to_width(text: &str, max_chars: usize) -> String {
    if text.chars().count() > max_chars {
        return truncate_text(text, max_chars);
    }

    let pad = max_chars.saturating_sub(text.chars().count());
    format!("{text}{}", " ".repeat(pad))
}

#[must_use]
fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let short: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{short}…")
}
