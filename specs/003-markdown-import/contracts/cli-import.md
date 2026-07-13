# Contract: `fireside import` CLI verb

## Invocation

```
fireside import <input.md> [output.fireside.json]
```

- `input`: required path to a Markdown file.
- `output`: optional path for the generated deck. Defaults to `input` with
  its extension replaced by `.fireside.json` (e.g. `talk.md` →
  `talk.fireside.json`), in the same directory as `input`.

## Exit behavior

| Condition | Exit code | Output |
|---|---|---|
| Successful import | 0 | stdout: `Imported <output>.` plus a summary line for anything v1 doesn't carry over (FR-023) |
| Output path already exists | 1 | stderr: `<output> already exists — pick another name.` (mirrors `fireside new`'s existing message shape) |
| No `##` headings found | 1 | stderr: message requiring at least one `##` section (FR-022) |
| Nested list found | 1 | stderr: message naming the line (FR-012) |
| Malformed branch-fence line | 1 | stderr: message naming the line and section heading |
| Unresolved branch target | 1 | stderr: message naming the link, its line, and the section heading (FR-018) |
| Content after a branch fence in the same section | 1 | stderr: message naming the line and section heading (FR-019) |
| Generated deck fails validation | 1 | stderr: the same diagnostics format `validate`/`present` already use (FR-021) |

No output file is written in any error case (FR-018, FR-019, FR-021,
FR-022).

## Internal module contract (`crates/fireside-cli/src/import.rs`, new module)

```rust
/// Parses `source` (Markdown) into a validated `Graph`, or a specific,
/// located `ImportError`. Pure — no file I/O, so it is unit-testable
/// without a filesystem.
pub fn import(source: &str) -> Result<Graph, ImportError>;
```

`main.rs`'s `Command::Import` handler owns all file I/O: reads the input
path, calls `import::import`, and on `Ok` writes the output (after the
`OutputExists`/overwrite check, which happens before parsing since it's
cheap and should fail fast); on `Err` maps `ImportError` to a
user-readable message and a non-zero exit, following the existing
`validate_file`/`present` pattern of `eprintln!` + `std::process::exit(1)`
rather than propagating `anyhow::Error` through `import()` itself (keeping
`import()` a pure, richly-typed function per constitution §V's stratified
error handling — `anyhow` stays at the CLI boundary in `main.rs`, not
inside the parsing logic).

## Behavioral guarantees this contract exists to make testable

- `import()` never performs file I/O — every test constructs it from an
  in-memory `&str`, no tempfile needed for parser-logic tests (only the
  CLI-wiring tests in `cli_e2e.rs` need real files).
- `import()` returns `Err` (never panics, never writes a partially-formed
  `Graph`) for every case in the Exit-behavior table above.
- A successful `import()` result always passes
  `fireside_engine::validate` with zero error-severity diagnostics — this
  is enforced *inside* `import()` (FR-021), not left to the caller.
