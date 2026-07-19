# Research: Presenter Polish

All five sub-features are small and build directly on existing mechanisms
(flash messages, `Session::visited()`, `Outcome`, the `add_title_banner`
width-guard pattern). No unknowns required external research; this document
records the five non-obvious design decisions made while grounding the spec
against the current code.

## 1. Where the reserved presenter key list lives

**Decision**: Define `RESERVED_PRESENTER_KEYS: [char; 12]` in
`fireside-engine` (next to the new `check_reserved_branch_keys` validation
function in `validation.rs`), and add a `fireside-tui` unit test that
imports it and asserts every key in the list is consumed by a global arm in
`App::on_present_key` before it could ever reach branch-option dispatch.

**Rationale**: The reserved keys are presenter-UI facts (`fireside-tui`
owns the key dispatch that actually reserves them), but the validation rule
that needs the list lives in `fireside-engine`, which per Constitution
Principle III **cannot** depend on `fireside-tui` (`fireside-tui` depends on
`fireside-engine`, not the reverse — adding the dependency the other way
would be a cycle). Since `fireside-tui` already depends on
`fireside-engine`, the constant's natural home is the lower crate, with the
higher crate re-checking against it. This also directly guards against a
repeat of the W1-1 bug (`demo.fireside.json`'s branch key silently shadowed
by the global `e` binding) — the two lists can no longer drift silently,
because the TUI-side test fails the moment `on_present_key` and the engine's
constant disagree.

**Alternatives considered**:

- *Duplicate the list as a literal array in both crates.* Rejected — this
  is exactly the "documented but code diverged" failure mode the feature
  exists to close.
- *Put the list in `fireside-core`.* Rejected — `fireside-core`'s own module
  doc states it is "a faithful Rust mirror of the... protocol... it performs
  no I/O, holds no state, and contains no rendering or validation logic".
  Reserved presenter keys are not a protocol concept (the spec has no
  opinion on keybindings); putting them there would misrepresent them as
  protocol-level.

## 2. How the CLI learns what to report after the TUI closes

**Decision**: Change `present`, `present_watching`, and `present_authoring`
in `fireside-tui/src/lib.rs` to return `Result<PresentSummary, TuiError>`
instead of `Result<(), TuiError>`, where:

```rust
pub struct PresentSummary {
    pub seen: usize,
    pub total: usize,
    pub elapsed: Duration,
}
```

populated from `session.visited().len()`, `graph.nodes.len()` (captured
before the session moves into `Session::new`), and `app.elapsed()` — all
three already tracked internally, none of it new state. The CLI (`present()`
and `demo()` in `main.rs`) prints the one-line summary itself after the call
returns `Ok`; `fireside-tui` still performs no `println!`/formatting of its
own.

**Rationale**: Constitution Principle III forbids `fireside-cli` from doing
"state management [or] rendering outside `fireside-tui`" — but a single
`println!` of a plain summary line is neither; it's the same class of
output `present()`/`import_file()` already produce directly in `main.rs`
today (e.g. "Created {path}.", the import-limitations note). Threading a
small summary struct back through the existing `Result` is a much smaller,
more idiomatic change than having the TUI print after `ratatui::restore()`
(which would also break the "TUI never touches stdout formatting" boundary
in the other direction).

**Alternatives considered**:

- *Have `fireside-tui` print the summary itself, after `ratatui::restore()`.*
  Rejected — moves formatting/output ownership into the TUI crate, which
  the rest of the crate deliberately avoids (compare `WriteBackError`'s
  `Display` impl, which formats a *message* but leaves the caller to print
  it).
- *Expose `App` accessors and have the CLI reach into a live `App` after the
  fact.* Not possible as structured — `App` is constructed and owned
  entirely inside `event_loop`/`present_authoring`; there is no `App` left
  for the caller to inspect once the terminal is torn down. A return value
  is the only channel.

## 3. What counts as a "graceful" quit

**Decision**: Any `Ok(PresentSummary)` return from `present_authoring`
counts as graceful and gets the exit summary — this covers both the `q` key
and in-TUI `Ctrl+C` (`app.rs` line 476-478 sets the same `quit` flag `q`
does; there is no OS-level signal involved, since raw mode intercepts
`Ctrl+C` as a keypress). Only an `Err(TuiError)` (unpresentable graph,
terminal I/O failure) suppresses the summary.

**Rationale**: `App` has exactly two ways to set `quit = true` (`q`, and the
in-loop `Ctrl+C` handler) and both exit the event loop identically, restore
the terminal identically, and return `Ok(())` identically — there is no
existing signal to distinguish "the user pressed `q`" from "the user pressed
Ctrl+C" without adding new state purely to suppress a summary line in a case
that behaves like a clean stop in every other respect. The feature
description's "on quitting the TUI with `q`" is read as "on a normal,
voluntary stop," which Ctrl+C already is in this app.

## 4. How the resume flash gets set

**Decision**: `set_flash` (currently a private `fn` on `App`) becomes
`pub(crate)`. In `present_authoring`, after `Session::new` and the existing
`session.goto(id)` call for `initial_node`, check the returned `Outcome`:
if it's `Outcome::Moved`, construct `App::new(session)` and immediately call
`app.set_flash("Resumed where you left off — --restart starts over",
FlashKind::Info)`.

**Rationale**: `Outcome::Moved` is exactly "the resume position existed and
was applied" — `Outcome::UnknownNode` (deck edited since last run, saved
node id no longer exists) already falls back to the entry node as a guarded
no-op, and correctly gets no resume flash under this check, satisfying the
spec's edge case without new logic. Keeping `set_flash` `pub(crate)` (not
fully `pub`) preserves the existing internal-only mutation boundary — the
call site is `lib.rs`, inside the same crate.

**Alternatives considered**:

- *Add an `initial_flash: Option<&str>` parameter to `App::new`.* Rejected —
  `App::new` has eleven direct call sites in `render/tests.rs` alone; a
  signature change would touch all of them for a caller (`present_authoring`)
  that already has the information it needs right after construction.

## 5. Width measurement for `art text`'s stderr note

**Decision**: Measure the same way `new.rs::add_title_banner` already does
— `art.lines().map(str::len).max()` — and compare against
`art::DEFAULT_ART_WIDTH` (76, already the same constant `--banner` compares
against). `art_text` prints `eprintln!("Note: this banner is N columns wide
— it will be flagged by \"fireside validate\" (ascii-art-too-wide, 76
columns).")`-style guidance to stderr when `widest > DEFAULT_ART_WIDTH`,
after printing the unchanged art to stdout.

**Rationale**: `DEFAULT_ART_WIDTH` and the byte-length-of-widest-line
measurement already exist and are already documented as matching
`ascii-art-too-wide`'s threshold — reusing them exactly is what "matching
the existing skip-note behavior" in the spec means, and keeps the two skip/
warn notes (`new --banner`'s and `art text`'s) measuring width identically
so they never disagree about whether the same banner is "too wide."

## 6. Wizard momentum: who launches the presenter

**Decision**: `new_deck`'s return type changes from `Result<()>` to
`Result<Option<PathBuf>>` — `Some(path)` means "the interactive wizard's
user said yes to the present-now prompt," `None` otherwise (including the
whole non-interactive `fireside new <name>` path, which never asks). The
`New` arm of `main.rs`'s command dispatch calls the existing `present(&path,
false)` when it gets `Some`.

**Rationale**: `new_deck` and `present` already live in the same crate but
different modules (`new.rs`, `main.rs`); routing the decision back to
`main.rs` to actually launch keeps `new.rs` from needing to know about
`resume`/`watch` (which `present` already wires together) and avoids
duplicating that wiring — a straightforward reuse of the exact function the
non-interactive `fireside <deck>` path already calls, not a new "exec"
mechanism. `--restart` is irrelevant here (freshly created decks have no
resume record), so `present(&path, false)` is called unconditionally.

## 7. Documenting the reserved-key rule

**Decision**: Add the rule as a fifth bullet to the "Recommended Checks"
list in `docs/src/content/docs/spec/validation.md` (~L68, alongside
`ascii-art-too-wide`/`ascii-art-empty`), *and* add one bullet to
`appendix-engine-extensions.md`'s "Behavior near the protocol's edges"
section, following the existing `malformed-link-url` precedent exactly (a
reference-engine-specific warning, not a protocol concept, still worth a
one-line pointer for anyone implementing a second engine).

**Rationale**: `malformed-link-url` is the closest existing precedent —
another WARNING-severity, reference-engine-specific validation rule that's
listed in both places. Matching it exactly is lower-risk than inventing a
new documentation pattern.

## 8. Rust/Node validator parity (discovered during implementation)

**Decision**: `reserved-branch-key` must also be implemented in
`protocol/validate.mjs` (a `checkReservedBranchKeys` function mirroring
`checkMalformedLinkUrls`'s shape, wired into `validate()`'s aggregator, plus
a line in the CLI `HELP` text's "Rules (warnings)" section), and proven
in lockstep with the Rust implementation via a new fixture:
`protocol/fixtures/valid/reserved-branch-key.json` with a matching entry
`"valid/reserved-branch-key.json": ["reserved-branch-key"]` in
`protocol/fixtures.expected.json`.

**Rationale**: `validation.rs`'s own module doc (line 1-6) states the file
uses "the same rules and rule names as `protocol/validate.mjs` so the Rust
and Node validators stay in lockstep" — this is a standing, tested
constraint, not a per-feature choice: `crates/fireside-engine/tests/
fixtures.rs` and `protocol/run-fixtures.mjs` both run the *same* fixture
corpus and assert identical rule-ids fire, per
`specs/004-spec-patch-0-1-1/contracts/fixture-corpus.md`. Every existing
WARNING-severity rule in this codebase (`malformed-link-url`,
`ascii-art-too-wide`, `ascii-art-empty`, `reveal-masked-by-container`, ...)
has both a Rust and a Node implementation plus a fixture proving parity;
`reserved-branch-key` follows the same pattern rather than becoming the
first exception. This was missed in the initial planning pass (which only
surveyed `fireside-engine/src/validation.rs`) and is corrected here before
implementation proceeds — see the updated `tasks.md` Phase 2.
