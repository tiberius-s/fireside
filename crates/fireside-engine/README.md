# fireside-engine

The protocol's behavior, with no UI attached:

- **`Session`** — the spec §3 traversal state machine: `next`, `choose`,
  `goto`, `back` over an immutable graph, with a history stack of node IDs.
  Every operation returns an `Outcome` so frontends can give the presenter
  feedback for every action — nothing is a silent no-op.
- **`validation`** — spec §4 Layer-2 semantic checks (same rules and rule
  names as `protocol/validate.mjs`) with presenter-friendly diagnostics.

No file I/O, no rendering, no terminal. Dependencies: `fireside-core`,
`thiserror`.
