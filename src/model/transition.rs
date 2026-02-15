//! Transition types for animating between slides.

use serde::{Deserialize, Serialize};

/// A transition effect applied when moving between slides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transition {
    /// No transition; instant switch.
    #[default]
    None,
    /// Fade between slides using character density interpolation.
    Fade,
    /// Slide content moves left to reveal the next slide.
    SlideLeft,
    /// Slide content moves right to reveal the next slide.
    SlideRight,
    /// Wipe transition from one edge.
    Wipe,
    /// Random cell reveal to the destination content.
    Dissolve,
    /// Matrix-style cascading characters resolving to content.
    Matrix,
    /// Sequential character-by-character reveal.
    Typewriter,
}
