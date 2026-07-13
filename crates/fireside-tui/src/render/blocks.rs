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
        ContentBlock::Image {
            src, alt, caption, ..
        } => image(src, alt.as_deref(), caption.as_deref(), width, tokens),
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

/// Code blocks with no language, or `"text"`/`"ascii"`, are the only way to
/// author ASCII art today — they get sized to their content and centered
/// instead of stretched full-width, which is what a real source listing
/// wants (see `code`).
fn is_ascii_art(language: Option<&str>) -> bool {
    matches!(language, None | Some("text") | Some("ascii"))
}

fn code(
    language: Option<&str>,
    source: &str,
    highlight: &[u32],
    line_numbers: bool,
    width: u16,
    tokens: &Tokens,
) -> Vec<Line<'static>> {
    let full_width = width as usize;
    let label = language.unwrap_or("code");
    let label_prefix = format!("─ {label} ");

    let total = source.lines().count();
    let num_width = if line_numbers {
        total.to_string().len()
    } else {
        0
    };
    let prefix = if line_numbers { num_width + 4 } else { 2 };

    let box_width = if is_ascii_art(language) {
        let content_max = source.lines().map(UnicodeWidthStr::width).max().unwrap_or(0);
        (prefix + content_max)
            .max(label_prefix.width())
            .min(full_width)
    } else {
        full_width
    };

    let mut top = label_prefix;
    let fill = box_width.saturating_sub(top.width());
    top.push_str(&"─".repeat(fill));

    let mut lines = vec![Line::styled(top, tokens.border)];
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
        let avail = box_width.saturating_sub(prefix);

        let mut content: Vec<Span<'static>> = match &colored {
            Some(rows) => clip_spans(rows[i].clone(), avail, tokens),
            None => {
                let style = if emphasized {
                    tokens.code_highlight
                } else {
                    tokens.code
                };
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
    lines.push(Line::styled("─".repeat(box_width), tokens.border));

    let pad = full_width.saturating_sub(box_width) / 2;
    if pad > 0 {
        for line in &mut lines {
            let mut spans = vec![Span::raw(" ".repeat(pad))];
            spans.extend(std::mem::take(&mut line.spans));
            line.spans = spans;
        }
    }
    lines
}

/// Clip a row of styled spans to `width` columns, marking any cut with an
/// ellipsis while preserving each span's style.
fn clip_spans(spans: Vec<Span<'static>>, width: usize, tokens: &Tokens) -> Vec<Span<'static>> {
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

/// A terminal cannot paint pixels, so an image becomes a designed
/// placeholder: a small framed plate with the picture's name, and the
/// caption beneath — centered, like a figure in a book.
fn image(
    src: &str,
    alt: Option<&str>,
    caption: Option<&str>,
    width: u16,
    tokens: &Tokens,
) -> Vec<Line<'static>> {
    let label = alt.unwrap_or(src);
    let w = usize::from(width);
    // Too narrow for a frame: a single quiet line.
    if width < 16 {
        let mut lines = markdown::wrap_styled(
            label,
            width,
            tokens.muted.add_modifier(Modifier::ITALIC),
            tokens,
        );
        if let Some(caption) = caption {
            lines.extend(markdown::wrap_styled(caption, width, tokens.muted, tokens));
        }
        return lines;
    }

    let inner = (w - 8).clamp(8, 36) as u16;
    let body = markdown::wrap_styled(label, inner, tokens.text, tokens);
    let text_w = body.iter().map(Line::width).max().unwrap_or(0).max(8);
    let plate_w = text_w + 8;
    let lead = " ".repeat(w.saturating_sub(plate_w) / 2);

    let mut lines = vec![Line::from(vec![
        Span::raw(lead.clone()),
        Span::styled("╭─ ".to_owned(), tokens.border),
        Span::styled("▨".to_owned(), tokens.accent),
        Span::styled(format!(" {}╮", "─".repeat(plate_w - 6)), tokens.border),
    ])];
    for row in body {
        let pad_l = (plate_w - 2).saturating_sub(row.width()) / 2;
        let pad_r = (plate_w - 2).saturating_sub(row.width()) - pad_l;
        let mut spans = vec![
            Span::raw(lead.clone()),
            Span::styled("│".to_owned(), tokens.border),
            Span::raw(" ".repeat(pad_l)),
        ];
        spans.extend(row.spans);
        spans.push(Span::raw(" ".repeat(pad_r)));
        spans.push(Span::styled("│".to_owned(), tokens.border));
        lines.push(Line::from(spans));
    }
    lines.push(Line::from(vec![
        Span::raw(lead),
        Span::styled(format!("╰{}╯", "─".repeat(plate_w - 2)), tokens.border),
    ]));

    if let Some(caption) = caption {
        for row in markdown::wrap_styled(caption, width, tokens.muted, tokens) {
            let pad = w.saturating_sub(row.width()) / 2;
            let mut spans = vec![Span::raw(" ".repeat(pad))];
            spans.extend(row.spans);
            lines.push(Line::from(spans));
        }
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
                spans.push(Span::raw(
                    " ".repeat((col_width as usize).saturating_sub(used)),
                ));
            }
        }
        lines.push(Line::from(spans));
    }
    lines
}

/// Center children on the container's axis. Prose (headings, text) centers
/// line by line, the way a title slide reads; everything else (code, lists,
/// images) moves as one unit so its internal alignment holds.
fn center(children: &[ContentBlock], width: u16, tokens: &Tokens) -> Vec<Line<'static>> {
    let inner_width = (u32::from(width) * 4 / 5) as u16;
    let mut lines = Vec::new();
    for (i, child) in children.iter().enumerate() {
        if i > 0 {
            lines.push(Line::default());
        }
        let flow = render_block(child, inner_width.max(1), tokens);
        let prose = matches!(
            child,
            ContentBlock::Heading { .. } | ContentBlock::Text { .. }
        );
        let unit_width = flow.iter().map(Line::width).max().unwrap_or(0);
        for line in flow {
            let w = if prose { line.width() } else { unit_width };
            let pad = usize::from(width).saturating_sub(w) / 2;
            let mut spans = vec![Span::raw(" ".repeat(pad))];
            spans.extend(line.spans);
            lines.push(Line::from(spans));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use fireside_core::Graph;

    fn flat(lines: &[Line<'_>]) -> Vec<String> {
        lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn h1_gets_an_underline_rule() {
        let block = ContentBlock::Heading {
            level: 1,
            text: "Hi".into(),
        };
        let lines = flat(&render_block(&block, 20, &Tokens::default()));
        assert_eq!(lines, ["Hi", "──"]);
    }

    #[test]
    fn h2_gets_an_accent_bar() {
        let block = ContentBlock::Heading {
            level: 2,
            text: "Section".into(),
        };
        let lines = flat(&render_block(&block, 20, &Tokens::default()));
        assert_eq!(lines, ["▎ Section"]);
    }

    #[test]
    fn divider_is_a_short_centered_rule() {
        let lines = flat(&render_block(
            &ContentBlock::Divider,
            30,
            &Tokens::default(),
        ));
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
                ContentBlock::Text {
                    body: "left".into(),
                },
                ContentBlock::Text {
                    body: "right".into(),
                },
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
                ContentBlock::Text {
                    body: "left".into(),
                },
                ContentBlock::Text {
                    body: "right".into(),
                },
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
    fn centered_code_keeps_its_internal_alignment() {
        let block = ContentBlock::Container {
            layout: Some(ContainerLayout::Center),
            children: vec![ContentBlock::Code {
                language: None,
                source: "short\na longer line".into(),
                highlight_lines: None,
                show_line_numbers: None,
            }],
        };
        let lines = flat(&render_block(&block, 40, &Tokens::default()));
        let lead_short = lines[1].find("short").expect("short present");
        let lead_long = lines[2].find("a longer line").expect("long present");
        assert_eq!(
            lead_short, lead_long,
            "code lines share one left edge: {lines:?}"
        );
    }

    #[test]
    fn image_renders_a_framed_plate_with_caption() {
        let block = ContentBlock::Image {
            src: "fire.png".into(),
            alt: Some("A campfire".into()),
            caption: Some("Warm".into()),
            width: None,
            height: None,
        };
        let lines = flat(&render_block(&block, 40, &Tokens::default()));
        assert!(lines[0].contains("╭─ ▨"), "framed top: {lines:?}");
        assert!(lines[1].contains("│"), "framed side: {lines:?}");
        assert!(lines[1].contains("A campfire"), "label shown: {lines:?}");
        assert!(lines[2].contains("╰"), "framed bottom: {lines:?}");
        assert!(lines[3].contains("Warm"), "caption beneath: {lines:?}");
        // The plate is centered.
        let lead = lines[0].chars().take_while(|c| *c == ' ').count();
        assert!(lead > 0, "centered plate: {lines:?}");
    }

    #[test]
    fn narrow_image_falls_back_to_a_quiet_line() {
        let block = ContentBlock::Image {
            src: "fire.png".into(),
            alt: Some("A campfire".into()),
            caption: None,
            width: None,
            height: None,
        };
        let lines = flat(&render_block(&block, 12, &Tokens::default()));
        assert!(lines[0].contains("A campfire"), "{lines:?}");
        assert!(!lines[0].contains('╭'), "no frame this narrow: {lines:?}");
    }

    #[test]
    fn ascii_art_code_block_centers_to_its_content_width() {
        let block = ContentBlock::Code {
            language: None,
            source: " /\\_/\\ \n( o.o )\n > ^ < ".into(),
            highlight_lines: None,
            show_line_numbers: None,
        };
        let lines = flat(&render_block(&block, 40, &Tokens::default()));
        let box_width = lines.iter().map(|l| l.width()).max().unwrap_or(0);
        assert!(box_width < 40, "box should not stretch full width: {lines:?}");
        // The bottom rule is pure pad + dashes, so its leading-space count
        // is exactly the centering pad with no ambiguity from content
        // whitespace — every other line must share that same prefix.
        let bottom = lines.last().expect("bottom rule present");
        let pad = bottom.len() - bottom.trim_start_matches(' ').len();
        let pad_str = " ".repeat(pad);
        assert!(
            lines.iter().all(|l| l.starts_with(&pad_str)),
            "every line shares the same leading pad {pad}: {lines:?}"
        );
        assert!(pad > 0, "content should be centered, not left-aligned: {lines:?}");
    }

    #[test]
    fn text_and_ascii_language_strings_center_like_no_language() {
        let source = " /\\_/\\ \n( o.o )\n > ^ < ";
        let none_lines = flat(&render_block(
            &ContentBlock::Code {
                language: None,
                source: source.into(),
                highlight_lines: None,
                show_line_numbers: None,
            },
            40,
            &Tokens::default(),
        ));
        for lang in ["text", "ascii"] {
            let lines = flat(&render_block(
                &ContentBlock::Code {
                    language: Some(lang.into()),
                    source: source.into(),
                    highlight_lines: None,
                    show_line_numbers: None,
                },
                40,
                &Tokens::default(),
            ));
            let box_width = lines.iter().map(|l| l.len()).max().unwrap_or(0);
            let none_box_width = none_lines.iter().map(|l| l.len()).max().unwrap_or(0);
            assert_eq!(
                box_width, none_box_width,
                "language {lang:?} should center identically to no language"
            );
        }
    }

    #[test]
    fn explicit_language_code_block_stays_full_width() {
        let block = ContentBlock::Code {
            language: Some("rust".into()),
            source: " /\\_/\\ \n( o.o )\n > ^ < ".into(),
            highlight_lines: None,
            show_line_numbers: None,
        };
        let lines = flat(&render_block(&block, 40, &Tokens::default()));
        assert!(lines[0].starts_with("─ rust "), "{lines:?}");
        assert_eq!(lines[0].chars().count(), 40, "top rule fills full width: {lines:?}");
        let bottom = lines.last().expect("bottom rule present");
        assert_eq!(
            bottom.chars().count(),
            40,
            "bottom rule fills full width: {lines:?}"
        );
        assert!(
            !bottom.starts_with(' '),
            "no centering pad on explicit-language blocks: {lines:?}"
        );
    }

    #[test]
    fn oversized_ascii_art_caps_and_clips_with_ellipsis() {
        let long_line = "x".repeat(200);
        let block = ContentBlock::Code {
            language: None,
            source: long_line,
            highlight_lines: None,
            show_line_numbers: None,
        };
        let lines = flat(&render_block(&block, 30, &Tokens::default()));
        let box_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        assert_eq!(box_width, 30, "box caps at available width: {lines:?}");
        assert!(lines[1].contains('…'), "overflow is marked: {lines:?}");
    }

    #[test]
    fn ascii_art_never_panics_across_a_range_of_widths() {
        let block = ContentBlock::Code {
            language: None,
            source: "x".repeat(200),
            highlight_lines: None,
            show_line_numbers: None,
        };
        for width in [0u16, 1, 2, 5, 10, 40, 200] {
            let _ = render_block(&block, width, &Tokens::default());
        }
    }

    #[test]
    fn empty_ascii_art_code_block_does_not_collapse_or_panic() {
        let block = ContentBlock::Code {
            language: None,
            source: String::new(),
            highlight_lines: None,
            show_line_numbers: None,
        };
        let lines = flat(&render_block(&block, 40, &Tokens::default()));
        assert!(lines[0].contains("code"), "top rule shows the label: {lines:?}");
        let last = lines.last().expect("bottom rule present");
        assert!(!last.is_empty(), "bottom rule is not empty: {lines:?}");
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
