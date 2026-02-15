//! Slide types — a single slide with layout, content, and navigation.

use serde::{Deserialize, Serialize};

use super::branch::BranchPoint;
use super::content::ContentBlock;
use super::layout::Layout;
use super::transition::Transition;

/// A unique identifier for a slide, used for branching navigation.
pub type SlideId = String;

/// A single slide within a deck.
///
/// Deserialized directly from the `slides` array in a presentation JSON file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slide {
    /// Optional unique identifier for this slide (used for branch targets).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<SlideId>,

    /// Layout variant for this slide.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,

    /// Transition effect when entering this slide.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition: Option<Transition>,

    /// Speaker notes (not rendered in the audience view).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker_notes: Option<String>,

    /// Navigation overrides and branching for this slide.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation: Option<Navigation>,

    /// The ordered list of content blocks that make up this slide.
    pub content: Vec<ContentBlock>,
}

/// Per-slide navigation control.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Navigation {
    /// Override the next slide by ID (instead of sequential advance).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<SlideId>,
    /// Slide ID to return to after completing a branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<SlideId>,
    /// Branch point definition, if this slide presents choices.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<BranchPoint>,
}

/// Legacy accessor for navigation fields — used by App to check overrides.
impl Slide {
    /// Get the next-slide override, if any.
    #[must_use]
    pub fn next_override(&self) -> Option<&str> {
        self.navigation.as_ref().and_then(|n| n.next.as_deref())
    }

    /// Get the branch point, if any.
    #[must_use]
    pub fn branch_point(&self) -> Option<&BranchPoint> {
        self.navigation.as_ref().and_then(|n| n.branch.as_ref())
    }

    /// Get the after (backtrack) target, if any.
    #[must_use]
    pub fn after_target(&self) -> Option<&str> {
        self.navigation.as_ref().and_then(|n| n.after.as_deref())
    }
}
