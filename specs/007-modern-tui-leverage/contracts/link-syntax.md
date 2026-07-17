# Contract: Inline link syntax (engine extension, non-normative)

Extends the existing inline-Markdown latitude the protocol already grants
to `text.body` and list items (`protocol/main.tsp`, and documented in
`docs/src/content/docs/spec/appendix-engine-extensions.md`). **Not a
protocol schema change** — no new field, no version bump, no `tsp-output/`
regeneration. This contract only documents the engine's chosen behavior
within latitude the spec already grants (same status as the existing
`**bold**`/`*italic*`/`` `code` `` markers).

## Syntax

```
[label](url)
```

- `label` MAY itself contain the other existing inline markers
  (`**bold**`, `*italic*`, `` `code` ``).
- `url` is taken verbatim as the link destination.
- Unmatched/malformed brackets render literally, same as an unmatched `**`
  or `` ` `` today (Appendix D: "unmatched markers render literally").

## Rendering contract

| Terminal capability                | Behavior                                                                 |
| ----------------------------------- | ------------------------------------------------------------------------- |
| Supports OSC 8 clickable links      | `label` renders as a distinctly-styled (e.g. underlined), clickable region; clicking/cmd-clicking opens `url`. |
| Does not support OSC 8              | `label` renders as plain readable text; no raw escape codes, no visible URL. |

## Validation contract

A new WARNING-level rule (name TBD in tasks, e.g. `malformed-link-url`)
fires when `url` is not a well-formed URL (missing scheme, empty, etc.).
WARNING, not ERROR — consistent with every other content-quality rule in
this codebase (a malformed link must not block presenting). Implemented
symmetrically in `fireside-engine::validation` and `protocol/validate.mjs`,
and added to the shared fixture corpus
(`protocol/fixtures/{valid,invalid}/*.json` + `fixtures.expected.json`) so
Rust/Node parity is tested, not just claimed — matching the precedent set
by `empty-traversal` and `reveal-masked-by-container`.
