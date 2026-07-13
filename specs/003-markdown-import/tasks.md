# Tasks: Markdown Authoring Frontend (`fireside import`)

**Input**: Design documents from `/specs/003-markdown-import/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md,
contracts/cli-import.md, quickstart.md

**Tests**: included — the constitution's Test Discipline principle (VII)
requires tests at the correct layer for every feature. This feature's
layer is unit tests directly against the pure `import::import(&str)`
function (no filesystem needed, per contracts/cli-import.md) plus one CLI
wiring test in `cli_e2e.rs`.

**Organization**: tasks are grouped by user story (spec.md priorities, both
US1 and US2 are P1; US3 is P2). Most implementation tasks touch the new
`crates/fireside-cli/src/import.rs` module; `[P]` is reserved for tasks in
a genuinely separate file with no dependency on unfinished work.

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup

- [X] T001 Run `cargo test --workspace` to confirm a clean baseline before
      touching `fireside-cli`
- [X] T002 Add `pulldown-cmark = "0.13"` to `crates/fireside-cli/Cargo.toml`
      under `[dependencies]` (already permitted per constitution v1.1.0 /
      ADR-006); run `cargo build -p fireside-cli` to confirm it resolves
      and builds clean under the workspace's pinned toolchain

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: shared plumbing every user story needs — the module skeleton,
slug helper, frontmatter parsing, and the first-pass node-id collection
that both linear traversal (US1) and forward-referencing branch targets
(US2) depend on.

**⚠️ CRITICAL**: no user-story task can begin until this phase is complete.

- [X] T003 In `crates/fireside-cli/src/main.rs`, extract the slug-building
      logic already inline in `new_deck` (lowercase, map non-alphanumeric
      to `-`, split and rejoin filtering empty segments) into `pub(crate)
      fn slugify(text: &str) -> String`; update `new_deck` to call it.
      `cargo test -p fireside-cli` must stay green — this is a pure
      refactor, no behavior change (research.md §6)
- [X] T004 Create `crates/fireside-cli/src/import.rs` with: `pub enum
      ImportError { NoHeadings, NestedList { line: usize }, UnresolvedBranchTarget
      { line: usize, target: String, section: String }, ContentAfterBranch
      { line: usize, section: String }, MalformedBranchLine { line: usize,
      section: String }, ValidationFailed(Vec<fireside_engine::Diagnostic>)
      }` plus a `Display` impl producing one human-readable line per
      variant naming the location (data-model.md `ImportError`); add `mod
      import;` to `crates/fireside-cli/src/main.rs`
- [X] T005 In `crates/fireside-cli/src/import.rs`, implement `fn
      split_frontmatter(source: &str) -> (Option<Frontmatter>, &str)`
      per research.md §4: detect a `---`-delimited block at byte offset 0,
      hand-parse flat `key: value` lines into a `struct Frontmatter { title:
      Option<String>, author: Option<String>, date: Option<String>,
      description: Option<String>, fireside_version: Option<String> }`
      (unrecognized keys ignored), and return the remaining source
      (frontmatter block excluded) for Markdown parsing
- [X] T006 [P] Add unit tests in `crates/fireside-cli/src/import.rs`'s
      `#[cfg(test)]` module for `split_frontmatter`: a file with
      title/author frontmatter returns both populated and the correct
      remaining source; a file with no frontmatter returns `None` and the
      full source unchanged; a file with an unrecognized frontmatter key
      ignores it without error
- [X] T007 In `crates/fireside-cli/src/import.rs`, implement `fn
      collect_node_ids(source: &str) -> Result<Vec<(String, String)>,
      ImportError>` (heading text, slugified id, deduplicated via
      `slugify` + a numeric suffix on collision) by a first pass over
      `pulldown_cmark::Parser::new(source)` collecting every `##`
      heading's inner text in document order; returns `Err(NoHeadings)` if
      none are found (FR-022)
- [X] T008 [P] Add unit tests for `collect_node_ids`: three distinct `##`
      headings produce three ids in order; two headings with identical
      text produce distinct ids (FR-005, second gets a `-2` suffix); a
      document with zero `##` headings returns `Err(NoHeadings)`

**Checkpoint**: `cargo test --workspace` green, `cargo clippy --workspace
--all-targets` silent. Foundation compiles and is unit-tested; nothing
user-visible yet (no CLI wiring, no full `import()` function).

---

## Phase 3: User Story 1 - Turn a Markdown talk into a presentable deck (Priority: P1) 🎯 MVP

**Goal**: `fireside import talk.md` produces a deck file with one node per
`##` heading, correct content blocks, linear traversal, and passes
`fireside validate`.

**Independent Test**: run `fireside import` on a three-section Markdown
file mixing prose/code/list content, confirm the output validates and
presents with three nodes in order (quickstart.md Scenario 1).

### Implementation for User Story 1

- [X] T009 [US1] In `crates/fireside-cli/src/import.rs`, implement `fn
      convert_section(source: &str, events: &[(pulldown_cmark::Event,
      Range<usize>)]) -> Result<Vec<fireside_core::ContentBlock>,
      ImportError>` per research.md §3: dispatch each event — H3-H6
      headings to `ContentBlock::Heading`, paragraphs to `ContentBlock::Text`
      (source-sliced per research.md §1), non-`branch` fenced code to
      `ContentBlock::Code`, `Event::Rule` to `ContentBlock::Divider`,
      images to `ContentBlock::Image`. List and branch-fence handling are
      T010/T015 — leave a `todo!()`-free placeholder path that just skips
      `List`/branch fences for now if needed to keep this task's diff
      focused, resolved by the very next task
- [X] T010 [US1] Extend `convert_section` in
      `crates/fireside-cli/src/import.rs` to handle `Tag::List`/`Tag::Item`:
      track list nesting depth via a counter; a flat list becomes
      `ContentBlock::List` (`ordered` from `List(start)`'s
      `Some`/`None`, items source-sliced per `Item`); entering a `List`
      while already inside an `Item` returns `Err(NestedList { line })`
      (FR-012) instead of a block
- [X] T011 [US1] Implement `fn split_sections(source: &str, node_ids:
      &[(String, String)]) -> Vec<Section>` in
      `crates/fireside-cli/src/import.rs`: the second pass — re-walks
      `pulldown_cmark::Parser::new_ext(source, options).into_offset_iter()`,
      splitting on `##` heading boundaries (research.md §2) and calling
      `convert_section` for each section's inner events; `Section` per
      data-model.md (`heading_text`, `id` from the T007 id list by
      position, `blocks`, `branch: None` for now — branch parsing is
      Phase 4)
- [X] T012 [US1] Implement `fn build_graph(frontmatter: Option<Frontmatter>,
      sections: Vec<Section>) -> fireside_core::Graph` in
      `crates/fireside-cli/src/import.rs`: maps `Frontmatter` fields onto
      `Graph`'s metadata (defaulting `fireside-version` to the current
      protocol version if absent), and for each section without a `branch`
      sets `traversal` to `TraversalSpec::Target(next section's id)` (or
      `None` for the last section) per FR-020
- [X] T013 [US1] Implement `pub fn import(source: &str) -> Result<Graph,
      ImportError>` in `crates/fireside-cli/src/import.rs` wiring
      T005/T007/T011/T012 together, then calling
      `fireside_engine::validate` on the built graph and returning
      `Err(ValidationFailed(diags))` if any diagnostic is
      `Severity::Error` (FR-021), else `Ok(graph)`
- [X] T014 [US1] Add unit tests in `crates/fireside-cli/src/import.rs` for
      `import()`: quickstart.md Scenario 1's three-section fixture produces
      a `Graph` with three nodes in order, correct content-block kinds per
      section, linear `next` traversal, and the last node terminal;
      frontmatter title/author land on `Graph.title`/`Graph.author`; an H1
      fallback title is used when no frontmatter title is present (FR-007)
- [X] T015 [US1] Add `Command::Import { input: PathBuf, output:
      Option<PathBuf> }` to the `Command` enum in
      `crates/fireside-cli/src/main.rs`; implement `fn import_file(input:
      &Path, output: Option<&Path>) -> Result<()>`: derive the default
      output path (input with `.fireside.json` extension, FR-002), refuse
      via `bail!` if it already exists with a message matching `new_deck`'s
      "already exists — pick another name" wording (FR-003), read the
      input file, call `import::import`, map `Err(ImportError)` to
      `eprintln!` + `std::process::exit(1)` (matching `validate_file`'s
      pattern), and on `Ok(graph)` write `graph.to_json_pretty()` to the
      output and print where it landed (FR-023); wire the new match arm in
      `main()`

**Checkpoint**: User Story 1 is fully functional and independently testable
via `quickstart.md` Scenario 1.

---

## Phase 4: User Story 2 - Give the audience a choice, from Markdown (Priority: P1)

**Goal**: a `branch` fence inside a node's section becomes a branch-point
with correctly resolved targets, including forward references; malformed
or unresolved branch syntax is rejected with a location, not silently
mishandled.

**Independent Test**: import a Markdown file where one section has a
`branch` fence linking to two later sections, confirm the resulting node's
branch-point options target the correct ids in order and the deck presents
the choice correctly (quickstart.md Scenario 2).

### Implementation for User Story 2

- [X] T016 [US2] In `crates/fireside-cli/src/import.rs`, implement `fn
      parse_branch_fence(body: &str, fence_line: usize) ->
      Result<BranchDeclaration, ImportError>` per research.md §5: first
      non-list line (if any) is the prompt; each subsequent non-blank line
      must match `- [label](#target)` with an optional trailing `` `key` ``,
      hand-parsed (find `[`/`]`, `(#`/`)`, optional backtick pair); a line
      matching neither the prompt nor the option shape returns
      `Err(MalformedBranchLine { line, section })`
- [X] T017 [US2] Extend `convert_section`/`split_sections` in
      `crates/fireside-cli/src/import.rs` so a fenced block with info
      string exactly `"branch"` is routed to `parse_branch_fence` instead
      of becoming a `ContentBlock::Code`, sets `Section.branch =
      Some(BranchDeclaration)`, and sets a `branch_seen` flag; any
      further content-producing event in the same section after that flag
      is set returns `Err(ContentAfterBranch { line, section })` (FR-019)
- [X] T018 [US2] Implement `fn resolve_branch_targets(sections: &mut
      [Section], node_ids: &[(String, String)]) -> Result<(), ImportError>`
      in `crates/fireside-cli/src/import.rs`: for each section's
      `BranchDeclaration`, resolve each `BranchOptionSource.target_slug`
      against `node_ids`, returning `Err(UnresolvedBranchTarget { line,
      target, section })` on the first miss (FR-018); on success, build
      the section's `BranchPoint`/`BranchOption`s in link order (FR-016,
      FR-017)
- [X] T019 [US2] Wire T018 into `build_graph`
      (`crates/fireside-cli/src/import.rs`, extending T012): a section
      with a resolved `BranchDeclaration` gets
      `TraversalSpec::Rules(Traversal { next: None, branch_point:
      Some(BranchPoint { prompt, options }) })` instead of the linear
      `next` target
- [X] T020 [US2] Add unit tests in `crates/fireside-cli/src/import.rs` for
      the full branch path: quickstart.md Scenario 2's fixture produces a
      branch-point with the correct prompt and two options (labels,
      targets, and the second option's `key` from its backtick suffix);
      Scenario 3's fixture (target renamed to a nonexistent slug) returns
      `Err(UnresolvedBranchTarget)` naming the bad link's line; a section
      with a paragraph after its `branch` fence returns
      `Err(ContentAfterBranch)`; a branch-fence line missing the `(#...)`
      part returns `Err(MalformedBranchLine)`

**Checkpoint**: User Stories 1 and 2 both independently pass;
`quickstart.md` Scenarios 2–3 confirmed.

---

## Phase 5: User Story 3 - Know exactly what didn't come through (Priority: P2)

**Goal**: nested lists and successful-but-incomplete imports give the
presenter specific, actionable feedback rather than silent data loss.

**Independent Test**: import a Markdown file with a nested bullet list and
confirm the failure names the nested list's line (quickstart.md
Scenario 4); import a normal file and confirm the success message notes
what v1 import doesn't carry over.

### Implementation for User Story 3

- [X] T021 [US3] Add a dedicated unit test in
      `crates/fireside-cli/src/import.rs` for T010's nested-list guard:
      quickstart.md Scenario 4's fixture (`- Top item` / `  - Nested item`)
      returns `Err(NestedList { line })` naming the nested item's line —
      this is a regression test for behavior already implemented in Phase
      3 (US1), giving it its own coverage per this story's priority
- [X] T022 [US3] In `crates/fireside-cli/src/main.rs`'s `import_file` (from
      T015), after a successful write, print a fixed one-line note listing
      what v1 Markdown import does not carry over (containers/columns,
      speaker notes, per-node view-mode/transition — per ADR-006 and
      FR-023), so every successful import restates the boundary rather
      than presenters discovering it by omission

**Checkpoint**: all three user stories independently pass; `quickstart.md`
Scenario 4 and the success-summary half of Scenario 5 confirmed.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T023 [P] Add one integration test to
      `crates/fireside-cli/tests/cli_e2e.rs` exercising the `import` verb
      end to end with real files: a fixture Markdown file imports to the
      default derived output path (FR-002) and the result validates via a
      follow-up `fireside validate` invocation; a second run against the
      same output path fails with the "already exists" message (FR-003)
      and does not modify the existing file
- [X] T024 Manually walk through `quickstart.md` Scenarios 1–5 by running
      the built `fireside` binary directly (no TUI is involved in this
      feature, so no tmux smoke test is needed — `import` is a plain CLI
      verb like `validate`), confirming every success and failure message
      reads clearly to a non-technical presenter
- [X] T025 Run `cargo test --workspace` and
      `cargo clippy --workspace --all-targets` and fix any findings
- [X] T026 [P] Run `graphify update .` to refresh the knowledge graph after
      the code change, per the constitution's Operational Constraints
- [X] T027 [P] Update the Progress Log in
      `.claude/plans/2026-07-12-strategic-improvement-plan.md`, checking
      off "P0 Stage D — Markdown authoring frontend" with the commit(s)
      and date

---

## Dependencies & Execution Order

- **Setup (Phase 1)**: no dependencies.
- **Foundational (Phase 2)**: depends on Setup; blocks every user story.
- **User Story 1 (Phase 3)**: depends on Foundational only. This is the
  MVP — it delivers a presentable deck from plain (non-branching)
  Markdown, which is already useful on its own.
- **User Story 2 (Phase 4)**: depends on Foundational and on User Story 1's
  `convert_section`/`split_sections`/`build_graph` (T009, T011, T012)
  existing, since branch handling extends the same functions rather than
  duplicating the section-walking machinery.
- **User Story 3 (Phase 5)**: depends on Foundational and on User Story 1's
  nested-list guard (T010) existing — it adds a dedicated regression test
  and a cross-cutting success-message change.
- **Polish (Phase 6)**: depends on all desired user stories being complete.

### Parallel Opportunities

- T006 and T008 (unit tests for independent Foundational functions) can run
  in parallel with each other once their respective implementation tasks
  (T005, T007) land.
- T023, T026, and T027 touch files untouched by the rest of Phase 6 — safe
  to run in parallel with each other once Phase 5 is done.
- All other tasks build on the same `import.rs` functions incrementally
  and are effectively sequential within their phase.

---

## Implementation Strategy

### MVP First (User Story 1 only)

1. Phase 1 (Setup) → Phase 2 (Foundational) → Phase 3 (User Story 1).
2. **Stop and validate**: run `quickstart.md` Scenario 1. This alone lets a
   presenter author a non-branching talk entirely in Markdown — the
   authoring-gap headline the strategic plan names.

### Incremental Delivery

1. Setup + Foundational → node-id collection and frontmatter parsing
   provably correct, nothing user-visible yet.
2. + User Story 1 → linear Markdown-to-deck import works end to end (MVP).
3. + User Story 2 → branching decks authored entirely in Markdown.
4. + User Story 3 → nested-list and success-summary feedback.
5. + Polish → CLI wiring e2e test, lint/test pass, manual quickstart walk,
   knowledge graph refresh, strategic-plan progress log update.
