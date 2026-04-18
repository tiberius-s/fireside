import "@typespec/json-schema";
import "@typespec/versioning";
import "./dist/src/index.js";

using TypeSpec.JsonSchema;
using Versioning;

/**
 * # Fireside Protocol — Domain Model
 *
 * A portable format for graph-structured presentations.
 *
 * A Fireside document is a directed graph where nodes contain structured
 * content and edges define traversal paths. Four operations govern
 * traversal: next, choose, goto, and back.
 *
 * This model is implementation-independent. Any runtime with JSON parsing
 * and a cursor plus history stack can build a conforming engine.
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

/** Supported protocol versions. */
enum Versions {
  v0_1_0: "0.1.0",
}

// ─── Scalar Types ────────────────────────────────────────────────────────────

/**
 * A unique string identifier for a Node within a Graph.
 * Used for graph edges (traversal targets) and goto operations.
 *
 * Node IDs MUST be unique within a Graph. They SHOULD be
 * kebab-case strings (e.g., "intro", "chapter-2", "code-demo").
 */
@minLength(1)
scalar NodeId extends string;

// ─── Enums ───────────────────────────────────────────────────────────────────

/**
 * Presentation frame mode for a node.
 *
 * View mode controls how much screen real estate content gets. It does
 * not control content arrangement — that is the container block's job.
 *
 * Engines SHOULD allow the presenter to toggle view modes at any time.
 * The node-level value is the initial suggestion, not a constraint.
 *
 * Resolution order:
 * 1. Node-level view-mode (explicit)
 * 2. Graph-level defaults
 * 3. Built-in default ("default")
 */
enum ViewMode {
  /** Standard presentation frame. */
  default: "default",

  /** Maximum content area, minimal frame. */
  fullscreen: "fullscreen",
}

/**
 * Pacing intent when transitioning between nodes.
 *
 * Transitions express pacing intent, not animation style. Engines
 * choose how to render each value. Unsupported transitions SHOULD
 * fall back to "none".
 */
enum Transition {
  /** Instant transition. No animation. */
  none: "none",

  /** Smooth transition. Engine chooses the visual effect. */
  fade: "fade",
}

// ─── Content Blocks ──────────────────────────────────────────────────────────

/**
 * A ContentBlock is an atomic content element within a Node.
 *
 * Content blocks use a tagged discriminated union keyed by the `kind` field.
 * Each variant represents a distinct type of presentable content.
 *
 * Conforming engines MUST support all 7 block kinds.
 *
 * Block order within a node's `content` array is significant. Blocks
 * MUST be rendered in array order.
 */
union ContentBlock {
  HeadingBlock,
  TextBlock,
  CodeBlock,
  ListBlock,
  ImageBlock,
  DividerBlock,
  ContainerBlock,
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
}

/**
 * A container for nested content blocks with layout control.
 *
 * Container layout hints control how children are arranged:
 *
 * - `"stack"` (default) — vertical stacking, top-aligned
 * - `"columns"` — side-by-side arrangement, left to right in array order
 * - `"center"` — centered vertically and horizontally
 *
 * There is no protocol limit on nesting depth. Engines MAY impose
 * practical limits.
 */
model ContainerBlock {
  kind: "container";

  /** The child content blocks within this container. */
  @minItems(1)
  children: ContentBlock[];

  /** Layout hint controlling how children are arranged. */
  layout?: string;
}

// ─── Traversal ───────────────────────────────────────────────────────────────

/**
 * A BranchOption is a single choice available at a BranchPoint.
 * It has a display label, an optional hotkey, and a target node.
 */
model BranchOption {
  /** Display label for this option. */
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
 * A BranchPoint is a decision point where the presenter must choose
 * between multiple options.
 *
 * When an engine encounters a BranchPoint, it MUST display the prompt
 * and options, then wait for the presenter to choose. The `next()`
 * operation is not valid at a node with a BranchPoint.
 */
model BranchPoint {
  /** The prompt displayed to the presenter. */
  prompt?: string;

  /** The available options. MUST contain at least one option. */
  @minItems(1)
  options: BranchOption[];
}

/**
 * Traversal defines how a presenter moves away from the current node.
 *
 * This is the object form, used when branching is needed. For simple
 * next-node edges, use the string shorthand on Node.traversal instead.
 *
 * A Traversal MUST NOT have both `next` and `branch-point`. Validators
 * MUST reject this as contradictory.
 */
model Traversal {
  /**
   * Override the next target. Navigate to this node on next().
   * MUST NOT coexist with branch-point.
   */
  next?: NodeId;

  /**
   * Present a BranchPoint with multiple options.
   * The presenter must choose before continuing. next() is blocked.
   * MUST NOT coexist with next.
   */
  `branch-point`?: BranchPoint;
}

// ─── Node ────────────────────────────────────────────────────────────────────

/**
 * A Node is a vertex in the graph — a discrete unit of content
 * that a presenter visits.
 *
 * ## Traversal
 *
 * The `traversal` field accepts either:
 * - A NodeId string (shorthand for a simple next edge)
 * - A Traversal object (for branching or explicit next)
 * - Absent (terminal node — dead end)
 *
 * There is no implicit sequential fallback. Every edge in the graph
 * is explicit. Array order is for document organization only.
 *
 * ## Resolution Order
 * View mode and transition use cascading resolution:
 * 1. Node-level value (explicit)
 * 2. Graph-level defaults
 * 3. Built-in default
 */
model Node {
  /** Unique identifier for this node. Required. */
  @minLength(1)
  id: NodeId;

  /** Human-readable node title for navigation UIs. */
  title?: string;

  /** Presentation frame mode for this node. */
  `view-mode`?: ViewMode;

  /** Pacing intent when entering this node. */
  transition?: Transition;

  /**
   * Notes visible only to the presenter, not the audience.
   */
  `speaker-notes`?: string;

  /**
   * How the presenter leaves this node.
   *
   * String: shorthand for a next edge (equivalent to `{ "next": "<id>" }`)
   * Object: full Traversal with next or branch-point
   * Absent: terminal node (dead end)
   */
  traversal?: NodeId | Traversal;

  /** The content blocks displayed at this node. */
  content: ContentBlock[];
}

// ─── Graph ───────────────────────────────────────────────────────────────────

/**
 * Default values applied to all nodes unless overridden at the node level.
 */
model NodeDefaults {
  /** Default view mode for all nodes. */
  `view-mode`?: ViewMode;

  /** Default transition for all nodes. */
  transition?: Transition;
}

/**
 * A Graph is the top-level document — a self-contained directed graph
 * of content nodes.
 *
 * ## Structure
 * 1. **Metadata** — Title, author, and descriptive fields
 * 2. **Defaults** — Default view mode and transition for all nodes
 * 3. **Nodes** — An ordered array of node objects
 *
 * ## Entry Point
 * The first node (index 0) is the graph's entry point.
 *
 * ## Validation
 * - All Node IDs MUST be unique within the graph
 * - All traversal targets MUST reference valid Node IDs
 * - BranchPoints MUST contain at least one option
 * - Nodes SHOULD be reachable from the entry point
 */
model Graph {
  /** Protocol version for this document. */
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

  /** Default values applied to all nodes unless overridden. */
  defaults?: NodeDefaults;

  /** The ordered array of nodes forming the graph. */
  @minItems(1)
  nodes: Node[];
}

// ─── Operations (Documentary) ────────────────────────────────────────────────

/**
 * The four traversal operations.
 *
 * These define the behavioral contract that conforming engines MUST
 * implement. They do not produce schema output.
 *
 * All operations except back() push the current node ID onto the
 * history stack before navigating. History is a stack of NodeIds,
 * not array indices.
 */
interface TraversalOps {
  /**
   * Advance to the next node.
   *
   * 1. If current node has a branch-point → BLOCKED, wait for choose()
   * 2. Push current node ID onto history
   * 3. If traversal is a string, navigate to that target
   * 4. If traversal.next exists, navigate to that target
   * 5. If no traversal, no-op (terminal node)
   */
  next(): void;

  /**
   * Select a branch option at a BranchPoint.
   *
   * 1. Push current node ID onto history
   * 2. Navigate to the selected option's target node
   */
  choose(option: BranchOption): void;

  /**
   * Jump directly to a node by its ID.
   *
   * 1. If the ID does not exist, no-op
   * 2. Push current node ID onto history
   * 3. Navigate to the target node
   */
  goto(target: NodeId): void;

  /**
   * Return to the previous node in the history stack.
   *
   * 1. Pop the top entry from the history stack
   * 2. Navigate to that node
   * 3. If the stack is empty, no-op
   * 4. Does NOT push to history
   */
  back(): void;
}
