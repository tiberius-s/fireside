//! Content blocks → styled lines.
//!
//! Every block renders to a flat `Vec<Line>` flow at a given width. Working
//! in lines (rather than widgets) keeps the hard parts simple: scrolling is
//! "skip n lines", measuring is `lines.len()`, container columns are a
//! side-by-side zip, and centering is a uniform left offset that preserves
//! the internal alignment of code boxes and lists.

use fireside_core::{ContainerLayout, ContentBlock};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use super::{markdown, syntax};
use crate::theme::Tokens;

/// Render a node's blocks to a line flow at `width` columns, with one blank
/// line between blocks.
#[must_use]
pub fn render_blocks(blocks: &[ContentBlock], width: u16, tokens: &Tokens) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            lines.push(Line::default());
        }
        lines.extend(render_block(block, width, tokens));
    }
    lines
}

fn render_block(block: &ContentBlock, width: u16, tokens: &Tokens) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }
    match block {
        ContentBlock::Heading { level, text } => heading(*level, text, width, tokens),
        ContentBlock::Text { body } => markdown::wrap_styled(body, width, tokens.text, tokens),
        ContentBlock::Code {
            language,
            source,
            highlight_lines,
            show_line_numbers,
        } => code(
            language.as_deref(),
            source,
            highlight_lines.as_deref().unwrap_or_default(),
            show_line_numbers.unwrap_or(false),
            width,
            tokens,
        ),
        ContentBlock::List { ordered, items } => {
            list(ordered.unwrap_or(false), items, width, tokens)
        }
        ContentBlock::Image { src, alt, caption, .. } => {
            image(src, alt.as_deref(), caption.as_deref(), width, tokens)
        }
        ContentBlock::Divider => divider(width, tokens),
        ContentBlock::Container { children, layout } => {
            container(children, layout.unwrap_or_default(), width, tokens)
        }
    }
}

fn heading(level: u8, text: &str, width: u16, tokens: &Tokens) -> Vec<Line<'static>> {
    let style = tokens.heading(level);
    match level {
        1 => {
            let mut lines = markdown::wrap_styled(text, width, style, tokens);
            let rule_width = lines
                .iter()
                .map(Line::width)
                .max()
                .unwrap_or(0)
                .min(width as usize);
            lines.push(Line::styled("─".repeat(rule_width), tokens.accent));
            lines
        }
        2 => {
            // A short accent bar marks the section without shouting.
            let body = markdown::wrap_styled(text, width.saturating_sub(2), style, tokens);
            body.into_iter()
                .enumerate()
                .map(|(row, line)| {
                    let lead = if row == 0 {
                        Span::styled("▎ ".to_owned(), tokens.accent)
                    } else {
                        Span::raw("  ".to_owned())
                    };
                    let mut spans = vec![lead];
                    spans.extend(line.spans);
                    Line::from(spans)
                })
                .collect()
        }
        _ => markdown::wrap_styled(text, width, style, tokens),
    }
}

/// A divider is a pause, not a wall: a short centered rule. The line is
/// padded on both sides to the full width so that outer containers (e.g.
/// `center`) never re-center it off axis.
fn divider(width: u16, tokens: &Tokens) -> Vec<Line<'static>> {
    let rule = usize::from((width / 3).clamp(2, 24).min(width));
    let pad = (usize::from(width) - rule) / 2;
    vec![Line::from(vec![
        Span::raw(" ".repeat(pad)),
        Span::styled("─".repeat(rule), tokens.border),
        Span::raw(" ".repeat(usize::from(width) - pad - rule)),
    ])]
}

fn code(
    language: Option<&str>,
    source: &str,
    highlight: &[u32],
    line_numbers: bool,
    width: u16,
    tokens: &Tokens,
) -> Vec<Line<'static>> {
    let width = width as usize;
    let label = language.unwrap_or("code");
    let mut top = format!("─ {label} ");
    let fill = width.saturating_sub(top.width());
    top.push_str(&"─".repeat(fill));

    let mut lines = vec![Line::styled(top, tokens.border)];
    let total = source.lines().count();
    let num_width = if line_numbers { total.to_string().len() } else { 0 };
    let colored = syntax::highlight(language, source, tokens);
    // When the author picked lines to highlight, focus means dimming the
    // rest — the chosen lines keep their full colors.
    let focused = !highlight.is_empty();

    for (i, raw) in source.lines().enumerate() {
        let n = i + 1;
        let emphasized = highlight.contains(&(n as u32));

        let mut spans = Vec::new();
        if line_numbers {
            let gutter = if emphasized {
                tokens.accent.add_modifier(Modifier::BOLD)
            } else {
                tokens.muted
            };
            spans.push(Span::styled(format!(" {n:num_width$} │ "), gutter));
        } else if emphasized {
            spans.push(Span::styled("▎ ".to_owned(), tokens.accent));
        } else {
            spans.push(Span::styled("  ".to_owned(), tokens.muted));
        }
        let prefix = if line_numbers { num_width + 4 } else { 2 };
        let avail = width.saturating_sub(prefix);

        let mut content: Vec<Span<'static>> = match &colored {
            Some(rows) => clip_spans(rows[i].clone(), avail, tokens),
            None => {
                let style = if emphasized { tokens.code_highlight } else { tokens.code };
                vec![Span::styled(clip(raw, avail), style)]
            }
        };
        if focused && !emphasized {
            for span in &mut content {
                span.style = span.style.add_modifier(Modifier::DIM);
            }
        }
        spans.extend(content);
        lines.push(Line::from(spans));
    }
    lines.push(Line::styled("─".repeat(width), tokens.border));
    lines
}

/// Clip a row of styled spans to `width` columns, marking any cut with an
/// ellipsis while preserving each span's style.
fn clip_spans(
    spans: Vec<Span<'static>>,
    width: usize,
    tokens: &Tokens,
) -> Vec<Span<'static>> {
    let total: usize = spans.iter().map(|s| s.content.width()).sum();
    if total <= width {
        return spans;
    }
    if width == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut used = 0;
    for span in spans {
        let w = span.content.width();
        if used + w < width {
            used += w;
            out.push(span);
            continue;
        }
        let mut text = String::new();
        for ch in span.content.chars() {
            let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if used + cw + 1 > width {
                break;
            }
            used += cw;
            text.push(ch);
        }
        if !text.is_empty() {
            out.push(Span::styled(text, span.style));
        }
        break;
    }
    out.push(Span::styled("…".to_owned(), tokens.muted));
    out
}

/// Clip a code line to `width` columns, marking the cut with an ellipsis.
fn clip(text: &str, width: usize) -> String {
    if text.width() <= width {
        return text.to_owned();
    }
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w + 1 > width {
            break;
        }
        used += w;
        out.push(ch);
    }
    out.push('…');
    out
}

fn list(ordered: bool, items: &[String], width: u16, tokens: &Tokens) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let marker = if ordered {
            format!("{:>2}. ", i + 1)
        } else {
            "  • ".to_owned()
        };
        let indent = marker.width();
        let body = markdown::wrap_styled(
            item,
            width.saturating_sub(indent as u16),
            tokens.text,
            tokens,
        );
        for (row, line) in body.into_iter().enumerate() {
            let lead = if row == 0 {
                Span::styled(marker.clone(), tokens.accent)
            } else {
                Span::raw(" ".repeat(indent))
            };
            let mut spans = vec![lead];
            spans.extend(line.spans);
            lines.push(Line::from(spans));
        }
    }
    lines
}

fn image(
    src: &str,
    alt: Option<&str>,
    caption: Option<&str>,
    width: u16,
    tokens: &Tokens,
) -> Vec<Line<'static>> {
    let label = alt.unwrap_or(src);
    let mut lines = markdown::wrap_styled(
        &format!("[image] {label}"),
        width,
        tokens.muted.add_modifier(Modifier::ITALIC),
        tokens,
    );
    if let Some(caption) = caption {
        lines.extend(markdown::wrap_styled(caption, width, tokens.muted, tokens));
    }
    lines
}

fn container(
    children: &[ContentBlock],
    layout: ContainerLayout,
    width: u16,
    tokens: &Tokens,
) -> Vec<Line<'static>> {
    match layout {
        ContainerLayout::Stack => render_blocks(children, width, tokens),
        ContainerLayout::Columns => columns(children, width, tokens),
        ContainerLayout::Center => center(children, width, tokens),
    }
}

const GUTTER: u16 = 2;

/// Side-by-side children: equal column widths, in array order.
fn columns(children: &[ContentBlock], width: u16, tokens: &Tokens) -> Vec<Line<'static>> {
    let n = children.len() as u16;
    if n == 0 {
        return Vec::new();
    }
    let col_width = width.saturating_sub(GUTTER * (n - 1)) / n;
    if col_width < 8 {
        // Too narrow to read side by side — gracefully fall back to a stack.
        return render_blocks(children, width, tokens);
    }

    let cols: Vec<Vec<Line<'static>>> = children
        .iter()
        .map(|c| render_block(c, col_width, tokens))
        .collect();
    let rows = cols.iter().map(Vec::len).max().unwrap_or(0);

    let mut lines = Vec::with_capacity(rows);
    for row in 0..rows {
        let mut spans = Vec::new();
        for (i, col) in cols.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" ".repeat(GUTTER as usize)));
            }
            let used = match col.get(row) {
                Some(line) => {
                    let w = line.width();
                    spans.extend(line.spans.iter().cloned());
                    w
                }
                None => 0,
            };
            if i + 1 < cols.len() {
                spans.push(Span::raw(" ".repeat((col_width as usize).saturating_sub(used))));
            }
        }
        lines.push(Line::from(spans));
    }
    lines
}

/// Center children as a unit: the flow keeps its internal alignment and is
/// offset to sit in the middle of the available width.
fn center(children: &[ContentBlock], width: u16, tokens: &Tokens) -> Vec<Line<'static>> {
    let inner_width = (u32::from(width) * 4 / 5) as u16;
    let flow = render_blocks(children, inner_width.max(1), tokens);
    let content_width = flow.iter().map(Line::width).max().unwrap_or(0);
    let offset = (width as usize).saturating_sub(content_width) / 2;
    flow.into_iter()
        .map(|line| {
            // Center each line individually so short headings/text sit in the
            // middle, while full-width elements stay put.
            let pad = if line.width() >= content_width {
                offset
            } else {
                offset + (content_width - line.width()) / 2
            };
            let mut spans = vec![Span::raw(" ".repeat(pad))];
            spans.extend(line.spans);
            Line::from(spans)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fireside_core::Graph;

    fn flat(lines: &[Line<'_>]) -> Vec<String> {
        lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect::<String>())
            .collect()
    }

    #[test]
    fn h1_gets_an_underline_rule() {
        let block = ContentBlock::Heading { level: 1, text: "Hi".into() };
        let lines = flat(&render_block(&block, 20, &Tokens::default()));
        assert_eq!(lines, ["Hi", "──"]);
    }

    #[test]
    fn h2_gets_an_accent_bar() {
        let block = ContentBlock::Heading { level: 2, text: "Section".into() };
        let lines = flat(&render_block(&block, 20, &Tokens::default()));
        assert_eq!(lines, ["▎ Section"]);
    }

    #[test]
    fn divider_is_a_short_centered_rule() {
        let lines = flat(&render_block(&ContentBlock::Divider, 30, &Tokens::default()));
        assert_eq!(lines.len(), 1);
        let rule = lines[0].trim();
        assert!(rule.chars().all(|c| c == '─'), "only rule chars: {rule:?}");
        assert!(rule.chars().count() < 30, "shorter than the full width");
        let lead = lines[0].chars().take_while(|c| *c == ' ').count();
        assert!((8..=12).contains(&lead), "centered, got lead {lead}");
    }

    #[test]
    fn code_renders_rules_line_numbers_and_clipping() {
        let block = ContentBlock::Code {
            language: Some("rust".into()),
            source: "fn main() {}\nlet x = 1;".into(),
            highlight_lines: Some(vec![2]),
            show_line_numbers: Some(true),
        };
        let lines = flat(&render_block(&block, 24, &Tokens::default()));
        assert!(lines[0].starts_with("─ rust "));
        assert!(lines[1].contains("1 │ fn main() {}"));
        assert!(lines[2].contains("2 │ let x = 1;"));
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn ordered_list_numbers_items_and_indents_wraps() {
        let block = ContentBlock::List {
            ordered: Some(true),
            items: vec!["first point that wraps onto another line".into()],
        };
        let lines = flat(&render_block(&block, 24, &Tokens::default()));
        assert!(lines[0].starts_with(" 1. first"));
        assert!(lines[1].starts_with("    "));
    }

    #[test]
    fn columns_render_side_by_side_in_array_order() {
        let block = ContentBlock::Container {
            layout: Some(ContainerLayout::Columns),
            children: vec![
                ContentBlock::Text { body: "left".into() },
                ContentBlock::Text { body: "right".into() },
            ],
        };
        let lines = flat(&render_block(&block, 30, &Tokens::default()));
        assert_eq!(lines.len(), 1);
        let pos_l = lines[0].find("left").expect("left present");
        let pos_r = lines[0].find("right").expect("right present");
        assert!(pos_l < pos_r);
    }

    #[test]
    fn narrow_columns_fall_back_to_stack() {
        let block = ContentBlock::Container {
            layout: Some(ContainerLayout::Columns),
            children: vec![
                ContentBlock::Text { body: "left".into() },
                ContentBlock::Text { body: "right".into() },
            ],
        };
        let lines = flat(&render_block(&block, 12, &Tokens::default()));
        assert!(lines.len() > 1);
    }

    #[test]
    fn center_offsets_content_into_the_middle() {
        let block = ContentBlock::Container {
            layout: Some(ContainerLayout::Center),
            children: vec![ContentBlock::Text { body: "hi".into() }],
        };
        let lines = flat(&render_block(&block, 20, &Tokens::default()));
        assert_eq!(lines[0].trim(), "hi");
        let leading = lines[0].len() - lines[0].trim_start().len();
        assert!((8..=10).contains(&leading), "centered, got {leading}");
    }

    #[test]
    fn hello_json_renders_without_panicking_at_any_width() {
        let graph = Graph::from_json(include_str!("../../../../docs/examples/hello.json"))
            .expect("hello parses");
        let tokens = Tokens::default();
        for node in &graph.nodes {
            for width in [0u16, 1, 7, 23, 80, 200] {
                let _ = render_blocks(&node.content, width, &tokens);
            }
        }
    }
}
