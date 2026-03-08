//! Node transition animation engine.
//!
//! Provides [`transition_lines`] which blends two sequences of ratatui [`Line`]s
//! according to a [`Transition`] kind and a `[0.0, 1.0]` progress value.  All
//! functions here are **pure** — they take inputs and return outputs without
//! touching any mutable state.
//!
//! # Transition kinds
//!
//! | Kind         | Effect                                              |
//! |--------------|-----------------------------------------------------|
//! | `None`       | Instant cut to destination                          |
//! | `Fade`       | Source dims out, brief blank, destination dims in   |
//! | `SlideLeft`  | Destination slides in from the right                |
//! | `SlideRight` | Destination slides in from the left                 |
//! | `Wipe`       | Left-to-right reveal of destination                 |
//! | `Dissolve`   | Random per-character reveal                         |
//! | `Matrix`     | Random character rain before settling on destination|
//! | `Typewriter` | Left-to-right character-by-character reveal         |

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use fireside_core::model::transition::Transition;

use crate::theme::Theme;

/// Blend `from_lines` → `to_lines` at the given `progress` (clamped to `[0,1]`)
/// using the specified `kind`.
///
/// Returns a new `Vec<Line<'static>>` of `max(from.len(), to.len())` rows, each
/// padded or clipped to `width` columns.  Span-level styles are preserved where
/// possible so syntax highlighting and heading colours remain visible throughout
/// the animation.
#[must_use]
pub(super) fn transition_lines(
    from_lines: &[Line<'_>],
    to_lines: &[Line<'_>],
    width: usize,
    kind: Transition,
    progress: f32,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let rows = from_lines.len().max(to_lines.len());
    let mut output = Vec::with_capacity(rows);
    let eased_progress = ease_out_cubic(progress.clamp(0.0, 1.0));
    let reveal = (eased_progress * width as f32).floor() as usize;

    for row in 0..rows {
        let from_line = from_lines.get(row);
        let to_line = to_lines.get(row);

        let line: Line<'static> = match kind {
            Transition::None => pad_line_styled(to_line, width),
            Transition::Fade => {
                // 3-phase: source dim (0–0.4) → blank (0.4–0.6) → dest appears (0.6–1.0)
                if eased_progress < 0.4 {
                    dim_line(from_line, width)
                } else if eased_progress < 0.6 {
                    blank_line(width)
                } else {
                    // Fade destination in: dim modifier until fully arrived
                    if eased_progress < 0.85 {
                        dim_line(to_line, width)
                    } else {
                        pad_line_styled(to_line, width)
                    }
                }
            }
            Transition::SlideLeft => {
                // Destination slides in from the right; blank prefix shrinks.
                let shift = ((1.0 - eased_progress) * width as f32).floor() as usize;
                prefix_blank_line(to_line, shift, width)
            }
            Transition::SlideRight => {
                // Destination slides in from the left; content starts past right edge.
                let shift = ((1.0 - eased_progress) * width as f32).floor() as usize;
                suffix_skip_line(to_line, shift, width)
            }
            Transition::Wipe => {
                // Left-to-right reveal; only `reveal` columns are shown.
                clip_line_at(to_line, reveal.min(width), width)
            }
            Transition::Dissolve => {
                // Per-cell random reveal — preserves destination styles.
                dissolve_row(from_line, to_line, row, width, progress)
            }
            Transition::Matrix => {
                // Character rain before settling on destination.
                matrix_row(to_line, row, width, progress, theme)
            }
            Transition::Typewriter => {
                // Left-to-right character-by-character reveal.
                clip_line_at(to_line, reveal.min(width), width)
            }
        };

        output.push(line);
    }

    output
}

// ── Span-level helpers ────────────────────────────────────────────────────────

/// A styled cell: one Unicode scalar value plus its ratatui [`Style`].
type StyledChar = (char, Style);

/// Flatten a [`Line`]'s spans into an ordered vec of `(char, style)` pairs.
fn line_to_styled_chars(line: &Line<'_>) -> Vec<StyledChar> {
    line.spans
        .iter()
        .flat_map(|span| {
            let style = span.style;
            span.content.chars().map(move |c| (c, style))
        })
        .collect()
}

/// Reconstruct a [`Line<'static>`] from `(char, style)` pairs, merging adjacent
/// cells that share the same [`Style`] into a single [`Span`].
fn styled_chars_to_line(cells: Vec<StyledChar>) -> Line<'static> {
    if cells.is_empty() {
        return Line::default();
    }
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut cur_style = cells[0].1;
    let mut cur_text = String::new();

    for (ch, style) in cells {
        if style == cur_style {
            cur_text.push(ch);
        } else {
            spans.push(Span::styled(cur_text.clone(), cur_style));
            cur_text = ch.to_string();
            cur_style = style;
        }
    }
    if !cur_text.is_empty() {
        spans.push(Span::styled(cur_text, cur_style));
    }
    Line::from(spans)
}

/// Return the styled chars of `line`, padded / clipped to exactly `width` cells.
fn normalised_chars(line: Option<&Line<'_>>, width: usize) -> Vec<StyledChar> {
    let mut chars = line.map(line_to_styled_chars).unwrap_or_default();
    if chars.len() > width {
        chars.truncate(width);
    } else {
        chars.extend(std::iter::repeat_n(
            (' ', Style::default()),
            width - chars.len(),
        ));
    }
    chars
}

/// Render `line` with [`Modifier::DIM`] applied to every span, padded to `width`.
fn dim_line(line: Option<&Line<'_>>, width: usize) -> Line<'static> {
    let chars = normalised_chars(line, width)
        .into_iter()
        .map(|(c, s)| (c, s.add_modifier(Modifier::DIM)))
        .collect();
    styled_chars_to_line(chars)
}

/// Render `line` preserving all styles, padded to `width`.
fn pad_line_styled(line: Option<&Line<'_>>, width: usize) -> Line<'static> {
    styled_chars_to_line(normalised_chars(line, width))
}

/// A blank line of `width` spaces with the default style.
fn blank_line(width: usize) -> Line<'static> {
    Line::from(Span::raw(" ".repeat(width)))
}

/// Prefix `line` with `shift` blank cells, clipping the total to `width`.
fn prefix_blank_line(line: Option<&Line<'_>>, shift: usize, width: usize) -> Line<'static> {
    let content = normalised_chars(line, width.saturating_sub(shift));
    let mut cells: Vec<StyledChar> = std::iter::repeat_n((' ', Style::default()), shift)
        .chain(content)
        .take(width)
        .collect();
    // Ensure exact width.
    cells.extend(std::iter::repeat_n(
        (' ', Style::default()),
        width.saturating_sub(cells.len()),
    ));
    styled_chars_to_line(cells)
}

/// Skip the first `shift` chars of `line`, padding the tail so total is `width`.
///
/// Used for `SlideRight` where the destination enters from off-screen left.
fn suffix_skip_line(line: Option<&Line<'_>>, shift: usize, width: usize) -> Line<'static> {
    let all_chars = line.map(line_to_styled_chars).unwrap_or_default();
    let total_content = all_chars.len();
    // The content is currently off-screen to the right by `shift` cols.
    // We want to show what would be visible if the slide has moved `shift` cols left.
    // i.e. show chars starting at index max(0, total_content - (width - shift)).
    let visible_start = total_content.saturating_sub(width.saturating_sub(shift));
    let visible: Vec<StyledChar> = all_chars
        .into_iter()
        .skip(visible_start)
        .take(width)
        .collect();
    let pad = width.saturating_sub(visible.len());
    let cells: Vec<StyledChar> = visible
        .into_iter()
        .chain(std::iter::repeat_n((' ', Style::default()), pad))
        .collect();
    styled_chars_to_line(cells)
}

/// Show only the first `reveal` cells of `line`, filling the rest with blanks.
fn clip_line_at(line: Option<&Line<'_>>, reveal: usize, width: usize) -> Line<'static> {
    let all_chars = line.map(line_to_styled_chars).unwrap_or_default();
    let cells: Vec<StyledChar> = all_chars
        .into_iter()
        .take(reveal)
        .chain(std::iter::repeat_n(
            (' ', Style::default()),
            width.saturating_sub(reveal),
        ))
        .take(width)
        .collect();
    styled_chars_to_line(cells)
}

/// Random per-cell reveal: destination cell appears when its hash ≤ `progress`.
fn dissolve_row(
    _from: Option<&Line<'_>>,
    to: Option<&Line<'_>>,
    row: usize,
    width: usize,
    progress: f32,
) -> Line<'static> {
    let to_chars = normalised_chars(to, width);
    let cells = to_chars
        .into_iter()
        .enumerate()
        .map(|(col, (ch, style))| {
            let hash = pseudo_rand(row as u32, col as u32, 7) as f32 / u32::MAX as f32;
            if hash <= progress {
                (ch, style)
            } else {
                (' ', Style::default())
            }
        })
        .collect();
    styled_chars_to_line(cells)
}

/// Matrix character rain: glitch chars until the hash threshold reveals the real glyph.
fn matrix_row(
    to: Option<&Line<'_>>,
    row: usize,
    width: usize,
    progress: f32,
    theme: &Theme,
) -> Line<'static> {
    let to_chars = normalised_chars(to, width);
    let matrix_glyphs = ['░', '▒', '▓'];
    let matrix_style = Style::default().fg(theme.heading_h2);

    let cells: Vec<StyledChar> = to_chars
        .into_iter()
        .enumerate()
        .map(|(col, (ch, style))| {
            let hash = pseudo_rand(row as u32, col as u32, 31) as f32 / u32::MAX as f32;
            if hash <= progress {
                (ch, style)
            } else {
                let idx = (pseudo_rand(row as u32, col as u32, 13) % 3) as usize;
                (matrix_glyphs[idx], matrix_style)
            }
        })
        .collect();
    styled_chars_to_line(cells)
}

// ── Pure helpers ─────────────────────────────────────────────────────────────

/// Cubic ease-out: fast start, slow finish.
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Deterministic pseudo-random hash for dissolve/matrix effects.
///
/// Produces a stable value for each `(row, col, salt)` triple so the
/// animation is reproducible across frames at the same progress value.
fn pseudo_rand(row: u32, col: u32, salt: u32) -> u32 {
    let mut value = row
        .wrapping_mul(374_761_393)
        .wrapping_add(col.wrapping_mul(668_265_263))
        .wrapping_add(salt.wrapping_mul(2_147_483_647));
    value ^= value >> 13;
    value = value.wrapping_mul(1_274_126_177);
    value ^ (value >> 16)
}
