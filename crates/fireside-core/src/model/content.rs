//! Content block types representing individual elements within a node.
//!
//! Each variant is discriminated by a `"kind"` tag in JSON (per the Fireside
//! protocol), and field names use kebab-case in the wire format.

use serde::{Deserialize, Serialize};

/// A single content element within a node.
///
/// Serialized with `#[serde(tag = "kind")]` so JSON looks like
/// `{ "kind": "heading", "level": 1, "text": "Hello" }`.
///
/// Includes the 7 core protocol block types plus `extension`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
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
        #[serde(
            default,
            skip_serializing_if = "Vec::is_empty",
            rename = "highlight-lines"
        )]
        highlight_lines: Vec<u32>,
        /// Whether to display line numbers.
        #[serde(default, rename = "show-line-numbers")]
        show_line_numbers: bool,
    },

    /// An ordered or unordered list.
    List {
        /// Whether the list is ordered (numbered). Defaults to false.
        #[serde(default)]
        ordered: bool,
        /// The list items (supports both string and object forms).
        items: Vec<ListItem>,
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

    /// A generic container with layout hint and nested content blocks.
    Container {
        /// Layout hint for how children are arranged.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        layout: Option<String>,
        /// Nested content blocks.
        #[serde(default)]
        children: Vec<ContentBlock>,
    },

    /// An extension block with a typed payload.
    ///
    /// Uses `kind: "extension"` with a required `type` identifier.
    Extension {
        /// The extension type identifier (e.g. `"acme.table"`).
        #[serde(rename = "type")]
        extension_type: String,
        /// Optional fallback content block for engines that don't support this extension.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fallback: Option<Box<ContentBlock>>,
        /// Extension-specific payload (arbitrary JSON).
        #[serde(flatten)]
        payload: serde_json::Value,
    },
}

/// A single item in a list.
///
/// Supports deserialization from both a bare JSON string and a full object
/// with `text` and optional `children` fields.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ListItem {
    /// The text content of this list item.
    pub text: String,
    /// Nested sub-items, if any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ListItem>,
}

impl<'de> serde::Deserialize<'de> for ListItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        struct ListItemVisitor;

        impl<'de> de::Visitor<'de> for ListItemVisitor {
            type Value = ListItem;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or a list item object with 'text' field")
            }

            fn visit_str<E>(self, v: &str) -> Result<ListItem, E>
            where
                E: de::Error,
            {
                Ok(ListItem {
                    text: v.to_owned(),
                    children: vec![],
                })
            }

            fn visit_map<A>(self, map: A) -> Result<ListItem, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                #[derive(Deserialize)]
                struct ListItemObj {
                    text: String,
                    #[serde(default)]
                    children: Vec<ListItem>,
                }
                let obj = ListItemObj::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(ListItem {
                    text: obj.text,
                    children: obj.children,
                })
            }
        }

        deserializer.deserialize_any(ListItemVisitor)
    }
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
        assert!(json.contains(r#""kind":"heading""#));
        let back: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(back, block);
    }

    #[test]
    fn roundtrip_text() {
        let block = ContentBlock::Text {
            body: "paragraph".into(),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains(r#""kind":"text""#));
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
        assert!(json.contains(r#""highlight-lines""#));
        assert!(json.contains(r#""show-line-numbers""#));
        let back: ContentBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(back, block);
    }

    #[test]
    fn deserialize_divider() {
        let json = r#"{"kind":"divider"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        assert_eq!(block, ContentBlock::Divider);
    }

    #[test]
    fn deserialize_list_with_string_items() {
        let json = r#"{"kind":"list","items":["First","Second","Third"]}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        if let ContentBlock::List { items, .. } = &block {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].text, "First");
        } else {
            panic!("expected List variant");
        }
    }

    #[test]
    fn deserialize_list_with_object_items() {
        let json = r#"{"kind":"list","items":[{"text":"First"},{"text":"Second","children":[{"text":"Nested"}]}]}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        if let ContentBlock::List { items, .. } = &block {
            assert_eq!(items.len(), 2);
            assert_eq!(items[1].children.len(), 1);
        } else {
            panic!("expected List variant");
        }
    }

    #[test]
    fn deserialize_container() {
        let json =
            r#"{"kind":"container","layout":"columns","children":[{"kind":"text","body":"A"}]}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        if let ContentBlock::Container { layout, children } = &block {
            assert_eq!(layout.as_deref(), Some("columns"));
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected Container variant");
        }
    }
}
