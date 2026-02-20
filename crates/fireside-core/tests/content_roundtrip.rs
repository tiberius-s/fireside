use fireside_core::model::content::{ContentBlock, ListItem};

fn roundtrip(block: &ContentBlock) {
    let json = serde_json::to_string(block).expect("content block should serialize");
    let decoded: ContentBlock =
        serde_json::from_str(&json).expect("content block should deserialize");
    assert_eq!(&decoded, block);
}

#[test]
fn heading_roundtrip() {
    let block = ContentBlock::Heading {
        level: 2,
        text: "Welcome".to_string(),
    };
    roundtrip(&block);
}

#[test]
fn text_roundtrip() {
    let block = ContentBlock::Text {
        body: "Body text".to_string(),
    };
    roundtrip(&block);
}

#[test]
fn code_roundtrip() {
    let block = ContentBlock::Code {
        language: Some("rust".to_string()),
        source: "fn main() {}".to_string(),
        highlight_lines: vec![1],
        show_line_numbers: true,
    };
    roundtrip(&block);
}

#[test]
fn list_roundtrip_with_bare_string_item() {
    let json =
        r#"{"kind":"list","ordered":false,"items":["First",{"text":"Second","children":[]}]}"#;
    let decoded: ContentBlock = serde_json::from_str(json).expect("list should deserialize");

    match decoded {
        ContentBlock::List { ordered, items } => {
            assert!(!ordered);
            assert_eq!(items[0].text, "First");
            assert!(items[0].children.is_empty());
            assert_eq!(items[1].text, "Second");
        }
        _ => panic!("expected list block"),
    }
}

#[test]
fn image_roundtrip() {
    let block = ContentBlock::Image {
        src: "assets/sample.ppm".to_string(),
        alt: "sample".to_string(),
        caption: Some("caption".to_string()),
    };
    roundtrip(&block);
}

#[test]
fn divider_roundtrip() {
    let block = ContentBlock::Divider;
    roundtrip(&block);
}

#[test]
fn container_roundtrip_with_children() {
    let block = ContentBlock::Container {
        layout: Some("split-horizontal".to_string()),
        children: vec![
            ContentBlock::Text {
                body: "left".to_string(),
            },
            ContentBlock::List {
                ordered: false,
                items: vec![ListItem {
                    text: "right".to_string(),
                    children: vec![],
                }],
            },
        ],
    };
    roundtrip(&block);
}

#[test]
fn extension_roundtrip_with_nested_fallback() {
    let block = ContentBlock::Extension {
        extension_type: "acme.widget".to_string(),
        fallback: Some(Box::new(ContentBlock::Container {
            layout: Some("stack".to_string()),
            children: vec![ContentBlock::Text {
                body: "fallback".to_string(),
            }],
        })),
        payload: serde_json::json!({
            "mode": "preview",
            "items": [1, 2, 3]
        }),
    };
    roundtrip(&block);
}
