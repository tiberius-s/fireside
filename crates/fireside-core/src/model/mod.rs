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
        /// Heading level from 1 (largest) to 6 (smallest).
        level: u8,
        /// The heading text content.
        text: String,
    },

    /// A block of prose text, optionally with inline Markdown formatting.
    Text {
        /// The text content.
        body: String,
    },

    /// A fenced code block with language annotation and optional highlighting.
    Code {
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
        /// Whether the list is ordered (numbered) or unordered (bulleted).
        #[serde(skip_serializing_if = "Option::is_none")]
        ordered: Option<bool>,
        /// The list items as strings.
        items: Vec<String>,
    },

    /// A visual element with source URI and accessibility metadata.
    Image {
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
    Divider,

    /// A container for nested content blocks with layout control.
    Container {
        /// The child content blocks within this container.
        children: Vec<ContentBlock>,
        /// Layout hint controlling how children are arranged.
        #[serde(skip_serializing_if = "Option::is_none")]
        layout: Option<ContainerLayout>,
    },
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
}
