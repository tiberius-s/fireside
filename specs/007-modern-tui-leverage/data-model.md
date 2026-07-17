# Phase 1 Data Model: Modern TUI Leverage

Neither new entity below touches the wire protocol (`protocol/main.tsp`) —
both are engine-extension / host-local concepts, per research.md §2 and §4.

## Resume Record

Host-local, one row per deck ever presented from this machine. Lives
outside the portable deck format entirely (not part of any `.fireside.json`
file).

| Field         | Type   | Notes                                                                 |
| ------------- | ------ | ---------------------------------------------------------------------|
| `fingerprint` | string | The deck's existing content fingerprint (`main.rs::fingerprint`), the map key. |
| `node_id`     | string | The last-current node's id at the moment of the most recent position change. |
| `updated_at`  | string (RFC 3339) | Informational only; not read by any logic — aids manual debugging of the state file. |

**Lifecycle**:
- Written on every node-change while presenting a file-backed deck (FR-005).
- Read once, at `present` startup, before the first frame draws (FR-006).
- Looked up by the *current* fingerprint of the file about to be presented;
  a fingerprint mismatch (content changed) or absent entry means "no
  record" (FR-003) — start at the graph's normal entry node.
- Cleared (row removed) when a session reaches a normal end — no further
  `next`/branch target exists — so a completed run does not leave a stale
  mid-deck pointer (FR-002 edge case).
- Never written or read for a presentation with no backing file (`fireside
  demo`) (FR-009).
- Ignored for one run when `--restart` is passed (FR-007), without deleting
  the underlying row.
- A `node_id` that no longer exists in the current graph (content changed
  in a way that changed the fingerprint too, or an edge case) is rejected
  by the existing `Session::goto` guarded-no-op behavior (`Outcome::UnknownNode`)
  — falls back to the entry node with no special-case code (FR-008).

## Link (inline content)

Not a new content-block kind — an inline fragment *within* the string body
of any content block that already carries text (`text`, `heading`, list
items — the same set that already supports `**bold**`/`*italic*`/`` `code` ``
per Appendix D).

| Field    | Type   | Notes                                                        |
| -------- | ------ | ------------------------------------------------------------- |
| `label`  | string | The visible, clickable text — may itself contain other inline styling. |
| `url`    | string | The link destination. Validated for a well-formed scheme (FR-015). |

**Authoring syntax**: `[label](url)`, parsed by the same hand-rolled inline
parser as the other markers (`fireside-tui/src/render/markdown.rs`).

**Rendering**: on capable terminals, an OSC 8-wrapped clickable region
(research.md §4); on incapable terminals, `label` alone, styled distinctly
(e.g. underlined) but with no functional click target (FR-013/FR-014).

**Validation**: a new WARNING-level rule (not an error — a malformed link
must not block presenting, consistent with every other content-quality rule
in this codebase) fires when `url` is not well-formed, symmetric across
`fireside-engine::validation` and `protocol/validate.mjs`, and covered by
the shared fixture corpus (FR-015).

## No change to existing entities

`Graph`, `Node`, `ContentBlock`, `Session`, `Outcome` are all unchanged in
shape. `Session::goto`'s existing guarded-no-op behavior is reused, not
modified, for resume fallback.
