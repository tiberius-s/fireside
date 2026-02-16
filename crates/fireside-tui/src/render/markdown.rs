//! Content block rendering to ratatui primitives.
//!
//! Each [`ContentBlock`] variant is converted into one or more ratatui `Line`s
//! or widgets for composition by the layout engine.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use fireside_core::model::content::{ContentBlock, ListItem};

use crate::theme::Theme;

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
        ContentBlock::Image { alt, src, caption } => {
            render_image_placeholder(alt, src, caption.as_deref(), theme)
        }
        ContentBlock::Divider => render_divider(width, theme),
        ContentBlock::Container { children, .. } => render_node_content(children, theme, width),
        ContentBlock::Extension { .. } => {
            // Render extension blocks as a placeholder
            vec![Line::from(Span::styled(
                "[extension block]",
                Style::default()
                    .fg(ratatui::style::Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ))]
        }
    }
}

/// Render all content blocks for a node into a flat list of lines.
#[must_use]
pub fn render_node_content<'a>(
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
