//! Graph and document-level types.
//!
//! A `Graph` is the top-level Fireside document, containing an ordered
//! collection of nodes with optional metadata and defaults.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::layout::Layout;
use super::node::{Node, NodeId};
use super::transition::Transition;

/// A complete Fireside graph loaded from a JSON document.
///
/// This is the runtime model used by the engine. It contains the
/// validated, indexed representation of the document.
#[derive(Debug, Clone)]
pub struct Graph {
    /// Document-level metadata.
    pub metadata: GraphMeta,
    /// The ordered list of nodes in the graph.
    pub nodes: Vec<Node>,
    /// Index mapping node IDs to their position in the `nodes` vec.
    pub node_index: HashMap<NodeId, usize>,
}

impl Graph {
    /// Build a `Graph` from a deserialized `GraphFile`.
    ///
    /// Constructs the node index and applies document-level defaults.
    ///
    /// # Errors
    ///
    /// Returns an error string if duplicate node IDs are found.
    pub fn from_file(file: GraphFile) -> Result<Self, String> {
        let mut node_index = HashMap::new();

        for (i, node) in file.nodes.iter().enumerate() {
            if let Some(ref id) = node.id {
                if node_index.contains_key(id) {
                    return Err(format!("duplicate node id: {id}"));
                }
                node_index.insert(id.clone(), i);
            }
        }

        // Apply document defaults to nodes that don't override them
        let default_layout = file.defaults.as_ref().and_then(|d| d.layout);
        let default_transition = file.defaults.as_ref().and_then(|d| d.transition);

        let nodes: Vec<Node> = file
            .nodes
            .into_iter()
            .map(|mut n| {
                if n.layout.is_none() {
                    n.layout = default_layout;
                }
                if n.transition.is_none() {
                    n.transition = default_transition;
                }
                n
            })
            .collect();

        Ok(Self {
            metadata: GraphMeta {
                title: file.title,
                fireside_version: file.fireside_version,
                author: file.author,
                date: file.date,
                description: file.description,
                version: file.version,
                tags: file.tags,
                theme: file.theme,
                font: file.font,
                extensions: file.extensions,
            },
            nodes,
            node_index,
        })
    }

    /// Returns the total number of nodes in the graph.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the graph contains no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Look up a node by its ID, returning a reference if found.
    #[must_use]
    pub fn node_by_id(&self, id: &str) -> Option<&Node> {
        self.node_index.get(id).map(|&idx| &self.nodes[idx])
    }

    /// Look up a node index by its ID.
    #[must_use]
    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.node_index.get(id).copied()
    }

    /// Rebuild the node index from the current node list.
    ///
    /// # Errors
    ///
    /// Returns an error string if duplicate node IDs are found.
    pub fn rebuild_index(&mut self) -> Result<(), String> {
        let mut node_index = HashMap::new();

        for (i, node) in self.nodes.iter().enumerate() {
            if let Some(ref id) = node.id {
                if node_index.contains_key(id) {
                    return Err(format!("duplicate node id: {id}"));
                }
                node_index.insert(id.clone(), i);
            }
        }

        self.node_index = node_index;
        Ok(())
    }
}

/// The raw Fireside JSON file structure.
///
/// This is the direct deserialization target matching the wire format.
/// Uses `"nodes"` (not `"slides"`) per the Fireside 0.1.0 protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphFile {
    /// Document title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Protocol version for this document.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "fireside-version"
    )]
    pub fireside_version: Option<String>,
    /// Author name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Document date.
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
    /// Default node settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defaults: Option<NodeDefaults>,
    /// Declared extension capabilities used by this graph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<ExtensionDeclaration>,
    /// Ordered nodes.
    pub nodes: Vec<Node>,
}

/// A declared extension capability used by a graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionDeclaration {
    /// Extension type identifier (e.g., "acme.table").
    pub r#type: String,
    /// Whether support for this extension is required for correct rendering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Default values applied to all nodes in the graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NodeDefaults {
    /// Default layout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
    /// Default transition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition: Option<Transition>,
}

/// Document-level metadata extracted for runtime use.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GraphMeta {
    /// The title of the document.
    pub title: Option<String>,
    /// Protocol version for this document.
    pub fireside_version: Option<String>,
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
    /// Declared extension capabilities used by this graph.
    pub extensions: Vec<ExtensionDeclaration>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_minimal_graph() {
        let json = r#"{
            "nodes": [
                {
                    "content": [
                        { "kind": "heading", "level": 1, "text": "Hello" }
                    ]
                }
            ]
        }"#;
        let file: GraphFile = serde_json::from_str(json).unwrap();
        let graph = Graph::from_file(file).unwrap();
        assert_eq!(graph.len(), 1);
    }

    #[test]
    fn defaults_applied_to_nodes() {
        let json = r#"{
            "defaults": { "layout": "center", "transition": "fade" },
            "nodes": [
                { "content": [] },
                { "layout": "fullscreen", "content": [] }
            ]
        }"#;
        let file: GraphFile = serde_json::from_str(json).unwrap();
        let graph = Graph::from_file(file).unwrap();
        assert_eq!(graph.nodes[0].layout, Some(Layout::Center));
        assert_eq!(graph.nodes[1].layout, Some(Layout::Fullscreen));
    }

    #[test]
    fn duplicate_node_ids_rejected() {
        let json = r#"{
            "nodes": [
                { "id": "dup", "content": [] },
                { "id": "dup", "content": [] }
            ]
        }"#;
        let file: GraphFile = serde_json::from_str(json).unwrap();
        let result = Graph::from_file(file);
        assert!(result.is_err());
    }
}
