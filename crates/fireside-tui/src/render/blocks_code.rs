//! Code block renderer.
//!
//! Renders a [`ContentBlock::Code`] into syntax-highlighted ratatui [`Line`]s
//! surrounded by a box-drawn chrome border.  Falls back to plain styling when
//! line numbers or highlight directives are requested, since the syntect
//! highlighter operates on the raw source as a unit.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::design::tokens::DesignTokens;
use crate::render::code::highlight_code;

/// Render a code block with optional syntax highlighting, line numbers, and
/// highlighted line markers.
pub(super) fn render_code<'a>(
    language: Option<&str>,
    source: &'a str,
    highlight_lines: &[u32],
    show_line_numbers: bool,
    tokens: &DesignTokens,
    width: u16,
) -> Vec<Line<'a>> {
    // Prefer syntect-highlighted output when no per-line directives are set.
    let has_line_directives = show_line_numbers || !highlight_lines.is_empty();
    if !has_line_directives
        && let Some(lang) = language
        && let Some(highlighted) = highlight_code(source, lang, &tokens.syntax_theme)
    {
        return add_code_chrome(highlighted, language, tokens, width);
    }

    let mut code_lines = Vec::new();
    for (index, raw_line) in source.lines().enumerate() {
        let line_number = (index + 1) as u32;
        let is_highlighted = highlight_lines.contains(&line_number);

        let mut line_style = Style::default().fg(tokens.code_fg).bg(tokens.code_bg);

        if is_highlighted {
            line_style = line_style.add_modifier(Modifier::BOLD);
        }

        let mut spans = Vec::new();
        if show_line_numbers {
            spans.push(Span::styled(
                format!("{line_number:>3} │ "),
                Style::default().fg(tokens.muted).bg(tokens.code_bg),
            ));
        }

        if is_highlighted {
            spans.push(Span::styled("▎ ", Style::default().fg(tokens.success)));
        }

        spans.push(Span::styled(
            raw_line
                .chars()
                .take((width as usize).saturating_sub(2))
                .collect::<String>(),
            line_style,
        ));
        code_lines.push(Line::from(spans));
    }

    add_code_chrome(code_lines, language, tokens, width)
}

/// Wrap `code_lines` with a box-drawing chrome header/footer.
///
/// The header uses a filled double-line top border (`╔═ lang ═╗`) with the
/// language name in gold (`heading_h3`) so code blocks visually pop against
/// dark slide backgrounds.  The box border colour is pine (`heading_h2`)
/// rather than the near-invisible `border_inactive` default.
pub(super) fn add_code_chrome<'a>(
    code_lines: Vec<Line<'a>>,
    language: Option<&str>,
    tokens: &DesignTokens,
    width: u16,
) -> Vec<Line<'a>> {
    let lang_label = language.unwrap_or("code");
    let content_width = width.max(20) as usize;
    let border_inner_width = content_width.saturating_sub(2).max(10);

    // Pine for the border frame; gold for the language tab.
    let border_style = Style::default().fg(tokens.heading_h2);
    let lang_style = Style::default()
        .fg(tokens.heading_h3)
        .add_modifier(Modifier::BOLD);

    let title = format!(" {lang_label} ");
    let top_fill = border_inner_width.saturating_sub(title.chars().count());

    let mut lines = Vec::new();

    // Top: ╔═ lang ══...══╗
    lines.push(Line::from(vec![
        Span::styled("╔═", border_style),
        Span::styled(title, lang_style),
        Span::styled(format!("{}╗", "═".repeat(top_fill)), border_style),
    ]));

    for mut line in code_lines {
        // Prepend a 1-space gutter inside the box so code doesn't touch the border.
        let mut spans = vec![Span::styled("║ ", border_style)];
        spans.append(&mut line.spans);
        lines.push(Line::from(spans));
    }

    // Bottom: ╚══...══╝
    lines.push(Line::from(Span::styled(
        format!("╚{}╝", "═".repeat(border_inner_width)),
        border_style,
    )));

    lines
}
