//! Content block types that represent individual elements within a slide.
//!
//! Each variant is discriminated by a `"type"` tag in JSON, matching the
//! slide schema at `schemas/slide.schema.json`.

use serde::{Deserialize, Serialize};

/// A single content element within a slide.
///
/// Serialized with `#[serde(tag = "type")]` so JSON looks like
/// `{ "type": "heading", "level": 1, "text": "Hello" }`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// A heading with a level (1–6) and inline text.
    Heading {
        /// Heading level: 1 = H1, 2 = H2, etc.
        level: u8,
        /// The heading text content.
        text: String,
    },

    /// A paragraph of text with optional inline formatting.
    Text {
        /// The paragraph body text.
        body: String,
    },

    /// A code block with optional syntax highlighting.
    Code {
        /// The language identifier (e.g. `"rust"`, `"python"`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        language: Option<String>,
        /// The raw source code.
        source: String,
        /// 1-based line numbers to visually highlight.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        highlight_lines: Vec<u32>,
        /// Whether to display line numbers.
        #[serde(default)]
        show_line_numbers: bool,
    },

    /// An ordered or unordered list.
    List {
        /// Whether the list is ordered (numbered). Defaults to false.
        #[serde(default)]
        ordered: bool,
        /// The list items.
        items: Vec<ListItem>,
    },

    /// A table with headers and data rows.
    Table {
        /// Column header labels.
        headers: Vec<String>,
        /// Table body rows, each an array of cell values.
        rows: Vec<Vec<String>>,
        /// Column alignments. Defaults to left-aligned.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        alignments: Vec<ColumnAlignment>,
    },

    /// A block quote containing nested content blocks.
    Blockquote {
        /// Nested content blocks within the quote.
        content: Vec<ContentBlock>,
        /// Optional attribution.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attribution: Option<String>,
    },

    /// An image reference with alt text and optional caption.
    Image {
        /// Path or URL to the image source.
        src: String,
        /// Alt text for accessibility.
        #[serde(default)]
        alt: String,
        /// Optional caption displayed below the image.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
    },

    /// A thematic break / horizontal rule.
    Divider,

    /// Content blocks revealed progressively (one per key press).
    Fragment {
        /// The blocks to reveal in order.
        blocks: Vec<ContentBlock>,
    },

    /// Vertical whitespace between content blocks.
    Spacer {
        /// Number of blank lines. Defaults to 1.
        #[serde(default = "default_spacer_lines")]
        lines: u16,
    },

    /// Side-by-side content columns within a single slide.
    Columns {
        /// Each column is an array of content blocks.
        cols: Vec<Vec<ContentBlock>>,
        /// Relative column widths (percentages). Must sum to 100.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        widths: Vec<u8>,
    },
}

fn default_spacer_lines() -> u16 {
    1
}

/// A single item in a list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    /// The text content of this list item.
    pub text: String,
    /// Nested sub-items, if any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ListItem>,
}

/// Column alignment for table cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnAlignment {
    /// Left-aligned (default).
    #[default]
    Left,
    /// Center-aligned.
    Center,
    /// Right-aligned.
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_heading() {
        let block = ContentBlock::Heading {
            level: 1,
            text: "Hello".into(),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"heading""#));
        let back: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(back, block);
    }

    #[test]
    fn roundtrip_text() {
        let block = ContentBlock::Text {
            body: "paragraph".into(),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""type":"text""#));
        let back: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(back, block);
    }

    #[test]
    fn roundtrip_code_with_highlight() {
        let block = ContentBlock::Code {
            language: Some("rust".into()),
            source: "fn main() {}".into(),
            highlight_lines: vec![1],
            show_line_numbers: true,
        };
        let json = serde_json::to_string(&block).unwrap();
        let back: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(back, block);
    }

    #[test]
    fn deserialize_divider() {
        let json = r#"{"type":"divider"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        assert_eq!(block, ContentBlock::Divider);
    }

    #[test]
    fn deserialize_spacer_default_lines() {
        let json = r#"{"type":"spacer"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        assert_eq!(block, ContentBlock::Spacer { lines: 1 });
    }
}
