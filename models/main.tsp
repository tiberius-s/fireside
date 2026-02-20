import "@typespec/json-schema";
import "@typespec/versioning";

using TypeSpec.JsonSchema;
using Versioning;

/**
 * # Fireside Protocol — Domain Model
 *
 * A portable format for branching presentations and lessons.
 *
 * Fireside defines a directed graph where vertices (Nodes) contain
 * structured content (ContentBlocks) and edges define traversal
 * paths. Four fundamental operations govern traversal: Next (linear
 * advance), Choose (branch selection), Goto (direct jump), and Back
 * (history retrace).
 *
 * This domain model is implementation-independent. Any runtime with
 * JSON parsing can build a conforming Fireside engine.
 *
 * ## Protocol Version
 * 0.1.0
 *
 * ## Schema Dialect
 * JSON Schema 2020-12
 */
@jsonSchema
@versioned(Versions)
namespace Fireside;

enum Versions {
  v0_1_0: "0.1.0",
}

// ─── Scalar Types ────────────────────────────────────────────────────────────

/**
 * A unique string identifier for a Node within a Graph.
 * Used for graph edges (traversal targets) and Goto operations.
 *
 * Node IDs MUST be unique within a Graph. They SHOULD be
 * kebab-case strings (e.g., "intro", "chapter-2", "boss-fight").
 */
@minLength(1)
scalar NodeId extends string;

// ─── Enums ───────────────────────────────────────────────────────────────────

/**
 * Spatial arrangement strategy for content blocks within a node.
 *
 * The engine resolves the effective layout by checking:
 * 1. Node-level layout (explicit)
 * 2. Graph-level defaults
 * 3. Built-in default ("default")
 */
enum Layout {
  /** Standard top-to-bottom stacking with configurable padding. */
  default: "default",

  /** Content centered both vertically and horizontally. */
  center: "center",

  /** Two-column split layout. */
  `split-horizontal`: "split-horizontal",

  /** Two-row split layout. */
  `split-vertical`: "split-vertical",

  /** Full terminal area, no chrome. */
  fullscreen: "fullscreen",

  /** Content anchored to the left with right margin. */
  `align-left`: "align-left",

  /** Content anchored to the right with left margin. */
  `align-right`: "align-right",

  /** Layout optimized for code-centric content with output panes. */
  `focus-code`: "focus-code",

  /** Agenda or section overview layout with title and item rail. */
  agenda: "agenda",

  /** Side-by-side comparison layout for before/after content. */
  compare: "compare",

  /** Image-leading layout with supporting text and metadata. */
  `image-left`: "image-left",

  /** Text-leading layout with trailing image or diagram panel. */
  `image-right`: "image-right",
}

/**
 * Animation effect applied when transitioning between nodes.
 *
 * Transitions control the visual effect when moving from one node
 * to another. Engines MAY implement a subset; unsupported transitions
 * SHOULD fall back to "none".
 */
enum Transition {
  /** No animation, instant switch. */
  none: "none",

  /** Crossfade between nodes. */
  fade: "fade",

  /** New node slides in from the right. */
  `slide-left`: "slide-left",

  /** New node slides in from the left. */
  `slide-right`: "slide-right",

  /** New node slides in from the bottom. */
  `slide-up`: "slide-up",

  /** New node slides in from the top. */
  `slide-down`: "slide-down",

  /** Old node dissolves into the new one. */
  dissolve: "dissolve",

  /** Matrix-style character rain effect. */
  matrix: "matrix",
}

// ─── Content Blocks ──────────────────────────────────────────────────────────

/**
 * A ContentBlock is an atomic content element within a Node.
 *
 * Content blocks use a tagged discriminated union keyed by the `kind` field.
 * Each variant represents a distinct type of content.
 *
 * ## Core Blocks (7)
 * Conforming engines MUST support all 7 core block kinds:
 * heading, text, code, list, image, divider, container.
 *
 * ## Extension Blocks
 * Extensions use the explicit `extension` block kind with a required
 * `type` identifier (e.g., `acme.table`, `org.video`).
 * Engines MUST render unsupported extensions using their `fallback` property.
 */
union ContentBlock {
  HeadingBlock,
  TextBlock,
  CodeBlock,
  ListBlock,
  ImageBlock,
  DividerBlock,
  ContainerBlock,
  ExtensionBlock,
}

/** A heading with a level (1–6) and text content. */
model HeadingBlock {
  kind: "heading";

  /** Heading level from 1 (largest) to 6 (smallest). */
  @minValue(1)
  @maxValue(6)
  level: int32;

  /** The heading text content. */
  text: string;
}

/** A block of prose text, optionally with inline Markdown formatting. */
model TextBlock {
  kind: "text";

  /** The text content. May contain inline Markdown formatting. */
  body: string;
}

/** A fenced code block with language annotation and optional highlighting. */
model CodeBlock {
  kind: "code";

  /** Programming language identifier for syntax highlighting. */
  language?: string;

  /** The source code content. */
  source: string;

  /** Line numbers to visually emphasize. */
  `highlight-lines`?: int32[];

  /** Whether to display line numbers. */
  `show-line-numbers`?: boolean;
}

/** An ordered or unordered list of items. */
model ListBlock {
  kind: "list";

  /** Whether the list is ordered (numbered) or unordered (bulleted). */
  ordered?: boolean;

  /** The list items as strings. */
  @minItems(1)
  items: string[];
}

/** A visual element with source URI and accessibility metadata. */
model ImageBlock {
  kind: "image";

  /** URI or file path to the image source. */
  src: string;

  /** Alternative text for accessibility. SHOULD be provided for all images. */
  alt?: string;

  /** Optional caption displayed below the image. */
  caption?: string;

  /** Desired display width (columns or percentage). */
  width?: int32;

  /** Desired display height (rows). */
  height?: int32;
}

/** A horizontal rule separating content sections. */
model DividerBlock {
  kind: "divider";

  /** Optional style variant for the divider. */
  style?: string;
}

/**
 * A generic container for nested content blocks.
 *
 * Replaces the previous `fragment` and `columns` block types with a single
 * unified container. The `layout` hint controls arrangement:
 *
 * - `"stack"` (default) — vertical stacking, progressive reveal
 * - `"columns"` — side-by-side column arrangement
 * - `"rows"` — horizontal row arrangement
 *
 * Engines MAY support additional layout hints through extension-defined values.
 */
model ContainerBlock {
  kind: "container";

  /** The child content blocks within this container. */
  @minItems(1)
  children: ContentBlock[];

  /** Layout hint controlling how children are arranged. */
  layout?: string;
}

/**
 * An extension content block using a typed extension identifier.
 *
 * Extension blocks allow protocol consumers to define custom content types
 * (e.g., `acme.table`, `org.video`, `edu.poll`) without modifying the core spec.
 *
 * ## Conformance
 * - `kind` MUST be `"extension"`
 * - `type` MUST uniquely identify the extension payload contract
 * - Engines MUST render unsupported extensions using the `fallback` property
 * - If no `fallback` is provided, engines SHOULD display a placeholder
 *
 * ## Extension Properties
 * Extension blocks MAY include arbitrary additional properties using the
 * spread record pattern. These properties are opaque to the core protocol.
 */
model ExtensionBlock {
  /** Extension discriminator for all non-core content. */
  kind: "extension";

  /** Extension type identifier (e.g., "acme.table", "org.video"). */
  type: string;

  /** Fallback content for engines that do not support this extension. */
  fallback?: ContentBlock;

  /** Optional publisher/namespace for discovery and registry matching. */
  publisher?: string;

  /** Optional extension schema or contract version. */
  `schema-version`?: string;

  /** Arbitrary extension-specific payload properties. */
  ...Record<unknown>;
}

// ─── Traversal ───────────────────────────────────────────────────────────────

/**
 * A BranchOption is a single choice available at a BranchPoint.
 * It has a display label, an optional hotkey, and a target node.
 */
model BranchOption {
  /** Display label for this option (e.g., "Fight the dragon"). */
  label: string;

  /** Keyboard shortcut to select this option (e.g., "a", "1"). */
  key?: string;

  /** The NodeId this option leads to. */
  @minLength(1)
  target: NodeId;

  /** Optional description providing more detail about this choice. */
  description?: string;
}

/**
 * A BranchPoint is a decision point at a node where the presenter
 * must choose between multiple options.
 *
 * When the engine encounters a BranchPoint, it MUST display the prompt
 * and options, then wait for the presenter to Choose.
 */
model BranchPoint {
  /** Optional identifier for this branch point. */
  id?: string;

  /** The prompt displayed to the presenter (e.g., "What do you do?"). */
  prompt?: string;

  /** The available options. MUST contain at least one option. */
  @minItems(1)
  options: BranchOption[];
}

/**
 * Traversal defines how a presenter moves away from the current node.
 * It contains optional overrides for traversal operations.
 *
 * If no traversal is specified, the default behavior is Next (advance
 * to the next node in array order).
 */
model Traversal {
  /**
   * Override the Next target. Instead of advancing to nodes[i+1],
   * Next will navigate to this node ID.
   */
  next?: NodeId;

  /**
   * Define a return point after completing a branch.
   * When the branch path reaches its end, the engine returns here.
   */
  after?: NodeId;

  /**
   * Present a BranchPoint with multiple options.
   * The presenter must Choose before continuing.
   */
  `branch-point`?: BranchPoint;
}

// ─── Node ────────────────────────────────────────────────────────────────────

/**
 * A Node is a vertex in the graph — a discrete unit of content
 * that a presenter visits. Each node has an optional identity, spatial
 * layout, transition effect, traversal rules, and zero or more
 * content blocks.
 *
 * ## Resolution Order
 * Properties use a cascading resolution:
 * 1. Node-level value (explicit)
 * 2. Graph-level defaults
 * 3. Built-in default
 */
model Node {
  /** Unique identifier for this node. Required for traversal targets. */
  @minLength(1)
  id?: NodeId;

  /** Human-readable node title for navigation and indexing UIs. */
  title?: string;

  /** Optional categorization tags for this node. */
  tags?: string[];

  /** Optional duration hint (ISO 8601 recommended). */
  duration?: string;

  /** Spatial arrangement strategy for this node's content blocks. */
  layout?: Layout;

  /** Animation effect when entering this node. */
  transition?: Transition;

  /**
   * Hidden notes visible only to the presenter, not the audience.
   * Presenter notes, GM tips, teaching hints.
   */
  `speaker-notes`?: string;

  /** Traversal overrides for leaving this node. */
  traversal?: Traversal;

  /** The content blocks displayed at this node. */
  content: ContentBlock[];
}

// ─── Graph ───────────────────────────────────────────────────────────────────

/**
 * Default values applied to all nodes in a graph unless
 * overridden at the node level.
 */
model NodeDefaults {
  /** Default layout for all nodes. */
  layout?: Layout;

  /** Default transition for all nodes. */
  transition?: Transition;
}

/**
 * Declares an extension type used by this graph and whether it is required.
 */
model ExtensionDeclaration {
  /** Extension type identifier (e.g., "acme.table"). */
  type: string;

  /** Whether engines MUST support this extension for correct rendering. */
  required?: boolean;
}

/**
 * A Graph is the top-level document — a self-contained directed graph
 * of nodes forming traversable content with metadata.
 *
 * ## Structure
 * A Graph is a JSON document with two sections:
 * 1. **Metadata** — Title, author, theme, and descriptive fields
 * 2. **Nodes** — An ordered array of node objects
 *
 * ## Graph Semantics
 * - The node array defines an implicit linear sequence (Next edges)
 * - Traversal overrides on each node define explicit graph edges
 * - The first node (index 0) is the graph's entry point
 * - All nodes SHOULD be reachable from the entry point
 *
 * ## Validation
 * - All Node IDs MUST be unique within the graph
 * - All traversal targets MUST reference valid Node IDs
 * - BranchPoints MUST contain at least one option
 */
model Graph {
  /** JSON Schema URI for self-describing documents. */
  $schema?: string;

  /** Protocol version for this document (machine-readable). */
  `fireside-version`?: Versions;

  /** The graph's display name. */
  title?: string;

  /** The graph creator's name. */
  author?: string;

  /** Creation or presentation date (ISO 8601 recommended). */
  date?: string;

  /** A brief summary of the graph's purpose. */
  description?: string;

  /** Semantic version of this graph document. */
  version?: string;

  /** Categorization tags for organization and filtering. */
  tags?: string[];

  /** Default theme name for the engine to use. */
  theme?: string;

  /** Preferred monospace font family. */
  font?: string;

  /** Default values applied to all nodes unless overridden. */
  defaults?: NodeDefaults;

  /** Declared extension capabilities used by this graph. */
  extensions?: ExtensionDeclaration[];

  /** The ordered array of nodes forming the graph. MUST contain at least one node. */
  @minItems(1)
  nodes: Node[];
}

// ─── Operations (Documentary) ────────────────────────────────────────────────

/**
 * The four fundamental traversal operations.
 *
 * These operations are defined for specification purposes — they describe
 * the behavioral contract that conforming engines MUST implement.
 * They do not produce schema output.
 */
interface TraversalOps {
  /**
   * Advance to the next node in sequence.
   *
   * Resolution order:
   * 1. If current node has `traversal.next`, go to that target
   * 2. If current node has `traversal.after` (branch return), go there
   * 3. Otherwise, advance to `nodes[currentIndex + 1]`
   * 4. If at the last node, the traversal is complete
   */
  next(): void;

  /**
   * Select a branch option at a BranchPoint.
   *
   * 1. Push the current node onto the history stack
   * 2. Navigate to the selected option's target node
   */
  choose(option: BranchOption): void;

  /**
   * Jump directly to a node by its ID.
   *
   * 1. Push the current node onto the history stack
   * 2. Navigate to the node matching the given ID
   * 3. If the ID does not exist, the operation is a no-op
   */
  goto(target: NodeId): void;

  /**
   * Return to the previous node in the history stack.
   *
   * 1. Pop the top entry from the history stack
   * 2. Navigate to that node
   * 3. If the stack is empty, the operation is a no-op
   */
  back(): void;
}
