# Quickstart: Validating Markdown Import

## Prerequisites

- `cargo build --workspace`

## Scenario 1 — Linear talk, no branching (User Story 1, P1)

```sh
cat > /tmp/talk.md <<'EOF'
---
title: My Talk
author: Ada Lovelace
---

# My Talk

## Welcome

Thanks for coming. Here's what we'll cover.

- Point one
- Point two

## The Code

​```rust
fn main() {
    println!("hello");
}
​```

## Thanks

Questions?
EOF
fireside import /tmp/talk.md
fireside validate /tmp/talk.fireside.json
fireside /tmp/talk.fireside.json
```

**Expected**: three nodes (`welcome`, `the-code`, `thanks`) in order, deck
title "My Talk", author "Ada Lovelace", `welcome` has a text block and a
list block, `the-code` has a `rust` code block, linear traversal
welcome → the-code → thanks, `thanks` is terminal. Validates and presents
clean.

## Scenario 2 — Branching (User Story 2, P1)

```sh
cat > /tmp/branch.md <<'EOF'
## Choose your path

​```branch
What would you like to see?
- [Explore the features](#core-features) `f`
- [Watch a demo](#code-demo) `d`
​```

## Core Features

Some features.

## Code Demo

​```rust
fn demo() {}
​```
EOF
fireside import /tmp/branch.md
fireside /tmp/branch.fireside.json
```

**Expected**: the `choose-your-path` node has a branch-point with prompt
"What would you like to see?" and two options targeting `core-features`
and `code-demo`, the second option's key is `d`. Presenting the deck shows
the choice menu and both branches work.

## Scenario 3 — Unresolved branch target is rejected (User Story 2)

Change `#code-demo` in Scenario 2's fence to `#nonexistent` and re-run
`fireside import`. **Expected**: import fails, exit 1, a message naming the
line and the bad link; `/tmp/branch.fireside.json` is not written (or not
overwritten if it already exists from Scenario 2 — delete it first to
confirm no *new* file is produced).

## Scenario 4 — Nested list is reported, not silently mangled (User Story 3, P2)

```sh
cat > /tmp/nested.md <<'EOF'
## Slide

- Top item
  - Nested item
EOF
fireside import /tmp/nested.md
```

**Expected**: import fails, exit 1, a message naming the nested list's
line; no output file is written.

## Scenario 5 — Edge cases

- No `##` headings at all → import fails with a clear "at least one `##`
  section is required" message.
- Re-running `fireside import /tmp/talk.md` after Scenario 1 already
  produced `/tmp/talk.fireside.json` → fails with "already exists — pick
  another name," matching `fireside new`'s existing wording for the same
  situation.
- A literal ` ```branch ` fence intended as a real code sample (not a
  branch declaration) is documented, reserved behavior — not tested here
  as a bug, but worth confirming the error message (if the fence body
  doesn't parse as valid branch syntax) points at the fence's line rather
  than failing silently or panicking.

## Automated coverage (see `tasks.md` for concrete test files)

- Unit tests directly against `import::import(&str) -> Result<Graph,
  ImportError>` in `crates/fireside-cli/src/import.rs` — one per Functional
  Requirement, no filesystem needed (per contracts/cli-import.md's "no file
  I/O" guarantee).
- One `cli_e2e.rs` integration test exercising the `import` verb end to end
  (real files, default output path derivation, overwrite refusal).
