//! Transition types for animating between nodes.
//!
//! Uses kebab-case serialization per the Fireside protocol.

use serde::{Deserialize, Serialize};

/// A transition effect applied when moving between nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Transition {
    /// No transition; instant switch.
    #[default]
    None,
    /// Fade between nodes using character density interpolation.
    Fade,
    /// Content moves left to reveal the next node.
    #[serde(rename = "slide-left")]
    SlideLeft,
    /// Content moves right to reveal the next node.
    #[serde(rename = "slide-right")]
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
