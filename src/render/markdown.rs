//! Content block rendering to ratatui primitives.
//!
//! Each [`ContentBlock`] variant is converted into one or more ratatui `Line`s
//! or widgets for composition by the layout engine.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::model::content::{ColumnAlignment, ContentBlock, ListItem};
use crate::model::theme::Theme;

use super::code::highlight_code;

/// Render a single content block into a list of styled `Line`s.
///
/// The caller is responsible for wrapping these lines into widgets
/// (e.g., `Paragraph`) and positioning them within the layout.
#[must_use]
pub fn render_block<'a>(block: &'a ContentBlock, theme: &Theme, width: u16) -> Vec<Line<'a>> {
    match block {
        ContentBlock::Heading { level, text } => render_heading(*level, text, theme),
        ContentBlock::Text { body } => render_text(body, theme, width),
        ContentBlock::Code {
            language, source, ..
        } => render_code(language.as_deref(), source, theme),
        ContentBlock::List { ordered, items } => render_list(*ordered, items, theme, 0),
        ContentBlock::Table {
            headers,
            rows,
            alignments,
        } => render_table(headers, rows, alignments, theme),
        ContentBlock::Blockquote {
            content,
            attribution,
        } => render_block_quote(content, attribution.as_deref(), theme, width),
        ContentBlock::Image { alt, src, caption } => {
            render_image_placeholder(alt, src, caption.as_deref(), theme)
        }
        ContentBlock::Divider => render_divider(width, theme),
        ContentBlock::Fragment { blocks } => {
            // For now, render all fragments (progressive reveal is a UI concern)
            render_slide_content(blocks, theme, width)
        }
        ContentBlock::Spacer { lines } => render_spacer(*lines),
        ContentBlock::Columns { cols, widths } => render_columns(cols, widths, theme, width),
    }
}

/// Render all content blocks for a slide into a flat list of lines.
#[must_use]
pub fn render_slide_content<'a>(
    blocks: &'a [ContentBlock],
    theme: &Theme,
    width: u16,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            // Add spacing between blocks
            lines.push(Line::default());
        }
        lines.extend(render_block(block, theme, width));
    }
    lines
}

fn render_heading<'a>(level: u8, text: &'a str, theme: &Theme) -> Vec<Line<'a>> {
    let color = match level {
        1 => theme.heading_h1,
        2 => theme.heading_h2,
        _ => theme.heading_h3,
    };

    let style = Style::default().fg(color).add_modifier(Modifier::BOLD);

    let prefix = match level {
        1 => "",
        2 => "  ",
        3 => "    ",
        _ => "      ",
    };

    vec![Line::from(vec![
        Span::raw(prefix),
        Span::styled(text.to_owned(), style),
    ])]
}

fn render_text<'a>(text: &'a str, theme: &Theme, width: u16) -> Vec<Line<'a>> {
    let style = Style::default().fg(theme.foreground);
    let wrapped = textwrap::wrap(text, width as usize);
    wrapped
        .into_iter()
        .map(|line| Line::from(Span::styled(line.into_owned(), style)))
        .collect()
}

fn render_code<'a>(language: Option<&str>, source: &'a str, theme: &Theme) -> Vec<Line<'a>> {
    // Try syntax highlighting first
    if let Some(lang) = language
        && let Some(highlighted) = highlight_code(source, lang, &theme.syntax_theme)
    {
        return highlighted;
    }

    // Fallback: render as plain styled text
    let style = Style::default()
        .fg(theme.code_foreground)
        .bg(theme.code_background);

    source
        .lines()
        .map(|line| Line::from(Span::styled(line.to_owned(), style)))
        .collect()
}

fn render_list<'a>(
    ordered: bool,
    items: &'a [ListItem],
    theme: &Theme,
    depth: usize,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let style = Style::default().fg(theme.foreground);
    let indent = "  ".repeat(depth);

    let bullet = match depth {
        0 => "•",
        1 => "◦",
        _ => "▪",
    };

    for (i, item) in items.iter().enumerate() {
        let marker = if ordered {
            format!("{indent}{}. ", i + 1)
        } else {
            format!("{indent}{bullet} ")
        };

        lines.push(Line::from(vec![
            Span::styled(marker, style.add_modifier(Modifier::DIM)),
            Span::styled(item.text.clone(), style),
        ]));

        // Render children with increased depth
        if !item.children.is_empty() {
            lines.extend(render_list(false, &item.children, theme, depth + 1));
        }
    }

    lines
}

fn render_table<'a>(
    headers: &'a [String],
    rows: &'a [Vec<String>],
    alignments: &[ColumnAlignment],
    theme: &Theme,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let header_style = Style::default()
        .fg(theme.foreground)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
    let cell_style = Style::default().fg(theme.foreground);

    // Calculate column widths
    let mut col_widths: Vec<usize> = headers.iter().map(String::len).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    // Render header row
    let header_spans: Vec<Span<'a>> = headers
        .iter()
        .enumerate()
        .flat_map(|(i, h)| {
            let width = col_widths.get(i).copied().unwrap_or(h.len());
            let aligned = align_text(h, width, alignments.get(i).copied());
            vec![Span::styled(aligned, header_style), Span::raw("  ")]
        })
        .collect();
    lines.push(Line::from(header_spans));

    // Separator
    let sep: String = col_widths
        .iter()
        .map(|w| "─".repeat(*w))
        .collect::<Vec<_>>()
        .join("──");
    lines.push(Line::from(Span::styled(
        sep,
        Style::default().fg(theme.code_border),
    )));

    // Render data rows
    for row in rows {
        let spans: Vec<Span<'a>> = row
            .iter()
            .enumerate()
            .flat_map(|(i, cell)| {
                let width = col_widths.get(i).copied().unwrap_or(cell.len());
                let aligned = align_text(cell, width, alignments.get(i).copied());
                vec![Span::styled(aligned, cell_style), Span::raw("  ")]
            })
            .collect();
        lines.push(Line::from(spans));
    }

    lines
}

fn align_text(text: &str, width: usize, alignment: Option<ColumnAlignment>) -> String {
    let alignment = alignment.unwrap_or(ColumnAlignment::Left);
    match alignment {
        ColumnAlignment::Left => format!("{:<width$}", text),
        ColumnAlignment::Center => format!("{:^width$}", text),
        ColumnAlignment::Right => format!("{:>width$}", text),
    }
}

fn render_block_quote<'a>(
    inner: &'a [ContentBlock],
    attribution: Option<&'a str>,
    theme: &Theme,
    width: u16,
) -> Vec<Line<'a>> {
    let border_style = Style::default().fg(theme.block_quote);
    let inner_width = width.saturating_sub(4);

    let inner_lines = render_slide_content(inner, theme, inner_width);

    let mut lines: Vec<Line<'a>> = inner_lines
        .into_iter()
        .map(|line| {
            let mut spans = vec![Span::styled("│ ", border_style)];
            spans.extend(line.spans);
            Line::from(spans)
        })
        .collect();

    if let Some(attr) = attribution {
        let attr_style = Style::default()
            .fg(theme.block_quote)
            .add_modifier(Modifier::ITALIC);
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(attr.to_owned(), attr_style),
        ]));
    }

    lines
}

fn render_image_placeholder<'a>(
    alt: &'a str,
    src: &'a str,
    caption: Option<&'a str>,
    theme: &Theme,
) -> Vec<Line<'a>> {
    let style = Style::default()
        .fg(theme.code_border)
        .add_modifier(Modifier::DIM);

    let mut lines = vec![Line::from(Span::styled(
        format!("🖼  [Image: {src}]"),
        style,
    ))];

    if !alt.is_empty() {
        lines.push(Line::from(Span::styled(format!("    {alt}"), style)));
    }

    if let Some(cap) = caption {
        let cap_style = Style::default()
            .fg(theme.foreground)
            .add_modifier(Modifier::ITALIC);
        lines.push(Line::from(Span::styled(cap.to_owned(), cap_style)));
    }

    lines
}

fn render_divider(width: u16, theme: &Theme) -> Vec<Line<'static>> {
    let style = Style::default().fg(theme.code_border);
    vec![Line::from(Span::styled("─".repeat(width as usize), style))]
}

fn render_spacer(lines: u16) -> Vec<Line<'static>> {
    (0..lines).map(|_| Line::default()).collect()
}

fn render_columns<'a>(
    cols: &'a [Vec<ContentBlock>],
    widths: &[u8],
    theme: &Theme,
    total_width: u16,
) -> Vec<Line<'a>> {
    let num_cols = cols.len();
    if num_cols == 0 {
        return Vec::new();
    }

    // Calculate column widths
    let gutter = 2u16;
    let available = total_width.saturating_sub(gutter * (num_cols as u16 - 1));

    let col_widths: Vec<u16> = if widths.len() == num_cols {
        widths
            .iter()
            .map(|&w| (u16::from(w) * available) / 100)
            .collect()
    } else {
        let each = available / num_cols as u16;
        vec![each; num_cols]
    };

    // Render each column independently
    let col_lines: Vec<Vec<Line<'a>>> = cols
        .iter()
        .zip(col_widths.iter())
        .map(|(blocks, &w)| render_slide_content(blocks, theme, w))
        .collect();

    // Find max height
    let max_height = col_lines.iter().map(Vec::len).max().unwrap_or(0);

    // Merge lines side-by-side
    let mut output = Vec::with_capacity(max_height);
    for row in 0..max_height {
        let mut spans: Vec<Span<'a>> = Vec::new();
        for (ci, col) in col_lines.iter().enumerate() {
            if ci > 0 {
                spans.push(Span::raw("  ")); // gutter
            }
            if let Some(line) = col.get(row) {
                spans.extend(line.spans.clone());
                // Pad to column width
                let line_width: usize = line.spans.iter().map(|s| s.content.len()).sum();
                let pad = (col_widths[ci] as usize).saturating_sub(line_width);
                if pad > 0 {
                    spans.push(Span::raw(" ".repeat(pad)));
                }
            } else {
                spans.push(Span::raw(" ".repeat(col_widths[ci] as usize)));
            }
        }
        output.push(Line::from(spans));
    }

    output
}
