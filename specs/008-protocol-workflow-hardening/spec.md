# Feature Specification: Protocol & Workflow Hardening

**Feature Branch**: `008-protocol-workflow-hardening`

**Created**: 2026-07-17

**Status**: Draft

**Input**: User description: "P2 — Protocol & workflow hardening: strengthen Fireside's testing and CI safety net without changing the wire format. Three parts, all from .claude/plans/2026-07-12-strategic-improvement-plan.md section 'P2 — Protocol & workflow hardening': (1) Property tests (proptest, dev-dependency only) — serde round-trip on arbitrary Graph values in fireside-core, and Session invariants (history reflects the actual path taken, visited nodes is a subset of all nodes) under arbitrary sequences of operations in fireside-engine. (2) Robustness fixtures added to the existing shared conformance corpus at protocol/fixtures/{valid,invalid}/ and protocol/fixtures.expected.json (consumed identically by fireside-engine's Rust fixture test and protocol/run-fixtures.mjs): deep container nesting (spec says engines MAY impose a limit — pick one, document it in an ADR, and add a validator rule + fixture), multi-codepoint/emoji/CJK-width headings and columns (render-correctness, not a validator rule — belongs in fireside-tui's scenario suite instead), a large deck (~1000 nodes) load/validate time fixture, and a rapid-reload-with-half-saved-invalid-JSON-mid-edit scenario for fireside-cli's watcher. (3) CI additions — confirm cargo-deny already runs in CI (it does, in .github/workflows/audit.yml) and decide whether the existing MSRV job (cargo check --workspace on Rust 1.88 in .github/workflows/rust.yml) already satisfies the plan's 'cargo msrv verify' ask or whether the dedicated cargo-msrv tool adds anything beyond what's already there."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Property tests guard the wire format and engine invariants (Priority: P1)

A maintainer changes a field in the protocol model or refactors the session
state machine. Before the change can merge, an automated test suite
generates many varied, randomized documents and operation sequences and
checks two guarantees that hand-written examples can't practically cover:
every document that round-trips through serialization comes back identical,
and every session, no matter what sequence of navigation actions a presenter
takes, only ever reports history and visited-node data that are actually
true of the path taken.

**Why this priority**: This is the deepest safety net Fireside has — it
catches classes of bugs (serialization asymmetry, invariant violations under
unusual-but-legal input) that example-based tests structurally can't find,
because the examples were written by someone who already assumed the
invariant held. It protects the two things the constitution calls
non-negotiable: the spec is the source of truth, and engine operations must
never silently misreport what happened.

**Independent Test**: Can be fully tested by running the property test suite
in isolation (`cargo test` in `fireside-core` and `fireside-engine`) and
confirming it fails when a deliberately introduced bug (e.g. a session
history field that isn't updated on `goto`) is present, then passes once
reverted.

**Acceptance Scenarios**:

1. **Given** an arbitrary, structurally valid `Graph` value, **When** it is
   serialized to the wire format and deserialized back, **Then** the
   resulting value is identical to the original.
2. **Given** an arbitrary sequence of navigation operations (`next`,
   `choose`, `goto`, `back`) applied to a session over an arbitrary valid
   graph, **When** the sequence completes, **Then** the session's reported
   history exactly reflects the sequence of nodes actually visited, and the
   set of visited nodes is a subset of the graph's declared nodes.
3. **Given** a deliberately broken invariant (introduced for verification
   purposes only, then reverted), **When** the property suite runs,
   **Then** it fails with a reproducible minimal counterexample rather than
   passing silently.

---

### User Story 2 - Expanded conformance corpus catches robustness regressions (Priority: P2)

A maintainer (or a third-party engine implementer relying on Fireside's
conformance corpus as documented in the spec) needs confidence that
validators — both the Rust reference implementation and the Node
implementation — agree not just on hand-picked simple cases, but on
documents that stress realistic boundary conditions: decks nested far
deeper than any real presentation would go, and decks large enough to
matter for load-time expectations. Both validators must reach the same
verdict on every such document, and a large deck must load and validate
within a time a presenter would experience as instant.

**Why this priority**: The corpus already exists and is proven to catch
cross-language drift (a prior deliberate-mismatch test confirmed this). This
story extends it to boundary conditions the current 14 fixtures don't cover,
closing the gap between "the validators agree on hand-picked examples" and
"the validators agree on realistic edge-of-scale documents." It's P2, not
P1, because it hardens existing guarantees rather than adding a new class of
guarantee.

**Independent Test**: Can be fully tested by adding new fixtures to
`protocol/fixtures/{valid,invalid}/` and their expected rule IDs to
`protocol/fixtures.expected.json`, then running both the Rust fixture test
and `protocol/run-fixtures.mjs` and confirming identical results.

**Acceptance Scenarios**:

1. **Given** a document whose containers are nested deeper than the
   documented maximum, **When** it is validated by either validator,
   **Then** both report the same diagnostic for exceeding the limit.
2. **Given** a document whose containers are nested exactly at the
   documented maximum, **When** it is validated, **Then** both validators
   accept it without that diagnostic.
3. **Given** a deck of approximately 1,000 nodes, **When** it is loaded and
   validated, **Then** the operation completes within the documented time
   budget on reference hardware.

---

### User Story 3 - Watcher survives a half-saved edit without losing state (Priority: P2)

A presenter is editing their deck's JSON file live while `fireside
validate --watch` or the TUI's live-reload is running. Their editor writes
the file in two steps (truncate, then write), so for a brief moment the
file on disk is incomplete or malformed JSON. The watcher must not crash,
must not discard the last known-good deck, and must recover automatically
and silently once the save completes and the file is valid JSON again.

**Why this priority**: This is a real, frequent occurrence (not a
contrived edge case) for anyone using the authoring loop shipped in P0 —
editors routinely write files non-atomically. It directly protects the
presenter-first experience: a crash or a silently-stuck-stale watcher mid-edit
would be a visible, confidence-breaking failure during exactly the workflow
this project built to be reliable.

**Independent Test**: Can be fully tested by driving `fireside-cli`'s
watcher against a file that is rewritten with an intentionally-truncated
intermediate write (simulating a non-atomic editor save) and asserting the
watcher reports the transient parse failure without crashing or losing the
prior valid state, then recovers on the next valid write.

**Acceptance Scenarios**:

1. **Given** a running watch session on a valid deck, **When** the deck
   file is overwritten with syntactically invalid JSON (simulating the
   mid-write moment of a non-atomic save), **Then** the watcher reports the
   parse error without crashing and continues showing the last valid state.
2. **Given** the watcher is in the error state from scenario 1, **When**
   the file is next overwritten with valid, well-formed JSON, **Then** the
   watcher recovers automatically and reflects the new content without
   requiring a restart.

---

### User Story 4 - Rendering stays correct with wide and multi-codepoint text (Priority: P3)

A presenter authors a deck containing emoji, CJK (Chinese/Japanese/Korean)
text, or other multi-codepoint characters in headings and multi-column
layouts. The TUI must measure and lay out this text using its true display
width, not its byte or codepoint count, so headings don't overflow their
box and columns stay aligned.

**Why this priority**: This is a rendering-correctness concern scoped to a
specific, less-common input class (non-Latin/wide-character content) rather
than a new capability — lower priority than the invariant and robustness
work above, but still worth closing given Fireside's terminal-rendering
surface is exactly where width-counting bugs hide silently until someone
hits them live.

**Independent Test**: Can be fully tested by adding scenario tests to
`fireside-tui`'s existing `TestBackend`-driven suite with headings and
column content containing emoji and CJK text, and asserting the rendered
cell layout matches true display width, no overflow or misalignment.

**Acceptance Scenarios**:

1. **Given** a heading containing emoji and/or CJK characters, **When** it
   is rendered at a fixed terminal width, **Then** it is measured and
   clipped (if needed) by true display width, matching the byte-for-byte
   behavior guarantee already given to ASCII headings.
2. **Given** a multi-column layout with wide characters in one column,
   **When** it is rendered, **Then** columns remain aligned exactly as they
   would with ASCII-only content of the same display width.

---

### Edge Cases

- What happens when a proptest-generated `Graph` includes edge-case values
  the existing hand-written fixtures never exercised (empty strings, zero
  and negative-adjacent numeric fields, empty containers)? The round-trip
  property must still hold, or the generator's value space must be
  documented as intentionally excluded with a reason.
- What happens when the watcher receives a rapid sequence of several
  invalid writes in a row before a valid one lands (not just one)? It must
  still recover on the first subsequent valid write, without needing the
  invalid streak to "settle."
- What happens when a container-nesting fixture is exactly one level over
  the documented limit versus far over it? Both must produce the same
  diagnostic (the rule is a threshold, not a graduated severity).
- What happens when the ~1,000-node performance fixture is run in a slower
  CI environment than reference hardware? The documented time budget must
  include enough margin that CI is not flaky; this is a design constraint
  on the budget, not a reason to skip the check.
- What happens to the property-test suite's run time in CI? It must stay
  within the existing CI budget (bounded case count per run), not turn into
  an unbounded fuzzing job.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST include a property-based test verifying that
  any structurally valid `Graph` value survives a serialize/deserialize
  round-trip unchanged.
- **FR-002**: The system MUST include a property-based test verifying that
  for any valid graph and any sequence of legal navigation operations, the
  session's reported history exactly matches the nodes actually visited, in
  order.
- **FR-003**: The system MUST include a property-based test verifying that
  for any valid graph and any sequence of legal navigation operations, every
  node the session reports as visited is a member of the graph's declared
  node set.
- **FR-004**: The system MUST define and document a maximum container
  nesting depth, enforced identically by both the Rust and Node validators.
- **FR-005**: The system MUST reject (or warn on, per the same
  severity convention used by existing structural rules) documents that
  exceed the documented nesting depth, with a diagnostic identifying the
  violation.
- **FR-006**: The shared conformance fixture corpus MUST include at least
  one fixture at the nesting-depth limit (accepted) and one fixture one
  level beyond it (rejected/warned), each verified to produce identical
  results from both validators.
- **FR-007**: The shared conformance fixture corpus MUST include a fixture
  representing a deck of approximately 1,000 nodes, with a documented,
  automatically-checked time budget for load-plus-validate time.
- **FR-008**: The `fireside-cli` file watcher MUST NOT crash or terminate
  the watch session when the watched file is momentarily invalid JSON or
  fails schema/semantic validation mid-edit.
- **FR-009**: The `fireside-cli` file watcher MUST continue displaying the
  last successfully loaded valid state while the watched file is in an
  invalid state, and MUST report the validation failure so the presenter
  gets feedback.
- **FR-010**: The `fireside-cli` file watcher MUST automatically recover
  and reflect new content, with no manual restart, as soon as the watched
  file becomes valid again.
- **FR-011**: The `fireside-tui` scenario test suite MUST include coverage
  asserting correct true-display-width measurement and layout for headings
  and multi-column content containing emoji and CJK characters.
- **FR-012**: The project's CI configuration MUST be reviewed against the
  plan's two asks — confirming license/advisory scanning runs on every
  relevant change, and confirming MSRV compliance is verified on every
  relevant change — with any gap found either closed or explicitly
  documented as already satisfied and why.
- **FR-013**: All new fixtures added to the shared conformance corpus MUST
  be consumed identically by both `fireside-engine`'s Rust fixture test and
  `protocol/run-fixtures.mjs`, preserving the existing cross-language parity
  guarantee.

### Key Entities

- **Property test**: A generated, randomized test run many times per test
  execution against a general invariant (e.g. "round-trips" or "history is
  truthful"), as opposed to a single hand-written example.
- **Conformance fixture**: A single JSON document under
  `protocol/fixtures/{valid,invalid}/` paired with its expected set of
  validator rule IDs in `protocol/fixtures.expected.json`, consumed
  identically by both validator implementations.
- **Nesting depth limit**: A newly-documented numeric bound on how many
  levels deep a `Container` block may nest inside other containers before a
  document is considered invalid.
- **Watcher state**: The `fireside-cli` file watcher's notion of "last known
  good deck" versus "current file is invalid," which must never collapse
  into a crash or silent staleness.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The property test suite catches 100% of a set of deliberately
  reintroduced historical/synthetic invariant violations (serialization
  asymmetry, history-tracking bugs) used to validate the suite itself before
  this feature is considered done.
- **SC-002**: Both validator implementations produce identical rule-ID
  verdicts on every fixture in the corpus, including all newly added
  robustness fixtures — zero divergences.
- **SC-003**: A ~1,000-node deck loads and validates in under 1 second on
  reference hardware, with the check automated so a future regression is
  caught rather than discovered live.
- **SC-004**: A presenter whose editor performs a non-atomic save while
  live-reload is active never sees the watcher crash or freeze on stale
  content; the tool always recovers within one subsequent valid save, with
  zero manual intervention.
- **SC-005**: Emoji- and CJK-bearing headings and columns render with zero
  overflow or misalignment regressions, verified by automated scenario
  tests rather than manual inspection alone.
- **SC-006**: CI enforces license/advisory scanning and MSRV compliance on
  every relevant change, with the project's actual CI configuration matching
  what is documented — zero silent gaps between claimed and enforced checks.

## Assumptions

- The container nesting depth limit is a new design decision, not dictated
  by the existing spec (which only says engines MAY impose one). A specific
  numeric value will be chosen during planning, comfortably above any
  realistic authored deck, and recorded in an ADR per constitution Principle
  I before implementation, consistent with how prior spec-adjacent decisions
  (e.g. ADR-007, ADR-009) were made.
- "Reference hardware" for the ~1,000-node performance budget means the
  environment the automated check actually runs in (CI runners), with
  enough margin that ordinary CI variance does not produce flaky failures.
- Property tests are additive dev-dependencies only (per the plan's explicit
  scope) — they do not change any production dependency, wire format, or
  public API surface, so no constitution crate-boundary amendment is
  expected for `fireside-core` or `fireside-engine`'s production
  dependency lists.
- The CI review (FR-012) may conclude that no code change is needed if the
  existing MSRV and cargo-deny jobs already satisfy the plan's ask — the
  requirement is to reach and document that decision, not to add tooling
  unconditionally.
- "Half-saved invalid JSON" is simulated deterministically in tests (writing
  a truncated/malformed payload directly) rather than depending on the
  timing behavior of any specific real text editor.
- Multi-codepoint/emoji/CJK rendering coverage (User Story 4) extends the
  existing `fireside-tui` `TestBackend` scenario suite; it does not require
  a new testing mechanism.
