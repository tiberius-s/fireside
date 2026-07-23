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
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::{markdown, syntax};
use crate::theme::Tokens;

/// A block whose reveal step has not yet been reached at `reveal_level` —
/// structurally absent, not merely styled invisible (see
/// `specs/006-incremental-reveal/contracts/reveal-field.md`).
fn is_revealed(block: &ContentBlock, reveal_level: u32) -> bool {
    block.reveal().unwrap_or(0) <= reveal_level
}

/// The subset of `blocks` visible at `reveal_level`, in order. Filtering
/// here (rather than skipping during layout) is what keeps a hidden block
/// from reserving space — e.g. an unrevealed `columns` child never affects
/// the column-width division.
fn visible_blocks(blocks: &[ContentBlock], reveal_level: u32) -> Vec<&ContentBlock> {
    blocks
        .iter()
        .filter(|b| is_revealed(b, reveal_level))
        .collect()
}

/// Render a node's blocks to a line flow at `width` columns, with one blank
/// line between blocks. `reveal_level` is the presenter's current reveal
/// threshold for this node (`0` when the node uses no reveal marks, or
/// while nothing has been revealed yet) — blocks not yet reached are
/// omitted entirely, not dimmed.
#[must_use]
pub fn render_blocks(
    blocks: &[ContentBlock],
    width: u16,
    tokens: &Tokens,
    reveal_level: u32,
) -> Vec<Line<'static>> {
    let visible = visible_blocks(blocks, reveal_level);
    let mut lines = Vec::new();
    for (i, block) in visible.into_iter().enumerate() {
        if i > 0 {
            lines.push(Line::default());
        }
        lines.extend(render_block(block, width, tokens, reveal_level));
    }
    lines
}

fn render_block(
    block: &ContentBlock,
    width: u16,
    tokens: &Tokens,
    reveal_level: u32,
) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }
    match block {
        ContentBlock::Heading { level, text, .. } => heading(*level, text, width, tokens),
        ContentBlock::Text { body, .. } => markdown::wrap_styled(body, width, tokens.text, tokens),
        ContentBlock::Code {
            language,
            source,
            highlight_lines,
            show_line_numbers,
            ..
        } => code(
            language.as_deref(),
            source,
            highlight_lines.as_deref().unwrap_or_default(),
            show_line_numbers.unwrap_or(false),
            width,
            tokens,
        ),
        ContentBlock::List { ordered, items, .. } => {
            list(ordered.unwrap_or(false), items, width, tokens)
        }
        ContentBlock::Image {
            src, alt, caption, ..
        } => image(src, alt.as_deref(), caption.as_deref(), width, tokens),
        ContentBlock::Divider { .. } => divider(width, tokens),
        ContentBlock::Container {
            children, layout, ..
        } => container(
            children,
            layout.unwrap_or_default(),
            width,
            tokens,
            reveal_level,
        ),
        ContentBlock::AsciiArt { art, alt, .. } => ascii_art(art, alt.as_deref(), width, tokens),
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

/// Box width for a sized-to-content, centered block: the content's widest
/// line plus `prefix`, or `label_width`, whichever is larger, capped at
/// `full_width`. Shared by `code()`'s ASCII-art path (spec 005) and
/// `ascii_art()` (spec 009) so both give the same "sized to itself,
/// centered" treatment from one formula.
fn centered_box_width<'a>(
    label_width: usize,
    lines: impl Iterator<Item = &'a str>,
    prefix: usize,
    full_width: usize,
) -> usize {
    let content_max = lines.map(UnicodeWidthStr::width).max().unwrap_or(0);
    (prefix + content_max).max(label_width).min(full_width)
}

fn code(
    language: Option<&str>,
    source: &str,
    highlight: &[u32],
    line_numbers: bool,
    width: u16,
    tokens: &Tokens,
) -> Vec<Line<'static>> {
    // P1-3: expand tabs before anything measures or highlights the source,
    // so a gofmt'd/tab-indented block keeps its indentation on screen.
    let source = super::expand_tabs(source);
    let source = source.as_str();
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
        centered_box_width(label_prefix.width(), source.lines(), prefix, full_width)
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

/// Pre-rendered ASCII/text art (spec 009): centered and sized to its own
/// widest line, with no syntax highlighting, line numbers, or
/// highlighted-line concept — this kind has none of those fields.
/// P2-1: rendered unframed and unlabeled — it's art, not a code listing,
/// and centering already sets it apart from body text — so the audience
/// never sees the implementation-jargon `─ ascii-art ─` label. `alt` is
/// accessibility metadata (`contracts/ascii-art-block.md`); when present
/// it is shown as a muted caption beneath the art, the same treatment
/// `image()` gives its `caption` field, rather than being discarded.
fn ascii_art(art: &str, alt: Option<&str>, width: u16, tokens: &Tokens) -> Vec<Line<'static>> {
    let full_width = width as usize;
    let box_width = art
        .lines()
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0)
        .min(full_width);

    let mut lines: Vec<Line<'static>> = art
        .lines()
        .map(|raw| Line::from(Span::styled(clip(raw, box_width), tokens.code)))
        .collect();

    let pad = full_width.saturating_sub(box_width) / 2;
    if pad > 0 {
        for line in &mut lines {
            let mut spans = vec![Span::raw(" ".repeat(pad))];
            spans.extend(std::mem::take(&mut line.spans));
            line.spans = spans;
        }
    }

    if let Some(alt) = alt {
        for row in markdown::wrap_styled(alt, width, tokens.muted, tokens) {
            let cap_pad = full_width.saturating_sub(row.width()) / 2;
            let mut spans = vec![Span::raw(" ".repeat(cap_pad))];
            spans.extend(row.spans);
            lines.push(Line::from(spans));
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
    reveal_level: u32,
) -> Vec<Line<'static>> {
    match layout {
        ContainerLayout::Stack => render_blocks(children, width, tokens, reveal_level),
        ContainerLayout::Columns => columns(children, width, tokens, reveal_level),
        ContainerLayout::Center => center(children, width, tokens, reveal_level),
    }
}

const GUTTER: u16 = 2;

/// Side-by-side children: equal column widths, in array order. A child not
/// yet revealed is excluded before the column count/width is computed, so
/// it never reserves a blank slot.
fn columns(
    children: &[ContentBlock],
    width: u16,
    tokens: &Tokens,
    reveal_level: u32,
) -> Vec<Line<'static>> {
    let visible = visible_blocks(children, reveal_level);
    let n = visible.len() as u16;
    if n == 0 {
        return Vec::new();
    }
    let col_width = width.saturating_sub(GUTTER * (n - 1)) / n;
    if col_width < 8 {
        // Too narrow to read side by side — gracefully fall back to a stack.
        return render_blocks(children, width, tokens, reveal_level);
    }

    let cols: Vec<Vec<Line<'static>>> = visible
        .into_iter()
        .map(|c| render_block(c, col_width, tokens, reveal_level))
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

/// The visible column span (start, end) of a line's non-space content, or
/// `None` for a blank line.
fn content_span(line: &Line<'static>) -> Option<(usize, usize)> {
    let mut col = 0usize;
    let mut start = None;
    let mut end = 0usize;
    for span in &line.spans {
        for ch in span.content.chars() {
            let w = ch.width().unwrap_or(0);
            if ch != ' ' {
                start.get_or_insert(col);
                end = col + w;
            }
            col += w;
        }
    }
    start.map(|s| (s, end))
}

/// Drops `n` columns of (assumed blank) leading content from a line's
/// spans — undoes the pad a self-centering child (ascii-art, code) already
/// applied while rendering itself at the container's narrower inner width,
/// so `center` can re-pad it against its own, wider axis instead of
/// compounding the two into a block shifted off true center.
fn strip_leading(line: Line<'static>, n: usize) -> Line<'static> {
    let mut remaining = n;
    let mut spans = Vec::with_capacity(line.spans.len());
    for span in line.spans {
        if remaining == 0 {
            spans.push(span);
            continue;
        }
        let content = span.content.into_owned();
        let w = UnicodeWidthStr::width(content.as_str());
        if w <= remaining {
            remaining -= w;
            continue;
        }
        let mut col = 0usize;
        let mut byte = content.len();
        for (i, ch) in content.char_indices() {
            if col >= remaining {
                byte = i;
                break;
            }
            col += ch.width().unwrap_or(0);
        }
        spans.push(Span::styled(content[byte..].to_owned(), span.style));
        remaining = 0;
    }
    Line::from(spans)
}

/// Center children on the container's axis. Prose (headings, text) centers
/// line by line, the way a title slide reads; everything else (code, lists,
/// images) moves as one unit so its internal alignment holds.
fn center(
    children: &[ContentBlock],
    width: u16,
    tokens: &Tokens,
    reveal_level: u32,
) -> Vec<Line<'static>> {
    let inner_width = (u32::from(width) * 4 / 5) as u16;
    let mut lines = Vec::new();
    for (i, child) in visible_blocks(children, reveal_level)
        .into_iter()
        .enumerate()
    {
        if i > 0 {
            lines.push(Line::default());
        }
        let flow = render_block(child, inner_width.max(1), tokens, reveal_level);
        let prose = matches!(
            child,
            ContentBlock::Heading { .. } | ContentBlock::Text { .. }
        );
        // Non-prose children render themselves already centered inside
        // `inner_width` (a uniform blank run on every line). Measuring the
        // block's *tight* bounding box — instead of trusting the raw
        // `Line::width()`, which still carries that self-applied pad —
        // and stripping it before re-padding against the full `width`
        // keeps this from double-counting that pad and shifting the
        // block off axis (see the title-slide ascii-art regression).
        let (leading, unit_width) = if prose {
            (0, 0)
        } else {
            let spans: Vec<(usize, usize)> = flow.iter().filter_map(content_span).collect();
            let left = spans.iter().map(|&(s, _)| s).min().unwrap_or(0);
            let right = spans.iter().map(|&(_, e)| e).max().unwrap_or(0);
            (left, right.saturating_sub(left))
        };
        for line in flow {
            let line = if !prose && leading > 0 {
                strip_leading(line, leading)
            } else {
                line
            };
            let w = if prose { line.width() } else { unit_width };
            let pad = usize::from(width).saturating_sub(w) / 2;
            let mut spans = vec![Span::raw(" ".repeat(pad))];
            spans.extend(line.spans);
            lines.push(Line::from(spans));
        }
    }
    lines
}

/// One child's on-screen extent within its container's own rendered line
/// flow, relative to the container's own row 0 (spec 014). `cols: None`
/// means the child spans the container's full width, exactly like a
/// top-level block (`Stack`/`Center`); `Some((x0, x1))` gives its column
/// sub-range for a side-by-side `Columns` layout, whose children share
/// the container's rows but not its columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ChildGeometry {
    pub(crate) rows: (usize, usize),
    pub(crate) cols: Option<(u16, u16)>,
}

/// Each of `children`'s own on-screen extents within their container's
/// rendered output at `width` columns — the same geometry `container`
/// itself draws, computed independently so hit-testing/selection can
/// never disagree with it (spec 014, mirroring the top-level
/// `editor::hit::block_extents`'s "recompute, don't observe" contract).
///
/// Callers MUST pass a `reveal_level` at least as high as every child's
/// own reveal step (the editor always calls this with `u32::MAX`, same as
/// `editor::hit::canvas_layout` does for the top-level case) — the
/// returned `Vec` is only guaranteed index-aligned with `children` under
/// that assumption; this function has no other caller.
#[must_use]
pub(crate) fn container_child_geometry(
    children: &[ContentBlock],
    layout: ContainerLayout,
    width: u16,
    tokens: &Tokens,
    reveal_level: u32,
) -> Vec<ChildGeometry> {
    if layout == ContainerLayout::Columns {
        return columns_child_geometry(children, width, tokens, reveal_level);
    }
    // `Stack` and `Center` both render each child independently (no
    // cross-child layout decision like column-width division), so the
    // same increasing-prefix technique `editor::hit::block_extents` uses
    // at the top level applies here unchanged, just against `container`
    // instead of `render_blocks` directly (for `Stack` the two are the
    // same function anyway).
    let mut out = Vec::with_capacity(children.len());
    let mut prev = 0usize;
    for i in 0..children.len() {
        let cumulative = container(&children[..=i], layout, width, tokens, reveal_level).len();
        let start = if i == 0 { 0 } else { prev + 1 };
        out.push(ChildGeometry {
            rows: (start, cumulative),
            cols: None,
        });
        prev = cumulative;
    }
    out
}

/// `Columns`' own child geometry: side-by-side column bands, mirroring
/// `columns()`'s exact `col_width` math (including its narrow-width
/// fallback to a stack) so the two can never disagree about where a
/// column sits. A child's row range is its own rendered height within its
/// column, not padded down to the tallest sibling — the blank space below
/// a shorter column isn't part of any child's clickable extent.
fn columns_child_geometry(
    children: &[ContentBlock],
    width: u16,
    tokens: &Tokens,
    reveal_level: u32,
) -> Vec<ChildGeometry> {
    let n = children.len() as u16;
    if n == 0 {
        return Vec::new();
    }
    let col_width = width.saturating_sub(GUTTER * (n - 1)) / n;
    if col_width < 8 {
        return container_child_geometry(
            children,
            ContainerLayout::Stack,
            width,
            tokens,
            reveal_level,
        );
    }
    let mut out = Vec::with_capacity(children.len());
    let mut x = 0u16;
    for child in children {
        let rows_len = render_block(child, col_width, tokens, reveal_level).len();
        out.push(ChildGeometry {
            rows: (0, rows_len),
            cols: Some((x, x + col_width)),
        });
        x += col_width + GUTTER;
    }
    out
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

    /// `render_block` at reveal level 0 — the vast majority of tests here
    /// don't exercise reveal at all, so this keeps them uncluttered.
    fn render(block: &ContentBlock, width: u16, tokens: &Tokens) -> Vec<Line<'static>> {
        render_block(block, width, tokens, 0)
    }

    #[test]
    fn h1_gets_an_underline_rule() {
        let block = ContentBlock::Heading {
            reveal: None,
            level: 1,
            text: "Hi".into(),
        };
        let lines = flat(&render(&block, 20, &Tokens::default()));
        assert_eq!(lines, ["Hi", "──"]);
    }

    /// Spec 008 US4: proves the H1 underline rule (sized to the text's
    /// rendered width, per `heading()`'s `Line::width().min(width)`) is
    /// measured by true display width, not `char` count — CJK ideographs
    /// are double-width, so a char-counting bug would produce a
    /// noticeably shorter (wrong) rule. The expected rule length is
    /// computed via the same `unicode-width` crate the production code
    /// uses (not a hand-picked magic number), so this test is about
    /// proving display-width measurement is used consistently, not about
    /// asserting a specific width value.
    #[test]
    fn heading_with_emoji_and_cjk_measures_by_display_width() {
        let text = "你好 🎉 world";
        let expected_width = UnicodeWidthStr::width(text);
        assert_ne!(
            expected_width,
            text.chars().count(),
            "the fixture must actually differ under width vs. char-count \
             measurement, or this test proves nothing"
        );

        let block = ContentBlock::Heading {
            reveal: None,
            level: 1,
            text: text.into(),
        };
        // Wide enough that the heading doesn't wrap — isolates the
        // underline-sizing behavior this test targets.
        let lines = flat(&render(&block, 40, &Tokens::default()));
        assert_eq!(lines[0], text);
        assert_eq!(lines[1].chars().count(), expected_width);
        assert!(lines[1].chars().all(|c| c == '─'));
    }

    /// Spec 008 US4: a heading with wide characters clipped/wrapped at a
    /// narrow width must never overflow that width when measured by true
    /// display width (a byte- or char-counting bug could either overflow
    /// visually or clip too aggressively).
    #[test]
    fn heading_with_cjk_wraps_without_overflowing_narrow_width() {
        let block = ContentBlock::Heading {
            reveal: None,
            level: 1,
            text: "你好世界这是一个很长的标题".into(),
        };
        let width = 10;
        let lines = render(&block, width, &Tokens::default());
        for line in &lines {
            assert!(
                line.width() <= width as usize,
                "line {line:?} overflows width {width}"
            );
        }
    }

    #[test]
    fn h2_gets_an_accent_bar() {
        let block = ContentBlock::Heading {
            reveal: None,
            level: 2,
            text: "Section".into(),
        };
        let lines = flat(&render(&block, 20, &Tokens::default()));
        assert_eq!(lines, ["▎ Section"]);
    }

    #[test]
    fn divider_is_a_short_centered_rule() {
        let lines = flat(&render(
            &ContentBlock::Divider { reveal: None },
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
            reveal: None,
            language: Some("rust".into()),
            source: "fn main() {}\nlet x = 1;".into(),
            highlight_lines: Some(vec![2]),
            show_line_numbers: Some(true),
        };
        let lines = flat(&render(&block, 24, &Tokens::default()));
        assert!(lines[0].starts_with("─ rust "));
        assert!(lines[1].contains("1 │ fn main() {}"));
        assert!(lines[2].contains("2 │ let x = 1;"));
        assert_eq!(lines.len(), 4);
    }

    /// P1-3: a gofmt'd (tab-indented) code block must keep its indentation
    /// on screen instead of ratatui silently dropping the raw `\t`.
    #[test]
    fn code_expands_tabs_instead_of_dropping_indentation() {
        let block = ContentBlock::Code {
            reveal: None,
            language: Some("go".into()),
            source: "func main() {\n\tfmt.Println(\"hi\")\n}".into(),
            highlight_lines: None,
            show_line_numbers: None,
        };
        let lines = flat(&render(&block, 40, &Tokens::default()));
        assert!(
            lines[2].contains("    fmt.Println"),
            "tab expanded to spaces, indentation preserved: {:?}",
            lines[2]
        );
        assert!(
            !lines[2].contains('\t'),
            "no raw tab should reach the rendered line: {:?}",
            lines[2]
        );
    }

    #[test]
    fn ordered_list_numbers_items_and_indents_wraps() {
        let block = ContentBlock::List {
            reveal: None,
            ordered: Some(true),
            items: vec!["first point that wraps onto another line".into()],
        };
        let lines = flat(&render(&block, 24, &Tokens::default()));
        assert!(lines[0].starts_with(" 1. first"));
        assert!(lines[1].starts_with("    "));
    }

    #[test]
    fn columns_render_side_by_side_in_array_order() {
        let block = ContentBlock::Container {
            reveal: None,
            layout: Some(ContainerLayout::Columns),
            children: vec![
                ContentBlock::Text {
                    reveal: None,
                    body: "left".into(),
                },
                ContentBlock::Text {
                    reveal: None,
                    body: "right".into(),
                },
            ],
        };
        let lines = flat(&render(&block, 30, &Tokens::default()));
        assert_eq!(lines.len(), 1);
        let pos_l = lines[0].find("left").expect("left present");
        let pos_r = lines[0].find("right").expect("right present");
        assert!(pos_l < pos_r);
    }

    /// Spec 008 US4: a column's right-hand neighbor starts at a fixed
    /// offset (`col_width + GUTTER`) computed purely from the container
    /// width — it must be identical whether the left column holds
    /// wide (CJK) or ASCII content of the same true display width. The
    /// ASCII comparison string's length is derived from the CJK string's
    /// *measured* width (not hand-picked), so this test is agnostic to
    /// the exact width the `unicode-width` crate assigns to any given
    /// character — it only asserts the two are measured consistently.
    #[test]
    fn columns_with_wide_characters_stay_aligned() {
        let cjk_left = "你好世界";
        let ascii_left = "a".repeat(UnicodeWidthStr::width(cjk_left));
        assert_ne!(
            cjk_left.chars().count(),
            ascii_left.chars().count(),
            "the fixture must actually exercise a char-count vs. \
             display-width difference, or this test proves nothing"
        );

        let build = |left: &str| ContentBlock::Container {
            reveal: None,
            layout: Some(ContainerLayout::Columns),
            children: vec![
                ContentBlock::Text {
                    reveal: None,
                    body: left.to_owned(),
                },
                ContentBlock::Text {
                    reveal: None,
                    body: "MARK".into(),
                },
            ],
        };

        let cjk_lines = flat(&render(&build(cjk_left), 30, &Tokens::default()));
        let ascii_lines = flat(&render(&build(&ascii_left), 30, &Tokens::default()));

        // `str::find` returns a *byte* offset, not a display column — CJK
        // characters are 3 bytes each in UTF-8, so the byte offset of
        // "MARK" legitimately differs even when its display column
        // doesn't. Measure the column position instead: the display width
        // of everything before "MARK".
        let column_of_mark = |line: &str| {
            let byte_pos = line.find("MARK").expect("MARK present");
            UnicodeWidthStr::width(&line[..byte_pos])
        };
        let cjk_col = column_of_mark(&cjk_lines[0]);
        let ascii_col = column_of_mark(&ascii_lines[0]);
        assert_eq!(
            cjk_col, ascii_col,
            "the right column must start at the same display column regardless \
             of whether the left column's content is CJK or ASCII, given equal \
             display width: cjk_lines={cjk_lines:?} ascii_lines={ascii_lines:?}"
        );
    }

    #[test]
    fn narrow_columns_fall_back_to_stack() {
        let block = ContentBlock::Container {
            reveal: None,
            layout: Some(ContainerLayout::Columns),
            children: vec![
                ContentBlock::Text {
                    reveal: None,
                    body: "left".into(),
                },
                ContentBlock::Text {
                    reveal: None,
                    body: "right".into(),
                },
            ],
        };
        let lines = flat(&render(&block, 12, &Tokens::default()));
        assert!(lines.len() > 1);
    }

    #[test]
    fn center_offsets_content_into_the_middle() {
        let block = ContentBlock::Container {
            reveal: None,
            layout: Some(ContainerLayout::Center),
            children: vec![ContentBlock::Text {
                reveal: None,
                body: "hi".into(),
            }],
        };
        let lines = flat(&render(&block, 20, &Tokens::default()));
        assert_eq!(lines[0].trim(), "hi");
        let leading = lines[0].len() - lines[0].trim_start().len();
        assert!((8..=10).contains(&leading), "centered, got {leading}");
    }

    #[test]
    fn centered_code_keeps_its_internal_alignment() {
        let block = ContentBlock::Container {
            reveal: None,
            layout: Some(ContainerLayout::Center),
            children: vec![ContentBlock::Code {
                reveal: None,
                language: None,
                source: "short\na longer line".into(),
                highlight_lines: None,
                show_line_numbers: None,
            }],
        };
        let lines = flat(&render(&block, 40, &Tokens::default()));
        let lead_short = lines[1].find("short").expect("short present");
        let lead_long = lines[2].find("a longer line").expect("long present");
        assert_eq!(
            lead_short, lead_long,
            "code lines share one left edge: {lines:?}"
        );
    }

    /// Regression for the title-slide bug: `code()`'s ascii-art path
    /// already centers itself within the narrower `inner_width` a `center`
    /// container renders it at (a uniform left pad, no matching right
    /// pad). Naively re-centering that already-padded result against the
    /// container's full width double-counts the child's own pad and
    /// shifts the whole block right of true center — exactly what shipped
    /// on the demo deck's welcome slide.
    #[test]
    fn centered_ascii_art_lands_on_true_center_not_shifted_by_its_own_inner_pad() {
        // Content wide enough that the label prefix never dominates
        // `centered_box_width`'s `.max(label_width)`, so the box width is
        // simply `prefix(2) + content_max(20)` — 22 — independent of the
        // formula under test.
        let source = format!("{}\n{}", "x".repeat(20), "y".repeat(10));
        let block = ContentBlock::Container {
            reveal: None,
            layout: Some(ContainerLayout::Center),
            children: vec![ContentBlock::Code {
                reveal: None,
                language: None,
                source,
                highlight_lines: None,
                show_line_numbers: None,
            }],
        };
        let width: usize = 60;
        let box_width = 22;
        // True center for a `box_width`-wide block in a `width`-wide
        // container — computed independently of `center()`'s own math.
        let expected_left = (width - box_width) / 2;

        let lines = flat(&render(&block, width as u16, &Tokens::default()));
        // The bottom rule is pure pad + dashes, so its leading-space count
        // is exactly the block's centering pad with no ambiguity from a
        // content row's own gutter indent — every line must share that
        // same prefix (block moves as one unit), and that prefix must be
        // true center, not shifted right by the child's own inner-width
        // self-centering pad (the bug: the demo deck's title-slide ascii
        // art landed ~5 columns right of center).
        let bottom = lines.last().expect("bottom rule present");
        let pad = bottom.len() - bottom.trim_start_matches(' ').len();
        let pad_str = " ".repeat(pad);
        assert!(
            lines.iter().all(|l| l.starts_with(&pad_str)),
            "block must move as one unit, every line sharing pad {pad}: {lines:?}"
        );
        assert_eq!(
            pad, expected_left,
            "expected the block on the true center axis: {lines:?}"
        );
    }

    #[test]
    fn image_renders_a_framed_plate_with_caption() {
        let block = ContentBlock::Image {
            reveal: None,
            src: "fire.png".into(),
            alt: Some("A campfire".into()),
            caption: Some("Warm".into()),
            width: None,
            height: None,
        };
        let lines = flat(&render(&block, 40, &Tokens::default()));
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
            reveal: None,
            src: "fire.png".into(),
            alt: Some("A campfire".into()),
            caption: None,
            width: None,
            height: None,
        };
        let lines = flat(&render(&block, 12, &Tokens::default()));
        assert!(lines[0].contains("A campfire"), "{lines:?}");
        assert!(!lines[0].contains('╭'), "no frame this narrow: {lines:?}");
    }

    #[test]
    fn ascii_art_code_block_centers_to_its_content_width() {
        let block = ContentBlock::Code {
            reveal: None,
            language: None,
            source: " /\\_/\\ \n( o.o )\n > ^ < ".into(),
            highlight_lines: None,
            show_line_numbers: None,
        };
        let lines = flat(&render(&block, 40, &Tokens::default()));
        let box_width = lines.iter().map(|l| l.width()).max().unwrap_or(0);
        assert!(
            box_width < 40,
            "box should not stretch full width: {lines:?}"
        );
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
        assert!(
            pad > 0,
            "content should be centered, not left-aligned: {lines:?}"
        );
    }

    #[test]
    fn text_and_ascii_language_strings_center_like_no_language() {
        let source = " /\\_/\\ \n( o.o )\n > ^ < ";
        let none_lines = flat(&render(
            &ContentBlock::Code {
                reveal: None,
                language: None,
                source: source.into(),
                highlight_lines: None,
                show_line_numbers: None,
            },
            40,
            &Tokens::default(),
        ));
        for lang in ["text", "ascii"] {
            let lines = flat(&render(
                &ContentBlock::Code {
                    reveal: None,
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
            reveal: None,
            language: Some("rust".into()),
            source: " /\\_/\\ \n( o.o )\n > ^ < ".into(),
            highlight_lines: None,
            show_line_numbers: None,
        };
        let lines = flat(&render(&block, 40, &Tokens::default()));
        assert!(lines[0].starts_with("─ rust "), "{lines:?}");
        assert_eq!(
            lines[0].chars().count(),
            40,
            "top rule fills full width: {lines:?}"
        );
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
            reveal: None,
            language: None,
            source: long_line,
            highlight_lines: None,
            show_line_numbers: None,
        };
        let lines = flat(&render(&block, 30, &Tokens::default()));
        let box_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        assert_eq!(box_width, 30, "box caps at available width: {lines:?}");
        assert!(lines[1].contains('…'), "overflow is marked: {lines:?}");
    }

    #[test]
    fn ascii_art_never_panics_across_a_range_of_widths() {
        let block = ContentBlock::Code {
            reveal: None,
            language: None,
            source: "x".repeat(200),
            highlight_lines: None,
            show_line_numbers: None,
        };
        for width in [0u16, 1, 2, 5, 10, 40, 200] {
            let _ = render(&block, width, &Tokens::default());
        }
    }

    #[test]
    fn empty_ascii_art_code_block_does_not_collapse_or_panic() {
        let block = ContentBlock::Code {
            reveal: None,
            language: None,
            source: String::new(),
            highlight_lines: None,
            show_line_numbers: None,
        };
        let lines = flat(&render(&block, 40, &Tokens::default()));
        assert!(
            lines[0].contains("code"),
            "top rule shows the label: {lines:?}"
        );
        let last = lines.last().expect("bottom rule present");
        assert!(!last.is_empty(), "bottom rule is not empty: {lines:?}");
    }

    #[test]
    fn ascii_art_block_renders_unframed_with_alt_as_caption() {
        let block = ContentBlock::AsciiArt {
            reveal: None,
            art: " /\\_/\\ \n( o.o )\n > ^ < ".into(),
            alt: Some("A sleepy cat".into()),
        };
        let lines = flat(&render(&block, 40, &Tokens::default()));
        assert!(
            !lines.iter().any(|l| l.contains("ascii-art")),
            "no implementation-jargon label: {lines:?}"
        );
        assert!(
            !lines.iter().any(|l| l.contains('─')),
            "no frame rules: {lines:?}"
        );
        assert!(lines.iter().any(|l| l.contains("o.o")), "{lines:?}");
        assert!(
            lines.last().unwrap().contains("A sleepy cat"),
            "alt shown as a caption beneath the art: {lines:?}"
        );
    }

    #[test]
    fn ascii_art_block_without_alt_has_no_caption() {
        let block = ContentBlock::AsciiArt {
            reveal: None,
            art: " /\\_/\\ \n( o.o )\n > ^ < ".into(),
            alt: None,
        };
        let lines = flat(&render(&block, 40, &Tokens::default()));
        assert_eq!(lines.len(), 3, "art lines only, no caption row: {lines:?}");
    }

    #[test]
    fn hello_json_renders_without_panicking_at_any_width() {
        let graph = Graph::from_json(include_str!("../../../../docs/examples/hello.json"))
            .expect("hello parses");
        let tokens = Tokens::default();
        for node in &graph.nodes {
            for width in [0u16, 1, 7, 23, 80, 200] {
                let _ = render_blocks(&node.content, width, &tokens, 0);
            }
        }
    }

    #[test]
    fn reveal_hides_a_block_until_its_step_is_reached() {
        let blocks = vec![
            ContentBlock::Text {
                reveal: None,
                body: "always".into(),
            },
            ContentBlock::Text {
                reveal: Some(1),
                body: "first reveal".into(),
            },
        ];
        let hidden = flat(&render_blocks(&blocks, 40, &Tokens::default(), 0));
        assert_eq!(hidden, ["always"], "reveal-gated block absent at level 0");
        let shown = flat(&render_blocks(&blocks, 40, &Tokens::default(), 1));
        assert_eq!(shown, ["always", "", "first reveal"]);
    }

    #[test]
    fn hidden_column_reserves_no_width_until_revealed() {
        let block = ContentBlock::Container {
            reveal: None,
            layout: Some(ContainerLayout::Columns),
            children: vec![
                ContentBlock::Text {
                    reveal: None,
                    body: "left".into(),
                },
                ContentBlock::Text {
                    reveal: Some(1),
                    body: "right".into(),
                },
            ],
        };
        let hidden = flat(&render_block(&block, 30, &Tokens::default(), 0));
        assert!(
            hidden.iter().any(|l| l.contains("left")),
            "left column visible: {hidden:?}"
        );
        assert!(
            !hidden.iter().any(|l| l.contains("right")),
            "right column absent, not blank: {hidden:?}"
        );
        // With the right column absent, "left" uses the space a single
        // column would use — not squeezed into a half-width column with
        // an empty second slot.
        let lead = hidden[0].find("left").unwrap_or(0);
        assert!(
            lead < 3,
            "left column not squeezed into half width: {hidden:?}"
        );

        let shown = flat(&render_block(&block, 30, &Tokens::default(), 1));
        assert_eq!(shown.len(), 1);
        let pos_l = shown[0].find("left").expect("left present");
        let pos_r = shown[0].find("right").expect("right present");
        assert!(
            pos_l < pos_r,
            "both columns side by side once revealed: {shown:?}"
        );
    }
}
