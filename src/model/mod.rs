//! Core data model types for Slideways.
//!
//! This module defines the structural types that represent a slide deck:
//! slides, content blocks, layouts, themes, transitions, and branching.
//! All types are JSON-native with serde Serialize/Deserialize derives.

pub mod branch;
pub mod content;
pub mod deck;
pub mod layout;
pub mod slide;
pub mod theme;
pub mod transition;

pub use branch::{BranchOption, BranchPoint};
pub use content::ContentBlock;
pub use deck::{PresentationFile, PresentationMeta, SlideDeck};
pub use layout::Layout;
pub use slide::{Navigation, Slide, SlideId};
pub use theme::Theme;
pub use transition::Transition;
