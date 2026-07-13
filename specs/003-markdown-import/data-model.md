# Phase 1 Data Model: Markdown Import

No protocol/wire-format entities are added or changed — the importer's job
is entirely to produce a value of the existing `fireside_core::Graph` type.
This feature adds only in-process types inside `fireside-cli`, used while
converting.

## `Frontmatter` (new, `fireside-cli`)

Parsed from an optional leading `---`-delimited block.

| Field | Type | Maps to |
|---|---|---|
| title | `Option<String>` | `Graph.title` |
| author | `Option<String>` | `Graph.author` |
| date | `Option<String>` | `Graph.date` |
| description | `Option<String>` | `Graph.description` |
| fireside_version | `Option<String>` | `Graph.fireside_version` (defaults to the current protocol version if absent) |

Unrecognized keys are ignored (spec Edge Cases), not an error.

## `Section` (new, `fireside-cli`, transient)

One `##`-delimited region of the source document — the unit that becomes
one `Node`.

| Field | Type | Notes |
|---|---|---|
| heading_text | `String` | Becomes `Node.title`; slugified for `Node.id`. |
| id | `NodeId` | Slug of `heading_text`, deduplicated against prior sections' ids. |
| blocks | `Vec<ContentBlock>` | Built by walking the section's events (research.md §3). |
| branch | `Option<BranchDeclaration>` | Set if a `branch` fence was found; mutually exclusive with the node getting linear `next` traversal. |

## `BranchDeclaration` (new, `fireside-cli`, transient)

Parsed form of a `branch` fence, before target resolution.

| Field | Type | Notes |
|---|---|---|
| prompt | `Option<String>` | First line, if it isn't a list item. |
| options | `Vec<BranchOptionSource>` | One per `- [label](#target)` line. |
| line | `usize` | The fence's starting line, for error messages if a later target fails to resolve. |

## `BranchOptionSource` (new, `fireside-cli`, transient)

| Field | Type | Notes |
|---|---|---|
| label | `String` | From the link text. |
| target_slug | `String` | From the link's `#`-anchor, resolved against known node ids after all sections are collected (research.md §5, two-pass). |
| key | `Option<String>` | From an optional trailing `` `key` ``. |
| line | `usize` | For an "unresolved link" error message (FR-018). |

## `ImportError` (new, `fireside-cli`)

One case per rejection reason the spec names; each carries enough location
information to satisfy FR-018/FR-019/FR-022's "name the line/link" bar.

| Variant | Carries | Corresponds to |
|---|---|---|
| `NoHeadings` | — | FR-022 |
| `NestedList` | line | FR-012 |
| `UnresolvedBranchTarget` | line, target slug, section heading | FR-018 |
| `ContentAfterBranch` | line, section heading | FR-019 |
| `MalformedBranchLine` | line, section heading | research.md §5 |
| `OutputExists` | path | FR-003 |
| `ValidationFailed` | diagnostics | FR-021 |

`fireside-cli`'s stratified error handling (constitution §V) means this
enum is surfaced to the user via `anyhow::Context` at the command boundary,
not propagated as a library type — consistent with how `validate_file`
already reports diagnostics.

## Relationships

- `Section` → `Node`: one-to-one, in document order; a `Section`'s `branch`
  (if present) becomes that `Node`'s `TraversalSpec::Rules(Traversal {
  next: None, branch_point: Some(BranchPoint { .. }) })`; otherwise the
  `Node` gets `TraversalSpec::Target(next_section.id)`, or no traversal at
  all for the last section.
- `BranchOptionSource` → `BranchOption`: one-to-one, once `target_slug` is
  resolved to a real `NodeId` from the id map built in pass one
  (research.md §5). `description` on `BranchOption` is never set by v1
  import (no Markdown construct maps to it).
- `Frontmatter` → `Graph`: direct field-for-field copy of the present
  fields; `Graph.defaults` and `Graph.version` are never set by import (no
  Markdown construct maps to them either).
