//! Slide deck and presentation-level metadata.
//!
//! Deserialized directly from a `.json` presentation file
//! matching `schemas/presentation.schema.json`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::layout::Layout;
use super::slide::{Slide, SlideId};
use super::transition::Transition;

/// A complete slide deck loaded from a presentation JSON file.
#[derive(Debug, Clone)]
pub struct SlideDeck {
    /// Presentation-level metadata.
    pub metadata: PresentationMeta,
    /// The ordered list of slides in the deck.
    pub slides: Vec<Slide>,
    /// Index mapping slide IDs to their position in the `slides` vec.
    pub slide_index: HashMap<SlideId, usize>,
}

impl SlideDeck {
    /// Build a `SlideDeck` from a deserialized `PresentationFile`.
    ///
    /// Constructs the slide index and applies deck-level defaults to slides.
    ///
    /// # Errors
    ///
    /// Returns an error string if duplicate slide IDs are found.
    pub fn from_presentation(file: PresentationFile) -> Result<Self, String> {
        let mut slide_index = HashMap::new();

        for (i, slide) in file.slides.iter().enumerate() {
            if let Some(ref id) = slide.id {
                if slide_index.contains_key(id) {
                    return Err(format!("duplicate slide id: {id}"));
                }
                slide_index.insert(id.clone(), i);
            }
        }

        // Apply presentation defaults to slides that don't override them
        let default_layout = file.defaults.as_ref().and_then(|d| d.layout);
        let default_transition = file.defaults.as_ref().and_then(|d| d.transition);

        let slides: Vec<Slide> = file
            .slides
            .into_iter()
            .map(|mut s| {
                if s.layout.is_none() {
                    s.layout = default_layout;
                }
                if s.transition.is_none() {
                    s.transition = default_transition;
                }
                s
            })
            .collect();

        Ok(Self {
            metadata: PresentationMeta {
                title: file.title,
                author: file.author,
                date: file.date,
                description: file.description,
                version: file.version,
                tags: file.tags,
                theme: file.theme,
                font: file.font,
            },
            slides,
            slide_index,
        })
    }

    /// Returns the total number of slides in the deck.
    #[must_use]
    pub fn len(&self) -> usize {
        self.slides.len()
    }

    /// Returns `true` if the deck contains no slides.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.slides.is_empty()
    }

    /// Look up a slide by its ID, returning a reference if found.
    #[must_use]
    pub fn slide_by_id(&self, id: &str) -> Option<&Slide> {
        self.slide_index.get(id).map(|&idx| &self.slides[idx])
    }

    /// Look up a slide index by its ID.
    #[must_use]
    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.slide_index.get(id).copied()
    }
}

/// The raw presentation JSON file structure.
///
/// This is the direct deserialization target matching
/// `schemas/presentation.schema.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationFile {
    /// Presentation title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Author name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Presentation date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Short description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Version string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Searchable tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Theme name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// Monospace font family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    /// Default slide settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defaults: Option<SlideDefaults>,
    /// Ordered slides.
    pub slides: Vec<Slide>,
}

/// Default values applied to all slides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SlideDefaults {
    /// Default layout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
    /// Default transition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition: Option<Transition>,
}

/// Presentation-level metadata (extracted from the file for runtime use).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PresentationMeta {
    /// The title of the presentation.
    pub title: Option<String>,
    /// The author.
    pub author: Option<String>,
    /// The date.
    pub date: Option<String>,
    /// Short description.
    pub description: Option<String>,
    /// Version string.
    pub version: Option<String>,
    /// Tags.
    pub tags: Vec<String>,
    /// Theme name.
    pub theme: Option<String>,
    /// Monospace font family.
    pub font: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_minimal_presentation() {
        let json = r#"{
            "slides": [
                {
                    "content": [
                        { "type": "heading", "level": 1, "text": "Hello" }
                    ]
                }
            ]
        }"#;
        let file: PresentationFile = serde_json::from_str(json).unwrap();
        let deck = SlideDeck::from_presentation(file).unwrap();
        assert_eq!(deck.len(), 1);
    }

    #[test]
    fn defaults_applied_to_slides() {
        let json = r#"{
            "defaults": { "layout": "center", "transition": "fade" },
            "slides": [
                { "content": [] },
                { "layout": "title", "content": [] }
            ]
        }"#;
        let file: PresentationFile = serde_json::from_str(json).unwrap();
        let deck = SlideDeck::from_presentation(file).unwrap();
        assert_eq!(deck.slides[0].layout, Some(Layout::Center));
        assert_eq!(deck.slides[1].layout, Some(Layout::Title));
    }

    #[test]
    fn duplicate_slide_id_rejected() {
        let json = r#"{
            "slides": [
                { "id": "dupe", "content": [] },
                { "id": "dupe", "content": [] }
            ]
        }"#;
        let file: PresentationFile = serde_json::from_str(json).unwrap();
        let result = SlideDeck::from_presentation(file);
        assert!(result.is_err());
    }
}
