//! Starter-deck templates for `fireside new` — one node graph per shape,
//! each demonstrating a different traversal pattern. Parameterized by the
//! author's deck name, so these stay `serde_json::json!` builders rather
//! than `include_str!` assets: `json!` handles string escaping for the
//! interpolated name, an asset with placeholder substitution wouldn't.

/// A straight-through talk with no branching — the simplest possible deck,
/// for a presenter who just wants to get on stage.
pub(crate) fn linear_template(name: &str) -> serde_json::Value {
    serde_json::json!({
        "fireside-version": "0.1.0",
        "title": name,
        "nodes": [
            {
                "id": "welcome",
                "title": "Welcome",
                "traversal": "context",
                "speaker-notes": "This is your title slide. Edit the heading and subtitle below to fit your talk.",
                "content": [
                    { "kind": "container", "layout": "center", "children": [
                        { "kind": "heading", "level": 1, "text": name },
                        { "kind": "text", "body": "Press **Space** to move forward. Press **?** any time for help." }
                    ]}
                ]
            },
            {
                "id": "context",
                "title": "Context",
                "traversal": "example",
                "speaker-notes": "Replace this list with your own key points.",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Say something" },
                    { "kind": "text", "body": "Text blocks support **inline markdown**." },
                    { "kind": "list", "items": [
                        "One point per line",
                        "Keep it short — the audience is listening, not reading",
                        "Add as many nodes as your talk needs"
                    ]}
                ]
            },
            {
                "id": "example",
                "title": "Example",
                "traversal": "closing",
                "speaker-notes": "Swap this code sample for a snippet from your own project.",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Show something" },
                    { "kind": "divider" },
                    { "kind": "code", "language": "json", "source": "{ \"kind\": \"text\", \"body\": \"like this\" }" }
                ]
            },
            {
                "id": "closing",
                "title": "Closing",
                "content": [
                    { "kind": "container", "layout": "center", "children": [
                        { "kind": "heading", "level": 1, "text": "Thanks!" },
                        { "kind": "text", "body": "Edit the .fireside.json file to make it yours." }
                    ]}
                ]
            }
        ]
    })
}

/// A three-slide starter that demonstrates the one Fireside idea people
/// need: explicit edges, including a branch that rejoins. The default
/// template.
pub(crate) fn branching_template(name: &str) -> serde_json::Value {
    serde_json::json!({
        "fireside-version": "0.1.0",
        "title": name,
        "nodes": [
            {
                "id": "welcome",
                "title": "Welcome",
                "traversal": "pick-a-path",
                "speaker-notes": "This is your title slide. Edit the heading and subtitle below to fit your talk.",
                "content": [
                    { "kind": "container", "layout": "center", "children": [
                        { "kind": "heading", "level": 1, "text": name },
                        { "kind": "text", "body": "Press **Space** to move forward. Press **?** any time for help." }
                    ]}
                ]
            },
            {
                "id": "pick-a-path",
                "title": "Pick a path",
                "traversal": { "branch-point": {
                    "prompt": "Decks can branch. Where to?",
                    "options": [
                        { "label": "Show me content blocks", "key": "a", "target": "blocks" },
                        { "label": "Skip to the end", "key": "b", "target": "the-end" }
                    ]
                }},
                "speaker-notes": "This is a branch point — presenters see a menu here. Add or remove options in traversal.branch-point.options.",
                "content": [
                    { "kind": "heading", "level": 2, "text": "A choice" },
                    { "kind": "text", "body": "Use the arrow keys and press Enter." }
                ]
            },
            {
                "id": "blocks",
                "title": "Content blocks",
                "traversal": "the-end",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Blocks" },
                    { "kind": "list", "items": [
                        "Headings, text with **inline markdown**",
                        "Code with `highlight-lines`",
                        "Lists, images, dividers, containers"
                    ]},
                    { "kind": "divider" },
                    { "kind": "code", "language": "json", "source": "{ \"kind\": \"text\", \"body\": \"like this\" }" }
                ]
            },
            {
                "id": "the-end",
                "title": "The end",
                "content": [
                    { "kind": "container", "layout": "center", "children": [
                        { "kind": "heading", "level": 1, "text": "That's it" },
                        { "kind": "text", "body": "Edit the .fireside.json file to make it yours." }
                    ]}
                ]
            }
        ]
    })
}

/// An agenda that lets the presenter jump into any exercise, then flows
/// forward through the rest in order — the hub-and-spoke pattern a workshop
/// needs, without looping back through the menu.
pub(crate) fn workshop_template(name: &str) -> serde_json::Value {
    serde_json::json!({
        "fireside-version": "0.1.0",
        "title": name,
        "nodes": [
            {
                "id": "welcome",
                "title": "Welcome",
                "traversal": "agenda",
                "speaker-notes": "This is your title slide. Edit the heading and subtitle below to fit your workshop.",
                "content": [
                    { "kind": "container", "layout": "center", "children": [
                        { "kind": "heading", "level": 1, "text": name },
                        { "kind": "text", "body": "Press **Space** to begin. Press **?** any time for help." }
                    ]}
                ]
            },
            {
                "id": "agenda",
                "title": "Agenda",
                "traversal": { "branch-point": {
                    "prompt": "Where should we start?",
                    "options": [
                        { "label": "Setup", "key": "a", "target": "setup" },
                        { "label": "Exercise 1", "key": "b", "target": "exercise-1" },
                        { "label": "Exercise 2", "key": "c", "target": "exercise-2" }
                    ]
                }},
                "speaker-notes": "Presenters can jump to any section from here; each section still flows into the next when they press Space. Add sections by adding an option here and a node below.",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Agenda" },
                    { "kind": "list", "items": [
                        "Setup",
                        "Exercise 1",
                        "Exercise 2"
                    ]}
                ]
            },
            {
                "id": "setup",
                "title": "Setup",
                "traversal": "exercise-1",
                "speaker-notes": "Walk through environment or prerequisite steps here.",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Setup" },
                    { "kind": "list", "ordered": true, "items": [
                        "Clone the repository",
                        "Install dependencies",
                        "Confirm everyone is ready"
                    ]}
                ]
            },
            {
                "id": "exercise-1",
                "title": "Exercise 1",
                "traversal": "exercise-2",
                "speaker-notes": "Replace this code sample with the first exercise.",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Exercise 1" },
                    { "kind": "code", "language": "json", "source": "{ \"kind\": \"text\", \"body\": \"like this\" }" }
                ]
            },
            {
                "id": "exercise-2",
                "title": "Exercise 2",
                "traversal": "wrap-up",
                "speaker-notes": "Replace this list with the second exercise's steps.",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Exercise 2" },
                    { "kind": "list", "items": [
                        "Step one",
                        "Step two",
                        "Step three"
                    ]}
                ]
            },
            {
                "id": "wrap-up",
                "title": "Wrap-up",
                "content": [
                    { "kind": "container", "layout": "center", "children": [
                        { "kind": "heading", "level": 1, "text": "That's it" },
                        { "kind": "text", "body": "Edit the .fireside.json file to make it yours." }
                    ]}
                ]
            }
        ]
    })
}
