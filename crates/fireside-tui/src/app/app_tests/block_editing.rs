//! Block editing helper tests — `update_block_from_inline_text` and metadata.

use super::*;

#[test]
fn update_block_from_inline_text_keeps_heading_level() {
    let updated = update_block_from_inline_text(
        ContentBlock::Heading {
            level: 3,
            text: "old".to_string(),
        },
        "new".to_string(),
    );

    assert_eq!(
        updated,
        ContentBlock::Heading {
            level: 3,
            text: "new".to_string(),
        }
    );
}

#[test]
fn update_block_from_inline_text_list_inserts_first_item_when_empty() {
    let updated = update_block_from_inline_text(
        ContentBlock::List {
            ordered: false,
            items: vec![],
        },
        "first".to_string(),
    );

    let ContentBlock::List { items, .. } = updated else {
        panic!("expected list block");
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].text, "first");
}

#[test]
fn update_block_from_inline_text_container_blank_clears_layout() {
    let updated = update_block_from_inline_text(
        ContentBlock::Container {
            layout: Some("row".to_string()),
            children: vec![],
        },
        "   ".to_string(),
    );

    let ContentBlock::Container { layout, .. } = updated else {
        panic!("expected container block");
    };
    assert!(layout.is_none());
}

#[test]
fn update_block_metadata_from_inline_text_sets_code_language() {
    let original = ContentBlock::Code {
        language: Some("rust".to_string()),
        source: "fn main() {}".to_string(),
        highlight_lines: vec![],
        show_line_numbers: false,
    };

    let result = update_block_metadata_from_inline_text(original, "python".to_string());
    let Ok(updated) = result else {
        panic!("expected Ok result");
    };

    let ContentBlock::Code { language, .. } = updated else {
        panic!("expected code block");
    };
    assert_eq!(language.as_deref(), Some("python"));
}

#[test]
fn update_block_metadata_from_inline_text_rejects_invalid_heading_level() {
    let original = ContentBlock::Heading {
        level: 1,
        text: "Hello".to_string(),
    };

    let result = update_block_metadata_from_inline_text(original, "9".to_string());
    let Err(err) = result else {
        panic!("expected Err result");
    };

    assert_eq!(err, "Heading level must be between 1 and 6");
}
