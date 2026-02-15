//! Presentation loader — reads JSON presentation files into a [`SlideDeck`].
//!
//! Replaces the former markdown parser pipeline. Presentations are now
//! first-class JSON documents matching `schemas/presentation.schema.json`.

use std::path::Path;

use anyhow::{Context, Result};

use crate::error::ParseError;
use crate::model::deck::{PresentationFile, SlideDeck};

/// Load a slide deck from a JSON presentation file on disk.
///
/// # Errors
///
/// Returns an error if the file cannot be read, the JSON is invalid,
/// or the deck contains no slides.
pub fn load_deck(path: &Path) -> Result<SlideDeck> {
    let source = std::fs::read_to_string(path).map_err(|e| ParseError::FileRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    load_deck_from_str(&source).with_context(|| format!("parsing {}", path.display()))
}

/// Load a slide deck from a JSON string.
///
/// # Errors
///
/// Returns an error if the JSON is malformed, doesn't match the schema,
/// or the deck contains no slides.
pub fn load_deck_from_str(source: &str) -> Result<SlideDeck> {
    let file: PresentationFile =
        serde_json::from_str(source).map_err(|e| ParseError::InvalidJson(e.to_string()))?;

    if file.slides.is_empty() {
        return Err(ParseError::EmptyDeck.into());
    }

    SlideDeck::from_presentation(file).map_err(|e| ParseError::DuplicateSlideId(e).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_simple_deck() {
        let json = r#"{
            "title": "Test Deck",
            "author": "Tester",
            "slides": [
                {
                    "content": [
                        { "type": "heading", "level": 1, "text": "Slide One" },
                        { "type": "text", "body": "Hello world" }
                    ]
                },
                {
                    "content": [
                        { "type": "heading", "level": 1, "text": "Slide Two" },
                        { "type": "text", "body": "Goodbye world" }
                    ]
                }
            ]
        }"#;
        let deck = load_deck_from_str(json).expect("should parse");
        assert_eq!(deck.slides.len(), 2);
        assert_eq!(deck.metadata.title.as_deref(), Some("Test Deck"));
    }

    #[test]
    fn empty_deck_returns_error() {
        let json = r#"{ "slides": [] }"#;
        let result = load_deck_from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn deck_with_branching() {
        let json = r#"{
            "slides": [
                {
                    "id": "start",
                    "content": [{ "type": "heading", "level": 1, "text": "Choose" }],
                    "navigation": {
                        "branch": {
                            "options": [
                                { "label": "Path A", "key": "a", "target": "path-a" },
                                { "label": "Path B", "key": "b", "target": "path-b" }
                            ]
                        }
                    }
                },
                {
                    "id": "path-a",
                    "content": [{ "type": "text", "body": "You chose A" }],
                    "navigation": { "next": "end" }
                },
                {
                    "id": "path-b",
                    "content": [{ "type": "text", "body": "You chose B" }],
                    "navigation": { "next": "end" }
                },
                {
                    "id": "end",
                    "content": [{ "type": "heading", "level": 1, "text": "The End" }]
                }
            ]
        }"#;
        let deck = load_deck_from_str(json).expect("should parse");
        assert_eq!(deck.slides.len(), 4);
        assert!(deck.slides[0].branch_point().is_some());
        assert_eq!(deck.index_of("path-a"), Some(1));
        assert_eq!(deck.index_of("path-b"), Some(2));
    }

    #[test]
    fn deck_with_all_content_types() {
        let json = r#"{
            "slides": [{
                "content": [
                    { "type": "heading", "level": 2, "text": "Title" },
                    { "type": "text", "body": "Paragraph" },
                    { "type": "code", "language": "rust", "source": "fn main() {}" },
                    { "type": "list", "items": [{ "text": "item 1" }, { "text": "item 2" }] },
                    { "type": "table", "headers": ["A", "B"], "rows": [["1", "2"]] },
                    { "type": "blockquote", "content": [{ "type": "text", "body": "wise words" }], "attribution": "— Someone" },
                    { "type": "image", "src": "./img.png", "alt": "A picture" },
                    { "type": "divider" },
                    { "type": "spacer", "lines": 2 },
                    { "type": "columns", "cols": [[{ "type": "text", "body": "Left" }], [{ "type": "text", "body": "Right" }]] }
                ]
            }]
        }"#;
        let deck = load_deck_from_str(json).expect("should parse all content types");
        assert_eq!(deck.slides[0].content.len(), 10);
    }
}
