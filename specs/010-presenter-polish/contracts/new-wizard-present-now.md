# Contract: interactive `fireside new` present-now prompt

## Scope

Only the interactive wizard path (`fireside new` with no name argument,
`new::interactive_new`). The non-interactive form (`fireside new <name>
[--template ...] [--banner]`) is completely unchanged — no prompt, no
behavior change, per spec FR-010.

## Prompt

After the existing "Created {path}." / present-it / check-it output, the
wizard asks:

```text
Present it now? [Y/n]:
```

- Bare Enter or `y`/`yes` (case-insensitive) → present now.
- `n`/`no` (case-insensitive) or any other input → do not present; wizard
  exits exactly as it does today (no behavior change to the "no" path).
- EOF/no input on stdin (piped, non-interactive invocation) → treated as
  "no" — falls back to the current no-prompt exit rather than blocking,
  per spec's Edge Cases.

This mirrors the existing `Add an ASCII title banner? [y/N]:` prompt's
input handling, with the default inverted (`[Y/n]` instead of `[y/N]`) per
the feature description.

## `new_deck` signature change

```rust
// before
pub(crate) fn new_deck(...) -> Result<()>;
// after
pub(crate) fn new_deck(...) -> Result<Option<PathBuf>>;
```

`Some(path)` only when the interactive prompt was answered yes; `None` in
every other case, including the entire non-interactive path (FR-010).

## Launch mechanism

`main.rs`'s `New` command arm calls the existing `present(&path, false)`
function (the same one the plain `fireside <deck>` invocation already uses)
when `new_deck` returns `Some(path)` — an in-process function call, not a
subprocess `exec`. `--restart` is not applicable (a freshly created deck has
no resume record to skip).

## Failure handling

If `present(&path, false)` itself fails (e.g. terminal init failure), that
error propagates exactly as it would from a direct `fireside <deck>`
invocation — no special-casing for having arrived via the wizard. The deck
file itself is already written to disk by this point regardless of what
happens next.
