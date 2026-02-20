# Active Context

## Current Focus

Execution of `.github/prompts/plan-fireside-tui-ux-initiative.prompt.md`
across phased milestones, while keeping protocol/docs aligned.

Current execution mode is TUI usability first:

- JSON-first configuration surfaces (`fireside.json`) for project config.
- No YAML/TOML expansion work for now unless explicitly re-approved.
- Export workflows (HTML/PDF) are deferred from active implementation scope.
- Competitive analysis is tracked in memory-bank, not user-facing docs.

## Recently Applied Direction

- Replaced `group` with `container` in protocol model and docs.
- Replaced `x-` prefix extension convention with explicit extension blocks:
  `kind: "extension"` + `type`.
- Standardized serialization guidance to `application/json`.
- Removed root `specs/` duplication by moving quick-reference docs into
  `docs/src/content/docs/reference/`.
- Enforced chapter ordering in docs sidebar: §1–§6 then appendices.

## Next Workstream

- Continue Phase 3/5/6 TUI slices from the UX initiative:
  graph exploration workflow polish, release usability refinements, and
  final integration cleanup.
- Keep task tracking explicitly phase-aligned so progress is easy to audit.

## Current Milestone Execution

- Implemented runtime handling for `traversal.after` in engine traversal to
  support branch rejoin behavior.
- Implemented project-directory edit support and shared project entry
  resolution using `fireside.json`.
- Implemented editor graph view overlay (`v`) with keyboard/mouse node
  navigation and jump-to-node integration.
- Expanded editor graph view with a richer ASCII edge-map topology view,
  mini-map side panel, and viewport controls (`PgUp`/`PgDn`, `Home`/`End`).
- Added multi-line branch fan-out edge rendering per node in graph overlay and
  synchronized overlay row hit-testing to variable-height entries.
- Added graph overlay shortcut to jump directly into presenter mode from the
  selected graph node (`p`).
- Added reverse handoff breadcrumb when entering editor from presenter (`e`),
  preserving current-node selection context in editor status.
- Refined graph fan-out topology rows with aligned branch connector labels and
  explicit per-edge connector glyphs for dense branch nodes.
- Upgraded in-app help overlay into categorized, mode-aware sections with
  active/dimmed shortcut states for presenter vs editor contexts.
- Added help overlay scrolling controls (`j/k`, arrows, page/home/end) and
  section jump keys (`1-6`) for small terminal sizes.
- Added compact help footer legend that maps section jump keys (`1-6`) and
  shows current scroll position for shortcut discoverability.
- Updated help footer section legend to be mode-sensitive so in-context
  section labels are emphasized and out-of-mode labels are dimmed.
- Completed a holistic release/usability audit across models, crates, and docs;
  full Rust checks and TypeSpec schema build are green, and docs build now
  uses a custom content-backed 404 route without Starlight missing-entry warnings.
- Next milestone slices: branch fan-out layout polish in graph map,
  graph-to-editor/presenter workflows, and release polish for TUI usability.
