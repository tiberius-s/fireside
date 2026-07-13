# Contract: `empty-traversal` validation rule

## Rule identifier

`empty-traversal`

## Severity

`Warning`

## Applies to

Every node in a `Graph` whose `traversal` field is present AND is the
object form (not the string shorthand) AND that object sets neither `next`
nor `branch-point`.

Pseudocode (language-neutral):

```text
for each node in graph.nodes:
  if node.traversal is absent: skip        # normal terminal, no warning
  if node.traversal is a string: skip      # never empty by construction
  t = node.traversal  # object form
  if t.next is absent and t["branch-point"] is absent:
    emit warning "empty-traversal" for node
```

## Message contract

The message MUST:
- name the affected node's id
- state that an empty traversal object behaves identically to an absent
  `traversal` field (terminal — only `back()` can leave)
- suggest the likely fix (remove the empty object, or add a `next`/
  `branch-point`) without being prescriptive about which

Reference wording (Rust): matches the tone of existing warnings in
`fireside-engine/src/validation.rs` — second person implied, plain
language, no jargon like "instance path."

## Non-goals

- MUST NOT change `Node::is_terminal()` or any traversal/session behavior.
- MUST NOT fire for a branch-point with zero options (that's the existing
  `empty-branch-options`/schema-level concern, a different case).
- MUST NOT fire for the string traversal shorthand under any input, since
  `NodeId` has a minimum length of 1 and cannot represent "empty."

## Implementations required

1. `fireside-engine/src/validation.rs`: a `check_empty_traversal` function
   in the same style as `check_self_loops`/`check_unreachable`, wired into
   `validate()`'s call chain, with unit test coverage in the existing
   `#[cfg(test)] mod tests` block.
2. `protocol/validate.mjs`: a `checkEmptyTraversal(graph)` function in the
   same style as `checkSelfLoops`/`checkReachability`, wired into the
   module's `validate()` function.

Both implementations MUST use the exact rule identifier string
`empty-traversal` and MUST be covered by the shared fixture
`valid/empty-traversal.json` (see `fixture-corpus.md`).
