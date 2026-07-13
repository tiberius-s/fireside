# Contract: `next()` with incremental reveal

## Revised algorithm

Supersedes the five-step algorithm in `main.tsp`'s current
`TraversalOps.next()` doc comment (ADR-007-era text). New algorithm:

1. Compute the current node's reveal steps (`reveal-field.md`'s
   derivation). If the current reveal threshold has not reached the
   largest step in that sequence:
   a. Advance the threshold to the next step in the sequence that is
      greater than the current threshold.
   b. Return/report a reveal outcome. Stop — do not proceed to step 2 in
      this call.
2. If current node has a branch-point → BLOCKED, wait for `choose()`.
3. Push current node ID onto history.
4. If traversal is a string, navigate to that target.
5. If `traversal.next` exists, navigate to that target.
6. If no traversal, no-op (terminal node) — end of path.

Step 1 is new; steps 2–6 are the pre-existing algorithm, unchanged and
reached only once step 1 has nothing left to do.

## Outcome contract

`next()` (and by extension any UI action bound to it) MUST distinguish
"a reveal step was consumed" from "the current node changed" in whatever
feedback mechanism the engine uses for keypress feedback — the reference
implementation does this via a dedicated `Outcome::Revealed` variant,
distinct from `Outcome::Moved`. This distinction matters because
navigation-triggered UI effects (e.g. a transition/fade, resetting a
branch-menu selection) MUST NOT fire on a reveal-only step, since no
navigation occurred.

## Interaction with `choose()`

If the current node has both pending reveal steps and a branch-point, an
engine's presenter-facing choose action MUST NOT be reachable until step 1
above has nothing left to do — i.e. the same gate that blocks `next()`
from reaching the branch-point check (step 2) MUST also block any UI path
that would call `choose()` directly. The reference implementation
enforces this by gating branch-key routing in the TUI event handler on the
same "has pending reveal" predicate `next()` uses internally, rather than
duplicating the reveal-step logic in the choose path.

## Interaction with `back()` and `goto()`

Both operations navigate to a node exactly as before. In addition, both
MUST reset that node's reveal threshold to `0` on arrival — the node
presents from its first reveal step regardless of any prior visit's
progress. `back()` does not route through the same internal navigation
helper as `next`/`choose`/`goto` in the reference implementation (it does
not push a new history entry), so this reset MUST be applied at `back()`'s
own call site, not assumed to come "for free" from a shared code path.
