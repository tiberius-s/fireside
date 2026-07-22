# Phase 1 Data Model: Authoring Editor (`fireside edit`)

This feature adds no protocol fields — every entity below either already
exists in `fireside-core::model` (linked, not redefined) or is new
in-memory/on-disk state introduced by this feature alone.

## Existing wire-format entities (unchanged, reused)

Reference: `crates/fireside-core/src/model/mod.rs`. Reproduced here only to
the depth the editor's forms/pickers need to map onto; the file itself is
canonical.

- **`Graph`** (`mod.rs:25`) — `nodes: Vec<Node>` (order-significant; first
  node is the entry point), plus optional `title`/`author`/`date`/
  `description`/`version`/`defaults`. This is "the Deck" in spec vocabulary.
- **`Node`** (`mod.rs:113`) — `id` (kebab-case, unique), `title`,
  `speaker_notes`, `content: Vec<ContentBlock>`, `traversal: Option<TraversalSpec>`.
  This is "the Slide." `traversal` absent ⇒ ending; `TraversalSpec::Target`
  or `Rules { next: Some(_) }` ⇒ single next; `Rules { branch_point: Some(_) }`
  ⇒ branch point. `next` and `branch_point` are mutually exclusive by schema
  (validation rejects both set) — the editor's construction must make this
  pairing unrepresentable, not merely validated (spec FR-023).
- **`BranchPoint`** (`mod.rs:236`) — `prompt: Option<String>`,
  `options: Vec<BranchOption>` (≥1, schema-required).
- **`BranchOption`** (`mod.rs:246`) — `label`, `key: Option<String>`
  (single presenter shortcut; the reserved-branch-key validator rule
  already exists and applies unchanged), `target: NodeId`, `description`.
  This is "a Branch answer" in spec vocabulary.
- **`ContentBlock`** (`mod.rs:285`) — 8 variants, each carrying its own
  `reveal: Option<u32>`: `Heading{level,text}`, `Text{body}`,
  `Code{language,source,highlight_lines,show_line_numbers}`,
  `List{ordered,items}`, `Image{src,alt,caption,width,height}`,
  `Divider{}`, `Container{children,layout}`, `AsciiArt{art,alt}`. This is
  "the Block" in spec vocabulary; the editor's plain-language names (FR-006)
  are: heading, text, code, list, picture, divider, columns/box/stack
  (`Container`, `layout` picks `ContainerLayout::Columns`/`Center`/`Stack`),
  text art (`AsciiArt`).
- **`Node::reveal_levels()`** (`mod.rs:196`) — the existing pure
  computation of distinct positive reveal steps used anywhere in a node's
  content (recursing into `Container` children). The editor's `[ Reveal ▾ ]`
  chip and `◇n` badges are a UI over this existing function plus a new
  authoring transform that renumbers/compacts values, never a parallel
  computation.

## New: `engine::authoring` entities (in-memory, pure, `fireside-engine`)

- **`Op`** — the closed set of authoring operations, each a pure
  `(Graph, Op) -> Result<Graph, AuthoringError>` transform (ADR-018 records
  the full enum; summarized here by spec story):
  - Slide ops (US3): `AddSlide`, `DeleteSlide`, `DuplicateSlide`,
    `RetitleSlide` (rewrites the id *and* every reference to it — `next`
    edges, choice targets, the start id — in one atomic op, per the slug
    algorithm below), `ReorderSlide` (linear-run only; cross-branch attempts
    return `AuthoringError::CrossesBranchBoundary`, never partially apply),
    `SetNext`, `ClearNext` (mark ending), `TurnIntoChoice`,
    `TurnBackIntoSlide`, `AddAnswer`/`RemoveAnswer`/`RetargetAnswer`.
  - Block ops (US1/US2): `EditBlock`, `AddBlock`, `DeleteBlock`,
    `MoveBlock` (reorder within one slide only — cross-slide move is out of
    scope per spec Assumptions), `SetRevealStep` (renumbers/compacts the
    node's distinct reveal values so displayed steps stay 1..n with no
    gaps, per `Node::reveal_levels`'s existing ordinal semantics).
- **`AuthoringError`** (`thiserror`, new type alongside `EngineError`,
  precedented by `fireside-tui`'s `TuiError`/`WriteBackError` pair) —
  variants for every op-level invariant violation: `DanglingTarget`,
  `DuplicateId`, `CrossesBranchBoundary`, `ReservedBranchKey`,
  `NotABranchPoint`, `EmptyLabel`, and so on. Every variant maps to one of
  spec FR-023's four unrepresentable-by-construction states, or to a
  plain-language toast the UI shows verbatim-adjacent (never the enum name
  itself — FR-024).
- **Id/slug algorithm** — deterministic, used by `AddSlide`/`RetitleSlide`:
  lowercase the title; map every run of non-alphanumeric characters to a
  single `-`; trim leading/trailing `-`; empty result falls back to
  `"slide"`; dedupe against every existing id in the graph with `-2`,
  `-3`, … suffixes. A proptest asserts no sequence of retitles can ever
  leave a dangling reference (every `next`/`target`/start-id reference to
  the renamed id is rewritten atomically in the same op).
- **Outline ordering function** — pure `Graph -> Vec<OutlineRow>`
  (`OutlineRow { node_id, display_number, reachable: bool }`): depth-first
  from the graph's entry node, following `next` before branch options in
  declared order, each node appearing once at its first visit (cycles
  terminate for free); nodes never visited this way are appended after a
  divider, in stable id (declaration) order. `display_number` is
  1-based position in this sequence — a display coordinate only, recomputed
  after every structural op, never persisted or used as an identifier.

## New: `EditorApp` state (in-memory, `fireside-tui`, TEA)

- **`EditorApp`** — the sole struct `editor::update()` mutates (Constitution
  IV, generalized). Owns:
  - `working_graph: Graph` — the in-progress, possibly-unsaved graph;
    `Op`s apply here via `engine::authoring`.
  - `saved_graph: Graph` (or an equality/hash marker against it) — for the
    dirty-state indicator (spec FR-018).
  - `selection: Selection` — `None | Slide(NodeId) | Block(NodeId, BlockPath) | OutlineRow(usize)`.
  - `drag: DragState` — `Idle | Lifting { origin } | Over { slot }`,
    covering both block-reorder and outline slide-reorder drags (spec
    FR-009, US3's outline drag).
  - `open_form: Option<FormState>` — the currently open block/slide/answer
    form, one at a time (spec principle: no invisible modes).
  - `history: Vec<HistorySnapshot>` — full `Graph` clones (not op
    inversion — decks in scope, ≤500 slides, clone cheaply; see
    `research.md` performance note), capped at 100 (spec FR-016), each
    paired with the `Selection` at that point so undo restores view
    context too; a parallel `redo` stack, cleared on any new op.
  - `terminal_size: (u16, u16)` — set at startup, updated on every resize
    event; `hit()` and layout both read this, never the renderer (keeps
    hit-testing pure per `research.md` §1).
  - `status: Vec<Diagnostic>` — the plain-language issues shown in the
    status banner (spec FR-026), sourced from the existing
    `fireside-engine::validation::rules()`.
  - `dirty_since_draft: bool` / `last_draft_write: Instant` — drive the
    periodic draft-sidecar write (spec FR-020).
- **`FormState`** — one variant per block kind (spec's 8 forms) plus slide
  metadata / branch-answer / wiring-picker forms; each wraps the
  minimal state that kind's form needs (heading/text forms wrap a promoted
  `EditableField`, per `research.md` §2).

## New: Draft sidecar (on-disk, `fireside-cli`)

- **Path**: `$XDG_STATE_HOME/fireside/drafts/<fnv1a64-hex-of-canonicalized-deck-path>.json`
  — same directory family, same hash function, and the same
  canonicalized-absolute-path keying `fireside-cli/src/session.rs` and
  `resume.rs` already use (`research.md` §3).
- **Schema**: `{ "schema": 1, "deck_path": <string>, "saved_at": <epoch seconds>, "deck": <full Graph JSON> }`.
- **Lifecycle**: written periodically while `dirty_since_draft` (spec
  FR-020) and on every structural op; deleted on successful save and on
  clean quit with no unsaved changes; read once at open time — if present
  and its `deck` differs from the on-disk file's parsed content, the editor
  prompts `[ Restore draft ] [ Open saved file ]` showing both timestamps
  (draft's `saved_at`, file's mtime). Writes are atomic (temp file +
  rename), matching every other state file in the project.

## Relationships

```text
Graph 1───* Node (order = presentation order for the entry point only;
             actual reachability is the outline algorithm's job)
Node   1───* ContentBlock (order = render order; Container nests further
             ContentBlock children)
Node   0..1─ BranchPoint 1───* BranchOption ──> Node.id (target)
Node   0..1─ TraversalSpec::next ──> Node.id

EditorApp ──working_graph──> Graph (mutated only via engine::authoring::Op)
EditorApp ──history────────> Vec<Graph clone> (undo/redo)
EditorApp ──selection───────> Node.id / (Node.id, BlockPath)

Draft sidecar ──deck_path──> canonicalized path of the Graph's source file
                              (independent of, not a field on, Graph)
```

## Validation / invariant summary (construction vs. detection)

| Invariant | Enforcement |
| --- | --- |
| No dangling `next`/target/start-id reference | Unrepresentable: every op that removes/renames a slide id rewrites every reference atomically (proptest-covered) |
| No duplicate slide id | Unrepresentable: `AddSlide`/`RetitleSlide` always dedupe via the slug algorithm |
| `next` and `branch_point` never both set | Unrepresentable: `TurnIntoChoice`/`TurnBackIntoSlide` are the only ops that change this axis, and each clears the other |
| Reveal steps never gapped | Unrepresentable: `SetRevealStep` renumbers to consecutive values as part of the op, consistent with `Node::reveal_levels()`'s existing ordinal semantics |
| Reserved branch key reused | Rejected at the op boundary (`AuthoringError::ReservedBranchKey`), surfaced in the answer form inline, per the existing `reserved-branch-key` validator rule |
| Anything else `fireside-engine::validation::rules()` flags | Detected, not prevented: shown in the status banner (spec FR-026), clickable to locate |
