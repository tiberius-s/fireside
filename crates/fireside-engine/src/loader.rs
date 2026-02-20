//! Graph loader — reads JSON Fireside documents into a [`Graph`].
//!
//! Handles file I/O and deserialization, delegating to core types.

use std::path::Path;

use anyhow::{Context, Result};
use fireside_core::error::CoreError;
use fireside_core::model::graph::{Graph, GraphFile, NodeDefaults};

/// Load a graph from a JSON file on disk.
///
/// # Errors
///
/// Returns an error if the file cannot be read, the JSON is invalid,
/// or the graph contains no nodes.
pub fn load_graph(path: &Path) -> Result<Graph> {
    let source = std::fs::read_to_string(path).map_err(|e| CoreError::FileRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    load_graph_from_str(&source).with_context(|| format!("parsing {}", path.display()))
}

/// Save a graph to a JSON file on disk.
///
/// # Errors
///
/// Returns an error if serialization or file writing fails.
pub fn save_graph(path: &Path, graph: &Graph) -> Result<()> {
    let file = graph_to_file(graph);
    let json =
        serde_json::to_string_pretty(&file).map_err(|e| CoreError::InvalidJson(e.to_string()))?;
    std::fs::write(path, format!("{json}\n")).map_err(|e| CoreError::FileRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

fn graph_to_file(graph: &Graph) -> GraphFile {
    GraphFile {
        title: graph.metadata.title.clone(),
        fireside_version: graph.metadata.fireside_version.clone(),
        author: graph.metadata.author.clone(),
        date: graph.metadata.date.clone(),
        description: graph.metadata.description.clone(),
        version: graph.metadata.version.clone(),
        tags: graph.metadata.tags.clone(),
        theme: graph.metadata.theme.clone(),
        font: graph.metadata.font.clone(),
        defaults: Some(NodeDefaults::default()),
        extensions: graph.metadata.extensions.clone(),
        nodes: graph.nodes.clone(),
    }
}

/// Load a graph from a JSON string.
///
/// # Errors
///
/// Returns an error if the JSON is malformed, doesn't match the schema,
/// or the graph contains no nodes.
pub fn load_graph_from_str(source: &str) -> Result<Graph> {
    let file: GraphFile =
        serde_json::from_str(source).map_err(|e| CoreError::InvalidJson(e.to_string()))?;

    if file.nodes.is_empty() {
        return Err(CoreError::EmptyGraph.into());
    }

    Graph::from_file(file)
        .map_err(|e| {
            let node_id = e
                .strip_prefix("duplicate node id: ")
                .map_or_else(|| e.clone(), ToOwned::to_owned);
            CoreError::DuplicateNodeId(node_id)
        })
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_simple_graph() {
        let json = r#"{
            "title": "Test Graph",
            "author": "Tester",
            "nodes": [
                {
                    "content": [
                        { "kind": "heading", "level": 1, "text": "Node One" },
                        { "kind": "text", "body": "Hello world" }
                    ]
                },
                {
                    "content": [
                        { "kind": "heading", "level": 1, "text": "Node Two" },
                        { "kind": "text", "body": "Goodbye world" }
                    ]
                }
            ]
        }"#;
        let graph = load_graph_from_str(json).expect("should parse");
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.metadata.title.as_deref(), Some("Test Graph"));
    }

    #[test]
    fn empty_graph_returns_error() {
        let json = r#"{ "nodes": [] }"#;
        let result = load_graph_from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn graph_with_branching() {
        let json = r#"{
            "nodes": [
                {
                    "id": "start",
                    "content": [{ "kind": "heading", "level": 1, "text": "Choose" }],
                    "traversal": {
                        "branch-point": {
                            "options": [
                                { "label": "Path A", "key": "a", "target": "path-a" },
                                { "label": "Path B", "key": "b", "target": "path-b" }
                            ]
                        }
                    }
                },
                {
                    "id": "path-a",
                    "content": [{ "kind": "text", "body": "You chose A" }],
                    "traversal": { "next": "end" }
                },
                {
                    "id": "path-b",
                    "content": [{ "kind": "text", "body": "You chose B" }],
                    "traversal": { "next": "end" }
                },
                {
                    "id": "end",
                    "content": [{ "kind": "text", "body": "The end" }]
                }
            ]
        }"#;
        let graph = load_graph_from_str(json).expect("should parse branching graph");
        assert_eq!(graph.nodes.len(), 4);
        assert!(graph.node_by_id("start").is_some());
        assert!(graph.node_by_id("path-a").is_some());
    }

    #[test]
    fn save_and_reload_roundtrip() {
        let json = r#"{
            "title": "Roundtrip",
            "nodes": [
                { "id": "n1", "content": [{ "kind": "text", "body": "Hello" }] }
            ]
        }"#;

        let graph = load_graph_from_str(json).expect("graph should parse");
        let temp_path = std::env::temp_dir().join("fireside-save-roundtrip.json");

        save_graph(&temp_path, &graph).expect("save should succeed");
        let reloaded = load_graph(&temp_path).expect("reload should succeed");
        let _ = std::fs::remove_file(&temp_path);

        assert_eq!(reloaded.nodes.len(), 1);
        assert_eq!(reloaded.metadata.title.as_deref(), Some("Roundtrip"));
    }
}
