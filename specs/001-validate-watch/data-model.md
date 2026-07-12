# Data Model: `validate --watch`

This feature has no persistent storage and introduces no new domain
entities in `fireside-core`. The relevant shapes already exist in
`fireside-engine`/`fireside-cli`; this document records how the watch loop
uses them.

## Existing types reused as-is

- **`fireside_core::Graph`** — the parsed deck. Unchanged.
- **`fireside_engine::Diagnostic`** (`severity`, `rule`, `message`, node
  reference) and **`fireside_engine::Severity`** (`Error` | `Warning` |
  `Info`) — the semantic validation output. Unchanged; the watch loop calls
  `fireside_engine::validate(&graph)` exactly as `validate_file` does today.
- **`serde_json::Error`** — the parse-failure type already handled by
  `parse_report()`/`strip_position()`.

## New shape: watch-cycle report

Not a struct — a rendering concern. Each poll cycle that detects a change
produces one of three outcomes, all rendered as plain stdout/stderr text
(no interactive UI, per FR-011):

| Outcome | Trigger | Rendering |
|---|---|---|
| Success | File parses and has zero validation errors (warnings/info allowed) | `✓ <path> — no problems found` (identical to today's `validate_file` success line) |
| Diagnostics | File parses but `validate()` returns one or more diagnostics | The existing per-diagnostic list + summary line from `validate_file`, extracted into a shared helper so watch and one-shot output match exactly |
| Parse failure | File is not valid JSON | The existing caret-block report from `parse_report()` |
| Missing/unreadable file | File does not exist or a transient read error occurs | A one-line message naming the file and the reason (mirrors the existing "could not read …" `anyhow` context used by one-shot `validate`'s error path), followed by continued watching (does not exit, per FR-008/FR-009) |

## Change-detection state

- **Fingerprint**: `(SystemTime, u64)` — modification time and byte length
  of the watched file, from the existing `fingerprint()` function. The watch
  loop keeps the last-seen fingerprint and re-runs the outcome table above
  only when it changes (or on the first check, per FR-003).
