//! Layer-2 semantic validation (spec §4).
//!
//! Checks graph integrity beyond what the JSON schema can express, with the
//! same rules and rule names as `protocol/validate.mjs` so the Rust and Node
//! validators stay in lockstep. Diagnostics are written for presenters, not
//! parser authors: each one names the node and says what to do about it.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use fireside_core::{ContentBlock, Graph, Node, TraversalSpec};

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
    check_container_nesting_depth(graph, &mut diags);
    check_empty_traversal(graph, &mut diags);
    check_reveal_masked_by_container(graph, &mut diags);
    check_ascii_art_too_wide(graph, &mut diags);
    check_ascii_art_empty(graph, &mut diags);
    check_malformed_link_urls(graph, &mut diags);
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

/// The reference implementation's chosen maximum `Container` nesting
/// depth (ADR-010). Unlike the content-quality warnings below, an
/// over-nested document risks pathological recursion in both validators
/// and the renderer, so this is ERROR severity — the same class as
/// `unique-node-ids`/`valid-traversal-target`.
const MAX_CONTAINER_NESTING_DEPTH: u32 = 8;

/// ERROR: a node's content nests `Container` blocks deeper than
/// [`MAX_CONTAINER_NESTING_DEPTH`] (ADR-010).
fn check_container_nesting_depth(graph: &Graph, diags: &mut Vec<Diagnostic>) {
    for node in &graph.nodes {
        let depth = node.content.iter().map(container_depth).max().unwrap_or(0);
        if depth > MAX_CONTAINER_NESTING_DEPTH {
            diags.push(Diagnostic::new(
                Severity::Error,
                "container-nesting-depth-exceeded",
                format!(
                    "\"{}\" nests containers {depth} levels deep, past the maximum of {MAX_CONTAINER_NESTING_DEPTH} — flatten the layout",
                    node.id
                ),
                Some(&node.id),
            ));
        }
    }
}

/// `0` for a non-container leaf; `1 + max(child depth)` for a `Container`
/// (data-model.md's formula, `specs/008-protocol-workflow-hardening/`).
fn container_depth(block: &ContentBlock) -> u32 {
    let ContentBlock::Container { children, .. } = block else {
        return 0;
    };
    1 + children.iter().map(container_depth).max().unwrap_or(0)
}

/// WARNING: a child block's own `reveal` value is lower than its
/// enclosing container's — the child can never actually appear before the
/// container does, so the lower number is misleading rather than
/// functional.
fn check_reveal_masked_by_container(graph: &Graph, diags: &mut Vec<Diagnostic>) {
    for node in &graph.nodes {
        walk_reveal_masking(&node.content, &node.id, diags);
    }
}

fn walk_reveal_masking(blocks: &[ContentBlock], node_id: &str, diags: &mut Vec<Diagnostic>) {
    for block in blocks {
        let ContentBlock::Container {
            children, reveal, ..
        } = block
        else {
            continue;
        };
        let container_level = reveal.unwrap_or(0);
        for child in children {
            let child_level = child.reveal().unwrap_or(0);
            if child_level < container_level {
                diags.push(Diagnostic::new(
                    Severity::Warning,
                    "reveal-masked-by-container",
                    format!(
                        "\"{node_id}\" has a block marked to reveal at step {child_level}, but it's nested inside a group that doesn't reveal until step {container_level} — it can't actually appear before its group does. Raise the block's reveal to {container_level} or higher, or lower the group's"
                    ),
                    Some(node_id),
                ));
            }
        }
        walk_reveal_masking(children, node_id, diags);
    }
}

/// The presentation card's usable width, in columns — "80-col terminal
/// minus card chrome" (spec 005's existing reasoning for the same class
/// of content). Widest-line measurement here counts Unicode scalar
/// values (`chars().count()`), not true display width: `fireside-engine`
/// cannot depend on `unicode-width` (crate boundary table, Principle
/// III), so this is a documented approximation, exact for the common
/// case (plain ASCII art) and only imprecise for wide/combining Unicode
/// characters — the same pragmatic tolerance every other content-quality
/// check in this validator already accepts (e.g. `malformed-link-url`'s
/// "looks like a URL" heuristic).
const MAX_ASCII_ART_WIDTH: usize = 76;

/// WARNING: an `AsciiArt` block's widest line exceeds
/// [`MAX_ASCII_ART_WIDTH`].
fn check_ascii_art_too_wide(graph: &Graph, diags: &mut Vec<Diagnostic>) {
    for node in &graph.nodes {
        walk_ascii_art(&node.content, &node.id, diags, |art, node_id, diags| {
            let widest = art
                .lines()
                .map(str::chars)
                .map(Iterator::count)
                .max()
                .unwrap_or(0);
            if widest > MAX_ASCII_ART_WIDTH {
                diags.push(Diagnostic::new(
                    Severity::Warning,
                    "ascii-art-too-wide",
                    format!(
                        "\"{node_id}\" has an ascii-art block {widest} columns wide, past the {MAX_ASCII_ART_WIDTH}-column limit — it may not fit the presentation card"
                    ),
                    Some(node_id),
                ));
            }
        });
    }
}

/// WARNING: an `AsciiArt` block's `art` is empty or whitespace-only.
fn check_ascii_art_empty(graph: &Graph, diags: &mut Vec<Diagnostic>) {
    for node in &graph.nodes {
        walk_ascii_art(&node.content, &node.id, diags, |art, node_id, diags| {
            if art.trim().is_empty() {
                diags.push(Diagnostic::new(
                    Severity::Warning,
                    "ascii-art-empty",
                    format!("\"{node_id}\" has an ascii-art block with no art content"),
                    Some(node_id),
                ));
            }
        });
    }
}

/// Walks `blocks` recursively (through `Container` children, like
/// `walk_reveal_masking`/`walk_link_urls`), calling `check` on every
/// `AsciiArt` block's `art` string.
fn walk_ascii_art(
    blocks: &[ContentBlock],
    node_id: &str,
    diags: &mut Vec<Diagnostic>,
    check: impl Fn(&str, &str, &mut Vec<Diagnostic>) + Copy,
) {
    for block in blocks {
        match block {
            ContentBlock::AsciiArt { art, .. } => check(art, node_id, diags),
            ContentBlock::Container { children, .. } => {
                walk_ascii_art(children, node_id, diags, check);
            }
            _ => {}
        }
    }
}

/// WARNING: a `[label](url)` link's destination doesn't look like a
/// well-formed URL (contracts/link-syntax.md) — a malformed link must not
/// block presenting, so this is a warning, not an error, matching every
/// other content-quality rule in this validator.
fn check_malformed_link_urls(graph: &Graph, diags: &mut Vec<Diagnostic>) {
    for node in &graph.nodes {
        walk_link_urls(&node.content, &node.id, diags);
    }
}

fn walk_link_urls(blocks: &[ContentBlock], node_id: &str, diags: &mut Vec<Diagnostic>) {
    for block in blocks {
        match block {
            ContentBlock::Text { body, .. } => check_text_links(body, node_id, diags),
            ContentBlock::Heading { text, .. } => check_text_links(text, node_id, diags),
            ContentBlock::List { items, .. } => {
                for item in items {
                    check_text_links(item, node_id, diags);
                }
            }
            ContentBlock::Container { children, .. } => walk_link_urls(children, node_id, diags),
            _ => {}
        }
    }
}

fn check_text_links(text: &str, node_id: &str, diags: &mut Vec<Diagnostic>) {
    for url in find_links(text) {
        if !is_well_formed_url(url) {
            diags.push(Diagnostic::new(
                Severity::Warning,
                "malformed-link-url",
                format!(
                    "\"{node_id}\" has a link whose destination \"{url}\" doesn't look like a well-formed URL (expected something like \"scheme://...\") — presenting still works, but the link won't be usefully clickable"
                ),
                Some(node_id),
            ));
        }
    }
}

/// Extracts every link destination found in `text`'s `[label](url)` syntax
/// — a minimal, independent mirror of `fireside-tui`'s inline-Markdown
/// parser (`fireside-engine` cannot depend on `fireside-tui` per the crate
/// boundary table, and only needs the URL portion to validate).
fn find_links(text: &str) -> Vec<&str> {
    let mut urls = Vec::new();
    let mut i = 0;
    while let Some(open_rel) = text[i..].find('[') {
        let open = i + open_rel;
        let Some(close_rel) = text[open + 1..].find(']') else {
            break;
        };
        let close = open + 1 + close_rel;
        if text[close + 1..].starts_with('(')
            && let Some(paren_close_rel) = text[close + 2..].find(')')
        {
            let paren_close = close + 2 + paren_close_rel;
            urls.push(&text[close + 2..paren_close]);
            i = paren_close + 1;
            continue;
        }
        i = close + 1;
    }
    urls
}

/// A pragmatic "does this look like a URL" check: a non-empty scheme
/// (starts with a letter, then letters/digits/`+`/`.`/`-`), a colon, and a
/// non-empty, whitespace-free remainder.
fn is_well_formed_url(url: &str) -> bool {
    let Some(colon) = url.find(':') else {
        return false;
    };
    let (scheme, rest) = (&url[..colon], &url[colon + 1..]);
    let mut chars = scheme.chars();
    let starts_ok = chars.next().is_some_and(char::is_alphabetic);
    let rest_ok = chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'));
    starts_ok && rest_ok && !rest.is_empty() && !rest.contains(char::is_whitespace)
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
mod proptest_support {
    //! Hand-written generators for graphs that may or may not be
    //! semantically valid — `validate()` must never panic on any of
    //! them. Deliberately its own generator rather than reusing
    //! `session.rs`'s (which only needs navigable graphs, no fuzzing
    //! toward invalid shapes) or `fireside-core`'s (`#[cfg(test)]`-private
    //! to that crate, unreachable from here across the crate boundary):
    //! this one leans into the shapes `validate`'s checks specifically
    //! look for — a small, reused id alphabet so duplicates and dangling
    //! targets are common rather than vanishingly rare, traversal that
    //! deliberately sets both `next` and `branch-point` sometimes, and
    //! container nesting bounded just past the validator's depth-8 limit
    //! so both at-limit and over-limit shapes actually occur.

    use proptest::collection::vec;
    use proptest::option;
    use proptest::prelude::*;

    use fireside_core::{
        BranchOption, BranchPoint, ContentBlock, Graph, Node, Traversal, TraversalSpec,
    };

    /// A handful of short, reused ids — deliberately not unique per node,
    /// so `unique-node-ids` and dangling-target shapes both occur often.
    fn arbitrary_id() -> impl Strategy<Value = String> {
        "[a-c][0-9]?".prop_map(String::from)
    }

    fn arbitrary_branch_option() -> impl Strategy<Value = BranchOption> {
        (arbitrary_id(), option::of(arbitrary_id())).prop_map(|(target, key)| BranchOption {
            label: "opt".to_owned(),
            key,
            target,
            description: None,
        })
    }

    fn arbitrary_traversal() -> impl Strategy<Value = Option<TraversalSpec>> {
        prop_oneof![
            2 => Just(None),
            3 => arbitrary_id().prop_map(|t| Some(TraversalSpec::Target(t))),
            2 => arbitrary_id().prop_map(|next| Some(TraversalSpec::Rules(Traversal {
                next: Some(next),
                branch_point: None,
            }))),
            2 => vec(arbitrary_branch_option(), 0..3).prop_map(|options| Some(
                TraversalSpec::Rules(Traversal { next: None, branch_point: Some(BranchPoint { prompt: None, options }) })
            )),
            // Deliberately invalid: both `next` and `branch-point` set.
            1 => (arbitrary_id(), vec(arbitrary_branch_option(), 0..3)).prop_map(|(next, options)| Some(
                TraversalSpec::Rules(Traversal {
                    next: Some(next),
                    branch_point: Some(BranchPoint { prompt: None, options }),
                })
            )),
        ]
    }

    fn arbitrary_leaf_block() -> impl Strategy<Value = ContentBlock> {
        let reveal = option::of(0u32..4);
        prop_oneof![
            reveal.clone().prop_map(|reveal| ContentBlock::Text {
                reveal,
                body: "text with a [link](not really a url)".to_owned(),
            }),
            reveal.prop_map(|reveal| ContentBlock::Divider { reveal }),
        ]
    }

    /// Nesting bounded to 10 (just past the validator's depth-8 limit,
    /// see ADR-010) so both at-limit and over-limit shapes actually get
    /// generated, not just the vastly more common shallow case.
    fn arbitrary_content_block() -> impl Strategy<Value = ContentBlock> {
        arbitrary_leaf_block().prop_recursive(10, 20, 3, |inner| {
            (option::of(0u32..4), vec(inner, 0..3)).prop_map(|(reveal, children)| {
                ContentBlock::Container {
                    reveal,
                    children,
                    layout: None,
                }
            })
        })
    }

    fn arbitrary_node() -> impl Strategy<Value = Node> {
        (
            arbitrary_id(),
            arbitrary_traversal(),
            vec(arbitrary_content_block(), 0..3),
        )
            .prop_map(|(id, traversal, content)| Node {
                id,
                title: None,
                view_mode: None,
                transition: None,
                speaker_notes: None,
                traversal,
                content,
            })
    }

    pub(super) fn arbitrary_graph() -> impl Strategy<Value = Graph> {
        vec(arbitrary_node(), 1..6).prop_map(|nodes| Graph {
            fireside_version: None,
            title: None,
            author: None,
            date: None,
            description: None,
            version: None,
            defaults: None,
            nodes,
        })
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
    fn reveal_masked_by_container_warns() {
        let diags = diags_for(
            r#"{"nodes":[{"id":"a","content":[
                {"kind":"container","reveal":2,"children":[
                    {"kind":"text","body":"x","reveal":1}
                ]}
            ]}]}"#,
        );
        let hits: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "reveal-masked-by-container")
            .collect();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].node.as_deref(), Some("a"));
        assert_eq!(hits[0].severity, Severity::Warning);
        assert!(!has_errors(&diags));
    }

    #[test]
    fn reveal_not_masked_when_child_reveal_is_greater_or_equal() {
        let diags = diags_for(
            r#"{"nodes":[{"id":"a","content":[
                {"kind":"container","reveal":1,"children":[
                    {"kind":"text","body":"x","reveal":1},
                    {"kind":"text","body":"y","reveal":2}
                ]}
            ]}]}"#,
        );
        assert!(!rules(&diags).contains(&"reveal-masked-by-container"));
    }

    #[test]
    fn ascii_art_too_wide_warns_on_oversized_art() {
        let wide_line = "x".repeat(MAX_ASCII_ART_WIDTH + 1);
        let diags = diags_for(&format!(
            r#"{{"nodes":[{{"id":"a","content":[{{"kind":"ascii-art","art":"{wide_line}"}}]}}]}}"#
        ));
        let hits: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "ascii-art-too-wide")
            .collect();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].node.as_deref(), Some("a"));
        assert_eq!(hits[0].severity, Severity::Warning);
        assert!(!has_errors(&diags));
    }

    #[test]
    fn ascii_art_empty_warns_on_blank_art() {
        let diags =
            diags_for(r#"{"nodes":[{"id":"a","content":[{"kind":"ascii-art","art":"   "}]}]}"#);
        let hits: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "ascii-art-empty")
            .collect();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].node.as_deref(), Some("a"));
        assert_eq!(hits[0].severity, Severity::Warning);
        assert!(!has_errors(&diags));
    }

    #[test]
    fn ascii_art_within_limits_produces_no_warning() {
        let diags = diags_for(
            r#"{"nodes":[{"id":"a","content":[{"kind":"ascii-art","art":"  o.o  \n /---\\ "}]}]}"#,
        );
        assert!(!rules(&diags).contains(&"ascii-art-too-wide"));
        assert!(!rules(&diags).contains(&"ascii-art-empty"));
    }

    #[test]
    fn malformed_link_url_warns() {
        let diags = diags_for(
            r#"{"nodes":[{"id":"a","content":[
                {"kind":"text","body":"see [here](not a url) for more"}
            ]}]}"#,
        );
        let hits: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "malformed-link-url")
            .collect();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].node.as_deref(), Some("a"));
        assert_eq!(hits[0].severity, Severity::Warning);
        assert!(
            !has_errors(&diags),
            "a malformed link must not block presenting"
        );
    }

    #[test]
    fn well_formed_link_url_does_not_warn() {
        let diags = diags_for(
            r#"{"nodes":[{"id":"a","content":[
                {"kind":"text","body":"see [here](https://example.com/docs) for more"},
                {"kind":"heading","level":2,"text":"[in a heading](mailto:a@b.com)"},
                {"kind":"list","items":["[in a list item](https://example.com)"]}
            ]}]}"#,
        );
        assert!(!rules(&diags).contains(&"malformed-link-url"));
    }

    #[test]
    fn text_with_no_links_never_warns() {
        let diags = diags_for(
            r#"{"nodes":[{"id":"a","content":[
            {"kind":"text","body":"plain text with [brackets] but no link syntax"}
        ]}]}"#,
        );
        assert!(!rules(&diags).contains(&"malformed-link-url"));
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

    proptest::proptest! {
        /// For any generated graph — deliberately including duplicate
        /// ids, dangling targets, conflicting `next`/`branch-point`
        /// traversal, empty branch options, malformed-looking links, and
        /// container nesting past the depth-8 limit — `validate()` must
        /// never panic, and every diagnostic that names a node must name
        /// a node that actually exists in the graph: a diagnostic
        /// pointing at a phantom node would be worse than useless to a
        /// presenter trying to act on it.
        #[test]
        fn validate_never_panics_and_only_names_real_nodes(
            graph in proptest_support::arbitrary_graph()
        ) {
            let ids: HashSet<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
            let diags = validate(&graph);
            for d in &diags {
                if let Some(node) = &d.node {
                    proptest::prop_assert!(
                        ids.contains(node.as_str()),
                        "diagnostic [{}] names node {node:?}, which doesn't exist in the graph",
                        d.rule
                    );
                }
            }
        }
    }
}
