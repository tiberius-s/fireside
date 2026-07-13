//! Layer-2 semantic validation (spec §4).
//!
//! Checks graph integrity beyond what the JSON schema can express, with the
//! same rules and rule names as `protocol/validate.mjs` so the Rust and Node
//! validators stay in lockstep. Diagnostics are written for presenters, not
//! parser authors: each one names the node and says what to do about it.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use fireside_core::{Graph, Node, TraversalSpec};

/// How serious a diagnostic is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Fine to know about; nothing needs fixing.
    Info,
    /// Probably a mistake, but the document is presentable.
    Warning,
    /// The document must not be presented until this is fixed.
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => f.write_str("info"),
            Self::Warning => f.write_str("warning"),
            Self::Error => f.write_str("error"),
        }
    }
}

/// A single validation finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// How serious this finding is.
    pub severity: Severity,
    /// Stable rule identifier (matches `protocol/validate.mjs`).
    pub rule: &'static str,
    /// Human-readable, presenter-friendly message.
    pub message: String,
    /// The node this finding is about, when there is one.
    pub node: Option<String>,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}] {}", self.severity, self.rule, self.message)
    }
}

impl Diagnostic {
    fn new(severity: Severity, rule: &'static str, message: String, node: Option<&str>) -> Self {
        Self {
            severity,
            rule,
            message,
            node: node.map(str::to_owned),
        }
    }
}

/// An outgoing edge collected from a node, for target checks.
struct Edge<'a> {
    target: &'a str,
    label: Option<&'a str>,
}

fn edges(node: &Node) -> Vec<Edge<'_>> {
    let mut out = Vec::new();
    if let Some(target) = node.next_target() {
        out.push(Edge {
            target,
            label: None,
        });
    }
    if let Some(bp) = node.branch_point() {
        for opt in &bp.options {
            out.push(Edge {
                target: &opt.target,
                label: Some(&opt.label),
            });
        }
    }
    out
}

/// Run every Layer-2 check and return all findings, errors first.
#[must_use]
pub fn validate(graph: &Graph) -> Vec<Diagnostic> {
    let ids: HashSet<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();

    let mut diags = Vec::new();
    check_unique_node_ids(graph, &mut diags);
    check_valid_targets(graph, &ids, &mut diags);
    check_next_branch_point_conflict(graph, &mut diags);
    check_branch_options(graph, &mut diags);
    check_empty_traversal(graph, &mut diags);
    check_reachability(graph, &ids, &mut diags);
    check_self_loops(graph, &mut diags);
    check_trivial_cycles(graph, &mut diags);
    check_dead_end_branches(graph, &mut diags);

    diags.sort_by_key(|d| std::cmp::Reverse(d.severity));
    diags
}

/// Whether any finding blocks presentation.
#[must_use]
pub fn has_errors(diags: &[Diagnostic]) -> bool {
    diags.iter().any(|d| d.severity == Severity::Error)
}

/// ERROR: node IDs must be unique (required check 1).
fn check_unique_node_ids(graph: &Graph, diags: &mut Vec<Diagnostic>) {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for (i, node) in graph.nodes.iter().enumerate() {
        if let Some(&first) = seen.get(node.id.as_str()) {
            diags.push(Diagnostic::new(
                Severity::Error,
                "unique-node-ids",
                format!(
                    "two nodes share the id \"{}\" (positions {} and {}) — rename one so every link knows where to go",
                    node.id,
                    first + 1,
                    i + 1
                ),
                Some(&node.id),
            ));
        } else {
            seen.insert(node.id.as_str(), i);
        }
    }
}

/// ERROR: every next/branch target must exist (required checks 2 and 3).
fn check_valid_targets(graph: &Graph, ids: &HashSet<&str>, diags: &mut Vec<Diagnostic>) {
    for node in &graph.nodes {
        for edge in edges(node) {
            if !ids.contains(edge.target) {
                let via = edge
                    .label
                    .map(|l| format!(" (via choice \"{l}\")"))
                    .unwrap_or_default();
                diags.push(Diagnostic::new(
                    Severity::Error,
                    "valid-traversal-target",
                    format!(
                        "\"{}\" points to \"{}\"{via}, but no node has that id",
                        node.id, edge.target
                    ),
                    Some(&node.id),
                ));
            }
        }
    }
}

/// ERROR: `next` and `branch-point` are mutually exclusive (required check 5).
fn check_next_branch_point_conflict(graph: &Graph, diags: &mut Vec<Diagnostic>) {
    for node in &graph.nodes {
        if node.next_target().is_some() && node.branch_point().is_some() {
            diags.push(Diagnostic::new(
                Severity::Error,
                "next-branch-point-conflict",
                format!(
                    "\"{}\" has both \"next\" and \"branch-point\" — keep the branch point and wire each option's return instead",
                    node.id
                ),
                Some(&node.id),
            ));
        }
    }
}

/// ERROR: branch points need at least one option, and option keys must be
/// unique within a branch point (required check 4).
fn check_branch_options(graph: &Graph, diags: &mut Vec<Diagnostic>) {
    for node in &graph.nodes {
        let Some(bp) = node.branch_point() else {
            continue;
        };
        if bp.options.is_empty() {
            diags.push(Diagnostic::new(
                Severity::Error,
                "empty-branch-options",
                format!(
                    "\"{}\" has a branch point with no options — the presenter would be stuck",
                    node.id
                ),
                Some(&node.id),
            ));
        }
        let mut seen: HashMap<&str, &str> = HashMap::new();
        for opt in &bp.options {
            let Some(key) = opt.key.as_deref() else {
                continue;
            };
            if let Some(other) = seen.get(key) {
                diags.push(Diagnostic::new(
                    Severity::Error,
                    "unique-branch-keys",
                    format!(
                        "\"{}\" assigns key \"{key}\" to both \"{other}\" and \"{}\" — keys must be unique within a branch point",
                        node.id, opt.label
                    ),
                    Some(&node.id),
                ));
            } else {
                seen.insert(key, &opt.label);
            }
        }
    }
}

/// WARNING: a present-but-vacuous `Traversal` object (`{}`) behaves like an
/// absent field — terminal — but is more likely an authoring mistake than
/// a deliberately omitted field.
fn check_empty_traversal(graph: &Graph, diags: &mut Vec<Diagnostic>) {
    for node in &graph.nodes {
        let Some(TraversalSpec::Rules(t)) = node.traversal.as_ref() else {
            continue;
        };
        if t.next.is_none() && t.branch_point.is_none() {
            diags.push(Diagnostic::new(
                Severity::Warning,
                "empty-traversal",
                format!(
                    "\"{}\" has an empty traversal object — it behaves like a terminal node (only back() can exit), same as leaving \"traversal\" out entirely. If that's what you want, remove the empty object; otherwise give it a \"next\" or a \"branch-point\"",
                    node.id
                ),
                Some(&node.id),
            ));
        }
    }
}

/// WARNING: nodes should be reachable from the entry point (recommended 1).
fn check_reachability(graph: &Graph, ids: &HashSet<&str>, diags: &mut Vec<Diagnostic>) {
    let Some(entry) = graph.entry() else {
        return;
    };
    let by_id: HashMap<&str, &Node> = graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    let mut reachable: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<&str> = VecDeque::from([entry.id.as_str()]);

    while let Some(id) = queue.pop_front() {
        if !reachable.insert(id) {
            continue;
        }
        let Some(node) = by_id.get(id) else { continue };
        for edge in edges(node) {
            if ids.contains(edge.target) && !reachable.contains(edge.target) {
                queue.push_back(edge.target);
            }
        }
    }

    for node in &graph.nodes {
        if !reachable.contains(node.id.as_str()) {
            diags.push(Diagnostic::new(
                Severity::Warning,
                "unreachable-node",
                format!(
                    "\"{}\" can never be reached from the start (\"{}\") — link to it or remove it",
                    node.id, entry.id
                ),
                Some(&node.id),
            ));
        }
    }
}

/// WARNING: a node pointing at itself is usually an accident (recommended 2).
fn check_self_loops(graph: &Graph, diags: &mut Vec<Diagnostic>) {
    for node in &graph.nodes {
        for edge in edges(node) {
            if edge.target == node.id {
                diags.push(Diagnostic::new(
                    Severity::Warning,
                    "self-loop",
                    format!("\"{}\" points to itself", node.id),
                    Some(&node.id),
                ));
            }
        }
    }
}

/// WARNING: two-node cycles (A → B → A) are likely accidental (recommended 4).
fn check_trivial_cycles(graph: &Graph, diags: &mut Vec<Diagnostic>) {
    let by_id: HashMap<&str, &Node> = graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    let mut reported: HashSet<(String, String)> = HashSet::new();

    for node in &graph.nodes {
        for edge in edges(node) {
            if edge.target == node.id {
                continue; // self-loops have their own rule
            }
            let Some(target) = by_id.get(edge.target) else {
                continue;
            };
            if edges(target).iter().any(|back| back.target == node.id) {
                let mut pair = [node.id.as_str(), edge.target];
                pair.sort_unstable();
                if reported.insert((pair[0].to_owned(), pair[1].to_owned())) {
                    diags.push(Diagnostic::new(
                        Severity::Warning,
                        "trivial-cycle",
                        format!(
                            "\"{}\" and \"{}\" point at each other — presenters can loop forever between them",
                            pair[0], pair[1]
                        ),
                        Some(&node.id),
                    ));
                }
            }
        }
    }
}

/// INFO: branch options leading to terminal nodes. Terminal nodes are a
/// legitimate ending pattern (recommended 5).
fn check_dead_end_branches(graph: &Graph, diags: &mut Vec<Diagnostic>) {
    let by_id: HashMap<&str, &Node> = graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    for node in &graph.nodes {
        let Some(bp) = node.branch_point() else {
            continue;
        };
        for opt in &bp.options {
            let Some(target) = by_id.get(opt.target.as_str()) else {
                continue;
            };
            if target.is_terminal() {
                diags.push(Diagnostic::new(
                    Severity::Info,
                    "dead-end-branch",
                    format!(
                        "choice \"{}\" at \"{}\" leads to \"{}\", which ends the path (going back is the only exit) — fine if that's the ending, otherwise give it a \"next\"",
                        opt.label, node.id, target.id
                    ),
                    Some(&node.id),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fireside_core::Graph;

    const HELLO: &str = include_str!("../../../docs/examples/hello.json");

    fn diags_for(json: &str) -> Vec<Diagnostic> {
        validate(&Graph::from_json(json).expect("fixture parses"))
    }

    fn rules(diags: &[Diagnostic]) -> Vec<&'static str> {
        diags.iter().map(|d| d.rule).collect()
    }

    #[test]
    fn canonical_example_has_no_errors_or_warnings() {
        let diags = diags_for(HELLO);
        assert!(!has_errors(&diags));
        assert!(diags.iter().all(|d| d.severity == Severity::Info));
        // The documented terminal pattern surfaces as info, not warning.
        assert_eq!(rules(&diags), ["dead-end-branch"]);
    }

    #[test]
    fn duplicate_ids_are_errors() {
        let diags = diags_for(r#"{"nodes":[{"id":"a","content":[]},{"id":"a","content":[]}]}"#);
        assert!(rules(&diags).contains(&"unique-node-ids"));
        assert!(has_errors(&diags));
    }

    #[test]
    fn dangling_targets_are_errors_for_both_edge_kinds() {
        let diags = diags_for(
            r#"{"nodes":[
                {"id":"a","traversal":"ghost","content":[]},
                {"id":"b","traversal":{"branch-point":{"options":[{"label":"x","target":"ghoul"}]}},"content":[]}
            ]}"#,
        );
        let targets: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "valid-traversal-target")
            .collect();
        assert_eq!(targets.len(), 2);
    }

    #[test]
    fn next_and_branch_point_together_is_an_error() {
        let diags = diags_for(
            r#"{"nodes":[
                {"id":"a","traversal":{"next":"b","branch-point":{"options":[{"label":"x","target":"b"}]}},"content":[]},
                {"id":"b","content":[]}
            ]}"#,
        );
        assert!(rules(&diags).contains(&"next-branch-point-conflict"));
    }

    #[test]
    fn duplicate_branch_keys_are_errors() {
        let diags = diags_for(
            r#"{"nodes":[
                {"id":"a","traversal":{"branch-point":{"options":[
                    {"label":"one","key":"x","target":"b"},
                    {"label":"two","key":"x","target":"b"}
                ]}},"content":[]},
                {"id":"b","content":[]}
            ]}"#,
        );
        assert!(rules(&diags).contains(&"unique-branch-keys"));
    }

    #[test]
    fn empty_traversal_object_warns() {
        let diags = diags_for(r#"{"nodes":[{"id":"a","traversal":{},"content":[]}]}"#);
        let hits: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "empty-traversal")
            .collect();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].node.as_deref(), Some("a"));
        assert_eq!(hits[0].severity, Severity::Warning);
        assert!(!has_errors(&diags));
    }

    #[test]
    fn absent_traversal_does_not_warn_as_empty() {
        let diags = diags_for(r#"{"nodes":[{"id":"a","content":[]}]}"#);
        assert!(!rules(&diags).contains(&"empty-traversal"));
    }

    #[test]
    fn populated_traversal_forms_do_not_warn_as_empty() {
        let diags = diags_for(
            r#"{"nodes":[
                {"id":"a","traversal":"b","content":[]},
                {"id":"b","traversal":{"next":"c"},"content":[]},
                {"id":"c","traversal":{"branch-point":{"options":[{"label":"x","target":"a"}]}},"content":[]}
            ]}"#,
        );
        assert!(!rules(&diags).contains(&"empty-traversal"));
    }

    #[test]
    fn unreachable_nodes_warn() {
        let diags =
            diags_for(r#"{"nodes":[{"id":"a","content":[]},{"id":"island","content":[]}]}"#);
        let unreachable: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "unreachable-node")
            .collect();
        assert_eq!(unreachable.len(), 1);
        assert_eq!(unreachable[0].node.as_deref(), Some("island"));
        assert!(!has_errors(&diags));
    }

    #[test]
    fn self_loops_and_trivial_cycles_warn_distinctly() {
        let diags = diags_for(
            r#"{"nodes":[
                {"id":"a","traversal":"a","content":[]},
                {"id":"b","traversal":"c","content":[]},
                {"id":"c","traversal":"b","content":[]}
            ]}"#,
        );
        let r = rules(&diags);
        assert!(r.contains(&"self-loop"));
        assert_eq!(r.iter().filter(|&&x| x == "trivial-cycle").count(), 1);
    }

    #[test]
    fn errors_sort_before_warnings_and_info() {
        let diags = diags_for(
            r#"{"nodes":[
                {"id":"a","traversal":"ghost","content":[]},
                {"id":"island","content":[]}
            ]}"#,
        );
        assert_eq!(diags[0].severity, Severity::Error);
    }
}
