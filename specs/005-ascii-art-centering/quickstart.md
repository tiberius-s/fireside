# Quickstart: ASCII art centering and clipping

## Prerequisites

- `cargo build -p fireside-tui`

## Scenario 1 — narrow ASCII art centers

Write a small Rust snippet (or run as a `cargo test` once implemented)
rendering:

```json
{ "kind": "code", "source": " /\\_/\\ \n( o.o )\n > ^ < " }
```

at `width: 40`. Expect: the box's top rule, three content rows, and bottom
rule are all identical width, less than 40, and every line is prefixed
with the same left pad — the box appears centered, not stretched.

## Scenario 2 — explicit language does not center

Same content, `"language": "rust"`, same `width: 40`. Expect: box
stretches to the full 40 columns, left-aligned, exactly as it renders
today (byte-for-byte match with pre-feature output).

## Scenario 3 — oversized ASCII art clips, doesn't break

Render a code block (no language) with one line 200 characters wide at
`width: 40`. Expect: no panic, box caps at 40, the long line ends with the
existing ellipsis marker at the cut point.

## Scenario 4 — 80×24 end-to-end

Drive the `TestBackend` scenario suite (`fireside-tui/src/render/mod.rs`)
at 80×24 with a node containing a narrow ASCII-art code block as its only
content block. Confirm the rendered screen shows the art centered
horizontally within the content area.

## Scenario 5 — composes with `container { layout: "center" }`

Re-run the existing `centered_code_keeps_its_internal_alignment` test
(container-centered code block with `language: None`) unmodified — it
must still pass, proving the two centering behaviors compose without
regressions.

## Full verification

```sh
cargo test -p fireside-tui
cargo test --workspace
cargo clippy --workspace --all-targets
```
