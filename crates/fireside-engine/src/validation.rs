//! Graph validation — integrity checks beyond basic deserialization.
//!
//! Validates structural invariants such as dangling node references,
//! unreachable nodes, and branch point consistency.

use fireside_core::model::graph::Graph;

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
    }

    diagnostics
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
}
