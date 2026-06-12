---
title: 'ADR-002: Retire node-level Layout in favor of view-mode and container layouts'
status: 'accepted'
date: '2026-06-11'
deciders: ['@tiberius']
---

# ADR-002: Retire node-level `Layout` in favor of `view-mode` + container layouts

## Status

Accepted, with the migration mechanism overtaken by ADR-004: the
presenter-first rewrite removed the `Layout` enum outright instead of carrying
the one-place rendering translation. The core decision — `view-mode` +
container layouts win — stands and is implemented.

## Context

The Rust core has a node-level `layout: Option<Layout>` field
(`crates/fireside-core/src/model/layout.rs`) with a flat enum of presentation
modes — `default`, `center`, `top`, `split-horizontal`, `split-vertical`,
`title`, `code-focus`, `fullscreen`, `align-left`, `align-right`, `blank` — a
pre-rewrite concept that conflates two independent concerns: how much screen
the node gets, and how content within it is arranged.

Protocol 0.1.0 (`protocol/main.tsp`) replaced this with two orthogonal
mechanisms: node-level `view-mode` (`default` | `fullscreen`) controls the
presentation frame, and `ContainerBlock.layout` (`stack` | `columns` |
`center`) controls content arrangement, with arbitrary nesting. The `Layout`
enum exists nowhere in the spec.

Options considered: spec the `Layout` enum (rejected — it re-conflates frame
and arrangement, and a closed list of ad-hoc slide templates does not compose,
whereas containers do); keep both systems side by side (rejected — two
competing layout authorities make rendering order-dependent and untestable).
Doing nothing leaves the TUI rendering off a model the protocol deleted.

## Decision

The spec 0.1.0 model wins. Rendering keys off `view-mode` and container
layouts (Task 17). Legacy `Layout` values get a single one-place rendering
translation (`legacy_layout_hint` in `crates/fireside-tui/src/render/layout.rs`)
so existing documents keep rendering sensibly, and the enum is removed from
`fireside-core` in a future major version.

The translation maps each legacy value as follows:

| Legacy `Layout` value | Translation |
| --------------------- | ----------- |
| `fullscreen` | `view-mode: fullscreen` |
| `code-focus` | `view-mode: fullscreen` |
| `center` | `view-mode: default`, node content area centered |
| `title` | `view-mode: default`, node content area centered |
| `default` | `view-mode: default` |
| `top` | `view-mode: default` (stack is top-aligned already) |
| `split-horizontal` | `view-mode: default` (migrate to a `columns` container) |
| `split-vertical` | `view-mode: default` (migrate to nested `stack` containers) |
| `align-left` | `view-mode: default` (stack is left-aligned already) |
| `align-right` | `view-mode: default` |
| `blank` | `view-mode: default` |

Values that translate to plain `default` and lose behavior (the splits, the
alignments) are exactly the cases the container model expresses better; the
editor templates emit container-based layouts going forward.

## Consequences

### Positive

- Rendering derives from the spec model only — conformance claims about
  presentation become testable.
- Frame and arrangement compose: any container layout works in any view mode,
  instead of 11 fixed combinations.
- The translation lives in one function, so deleting it in the next major is a
  one-place change.

### Negative or Trade-offs

- Legacy documents using `split-horizontal`/`split-vertical` lose their
  two-pane rendering until migrated to container `columns`/`stack`.
- `align-left`/`align-right` have no container equivalent in 0.1.0; that
  expressiveness is dropped deliberately.
- Until the future major lands, `fireside-core` carries a deserialized-but-
  obsolete enum that may confuse contributors (mitigated by doc comments
  pointing here).

### Neutral / Follow-up

- Superseded in part by ADR-004: the rewrite deleted `Layout` from the model
  entirely, so the translation table above is now historical guidance for
  migrating old documents by hand (legacy `layout` fields are ignored on
  read).
