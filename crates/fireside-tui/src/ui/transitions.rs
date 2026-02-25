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
//! | `Fade`       | Dim source until progress = 0.5, then show dest     |
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
/// padded or clipped to `width` columns.
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
        let from_text = from_lines.get(row).map_or_else(String::new, line_to_text);
        let to_text = to_lines.get(row).map_or_else(String::new, line_to_text);
        let line = match kind {
            Transition::None => clip_pad(&to_text, width),
            Transition::Fade => {
                if progress < 0.5 {
                    clip_pad(&from_text, width)
                } else {
                    clip_pad(&to_text, width)
                }
            }
            Transition::SlideLeft => {
                let shift = ((1.0 - eased_progress) * width as f32).floor() as usize;
                clip_pad(&format!("{}{}", " ".repeat(shift), to_text), width)
            }
            Transition::SlideRight => {
                let shift = ((1.0 - eased_progress) * width as f32).floor() as usize;
                let padded = format!("{}{}", " ".repeat(width), to_text);
                let start = width.saturating_sub(shift).min(padded.chars().count());
                clip_pad(&padded.chars().skip(start).collect::<String>(), width)
            }
            Transition::Wipe => {
                let visible = take_chars(&to_text, reveal.min(width));
                clip_pad(&visible, width)
            }
            Transition::Dissolve => {
                let mut chars = Vec::with_capacity(width);
                let to_chars = to_text.chars().collect::<Vec<_>>();
                for col in 0..width {
                    let next = to_chars.get(col).copied().unwrap_or(' ');
                    let hash = pseudo_rand(row as u32, col as u32, 7) as f32 / u32::MAX as f32;
                    chars.push(if hash <= progress { next } else { ' ' });
                }
                chars.into_iter().collect::<String>()
            }
            Transition::Matrix => {
                let mut chars = Vec::with_capacity(width);
                let to_chars = to_text.chars().collect::<Vec<_>>();
                let matrix_chars = ['░', '▒', '▓'];
                for col in 0..width {
                    let next = to_chars.get(col).copied().unwrap_or(' ');
                    let hash = pseudo_rand(row as u32, col as u32, 31) as f32 / u32::MAX as f32;
                    if hash <= progress {
                        chars.push(next);
                    } else {
                        let idx = (pseudo_rand(row as u32, col as u32, 13) % 3) as usize;
                        chars.push(matrix_chars[idx]);
                    }
                }
                chars.into_iter().collect::<String>()
            }
            Transition::Typewriter => {
                let visible = take_chars(&to_text, reveal.min(width));
                clip_pad(&visible, width)
            }
        };

        let mut style = Style::default().fg(theme.foreground);
        if matches!(kind, Transition::Fade) && progress < 0.5 {
            style = style.add_modifier(Modifier::DIM);
        }
        if matches!(kind, Transition::Matrix) {
            style = Style::default().fg(theme.heading_h2);
        }

        output.push(Line::from(Span::styled(line, style)));
    }

    output
}

// ── Pure helpers ─────────────────────────────────────────────────────────────

/// Cubic ease-out: fast start, slow finish.
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Flatten a styled [`Line`] to a plain-text string.
fn line_to_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<Vec<_>>()
        .join("")
}

/// Clip `text` to `width` columns and right-pad with spaces to fill the remainder.
fn clip_pad(text: &str, width: usize) -> String {
    let clipped = take_chars(text, width);
    let pad = width.saturating_sub(clipped.chars().count());
    format!("{clipped}{}", " ".repeat(pad))
}

/// Take the first `max_chars` Unicode scalar values from `text`.
fn take_chars(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
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
