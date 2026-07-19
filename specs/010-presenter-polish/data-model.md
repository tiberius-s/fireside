# Data Model: Presenter Polish

No persistent storage or protocol/wire-format changes — every entity below
is either a new in-memory value used only for one call's return path, or an
existing value read, not redefined. This document maps the spec's Key
Entities to their concrete shape, decided in `research.md`.

## `PresentSummary` (new)

Returned by `fireside_tui::{present, present_watching, present_authoring}`
on a graceful stop, replacing their current `Ok(())`.

| Field     | Type       | Source                                                    |
| --------- | ---------- | ----------------------------------------------------------- |
| `seen`    | `usize`    | `session.visited().len()` at the moment the loop exits      |
| `total`   | `usize`    | `graph.nodes.len()`, captured before the graph moves into `Session::new` |
| `elapsed` | `Duration` | `app.elapsed()` at the moment the loop exits                |

Lifecycle: constructed once, immediately before `present_authoring` returns
`Ok`; consumed by the CLI to format `Presented {seen}/{total} slides in
{mm}:{ss}.` and discarded. Never serialized, never crosses a process
boundary.

## `RESERVED_PRESENTER_KEYS` (new)

A fixed, `fireside-engine`-owned constant:

```rust
pub const RESERVED_PRESENTER_KEYS: [char; 12] =
    ['e', 'f', 'g', 'h', 'j', 'k', 'm', 'n', 'p', 'q', 's', 't'];
```

Consumed by:

- `fireside-engine::validation::check_reserved_branch_keys` (new function),
  which compares every branch option's `key` against this list.
- A `fireside-tui` regression test asserting `App::on_present_key` consumes
  every one of these chars globally (never lets one reach branch dispatch).

Not part of the wire protocol — a reference-engine implementation detail,
documented per research.md §7.

## `reserved-branch-key` diagnostic (new validation rule)

Follows the existing `Diagnostic` shape (`fireside-engine::validation`,
same as `unique-branch-keys` and every other Layer-2 rule) — no new type.

| Field      | Value for this rule                                                       |
| ---------- | --------------------------------------------------------------------------- |
| `severity` | `Severity::Warning`                                                         |
| `rule`     | `"reserved-branch-key"`                                                     |
| `message`  | Names the colliding key, the owning node's id, and the option's label       |
| `node_id`  | `Some(&node.id)` — the branch point's owning node                           |

## Resume flash (state transition, no new type)

Not a new struct — a new call to the existing `App::set_flash` (widened from
private to `pub(crate)`), made from `present_authoring` immediately after
construction, conditional on `session.goto(id)` having returned
`Outcome::Moved` for a `Some(initial_node)`. Reuses the existing `Flash` /
`FlashKind::Info` shape unchanged.

## `new_deck` return value (changed)

| Before                | After                                                              |
| ---------------------- | ------------------------------------------------------------------- |
| `Result<()>`            | `Result<Option<PathBuf>>` — `Some(path)` only when the interactive wizard's present-now prompt was answered yes |

Consumed by `main.rs`'s `New` command arm, which calls the existing
`present(&path, false)` when it receives `Some`.

## `art text` width note (no new type)

A single `eprintln!` in `art_text`, gated on the same
`widest > DEFAULT_ART_WIDTH` comparison `new.rs::add_title_banner` already
performs — no new struct, no change to `render_text_banner`'s return value
or to stdout output.
