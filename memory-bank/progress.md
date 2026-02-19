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

## In Progress

- Bring Rust reference implementation vocabulary fully in line with protocol
  naming where legacy terms remain.
- UX initiative phase execution from `.github/prompts/plan-fireside-tui-ux-initiative.prompt.md`
  with memory-bank milestone sync in progress.
- Phase 5/6 TUI usability slices: Mermaid hardening, settings polish,
  and hot-reload workflow.

## Known Follow-Up

- Validate generated schema pages and examples against the latest TypeSpec
  output after each protocol model change.
- Keep competitive analysis as internal context in memory-bank unless a public
  docs publication is explicitly requested.
- Keep export formats (HTML/PDF) deferred while TUI usability milestones are
  still in progress.
