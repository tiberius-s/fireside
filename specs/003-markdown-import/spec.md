# Feature Specification: Markdown Authoring Frontend (`fireside import`)

**Feature Branch**: `003-markdown-import`

**Created**: 2026-07-12

**Status**: Draft

**Input**: User description: "Markdown authoring frontend: a new `fireside
import <file.md> [output.fireside.json]` CLI verb per ADR-006. Compiles a
Markdown document into protocol-0.1.0 JSON so a presenter can author a deck
in Markdown instead of hand-writing graph JSON, matching how
presenterm/slides/patat/sli.dev all author in Markdown. Optional flat-YAML
frontmatter sets deck metadata. Every H2 heading starts a new node; H3-H6
become heading blocks; paragraphs become text blocks; fenced code becomes
code blocks; flat lists become list blocks (nested lists are a stated v1
limitation); images become image blocks; a bare `---` becomes a divider.
Branching via a fenced `branch` block containing a link list, resolved
against heading-slug node ids; must be the last thing in its section.
Nodes without a branch fence get linear traversal. The importer validates
its output before writing and refuses to write on error, reporting the
offending line/link. Out of scope: containers/columns, speaker notes,
per-node view-mode/transition, nested list items, any Markdown export
direction."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Turn a Markdown talk into a presentable deck (Priority: P1)

A presenter writes their talk as an ordinary Markdown document — headings
for each slide, paragraphs, code samples, bullet lists, images — the same
way they'd write a README or a blog post. They run `fireside import
talk.md` and get a deck file they can immediately present with `fireside
talk.fireside.json`, without ever having hand-written a line of JSON.

**Why this priority**: This is the entire point of the feature — the
authoring gap the strategic plan names as the last stage of the P0
authoring path. Every comparable presentation tool authors this way;
without it, Fireside is the only one that doesn't.

**Independent Test**: Write a Markdown file with a title, three `##`
sections each containing a mix of a paragraph, a code block, and a list,
run `fireside import`, and confirm the resulting file presents correctly
(`fireside <output>`) and validates clean (`fireside validate <output>`),
with each `##` section becoming its own slide in document order.

**Acceptance Scenarios**:

1. **Given** a Markdown file with three `##` headings and prose/code/list
   content under each, **When** the presenter runs `fireside import
   deck.md`, **Then** a deck file is produced with three nodes in document
   order, each containing the corresponding content blocks, and the deck
   presents and validates without errors.
2. **Given** a Markdown file with YAML frontmatter specifying a title and
   author, **When** the presenter imports it, **Then** the produced deck's
   metadata reflects the frontmatter values.
3. **Given** a Markdown file with no frontmatter but an `#` (H1) heading
   before the first `##`, **When** the presenter imports it, **Then** the
   produced deck's title is the H1 text.
4. **Given** two `##` headings that would produce the same slug (e.g.
   identical heading text), **When** the presenter imports the file,
   **Then** the importer gives the second node a distinguishing id instead
   of silently colliding with or overwriting the first.

---

### User Story 2 - Give the audience a choice, from Markdown (Priority: P1)

A presenter wants one slide in their talk to offer a choice — "see the
demo" or "see the architecture" — exactly like Fireside's branching decks
already support, but without hand-writing the branch-point JSON. They write
a fenced `branch` block under that slide's heading, listing each option as
a Markdown link to another `##` heading in the same document.

**Why this priority**: Branching is Fireside's differentiator among
presentation tools; a Markdown authoring path that can't express it would
only support the least interesting subset of decks. This is P1 alongside
linear import because a real deck without branching support isn't a
meaningful MVP for this project specifically.

**Independent Test**: Write a Markdown file where one `##` section contains
a `branch` fence with two links to two other `##` sections, import it, and
confirm the resulting node has a branch-point with two options whose
targets match the linked headings' ids, and that presenting the deck offers
the choice correctly.

**Acceptance Scenarios**:

1. **Given** a node's section contains a `branch` fence with a prompt line
   and two links to other headings, **When** the file is imported, **Then**
   that node's traversal is a branch-point with a matching prompt and two
   options whose targets are the linked headings' ids, in the order listed.
2. **Given** a branch option link includes a trailing key in backticks
   (e.g. `` `d` ``), **When** the file is imported, **Then** that option's
   author-declared key matches the backtick value.
3. **Given** a branch fence link's target does not match any `##` heading's
   id in the document, **When** the presenter runs the import, **Then** the
   import fails with a message naming the bad link and its line number,
   and no output file is written.
4. **Given** content appears after a `branch` fence within the same node's
   section (before the next `##` heading), **When** the presenter runs the
   import, **Then** the import fails with a message pointing at the
   misplaced content, rather than silently dropping or reordering it.

---

### User Story 3 - Know exactly what didn't come through (Priority: P2)

A presenter's Markdown uses a construct the importer doesn't support — a
nested bullet list, for instance. Rather than the importer silently
flattening, dropping, or mis-mapping that content, the presenter gets a
clear, specific message telling them what wasn't imported and where, so
they can either restructure their Markdown or accept the limitation
knowingly.

**Why this priority**: Silent data loss during import is worse than no
import feature at all — a presenter who doesn't notice a dropped bullet
point until they're on stage has been actively harmed by the tool. This is
P2 because it's a robustness/trust requirement layered on top of the P1
import mechanics, not new user-facing capability.

**Independent Test**: Write a Markdown file containing a nested bullet list
under a `##` heading, run the import, and confirm the tool reports a clear
diagnostic naming the nested list and its line, rather than exiting 0 with
a silently mangled deck.

**Acceptance Scenarios**:

1. **Given** a Markdown file containing a nested (multi-level) list,
   **When** the presenter imports it, **Then** the import reports a clear
   diagnostic identifying the nested list's location and does not produce
   a deck that silently drops or flattens the nested items.
2. **Given** a Markdown file whose generated deck would otherwise pass
   every check, **When** the import completes successfully, **Then** the
   presenter is told the deck was written and where, plus a one-line
   summary of anything intentionally not carried over (e.g. "no speaker
   notes in v1 Markdown import — hand-edit the file to add them").

### Edge Cases

- What happens when the Markdown file has zero `##` headings? The import
  fails with a clear message that at least one `##` section is required —
  producing a deck with zero presentable nodes is not a useful outcome.
- What happens when a fenced code block's info string is exactly `branch`
  but appears where a code sample was clearly intended (e.g. a "branch
  prediction" code example)? The importer treats any ` ```branch ` fence as
  a branch-point declaration — this is a stated, documented reserved word;
  authors needing a literal code sample in a language/tag called "branch"
  must pick a different fence tag (this is an acceptable, documented
  constraint, not a bug).
- What happens when the output path already exists? The import refuses to
  overwrite silently and reports that the target already exists, consistent
  with `fireside new`'s existing "please pick another name" behavior for
  the same situation.
- What happens when no output path is given? The output path is derived
  from the input filename (e.g. `talk.md` → `talk.fireside.json`) in the
  same directory as the input.
- What happens when the frontmatter contains a key the importer doesn't
  recognize? Unrecognized frontmatter keys are ignored, not an error —
  consistent with the protocol's own "unknown fields are ignored" rule.
- What happens when a link inside ordinary prose (not inside a `branch`
  fence) points to a `#slug` anchor? It is left as ordinary Markdown text
  in the resulting `text` block's body — only links inside a `branch` fence
  are interpreted as branch options.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide an `import` command that accepts a
  Markdown input file path and an optional output file path.
- **FR-002**: When no output path is given, the system MUST derive one from
  the input file name.
- **FR-003**: The system MUST refuse to overwrite an existing output file
  and MUST report that the target already exists.
- **FR-004**: The system MUST treat every `##` (level-2) heading in the
  input as the start of a new presentable node, in document order.
- **FR-005**: The system MUST derive each node's identifier from its
  heading text using a URL-safe slug, and MUST make identifiers unique
  when two headings would otherwise produce the same slug.
- **FR-006**: The system MUST use each `##` heading's text verbatim as that
  node's display title.
- **FR-007**: The system MUST support optional frontmatter at the start of
  the file for deck-level metadata (title, author, date, description), and
  MUST prefer frontmatter values over any fallback (such as a leading `#`
  heading) when both are present.
- **FR-008**: The system MUST convert level 3-6 headings within a node's
  section into heading content on that same node, not into new nodes.
- **FR-009**: The system MUST convert paragraphs within a node's section
  into text content, preserving inline formatting as written.
- **FR-010**: The system MUST convert fenced code blocks (other than the
  reserved branch-declaration fence) within a node's section into code
  content, using the fence's language tag when present.
- **FR-011**: The system MUST convert single-level bulleted or numbered
  lists within a node's section into list content, preserving list order
  and marking numbered lists as ordered.
- **FR-012**: The system MUST detect nested (multi-level) lists and MUST
  report a clear diagnostic identifying the location rather than silently
  flattening, dropping, or otherwise mis-converting the nested items.
- **FR-013**: The system MUST convert Markdown images within a node's
  section into image content, carrying over the alt text and, when present,
  the title attribute as a caption.
- **FR-014**: The system MUST convert a standalone thematic break (a bare
  `---` outside of frontmatter) into a divider between content.
- **FR-015**: The system MUST support declaring a branch point via a
  specially-tagged fenced block containing a list of links, where each
  link's destination resolves to another node's identifier and each link's
  text becomes that option's label.
- **FR-016**: The system MUST support an optional leading prompt line
  within a branch declaration, carried into the resulting branch point's
  prompt.
- **FR-017**: The system MUST support an optional author-chosen key per
  branch option, distinguishable from the option's label and target.
- **FR-018**: The system MUST reject an import where a branch option's
  link does not resolve to any node in the document, reporting which link
  and where it appears, and MUST NOT produce an output file in that case.
- **FR-019**: The system MUST reject an import where content appears after
  a branch declaration within the same node's section, reporting the
  location, and MUST NOT silently drop or relocate that content.
- **FR-020**: For any node that does not declare a branch point, the system
  MUST set that node's traversal to advance to the next node in document
  order; the final node in the document MUST be left with no forward
  traversal.
- **FR-021**: Before writing any output, the system MUST validate the
  generated deck using the same semantic validation rules the `validate`
  command already applies, and MUST refuse to write output if that
  validation reports any error-level diagnostic.
- **FR-022**: The system MUST reject an import of a Markdown file with no
  `##` headings, reporting that at least one is required.
- **FR-023**: On a successful import, the system MUST report where the
  output was written and MUST summarize anything from the source Markdown
  that v1 import does not carry over (e.g. nested lists already reported
  per FR-012; the general absence of speaker-notes/layout/transition
  authoring is documented guidance, not a per-run diagnostic).

### Key Entities

- **Source document**: the Markdown file being imported, optionally
  beginning with a flat key-value frontmatter block.
- **Node section**: the portion of the source document from one `##`
  heading up to (but not including) the next `##` heading or end of file;
  maps one-to-one to a produced node.
- **Branch declaration**: a specially-tagged fenced block within a node
  section, containing an optional prompt and an ordered list of links, each
  resolving to a target node and carrying a label and optional key.
- **Import diagnostic**: a message identifying a specific problem in the
  source document (an unresolved branch link, content after a branch
  declaration, a nested list, or an empty document) together with enough
  location information for the presenter to find and fix it.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A presenter can go from a plain-text Markdown talk outline to
  a presentable, validated deck using a single command, with zero manual
  JSON editing required for a talk that only needs headings, prose, code,
  lists, images, and dividers.
- **SC-002**: A presenter can express a branching choice entirely in
  Markdown and have it present identically to a hand-written branch-point
  deck.
- **SC-003**: Every import that fails does so with a message specific
  enough that the presenter can locate and fix the problem without needing
  to inspect the generated JSON or consult documentation.
- **SC-004**: No import ever silently drops or corrupts source content —
  every case v1 cannot represent is either converted correctly or reported,
  never silently mishandled.

## Assumptions

- "Flat-YAML frontmatter" means simple `key: value` lines only; the
  importer does not need a general YAML parser, since deck metadata has no
  nested structure.
- The branch-declaration fence's reserved tag and exact inline syntax
  (prompt line, link list, optional backticked key) follow ADR-006
  (`.claude/adrs/adr-006-markdown-import.md`) exactly; this spec does not
  restate the literal syntax, only the behavior it produces.
- Containers/columns, speaker notes, and per-node view-mode/transition are
  out of scope for v1 import, per ADR-006 — a presenter needing those edits
  the generated JSON by hand (or, for heading/text content, via the
  quick-edit modal from `specs/002-quick-edit-modal/`).
- There is no Markdown-export direction (JSON back to Markdown) in this
  feature or planned as part of it.
- Import is a one-shot batch operation, not a live/watched process (unlike
  `validate --watch` or `present`'s live reload) — rerunning `fireside
  import` is how a presenter picks up source changes.
