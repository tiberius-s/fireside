//! Graph validation — integrity checks beyond basic deserialization.
//!
//! Validates structural invariants such as dangling node references,
//! unreachable nodes, and branch point consistency.

use fireside_core::model::graph::Graph;
use fireside_core::model::{content::ContentBlock, node::Node};

use crate::error::EngineError;

/// Validation diagnostic with severity level.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity of the issue.
    pub severity: Severity,
    /// Human-readable description of the issue.
    pub message: String,
    /// Optional node ID where the issue was found.
    pub node_id: Option<String>,
}

/// Severity level for validation diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// A problem that prevents correct traversal.
    Error,
    /// A potential issue that may cause unexpected behavior.
    Warning,
}

/// Validate a graph for structural integrity.
///
/// Returns a list of diagnostics. An empty list means the graph is valid.
#[must_use]
pub fn validate_graph(graph: &Graph) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Check for empty graph
    if graph.is_empty() {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            message: "graph contains no nodes".into(),
            node_id: None,
        });
        return diagnostics;
    }

    // Check for dangling traversal references
    for node in &graph.nodes {
        let node_id_str = node.id.as_deref().unwrap_or("<anonymous>");

        if let Some(ref traversal) = node.traversal {
            // Check next reference
            if let Some(ref next_id) = traversal.next
                && graph.node_by_id(next_id).is_none()
            {
                diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    message: format!(
                        "node '{node_id_str}' traversal.next references unknown node '{next_id}'"
                    ),
                    node_id: node.id.clone(),
                });
            }

            // Check after reference
            if let Some(ref after_id) = traversal.after
                && graph.node_by_id(after_id).is_none()
            {
                diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    message: format!(
                        "node '{node_id_str}' traversal.after references unknown node '{after_id}'"
                    ),
                    node_id: node.id.clone(),
                });
            }

            // Check branch option targets
            if let Some(ref bp) = traversal.branch_point {
                for option in &bp.options {
                    if graph.node_by_id(&option.target).is_none() {
                        diagnostics.push(Diagnostic {
                            severity: Severity::Error,
                            message: format!(
                                "node '{node_id_str}' branch option '{}' targets unknown node '{}'",
                                option.label, option.target
                            ),
                            node_id: node.id.clone(),
                        });
                    }
                }
            }
        }

        diagnostics.extend(validate_node_content(node));
    }

    diagnostics
}

fn validate_node_content(node: &Node) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (index, block) in node.content.iter().enumerate() {
        for warning in validate_content_block(block) {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                message: format!("block #{}: {}", index + 1, warning.message),
                node_id: node.id.clone(),
            });
        }
    }

    diagnostics
}

/// Validate a content block for authoring quality warnings.
///
/// This function emits Warning-severity diagnostics only.
#[must_use]
pub fn validate_content_block(block: &ContentBlock) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    validate_content_block_with_path(block, "", &mut diagnostics);
    diagnostics
}

fn validate_content_block_with_path(
    block: &ContentBlock,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let prefix = if path.is_empty() {
        String::new()
    } else {
        format!("{path}: ")
    };

    match block {
        ContentBlock::Heading { text, .. } => {
            if text.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{prefix}heading text is empty"),
                    node_id: None,
                });
            }
        }
        ContentBlock::Text { body } => {
            if body.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{prefix}text body is empty"),
                    node_id: None,
                });
            }
        }
        ContentBlock::Code {
            language, source, ..
        } => {
            if source.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{prefix}code source is empty"),
                    node_id: None,
                });
            }

            if language
                .as_deref()
                .is_some_and(|lang| lang.trim().is_empty())
            {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{prefix}code language is blank"),
                    node_id: None,
                });
            }
        }
        ContentBlock::List { items, .. } => {
            if items.is_empty() {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{prefix}list has no items"),
                    node_id: None,
                });
            }

            for (item_index, item) in items.iter().enumerate() {
                if item.text.trim().is_empty() {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        message: format!("{prefix}list item #{} is empty", item_index + 1),
                        node_id: None,
                    });
                }
            }
        }
        ContentBlock::Image { src, alt, .. } => {
            if src.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{prefix}image src is empty"),
                    node_id: None,
                });
            }

            if alt.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{prefix}image alt text is empty"),
                    node_id: None,
                });
            }
        }
        ContentBlock::Divider => {}
        ContentBlock::Container { children, .. } => {
            if children.is_empty() {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{prefix}container has no children"),
                    node_id: None,
                });
            }

            for (child_index, child) in children.iter().enumerate() {
                let child_path = if path.is_empty() {
                    format!("container child #{}", child_index + 1)
                } else {
                    format!("{path} > child #{}", child_index + 1)
                };
                validate_content_block_with_path(child, &child_path, diagnostics);
            }
        }
        ContentBlock::Extension {
            extension_type,
            fallback,
            ..
        } => {
            if extension_type.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{prefix}extension type is empty"),
                    node_id: None,
                });
            }

            if fallback.is_none() {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    message: format!("{prefix}extension has no fallback block"),
                    node_id: None,
                });
            }
        }
    }
}

/// Validate a graph and return an error if any Error-severity diagnostics exist.
///
/// # Errors
///
/// Returns the first error-severity diagnostic as an `EngineError`.
pub fn validate_or_error(graph: &Graph) -> Result<(), EngineError> {
    let diagnostics = validate_graph(graph);
    for d in &diagnostics {
        if d.severity == Severity::Error {
            return Err(EngineError::DanglingReference(d.message.clone()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::load_graph_from_str;

    #[test]
    fn valid_graph_has_no_errors() {
        let json = r#"{
            "nodes": [
                { "id": "a", "content": [{ "kind": "text", "body": "A" }], "traversal": { "next": "b" } },
                { "id": "b", "content": [{ "kind": "text", "body": "B" }] }
            ]
        }"#;
        let graph = load_graph_from_str(json).unwrap();
        let diags = validate_graph(&graph);
        assert!(diags.is_empty(), "expected no diagnostics, got: {diags:?}");
    }

    #[test]
    fn dangling_next_reference_detected() {
        let json = r#"{
            "nodes": [
                { "id": "a", "content": [], "traversal": { "next": "missing" } }
            ]
        }"#;
        let graph = load_graph_from_str(json).unwrap();
        let diags = validate_graph(&graph);
        assert!(!diags.is_empty());
        assert_eq!(diags[0].severity, Severity::Error);
    }

    #[test]
    fn dangling_branch_target_detected() {
        let json = r#"{
            "nodes": [
                {
                    "id": "start",
                    "content": [],
                    "traversal": {
                        "branch-point": {
                            "options": [
                                { "label": "Go", "key": "a", "target": "nowhere" }
                            ]
                        }
                    }
                }
            ]
        }"#;
        let graph = load_graph_from_str(json).unwrap();
        let diags = validate_graph(&graph);
        assert!(!diags.is_empty());
    }

    #[test]
    fn empty_content_fields_emit_warnings() {
        let json = r#"{
            "nodes": [
                {
                    "id": "warn",
                    "content": [
                        { "kind": "heading", "level": 1, "text": "" },
                        { "kind": "image", "src": "", "alt": "" }
                    ]
                }
            ]
        }"#;

        let graph = load_graph_from_str(json).unwrap();
        let diags = validate_graph(&graph);

        assert!(diags.iter().any(|diag| {
            diag.severity == Severity::Warning && diag.message.contains("heading text is empty")
        }));
        assert!(diags.iter().any(|diag| {
            diag.severity == Severity::Warning && diag.message.contains("image src is empty")
        }));
    }
}
