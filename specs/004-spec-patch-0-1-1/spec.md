# Feature Specification: Protocol spec patch 0.1.1

**Feature Branch**: `004-spec-patch-0-1-1`

**Created**: 2026-07-12

**Status**: Draft

**Input**: User description: "Protocol spec patch 0.1.1: resolve the 7 audit ambiguities catalogued in the strategic plan per ADR-007, plus a shared conformance fixture corpus."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A third-party engine implementer reads an unambiguous spec (Priority: P1)

Someone building an independent Fireside engine (not the reference Rust/TUI
implementation) reads `protocol/main.tsp` and its companion docs to decide
how their engine should behave for seven behaviors the current spec text
leaves unstated or misstates: branch-option key uniqueness severity, the
empty traversal object, the `choose()` contract, ViewMode toggle
persistence, inline Markdown in list items, image overflow handling, and
history growth. Today they would have to guess, or reverse-engineer the
reference implementation's source, for six of these; for the seventh
(branch-key uniqueness) the docs actively mislead them into thinking it's
a warning when the reference implementation treats it as a hard error.

**Why this priority**: This is the core problem the audit identified — spec
drift undermines the protocol's entire value proposition as a portable,
implementation-independent format (Principle I). Without this fix, every
future third-party implementer re-derives the same behaviors by trial and
error or by reading Rust source they may not be able to read.

**Independent Test**: Read `protocol/main.tsp` and
`docs/src/content/docs/spec/` cover to cover with no access to the Rust or
Node source. Confirm all seven behaviors are stated unambiguously and match
what the reference implementation actually does (verified by cross-checking
against the reference source separately).

**Acceptance Scenarios**:

1. **Given** the published spec, **When** a reader looks up branch-option
   key collisions, **Then** the docs state it is an ERROR-severity check
   (not a recommendation), matching the reference validators.
2. **Given** the published spec, **When** a reader looks up what happens
   when a node's `traversal` is `{}` (an object with neither `next` nor
   `branch-point`), **Then** the docs state it is treated as a terminal
   node and SHOULD produce a diagnostic, since an empty object more likely
   indicates an authoring mistake than deliberate intent.
3. **Given** the published spec, **When** a reader looks up the `choose()`
   operation, **Then** the docs state the selected option MUST be
   validated as belonging to the current node's branch point, not an
   arbitrary caller-supplied object.
4. **Given** the published spec, **When** a reader looks up ViewMode
   toggling, **Then** the docs state whether the toggle persists across
   node transitions or resets.
5. **Given** the published spec, **When** a reader looks up `ListBlock`,
   **Then** the docs state whether `items` entries may contain inline
   Markdown.
6. **Given** the published spec, **When** a reader looks up `ImageBlock`
   width/height, **Then** the docs state the overflow-handling rule
   (clamp to content area) and are honest about which reference renderer
   capabilities currently implement it.
7. **Given** the published spec, **When** a reader looks up history
   growth, **Then** the docs state that engines MAY cap it.

---

### User Story 2 - A presenter gets a diagnostic for a likely-accidental empty traversal (Priority: P1)

A presenter hand-editing a deck's JSON (or working through the quick-edit
modal / Markdown import output) leaves a node's `traversal` as `{}` —
perhaps from copy-pasting a template or an incomplete edit. Today this is
silently treated as a terminal node with zero feedback, indistinguishable
from an intentional dead end. The presenter has no way to know whether
this was deliberate or a mistake until they present and discover the node
has no way forward.

**Why this priority**: Directly serves Principle II (presenter-first):
every blocked or surprising state should give feedback. This is the one
ambiguity in the set that requires new validator behavior, not just
documentation, so it's the behavioral heart of this patch.

**Independent Test**: Validate a document containing a node with
`"traversal": {}` using both `fireside validate` (Rust) and
`node protocol/validate.mjs` (Node). Confirm both report a warning naming
the affected node, and that the node is still treated as presentable
(warnings don't block presentation).

**Acceptance Scenarios**:

1. **Given** a document with a node whose `traversal` is `{}`, **When**
   validated, **Then** a warning is reported identifying the node and
   explaining that an empty traversal object behaves like an absent one
   (terminal) but is usually accidental.
2. **Given** a document with a node whose `traversal` is entirely absent
   (no `traversal` key at all), **When** validated, **Then** no warning is
   reported — an absent field is the normal, unambiguous way to declare a
   terminal node.
3. **Given** a document with a node whose `traversal` is `{"next": "x"}`
   or a valid branch-point, **When** validated, **Then** no empty-traversal
   warning is reported.

---

### User Story 3 - The project can prove, not just claim, that the Rust and Node validators agree (Priority: P2)

The project currently asserts that `fireside-engine`'s Rust validator and
`protocol/validate.mjs` implement "the same rules with the same names," but
nothing tests this beyond both files happening to use matching string
literals. A maintainer changing one validator (e.g., adding the new
`empty-traversal` rule) could silently drift from the other with no test
failure to catch it.

**Why this priority**: Lower priority than the presenter-facing behavior
change, but it's what makes every future rule change safe, and it directly
supports Priority 2's fixture-corpus goal from the strategic plan of
seeding a conformance suite a third-party engine can run.

**Independent Test**: Run the fixture corpus against both validators
independently (`cargo test` for Rust, an npm script for Node) with no
knowledge of the other's internals; confirm each fixture's expected
rule-id set is documented once and both validators are checked against it.

**Acceptance Scenarios**:

1. **Given** the fixture corpus, **When** each fixture is validated by the
   Rust validator, **Then** the exact set of rule-ids it fires matches the
   fixture's documented expectation.
2. **Given** the same fixture corpus, **When** each fixture is validated by
   `validate.mjs`, **Then** the exact set of rule-ids it fires matches the
   same documented expectation used in Scenario 1.
3. **Given** the corpus's single clean `valid.json` fixture, **When**
   validated by either validator, **Then** zero diagnostics are reported.

### Edge Cases

- A node's `traversal` given as the string shorthand (e.g.,
  `"traversal": "next-id"`) is never subject to the empty-traversal check —
  the check only applies to the object form, since the string form cannot
  be "empty."
- A branch-point with zero options already produces a separate
  `empty-branch-options`/schema-level error; this is a different case from
  an empty `Traversal` object and is not affected by this patch.
- `docs/examples/hello.json`, the canonical example, is not modified to
  use `"fireside-version": "0.1.1"` — this patch doesn't require any
  document to declare the new version, only permits it.
- Fixture files that are themselves malformed JSON or fail schema
  validation are out of scope for this corpus — it exercises Layer-2
  semantic rules only, on schema-valid documents.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The protocol version enum MUST gain a `"0.1.1"` value
  alongside the existing `"0.1.0"` value; existing `"0.1.0"` documents
  MUST remain valid.
- **FR-002**: Both the Rust validator (`fireside-engine`) and the Node
  validator (`protocol/validate.mjs`) MUST report a warning-severity
  diagnostic, under a shared rule identifier, when a node's `traversal` is
  present in object form but declares neither `next` nor `branch-point`.
- **FR-003**: This new diagnostic MUST NOT change engine traversal
  behavior — a node with `"traversal": {}` MUST continue to behave exactly
  as a node with no `traversal` field (terminal).
- **FR-004**: The published validation documentation MUST classify branch
  option key collisions as a required (error-severity) check, matching
  actual validator behavior.
- **FR-005**: The published validation documentation MUST list the new
  empty-traversal check under recommended (warning-severity) checks.
- **FR-006**: The published traversal documentation MUST state that a
  `choose()` implementation must reject or otherwise not accept an option
  that does not belong to the current node's branch point.
- **FR-007**: The published spec MUST state whether a presenter's runtime
  view-mode toggle persists across node transitions.
- **FR-008**: The published spec MUST state whether list-block items may
  contain inline Markdown formatting.
- **FR-009**: The published spec MUST state the intended overflow-handling
  rule for image width/height, and MUST distinguish that rule from what
  the current reference renderer actually implements today.
- **FR-010**: The published spec MUST state that engines are permitted,
  but not required, to cap history length.
- **FR-011**: A shared set of fixture documents MUST exist, each isolating
  one validation rule (including at least one fixture with zero
  diagnostics), with each fixture's expected outcome documented once in a
  way both validator test suites can reference.
- **FR-012**: Both the Rust and Node validator test suites MUST run every
  fixture in the shared corpus and assert the fired rule-id set matches
  the fixture's documented expectation.
- **FR-013**: Generated protocol schema artifacts MUST be regenerated and
  kept consistent with the updated protocol version enum.

### Key Entities

- **Fixture document**: A minimal Fireside JSON document designed to
  exercise exactly one validation rule (or zero rules, for the clean
  case), paired with the specific set of rule-ids validators are expected
  to report for it.
- **Empty-traversal diagnostic**: A new warning-severity finding reported
  for nodes whose `traversal` object form sets neither `next` nor
  `branch-point`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All seven ambiguities catalogued in the strategic plan's
  audit have a corresponding, unambiguous statement in the published spec
  or its appendices, each traceable to a specific requirement above.
- **SC-002**: A document with an empty `traversal` object produces an
  identical warning message tone and rule identifier from both the Rust
  and Node validators.
- **SC-003**: Every fixture in the shared corpus produces the exact same
  set of rule identifiers from both validators, with zero mismatches.
- **SC-004**: The canonical example document continues to validate with
  zero errors and only its existing informational diagnostic after all
  changes land.

## Assumptions

- The reference implementation's existing behavior for six of the seven
  ambiguities (branch-key uniqueness severity, empty-traversal terminal
  handling, choose() option scoping, ViewMode persistence, list-item
  Markdown, unbounded history) is accepted as correct and is being
  documented, not redesigned.
- The image width/height clamp rule is written as forward-looking
  guidance for the still-deferred real image rendering (P1 in the
  strategic plan); no renderer code changes ship in this feature.
- Protocol version 0.1.1 is additive only — no field, block kind, or
  required property changes shape, so no migration guidance is needed for
  existing documents.
- The fixture corpus's scope is the existing nine Layer-2 semantic rules
  (including the new empty-traversal rule); Layer-1 JSON Schema validation
  is not part of this corpus.
