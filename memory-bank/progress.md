# Progress

## Completed

- Protocol baseline normalized to `0.1.0`.
- TypeSpec versioning setup integrated.
- Docs site configured for static output with explicit spec chapter ordering.
- Core model updated to `container` + typed `extension` shape.
- Key spec, guide, schema-reference, ADR, and memory-bank pages updated.
- Root `specs/` quick-reference duplication removed; content moved to
  `docs/src/content/docs/reference/`.
- Runtime traversal now supports `after` branch-rejoin semantics in
  `TraversalEngine::next` with tests.
- CLI project flow now uses JSON project config (`fireside.json`) for
  open/edit resolution and scaffold generation.
- Presenter hot-reload loop and graph-reload safety behavior were validated with
  targeted tests, CLI tests, and crate-level clippy checks.
- Mermaid extension rendering was hardened (payload normalization, truncation
  safeguards, and preview overflow messaging) with focused tests.
- Settings handling was polished (key aliases, nested settings support,
  timeout bounds, and XDG-aware config path resolution) with tests.
- Editor mode now includes a graph-view overlay with ASCII edge summaries,
  keyboard/mouse navigation, and Enter/click jump-to-node behavior.
- Graph-view overlay now includes richer boxed edge-map rows, a mini-map
  side panel, and viewport navigation controls (`PgUp/PgDn/Home/End`).
- Graph-view edge map now renders multi-line branch fan-out edges per node,
  with paging and click-to-jump behavior aligned to variable row heights.
- Graph-view overlay now supports direct graph-to-presenter handoff (`p`) at
  the selected node, preserving traversal/editor selection sync.
- Presenter mode now hands off to editor mode (`e`) with a status breadcrumb
  that confirms current-node context (`Presenter → editor @ node #N`).
- Graph fan-out rows now align edge-kind and target labels with explicit
  connector glyphs (`├╼`/`└╼`) for clearer branch topology scanning.
- Help overlay now provides categorized, mode-aware shortcut sections with
  active/dimmed entries across presenter and editor modes.
- Help overlay now supports scroll navigation and section jumps (`1-6`) so
  full shortcut docs remain usable on compact terminals.
- Help overlay now includes a footer section index legend and scroll indicator
  to improve discoverability of jump/navigation controls.
- Help footer section legend now reflects mode context by emphasizing active
  sections and dimming out-of-mode sections.
- Holistic release/usability audit now passes end-to-end for Rust workspace
  checks and TypeSpec schema generation; docs static build is clean after
  resolving Starlight 404 entry handling.
- Crate READMEs written for all four workspace members (`fireside-core`,
  `fireside-engine`, `fireside-tui`, `fireside-cli`): teaching-quality Rust
  documentation with architecture rationale, annotated code examples, module
  maps, and dependency tables.
- Root `README.md` updated with full feature set documentation: all CLI
  subcommands with examples, complete keybinding tables for presenter and
  editor modes, iTerm2 theme import flow, and cross-links to crate READMEs.

## In Progress

- Bring Rust reference implementation vocabulary fully in line with protocol
  naming where legacy terms remain.
- UX initiative phase execution from `.github/prompts/plan-fireside-tui-ux-initiative.prompt.md`
  with memory-bank milestone sync in progress.
- Phase 5/6 TUI usability slices: release polish and remaining milestone
  integration cleanup.

## Known Follow-Up

- Validate generated schema pages and examples against the latest TypeSpec
  output after each protocol model change.
- Keep competitive analysis as internal context in memory-bank unless a public
  docs publication is explicitly requested.
- Keep export formats (HTML/PDF) deferred while TUI usability milestones are
  still in progress.
