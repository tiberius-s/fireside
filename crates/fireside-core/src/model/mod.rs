//! The Fireside 0.1.0 protocol data model.
//!
//! Every type mirrors the generated JSON schemas in
//! `protocol/tsp-output/schemas/` exactly: kebab-case property names, the
//! `"kind"` discriminator for content blocks, closed enums, and nothing the
//! protocol does not define. Unknown properties in documents are ignored on
//! read (the schema layer owns strictness) and absent optional fields stay
//! absent on write, so load → save round-trips are faithful.

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// A unique string identifier for a node within a graph.
///
/// IDs MUST be unique within a graph and SHOULD be kebab-case.
pub type NodeId = String;

// ─── Graph ───────────────────────────────────────────────────────────────────

/// The top-level Fireside document: metadata, optional defaults, and the
/// ordered array of nodes. The first node is the entry point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Graph {
    /// Protocol version for this document (e.g. `"0.1.0"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fireside_version: Option<String>,

    /// The graph's display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The graph creator's name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Creation or presentation date (ISO 8601 recommended).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,

    /// A brief summary of the graph's purpose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Semantic version of this graph document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Default values applied to all nodes unless overridden.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defaults: Option<NodeDefaults>,

    /// The ordered array of nodes forming the graph.
    pub nodes: Vec<Node>,
}

impl Graph {
    /// Parse a graph from JSON text.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Parse`] when the text is not valid JSON or does
    /// not match the protocol data model.
    pub fn from_json(text: &str) -> Result<Self, CoreError> {
        Ok(serde_json::from_str(text)?)
    }

    /// Serialize the graph as pretty-printed JSON.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Parse`] if serialization fails (practically
    /// unreachable for this model).
    pub fn to_json_pretty(&self) -> Result<String, CoreError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Look up a node by id.
    #[must_use]
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// The entry node — the first node in the array.
    ///
    /// The schema requires at least one node, but a hand-built [`Graph`]
    /// value may be empty, so this returns an `Option`.
    #[must_use]
    pub fn entry(&self) -> Option<&Node> {
        self.nodes.first()
    }
}

/// Default values applied to all nodes unless overridden at the node level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NodeDefaults {
    /// Default view mode for all nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_mode: Option<ViewMode>,

    /// Default transition for all nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition: Option<Transition>,
}

// ─── Node ────────────────────────────────────────────────────────────────────

/// A vertex in the graph — a discrete unit of content a presenter visits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Node {
    /// Unique identifier for this node.
    pub id: NodeId,

    /// Human-readable node title for navigation UIs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Presentation frame mode for this node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_mode: Option<ViewMode>,

    /// Pacing intent when entering this node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition: Option<Transition>,

    /// Notes visible only to the presenter, not the audience.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker_notes: Option<String>,

    /// How the presenter leaves this node. Absent means terminal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traversal: Option<TraversalSpec>,

    /// The content blocks displayed at this node, in render order.
    pub content: Vec<ContentBlock>,
}

impl Node {
    /// The explicit next target, whichever traversal form declares it.
    #[must_use]
    pub fn next_target(&self) -> Option<&str> {
        match self.traversal.as_ref()? {
            TraversalSpec::Target(id) => Some(id),
            TraversalSpec::Rules(t) => t.next.as_deref(),
        }
    }

    /// The branch point at this node, if any.
    #[must_use]
    pub fn branch_point(&self) -> Option<&BranchPoint> {
        match self.traversal.as_ref()? {
            TraversalSpec::Target(_) => None,
            TraversalSpec::Rules(t) => t.branch_point.as_ref(),
        }
    }

    /// Whether this node is terminal: no next edge and no branch point.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.next_target().is_none() && self.branch_point().is_none()
    }

    /// Resolve the effective view mode: node value, then graph defaults,
    /// then the built-in default.
    #[must_use]
    pub fn resolved_view_mode(&self, defaults: Option<&NodeDefaults>) -> ViewMode {
        self.view_mode
            .or_else(|| defaults.and_then(|d| d.view_mode))
            .unwrap_or_default()
    }

    /// Resolve the effective transition: node value, then graph defaults,
    /// then the built-in default.
    #[must_use]
    pub fn resolved_transition(&self, defaults: Option<&NodeDefaults>) -> Transition {
        self.transition
            .or_else(|| defaults.and_then(|d| d.transition))
            .unwrap_or_default()
    }

    /// The distinct positive `reveal` values used anywhere in this node's
    /// content, recursively through `Container` children, sorted
    /// ascending. An empty result means the node uses no reveal marks —
    /// `next()` never pauses for reveal on such a node. Steps are ordinal
    /// over these distinct values, not raw integer magnitudes, so a gap
    /// in an author's numbering can never produce a step that reveals
    /// nothing.
    #[must_use]
    pub fn reveal_levels(&self) -> Vec<u32> {
        let mut levels = Vec::new();
        collect_reveal_levels(&self.content, &mut levels);
        levels.sort_unstable();
        levels.dedup();
        levels
    }
}

fn collect_reveal_levels(blocks: &[ContentBlock], out: &mut Vec<u32>) {
    for block in blocks {
        if let Some(level) = block.reveal()
            && level > 0
        {
            out.push(level);
        }
        collect_reveal_levels(block.children(), out);
    }
}

// ─── Traversal ───────────────────────────────────────────────────────────────

/// The two wire forms of `Node.traversal`: a string shorthand for a simple
/// next edge, or the object form for branching.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TraversalSpec {
    /// String shorthand — equivalent to `{ "next": "<id>" }`.
    Target(NodeId),

    /// Object form with `next` or `branch-point`.
    Rules(Traversal),
}

/// The object form of traversal. A document MUST NOT set both `next` and
/// `branch-point`; validation rejects that as contradictory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Traversal {
    /// Navigate to this node on next().
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<NodeId>,

    /// Present a choice; next() is blocked until the presenter chooses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_point: Option<BranchPoint>,
}

/// A decision point where the presenter must choose between options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BranchPoint {
    /// The prompt displayed to the presenter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// The available options. The schema requires at least one.
    pub options: Vec<BranchOption>,
}

/// A single choice available at a branch point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BranchOption {
    /// Display label for this option.
    pub label: String,

    /// Keyboard shortcut to select this option (e.g. `"a"`, `"1"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The node this option leads to.
    pub target: NodeId,

    /// Optional description providing more detail about this choice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ─── Content blocks ──────────────────────────────────────────────────────────

/// An atomic content element within a node, discriminated by `kind`.
///
/// Numeric fields use the natural Rust width for their meaning (heading
/// levels are 1–6, sizes are terminal cells); the schemas' `int32` is a
/// TypeSpec artifact, and out-of-domain values fail at parse with a clear
/// serde error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "kebab-case"
)]
pub enum ContentBlock {
    /// A heading with a level (1–6) and text content.
    Heading {
        /// The incremental-reveal step at which this block becomes
        /// visible. `None` and `Some(0)` are equivalent: visible
        /// immediately. See [`Node::reveal_levels`].
        #[serde(skip_serializing_if = "Option::is_none")]
        reveal: Option<u32>,
        /// Heading level from 1 (largest) to 6 (smallest).
        level: u8,
        /// The heading text content.
        text: String,
    },

    /// A block of prose text, optionally with inline Markdown formatting.
    Text {
        /// The incremental-reveal step at which this block becomes
        /// visible. See [`ContentBlock::Heading::reveal`].
        #[serde(skip_serializing_if = "Option::is_none")]
        reveal: Option<u32>,
        /// The text content.
        body: String,
    },

    /// A fenced code block with language annotation and optional highlighting.
    Code {
        /// The incremental-reveal step at which this block becomes
        /// visible. See [`ContentBlock::Heading::reveal`].
        #[serde(skip_serializing_if = "Option::is_none")]
        reveal: Option<u32>,
        /// Programming language identifier for syntax highlighting.
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
        /// The source code content.
        source: String,
        /// Line numbers (1-based) to visually emphasize.
        #[serde(skip_serializing_if = "Option::is_none")]
        highlight_lines: Option<Vec<u32>>,
        /// Whether to display line numbers.
        #[serde(skip_serializing_if = "Option::is_none")]
        show_line_numbers: Option<bool>,
    },

    /// An ordered or unordered list of items.
    List {
        /// The incremental-reveal step at which this block becomes
        /// visible. See [`ContentBlock::Heading::reveal`].
        #[serde(skip_serializing_if = "Option::is_none")]
        reveal: Option<u32>,
        /// Whether the list is ordered (numbered) or unordered (bulleted).
        #[serde(skip_serializing_if = "Option::is_none")]
        ordered: Option<bool>,
        /// The list items as strings.
        items: Vec<String>,
    },

    /// A visual element with source URI and accessibility metadata.
    Image {
        /// The incremental-reveal step at which this block becomes
        /// visible. See [`ContentBlock::Heading::reveal`].
        #[serde(skip_serializing_if = "Option::is_none")]
        reveal: Option<u32>,
        /// URI or file path to the image source.
        src: String,
        /// Alternative text for accessibility.
        #[serde(skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
        /// Optional caption displayed below the image.
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
        /// Desired display width in terminal cells (columns).
        #[serde(skip_serializing_if = "Option::is_none")]
        width: Option<u16>,
        /// Desired display height in terminal cells (rows).
        #[serde(skip_serializing_if = "Option::is_none")]
        height: Option<u16>,
    },

    /// A horizontal rule separating content sections.
    Divider {
        /// The incremental-reveal step at which this block becomes
        /// visible. See [`ContentBlock::Heading::reveal`].
        #[serde(skip_serializing_if = "Option::is_none")]
        reveal: Option<u32>,
    },

    /// A container for nested content blocks with layout control.
    Container {
        /// The incremental-reveal step at which this block becomes
        /// visible. See [`ContentBlock::Heading::reveal`]. Hiding a
        /// container hides every one of its children regardless of
        /// their own `reveal` values.
        #[serde(skip_serializing_if = "Option::is_none")]
        reveal: Option<u32>,
        /// The child content blocks within this container.
        children: Vec<ContentBlock>,
        /// Layout hint controlling how children are arranged.
        #[serde(skip_serializing_if = "Option::is_none")]
        layout: Option<ContainerLayout>,
    },

    /// Pre-rendered ASCII/text art, generated at authoring time. See
    /// [`ADR-012`](https://github.com/tiberius-s/fireside/blob/main/.claude/adrs/adr-012-ascii-art-protocol-change.md)
    /// for why this is a new block kind rather than an additive field.
    AsciiArt {
        /// The incremental-reveal step at which this block becomes
        /// visible. See [`ContentBlock::Heading::reveal`].
        #[serde(skip_serializing_if = "Option::is_none")]
        reveal: Option<u32>,
        /// The pre-rendered multi-line art content, as plain text.
        art: String,
        /// Alternative text description, for anyone who can't see the
        /// art.
        #[serde(skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
    },
}

impl ContentBlock {
    /// This block's own reveal marker, if any. `None` is equivalent to
    /// `Some(0)` for step-visibility purposes: always visible.
    #[must_use]
    pub fn reveal(&self) -> Option<u32> {
        match self {
            Self::Heading { reveal, .. }
            | Self::Text { reveal, .. }
            | Self::Code { reveal, .. }
            | Self::List { reveal, .. }
            | Self::Image { reveal, .. }
            | Self::Divider { reveal }
            | Self::AsciiArt { reveal, .. }
            | Self::Container { reveal, .. } => *reveal,
        }
    }

    fn children(&self) -> &[ContentBlock] {
        match self {
            Self::Container { children, .. } => children,
            _ => &[],
        }
    }
}

// ─── Enums ───────────────────────────────────────────────────────────────────

/// Presentation frame mode for a node. Controls how much screen real estate
/// content gets — content arrangement is the container's job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ViewMode {
    /// Standard presentation frame.
    #[default]
    Default,
    /// Maximum content area, minimal frame.
    Fullscreen,
}

/// Pacing intent when transitioning between nodes. Engines choose the
/// visual effect; unsupported values fall back to `none`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Transition {
    /// Instant transition. No animation.
    #[default]
    None,
    /// Smooth transition. Engine chooses the visual effect.
    Fade,
}

/// Layout hint controlling how a container's children are arranged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContainerLayout {
    /// Vertical stacking, top-aligned (default).
    #[default]
    Stack,
    /// Side-by-side arrangement, left to right in array order.
    Columns,
    /// Centered vertically and horizontally.
    Center,
}

#[cfg(test)]
mod proptest_support {
    //! Hand-written `proptest::Strategy` generators for the wire-format
    //! types, per `specs/008-protocol-workflow-hardening/research.md` §2.
    //! Written by hand rather than via `proptest-derive` so the recursive
    //! `Container` case can be bounded with `prop_recursive` directly, and
    //! so `fireside-core`'s production dependency list (Principle III)
    //! never needs a proc-macro crate — this module is `#[cfg(test)]`
    //! only.

    use proptest::collection::vec;
    use proptest::option;
    use proptest::prelude::*;

    use super::{
        BranchOption, BranchPoint, ContainerLayout, ContentBlock, Graph, Node, NodeDefaults,
        Transition, Traversal, TraversalSpec, ViewMode,
    };

    /// Short, printable strings — arbitrary Unicode `String` is valid input
    /// too, but keeps failing cases small and readable when shrunk.
    fn arbitrary_string() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 _.,!?-]{0,12}"
    }

    fn arbitrary_view_mode() -> impl Strategy<Value = ViewMode> {
        prop_oneof![Just(ViewMode::Default), Just(ViewMode::Fullscreen)]
    }

    fn arbitrary_transition() -> impl Strategy<Value = Transition> {
        prop_oneof![Just(Transition::None), Just(Transition::Fade)]
    }

    fn arbitrary_container_layout() -> impl Strategy<Value = ContainerLayout> {
        prop_oneof![
            Just(ContainerLayout::Stack),
            Just(ContainerLayout::Columns),
            Just(ContainerLayout::Center),
        ]
    }

    /// A non-container leaf block: every `ContentBlock` variant except
    /// `Container` itself, which `arbitrary_content_block` wraps this in
    /// via `prop_recursive`.
    fn arbitrary_leaf_block() -> impl Strategy<Value = ContentBlock> {
        let reveal = option::of(any::<u32>());
        prop_oneof![
            (reveal.clone(), 1u8..=6, arbitrary_string()).prop_map(|(reveal, level, text)| {
                ContentBlock::Heading {
                    reveal,
                    level,
                    text,
                }
            }),
            (reveal.clone(), arbitrary_string())
                .prop_map(|(reveal, body)| ContentBlock::Text { reveal, body }),
            (
                reveal.clone(),
                option::of(arbitrary_string()),
                arbitrary_string(),
                option::of(vec(any::<u32>(), 0..4)),
                option::of(any::<bool>()),
            )
                .prop_map(
                    |(reveal, language, source, highlight_lines, show_line_numbers)| {
                        ContentBlock::Code {
                            reveal,
                            language,
                            source,
                            highlight_lines,
                            show_line_numbers,
                        }
                    }
                ),
            (
                reveal.clone(),
                option::of(any::<bool>()),
                vec(arbitrary_string(), 0..5),
            )
                .prop_map(|(reveal, ordered, items)| ContentBlock::List {
                    reveal,
                    ordered,
                    items
                }),
            (
                reveal.clone(),
                arbitrary_string(),
                option::of(arbitrary_string()),
                option::of(arbitrary_string()),
                option::of(any::<u16>()),
                option::of(any::<u16>()),
            )
                .prop_map(|(reveal, src, alt, caption, width, height)| {
                    ContentBlock::Image {
                        reveal,
                        src,
                        alt,
                        caption,
                        width,
                        height,
                    }
                }),
            reveal
                .clone()
                .prop_map(|reveal| ContentBlock::Divider { reveal }),
            (reveal, arbitrary_string(), option::of(arbitrary_string()))
                .prop_map(|(reveal, art, alt)| ContentBlock::AsciiArt { reveal, art, alt }),
        ]
    }

    /// Bounds `Container` nesting to a shallow depth during generation —
    /// independent of (and much smaller than) the validator's depth-8
    /// limit added by this same feature; this bound only exists to keep
    /// generated cases small and shrinking fast.
    fn arbitrary_content_block() -> impl Strategy<Value = ContentBlock> {
        arbitrary_leaf_block().prop_recursive(3, 12, 4, |inner| {
            (
                option::of(any::<u32>()),
                vec(inner, 1..4),
                option::of(arbitrary_container_layout()),
            )
                .prop_map(|(reveal, children, layout)| ContentBlock::Container {
                    reveal,
                    children,
                    layout,
                })
        })
    }

    fn arbitrary_branch_option() -> impl Strategy<Value = BranchOption> {
        (
            arbitrary_string(),
            option::of(arbitrary_string()),
            arbitrary_string(),
            option::of(arbitrary_string()),
        )
            .prop_map(|(label, key, target, description)| BranchOption {
                label,
                key,
                target,
                description,
            })
    }

    fn arbitrary_branch_point() -> impl Strategy<Value = BranchPoint> {
        (
            option::of(arbitrary_string()),
            vec(arbitrary_branch_option(), 1..4),
        )
            .prop_map(|(prompt, options)| BranchPoint { prompt, options })
    }

    fn arbitrary_traversal_spec() -> impl Strategy<Value = TraversalSpec> {
        prop_oneof![
            arbitrary_string().prop_map(TraversalSpec::Target),
            (
                option::of(arbitrary_string()),
                option::of(arbitrary_branch_point())
            )
                .prop_map(|(next, branch_point)| TraversalSpec::Rules(Traversal {
                    next,
                    branch_point,
                })),
        ]
    }

    pub(super) fn arbitrary_node() -> impl Strategy<Value = Node> {
        (
            arbitrary_string(),
            option::of(arbitrary_string()),
            option::of(arbitrary_view_mode()),
            option::of(arbitrary_transition()),
            option::of(arbitrary_string()),
            option::of(arbitrary_traversal_spec()),
            vec(arbitrary_content_block(), 0..4),
        )
            .prop_map(
                |(id, title, view_mode, transition, speaker_notes, traversal, content)| Node {
                    id,
                    title,
                    view_mode,
                    transition,
                    speaker_notes,
                    traversal,
                    content,
                },
            )
    }

    fn arbitrary_node_defaults() -> impl Strategy<Value = NodeDefaults> {
        (
            option::of(arbitrary_view_mode()),
            option::of(arbitrary_transition()),
        )
            .prop_map(|(view_mode, transition)| NodeDefaults {
                view_mode,
                transition,
            })
    }

    /// An arbitrary `Graph`. Deliberately does **not** enforce
    /// protocol-level semantic validity (unique node ids, resolvable
    /// traversal targets) — the round-trip property this feeds
    /// (`graph_round_trips_through_json`) is a pure serde property,
    /// independent of `fireside-engine::validate`.
    pub(super) fn arbitrary_graph() -> impl Strategy<Value = Graph> {
        (
            option::of(arbitrary_string()),
            option::of(arbitrary_string()),
            option::of(arbitrary_string()),
            option::of(arbitrary_string()),
            option::of(arbitrary_string()),
            option::of(arbitrary_string()),
            option::of(arbitrary_node_defaults()),
            vec(arbitrary_node(), 0..6),
        )
            .prop_map(
                |(fireside_version, title, author, date, description, version, defaults, nodes)| {
                    Graph {
                        fireside_version,
                        title,
                        author,
                        date,
                        description,
                        version,
                        defaults,
                        nodes,
                    }
                },
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELLO: &str = include_str!("../../../../docs/examples/hello.json");

    #[test]
    fn canonical_example_parses() {
        let graph = Graph::from_json(HELLO).expect("hello.json must parse");
        assert_eq!(graph.title.as_deref(), Some("Hello, Fireside"));
        assert_eq!(graph.nodes.len(), 6);
        assert_eq!(graph.entry().expect("non-empty").id, "intro");
    }

    #[test]
    fn canonical_example_round_trips() {
        let graph = Graph::from_json(HELLO).expect("parse");
        let json = graph.to_json_pretty().expect("serialize");
        let again = Graph::from_json(&json).expect("re-parse");
        assert_eq!(graph, again);
    }

    #[test]
    fn round_trip_preserves_absent_fields() {
        let graph = Graph::from_json(r#"{"nodes":[{"id":"a","content":[]}]}"#).expect("parse");
        let json = serde_json::to_string(&graph).expect("serialize");
        assert_eq!(json, r#"{"nodes":[{"id":"a","content":[]}]}"#);
    }

    #[test]
    fn traversal_string_shorthand() {
        let node: Node =
            serde_json::from_str(r#"{"id":"a","traversal":"b","content":[]}"#).expect("parse");
        assert_eq!(node.next_target(), Some("b"));
        assert!(node.branch_point().is_none());
        assert!(!node.is_terminal());
    }

    #[test]
    fn traversal_object_next() {
        let node: Node =
            serde_json::from_str(r#"{"id":"a","traversal":{"next":"b"},"content":[]}"#)
                .expect("parse");
        assert_eq!(node.next_target(), Some("b"));
    }

    #[test]
    fn traversal_branch_point() {
        let node: Node = serde_json::from_str(
            r#"{"id":"a","traversal":{"branch-point":{"prompt":"?","options":[{"label":"L","key":"1","target":"b"}]}},"content":[]}"#,
        )
        .expect("parse");
        let bp = node.branch_point().expect("branch point");
        assert_eq!(bp.prompt.as_deref(), Some("?"));
        assert_eq!(bp.options[0].target, "b");
        assert!(node.next_target().is_none());
        assert!(!node.is_terminal());
    }

    #[test]
    fn absent_traversal_is_terminal() {
        let node: Node = serde_json::from_str(r#"{"id":"a","content":[]}"#).expect("parse");
        assert!(node.is_terminal());
    }

    #[test]
    fn empty_traversal_object_is_terminal() {
        let node: Node =
            serde_json::from_str(r#"{"id":"a","traversal":{},"content":[]}"#).expect("parse");
        assert!(node.is_terminal());
    }

    #[test]
    fn unknown_fields_are_ignored() {
        // Legacy/foreign fields (`after`, `theme`, node `layout`) parse fine;
        // the schema layer owns strictness.
        let graph = Graph::from_json(
            r#"{"theme":"x","nodes":[{"id":"a","layout":"center","traversal":{"after":"z"},"content":[]}]}"#,
        )
        .expect("parse");
        assert!(graph.nodes[0].is_terminal());
    }

    #[test]
    fn closed_enums_reject_unknown_values() {
        assert!(serde_json::from_str::<ViewMode>(r#""cinema""#).is_err());
        assert!(serde_json::from_str::<Transition>(r#""slide-left""#).is_err());
        assert!(serde_json::from_str::<ContainerLayout>(r#""split-horizontal""#).is_err());
    }

    #[test]
    fn content_blocks_use_kebab_case_wire_format() {
        let block: ContentBlock = serde_json::from_str(
            r#"{"kind":"code","source":"x","highlight-lines":[1],"show-line-numbers":true}"#,
        )
        .expect("parse");
        let json = serde_json::to_string(&block).expect("serialize");
        assert!(json.contains(r#""kind":"code""#));
        assert!(json.contains(r#""highlight-lines""#));
        assert!(json.contains(r#""show-line-numbers""#));
    }

    #[test]
    fn ascii_art_block_round_trips_with_kebab_case_wire_format() {
        let block: ContentBlock =
            serde_json::from_str(r#"{"kind":"ascii-art","art":"x","alt":"y","reveal":1}"#)
                .expect("parse");
        assert_eq!(block.reveal(), Some(1));
        let ContentBlock::AsciiArt { art, alt, .. } = &block else {
            panic!("expected AsciiArt");
        };
        assert_eq!(art, "x");
        assert_eq!(alt.as_deref(), Some("y"));

        let json = serde_json::to_string(&block).expect("serialize");
        assert!(json.contains(r#""kind":"ascii-art""#));
        assert!(json.contains(r#""art":"x""#));
        assert!(json.contains(r#""alt":"y""#));

        let no_alt: ContentBlock =
            serde_json::from_str(r#"{"kind":"ascii-art","art":"x"}"#).expect("parse");
        let json = serde_json::to_string(&no_alt).expect("serialize");
        assert!(!json.contains("alt"), "absent alt stays absent: {json}");
    }

    #[test]
    fn unknown_kind_produces_clear_parse_error() {
        let err = Graph::from_json(r#"{"nodes":[{"id":"a","content":[{"kind":"not-a-kind"}]}]}"#)
            .expect_err("unrecognized kind must fail to parse");
        let message = err.to_string();
        assert!(
            message.contains("unknown variant"),
            "expected an unknown-variant error, got: {message}"
        );
    }

    #[test]
    fn view_mode_resolution_cascade() {
        let defaults = NodeDefaults {
            view_mode: Some(ViewMode::Fullscreen),
            transition: None,
        };
        let mut node: Node = serde_json::from_str(r#"{"id":"a","content":[]}"#).expect("parse");

        assert_eq!(node.resolved_view_mode(None), ViewMode::Default);
        assert_eq!(
            node.resolved_view_mode(Some(&defaults)),
            ViewMode::Fullscreen
        );
        assert_eq!(node.resolved_transition(Some(&defaults)), Transition::None);

        node.view_mode = Some(ViewMode::Default);
        assert_eq!(node.resolved_view_mode(Some(&defaults)), ViewMode::Default);
    }

    #[test]
    fn reveal_field_round_trips_and_defaults_to_none() {
        let block: ContentBlock =
            serde_json::from_str(r#"{"kind":"text","body":"x","reveal":2}"#).expect("parse");
        assert_eq!(block.reveal(), Some(2));
        let json = serde_json::to_string(&block).expect("serialize");
        assert!(json.contains(r#""reveal":2"#));

        let unmarked: ContentBlock =
            serde_json::from_str(r#"{"kind":"text","body":"x"}"#).expect("parse");
        assert_eq!(unmarked.reveal(), None);
        let json = serde_json::to_string(&unmarked).expect("serialize");
        assert!(
            !json.contains("reveal"),
            "absent reveal stays absent on write: {json}"
        );
    }

    #[test]
    fn reveal_levels_collects_distinct_positive_values_recursively() {
        let node: Node = serde_json::from_str(
            r#"{"id":"a","content":[
                {"kind":"text","body":"always"},
                {"kind":"text","body":"x","reveal":1},
                {"kind":"text","body":"y","reveal":0},
                {"kind":"container","children":[
                    {"kind":"text","body":"z","reveal":1},
                    {"kind":"text","body":"w","reveal":3}
                ]}
            ]}"#,
        )
        .expect("parse");
        assert_eq!(node.reveal_levels(), vec![1, 3]);
    }

    #[test]
    fn reveal_levels_is_empty_when_no_block_uses_reveal() {
        let graph = Graph::from_json(HELLO).expect("parse");
        assert!(graph.nodes[0].reveal_levels().is_empty());
    }

    proptest::proptest! {
        /// Any structurally valid `Graph` value survives a
        /// serialize/deserialize round trip unchanged (spec 008 US1,
        /// FR-001). Uses `to_json_pretty`/`from_json` — the same pair every
        /// real caller (`fireside-cli`) uses — rather than the compact
        /// `serde_json::to_string` used by some unit tests above, so the
        /// property matches production round-trip behavior exactly.
        #[test]
        fn graph_round_trips_through_json(graph in proptest_support::arbitrary_graph()) {
            let json = graph.to_json_pretty().expect("serialize");
            let again = Graph::from_json(&json).expect("re-parse");
            proptest::prop_assert_eq!(graph, again);
        }

        /// `reveal_levels()` is always sorted ascending, free of
        /// duplicates, and contains no non-positive values — regardless
        /// of what `reveal` values (repeats, zeros, out-of-order nesting)
        /// the node's content actually uses. The engine's reveal-gating
        /// (`Session::next`/`has_pending_reveal`) trusts this ordering
        /// without re-sorting, so a regression here would silently break
        /// reveal progression rather than fail loudly.
        #[test]
        fn reveal_levels_are_sorted_deduped_and_positive(node in proptest_support::arbitrary_node()) {
            let levels = node.reveal_levels();
            proptest::prop_assert!(
                levels.iter().all(|&l| l > 0),
                "no non-positive reveal levels: {levels:?}"
            );
            let mut sorted = levels.clone();
            sorted.sort_unstable();
            sorted.dedup();
            proptest::prop_assert_eq!(&levels, &sorted, "levels must already be sorted and deduped");
        }
    }
}
