//! TUI application unit tests.
//!
//! Tests are split into focused sub-modules to keep each file
//! manageable:
//!
//! | Module               | Coverage                                      |
//! |----------------------|-----------------------------------------------|
//! | [`presenter`]        | Reload, hot-reload, branch overlay, graph overlay |
//! | [`block_editing`]    | `update_block_from_inline_text` and metadata helpers |
//! | [`overlays`]         | Help overlay and Goto mode                    |
//! | [`editor_interaction`] | Block ops, dirty-exit/quit, picker, search, notes, mouse, metadata edit |
//!
//! Shared fixture helpers (`graph_with_ids`, `branch_graph`, etc.) live in
//! this `mod.rs` and are accessible to all sub-modules via `use super::*;`.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use fireside_core::model::branch::{BranchOption, BranchPoint};
use fireside_core::model::content::ContentBlock;
use fireside_core::model::graph::{Graph, GraphFile};
use fireside_core::model::node::Node;
use fireside_core::model::traversal::Traversal;
use fireside_engine::PresentationSession;

use crate::app::app_helpers::{
    update_block_from_inline_text, update_block_metadata_from_inline_text,
};
use crate::app::{App, AppMode};
use crate::event::Action;
use crate::theme::Theme;

mod block_editing;
mod editor_interaction;
mod overlays;
mod presenter;

// ── Shared fixtures ──────────────────────────────────────────────────────────

pub(super) fn graph_with_ids(ids: &[&str]) -> Graph {
    let file = GraphFile {
        title: None,
        fireside_version: None,
        author: None,
        date: None,
        description: None,
        version: None,
        tags: Vec::new(),
        theme: None,
        font: None,
        defaults: None,
        extensions: Vec::new(),
        nodes: ids
            .iter()
            .map(|id| Node {
                id: Some((*id).to_string()),
                title: None,
                tags: Vec::new(),
                duration: None,
                layout: None,
                transition: None,
                speaker_notes: None,
                traversal: None,
                content: Vec::new(),
            })
            .collect(),
    };

    Graph::from_file(file).expect("graph should be valid")
}

pub(super) fn branch_graph() -> Graph {
    let mut start = Node {
        id: Some("start".to_string()),
        title: None,
        tags: Vec::new(),
        duration: None,
        layout: None,
        transition: None,
        speaker_notes: None,
        traversal: None,
        content: Vec::new(),
    };
    start.traversal = Some(Traversal {
        next: None,
        after: None,
        branch_point: Some(BranchPoint {
            id: Some("branch-0".to_string()),
            prompt: Some("Choose path".to_string()),
            options: vec![
                BranchOption {
                    label: "Path A".to_string(),
                    key: '1',
                    target: "path-a".to_string(),
                },
                BranchOption {
                    label: "Path B".to_string(),
                    key: '2',
                    target: "path-b".to_string(),
                },
            ],
        }),
    });

    let path_a = Node {
        id: Some("path-a".to_string()),
        title: None,
        tags: Vec::new(),
        duration: None,
        layout: None,
        transition: None,
        speaker_notes: None,
        traversal: None,
        content: Vec::new(),
    };
    let path_b = Node {
        id: Some("path-b".to_string()),
        title: None,
        tags: Vec::new(),
        duration: None,
        layout: None,
        transition: None,
        speaker_notes: None,
        traversal: None,
        content: Vec::new(),
    };

    Graph::from_file(GraphFile {
        title: None,
        fireside_version: None,
        author: None,
        date: None,
        description: None,
        version: None,
        tags: Vec::new(),
        theme: None,
        font: None,
        defaults: None,
        extensions: Vec::new(),
        nodes: vec![start, path_a, path_b],
    })
    .expect("branch graph should be valid")
}

pub(super) fn graph_with_content_blocks() -> Graph {
    let file = GraphFile {
        title: None,
        fireside_version: None,
        author: None,
        date: None,
        description: None,
        version: None,
        tags: Vec::new(),
        theme: None,
        font: None,
        defaults: None,
        extensions: Vec::new(),
        nodes: vec![Node {
            id: Some("node-1".to_string()),
            title: None,
            tags: Vec::new(),
            duration: None,
            layout: None,
            transition: None,
            speaker_notes: None,
            traversal: None,
            content: vec![
                ContentBlock::Heading {
                    level: 1,
                    text: "Title".to_string(),
                },
                ContentBlock::Text {
                    body: "Body paragraph".to_string(),
                },
                ContentBlock::Code {
                    language: Some("rust".to_string()),
                    source: "fn main() {}".to_string(),
                    highlight_lines: Vec::new(),
                    show_line_numbers: false,
                },
            ],
        }],
    };

    Graph::from_file(file).expect("graph with content blocks should be valid")
}
