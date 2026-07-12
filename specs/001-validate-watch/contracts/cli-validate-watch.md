# CLI Contract: `fireside validate --watch`

## Invocation

```text
fireside validate [--watch] <file>
```

- `<file>` — path to a deck JSON file (required, same as today).
- `--watch` — optional flag. Absent: today's exact behavior (FR-002).
  Present: enters the watch loop described below.

## Without `--watch` (unchanged contract)

- Runs one check, prints the result, exits.
- Exit code `0` if zero errors (warnings/info allowed); exit code `1` if
  the file is unreadable, malformed JSON, or has one or more validation
  errors.
- This contract is not modified by this feature (FR-002); existing
  `cli_e2e.rs` tests (`validate_hello_exits_zero`,
  `validate_missing_file_fails_with_readable_error`,
  `validate_reports_dangling_targets_in_plain_language`) continue to pass
  unchanged and remain the source of truth for it.

## With `--watch`

- **Startup**: performs one check immediately and prints its result before
  entering the poll loop (FR-003). Does not wait for a file change first.
- **Loop**: polls the file's fingerprint (mtime + size) roughly every
  250ms. On a change, re-checks and prints a new result (FR-004, FR-005).
- **Output shape per check** — exactly one of:
  - `✓ <path> — no problems found` (success)
  - the diagnostic list + summary line, identical in format to non-watch
    `validate`'s output (FR-007)
  - the caret-block parse report, identical in format to the report
    `fireside <file>`/`fireside validate <file>` already produce for a
    malformed file (FR-006)
  - a one-line "file missing/unreadable" message (FR-009), for a file that
    does not exist or a transient mid-save read failure (FR-008)
- **Exit codes**: the process does not exit on a validation failure while
  watching — that is the point of the feature. It exits `0` on a clean
  interrupt (Ctrl-C / SIGINT, FR-010). It does not exit due to a bad file;
  it keeps watching.
- **Streams**: all output goes to stdout (matching non-watch `validate`,
  which also uses stdout for its report); no interactive terminal UI is
  drawn (FR-011).

## Out of scope for this contract

- No new subcommand — `--watch` is a flag on `validate` only (constraint
  from spec Input).
- No configurable poll interval, no `--once` alias, no JSON/machine-readable
  output mode — none of these were requested; adding them is future scope,
  not part of this feature.
