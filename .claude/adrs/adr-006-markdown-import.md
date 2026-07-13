---
title: 'ADR-006: Markdown authoring frontend (`fireside import`)'
status: 'accepted'
date: '2026-07-12'
deciders: ['@tiberius']
---

# ADR-006: Markdown authoring frontend (`fireside import`)

## Status

Accepted.

## Context

The strategic plan (`.claude/plans/2026-07-12-strategic-improvement-plan.md`,
P0 Stage D) names Markdown authoring as the last stage of the authoring
gap: "A `fireside import deck.md` compiler... attacks the authoring gap
without touching the wire format and matches how every competitor authors."
Every comparable tool — presenterm, slides, patat, sli.dev — authors decks
in Markdown; Fireside presenters still hand-write graph JSON even after
Stage C's quick-edit modal (ADR-005), which only edits existing text, never
creates structure. The plan flagged one open question before committing to
this stage: "the branch-point syntax is the hard part... needs a throwaway
prototype and a taste test," because nothing else does graph-structured,
branching presentations — there is no prior art to copy.

That taste test happened directly with the user: three concrete syntax
candidates were mocked up (a link list inside a fenced ` ```branch ` block;
a bare link list marked by an HTML comment; a YAML-in-a-fence block mapping
1:1 onto the protocol's `BranchPoint`/`BranchOption` fields) and the user
picked the fenced link-list form. This ADR records that choice and the full
Markdown-to-`Graph` mapping built around it.

A second, separate question this ADR must resolve is the crate boundary
table (`.specify/memory/constitution.md` §III): `fireside-cli`'s permitted
dependencies today are `clap`, `anyhow`, `serde_json`, and the workspace
crates — no Markdown parser. Hand-rolling a Markdown parser (headings,
paragraphs, fenced code, lists, images, thematic breaks, inline links) to
avoid a new dependency was considered and rejected: Markdown's edge cases
(nested emphasis, lazy paragraph continuation, fence info-string parsing)
are exactly the kind of thing a hand-rolled parser gets subtly wrong, and
this is authoring tooling for non-technical presenters, where a parser bug
means a confusing, hard-to-diagnose import failure. `pulldown-cmark` 0.13
(the parser used by `mdBook` and `rustdoc` itself) was verified to build
clean under the workspace's MSRV 1.88, and its own dependency footprint is
light (`memchr`, `bitflags`, `unicase`, `pulldown-cmark-escape`,
`unicode-width` — the last already a `fireside-tui` dependency). YAML
frontmatter parsing does not need a full YAML library: deck-level metadata
(title, author, date, description) is flat `key: value` pairs, so a
hand-rolled line parser for the frontmatter block only is sufficient and
keeps the new-dependency surface to one crate.

## Decision

We add a new `fireside-cli` verb, `fireside import <file.md> [output.fireside.json]`,
that compiles a Markdown document into protocol-0.1.0 JSON. This requires a
**deliberate amendment to the crate boundary table**: `fireside-cli` gains
`pulldown-cmark` as a permitted dependency, flagged here per the
constitution's rule that any boundary-table violation must be called out
explicitly with the alternative considered (hand-rolling, rejected above).
No other crate's boundary changes — `fireside-core`, `fireside-engine`, and
`fireside-tui` are untouched by this feature.

The Markdown-to-`Graph` mapping:

- **Frontmatter** (optional, `---`-delimited at the very top of the file,
  flat `key: value` lines only — no nested YAML): `title`, `author`, `date`,
  `description`, `fireside-version` (defaults to the current protocol
  version if absent) become the `Graph`'s top-level metadata fields.
- **Nodes**: every `##` (H2) heading starts a new node. The node `id` is a
  GitHub-style slug of the heading text (lowercase, spaces to hyphens,
  punctuation stripped), deduplicated with a numeric suffix on collision.
  The node's `title` is the heading text verbatim. An H1 before the first
  H2 is not a node; it is a fallback deck title only if frontmatter did not
  supply one.
- **Content blocks** within a node's section, in document order:
  - H3–H6 headings become `heading` content blocks (sub-headings on a
    slide, not new nodes).
  - Paragraphs become `text` content blocks; inline Markdown is passed
    through as-is, since the protocol's `TextBlock` already allows it.
  - Fenced code blocks (any info string except the reserved `branch` tag)
    become `code` blocks: language from the info string, source from the
    fence body. `highlight-lines`/`show-line-numbers` are not expressible
    from plain Markdown and stay absent — hand-edit or the ADR-005
    quick-edit modal (text/heading only, so not this) remain the paths to
    add them.
  - Flat bulleted/numbered lists become `list` blocks (`ordered` set
    accordingly). **Nested list items are a stated v1 limitation**: the
    importer reports a clear diagnostic naming the offending line rather
    than silently flattening or dropping nested items.
  - Images (`![alt](src "caption")`) become `image` blocks; `width`/`height`
    are not expressible from Markdown and stay absent.
  - A thematic break (`---` alone on a line, anywhere after the
    frontmatter) becomes a `divider` block.
- **Branching**: a fenced block tagged ` ```branch `, whose body is an
  optional first-line prompt followed by a Markdown link list — one option
  per link, the href a `#slug` anchor resolved against node ids using the
  same slugification as headings, an optional trailing `` `key` `` in
  backticks setting the author hotkey. This fence sets the node's
  `traversal` to a branch-point instead of becoming a content block, and
  must be the last thing in its node's section — content after a branch
  fence in the same section is an authoring error, reported with the
  offending line, not silently dropped or reordered.
- **Traversal for non-branch nodes**: linear — `next` is the following
  node's id in document order; the last node in the document gets no
  traversal (terminal).
- **Validation before write**: the importer runs the existing
  `fireside-engine::validate` (Layer-2) over the generated graph before
  writing the output file, and refuses to write on any error-severity
  diagnostic. Auto-generated linear structure makes most validator rules
  structurally unreachable, but a bad branch-fence target (a link to a
  `#slug` that doesn't match any node) is a realistic authoring mistake;
  the importer catches this at parse time with a message naming the bad
  link and its line, rather than surfacing a generic post-hoc validator
  error disconnected from the source Markdown.

**Explicitly out of scope for v1**, stated so rather than silently
unsupported: container/columns layout (imported decks are single-column;
no Markdown construct maps to `ContainerLayout`), speaker notes, per-node
`view-mode`/`transition` authoring, and nested list items. All remain
hand-JSON-edit territory, and heading/text content becomes editable via the
ADR-005 quick-edit modal once imported. This is progressive enhancement of
the existing hand-JSON authoring path, not a replacement for it — matching
how ADR-005 scoped Stage C.

This decision does not touch `protocol/main.tsp` at all: the importer only
ever emits documents already valid under protocol 0.1.0 as published. No
spec change, no version bump, no new ADR-track spec-kit feature beyond this
ADR itself is required for the wire format.

## Consequences

### Positive

- Closes the authoring gap the strategic plan named as the last P0 stage:
  a presenter can write a talk in the same Markdown-with-headings shape
  every other tool in this space uses, instead of hand-authoring JSON.
- The branch-fence syntax degrades gracefully — any plain Markdown viewer
  (GitHub, a text editor's preview pane) renders it as a labeled code
  block, not garbage — so an author previewing the source file elsewhere
  still gets something legible.
- Reuses the existing Layer-2 validator rather than inventing import-time
  validation rules, keeping "what makes a deck valid" defined in exactly
  one place.

### Negative or Trade-offs

- `pulldown-cmark` is a new dependency on the crate boundary table — a
  deliberate, flagged exception, but a maintenance surface that did not
  exist before (version bumps, `cargo deny`/audit exposure) confined to
  `fireside-cli`.
- v1 import cannot express containers, speaker notes, or per-node
  view-mode/transition — an author who wants those must still hand-edit
  the generated JSON, so the authoring gap narrows but does not fully
  close for advanced decks.
- The branch-fence syntax is novel (invented for this project, not
  borrowed from prior art); presenters have to learn it specifically, even
  though it was designed to read naturally.
- Round-tripping is one-directional: there is no `fireside export` back to
  Markdown, so a deck edited in JSON (via quick-edit or by hand) after
  import cannot be re-exported to keep a Markdown source in sync. Authors
  choose one authoring surface per deck going forward, or accept drift.

### Neutral / Follow-up

- If nested list items or container/columns import turn out to matter in
  practice, that is a v1.1 decision, not a reason to delay this ADR.
- A future `fireside export` (JSON back to Markdown) is conceivable but
  explicitly deferred; it was not asked for and adds its own lossiness
  questions (containers, transitions) that don't need answering yet.
