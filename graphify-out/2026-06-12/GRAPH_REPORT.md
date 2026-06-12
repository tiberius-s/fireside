# Graph Report - fireside  (2026-06-12)

## Corpus Check
- 108 files · ~56,207 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1152 nodes · 1550 edges · 100 communities (85 shown, 15 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 36 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ad7328c9`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_TUI App Helpers|TUI App Helpers]]
- [[_COMMUNITY_CLI Commands|CLI Commands]]
- [[_COMMUNITY_Block Rendering Types|Block Rendering Types]]
- [[_COMMUNITY_Agent & Skill Governance|Agent & Skill Governance]]
- [[_COMMUNITY_TUI App Core|TUI App Core]]
- [[_COMMUNITY_Editor Detail Pane|Editor Detail Pane]]
- [[_COMMUNITY_Graph Overlay UI|Graph Overlay UI]]
- [[_COMMUNITY_Editor Navigation|Editor Navigation]]
- [[_COMMUNITY_Design Templates|Design Templates]]
- [[_COMMUNITY_Editor Interaction Tests|Editor Interaction Tests]]
- [[_COMMUNITY_Theme & Content Types|Theme & Content Types]]
- [[_COMMUNITY_Traversal Engine|Traversal Engine]]
- [[_COMMUNITY_Protocol Package|Protocol Package]]
- [[_COMMUNITY_Docs Site Package|Docs Site Package]]
- [[_COMMUNITY_Presentation Session|Presentation Session]]
- [[_COMMUNITY_Keybindings Config|Keybindings Config]]
- [[_COMMUNITY_Window Chrome UI|Window Chrome UI]]
- [[_COMMUNITY_Graph Loading Tests|Graph Loading Tests]]
- [[_COMMUNITY_Docs Site Pages|Docs Site Pages]]
- [[_COMMUNITY_Graph Loader|Graph Loader]]
- [[_COMMUNITY_Protocol TS Config|Protocol TS Config]]
- [[_COMMUNITY_Protocol JS Validator|Protocol JS Validator]]
- [[_COMMUNITY_Project Cheat Sheets|Project Cheat Sheets]]
- [[_COMMUNITY_Markdown Lint Config|Markdown Lint Config]]
- [[_COMMUNITY_Hello Example Document|Hello Example Document]]
- [[_COMMUNITY_Crate READMEs|Crate READMEs]]
- [[_COMMUNITY_Graph & Command Docs|Graph & Command Docs]]
- [[_COMMUNITY_Editor Command Tests|Editor Command Tests]]
- [[_COMMUNITY_Layout Rendering|Layout Rendering]]
- [[_COMMUNITY_TypeSpec Linter Rules|TypeSpec Linter Rules]]
- [[_COMMUNITY_Timeline UI|Timeline UI]]
- [[_COMMUNITY_TEA Architecture Docs|TEA Architecture Docs]]
- [[_COMMUNITY_Documentation Standards|Documentation Standards]]
- [[_COMMUNITY_Validate Command|Validate Command]]
- [[_COMMUNITY_Command History Tests|Command History Tests]]
- [[_COMMUNITY_Claude Hooks Settings|Claude Hooks Settings]]
- [[_COMMUNITY_Claude Local Permissions|Claude Local Permissions]]
- [[_COMMUNITY_Docs TS Config|Docs TS Config]]
- [[_COMMUNITY_Git Hooks Install|Git Hooks Install]]
- [[_COMMUNITY_Docs Content Config|Docs Content Config]]
- [[_COMMUNITY_Model Module|Model Module]]
- [[_COMMUNITY_Render Module|Render Module]]
- [[_COMMUNITY_TypeSpec Diagnostics|TypeSpec Diagnostics]]
- [[_COMMUNITY_Community 92|Community 92]]
- [[_COMMUNITY_Community 93|Community 93]]
- [[_COMMUNITY_Community 96|Community 96]]
- [[_COMMUNITY_Community 98|Community 98]]
- [[_COMMUNITY_Community 99|Community 99]]
- [[_COMMUNITY_Community 100|Community 100]]
- [[_COMMUNITY_Community 101|Community 101]]
- [[_COMMUNITY_Community 102|Community 102]]
- [[_COMMUNITY_Community 103|Community 103]]
- [[_COMMUNITY_Community 104|Community 104]]
- [[_COMMUNITY_Community 105|Community 105]]
- [[_COMMUNITY_Community 106|Community 106]]
- [[_COMMUNITY_Community 107|Community 107]]
- [[_COMMUNITY_Community 108|Community 108]]
- [[_COMMUNITY_Community 109|Community 109]]
- [[_COMMUNITY_Community 110|Community 110]]
- [[_COMMUNITY_Community 111|Community 111]]
- [[_COMMUNITY_Community 112|Community 112]]
- [[_COMMUNITY_Community 113|Community 113]]
- [[_COMMUNITY_Community 114|Community 114]]
- [[_COMMUNITY_Community 115|Community 115]]
- [[_COMMUNITY_Community 116|Community 116]]
- [[_COMMUNITY_Community 117|Community 117]]
- [[_COMMUNITY_Community 118|Community 118]]
- [[_COMMUNITY_Community 119|Community 119]]
- [[_COMMUNITY_Community 120|Community 120]]
- [[_COMMUNITY_Community 121|Community 121]]
- [[_COMMUNITY_Community 122|Community 122]]
- [[_COMMUNITY_Community 123|Community 123]]
- [[_COMMUNITY_Community 125|Community 125]]
- [[_COMMUNITY_Community 126|Community 126]]
- [[_COMMUNITY_Community 127|Community 127]]
- [[_COMMUNITY_Community 128|Community 128]]
- [[_COMMUNITY_Community 129|Community 129]]
- [[_COMMUNITY_Community 130|Community 130]]
- [[_COMMUNITY_Community 131|Community 131]]
- [[_COMMUNITY_Community 132|Community 132]]
- [[_COMMUNITY_Community 133|Community 133]]
- [[_COMMUNITY_Community 135|Community 135]]
- [[_COMMUNITY_Community 136|Community 136]]
- [[_COMMUNITY_Community 138|Community 138]]
- [[_COMMUNITY_Community 139|Community 139]]
- [[_COMMUNITY_Community 140|Community 140]]
- [[_COMMUNITY_Community 141|Community 141]]
- [[_COMMUNITY_Community 142|Community 142]]
- [[_COMMUNITY_Community 143|Community 143]]
- [[_COMMUNITY_Community 144|Community 144]]
- [[_COMMUNITY_Community 145|Community 145]]
- [[_COMMUNITY_Community 146|Community 146]]
- [[_COMMUNITY_Community 148|Community 148]]
- [[_COMMUNITY_Community 149|Community 149]]
- [[_COMMUNITY_Community 151|Community 151]]
- [[_COMMUNITY_Community 153|Community 153]]
- [[_COMMUNITY_Community 160|Community 160]]
- [[_COMMUNITY_Community 163|Community 163]]

## God Nodes (most connected - your core abstractions)
1. `App` - 27 edges
2. `Session` - 20 edges
3. `Diagnostic` - 19 edges
4. `render_block()` - 17 edges
5. `Graphify Skill Pipeline` - 17 edges
6. `app()` - 16 edges
7. `screen()` - 15 edges
8. `Node` - 14 edges
9. `hello_session()` - 14 edges
10. `validate()` - 14 edges

## Surprising Connections (you probably didn't know these)
- `TEA (The Elm Architecture) Discipline` --semantically_similar_to--> `Core Runtime Guarantees`  [INFERRED] [semantically similar]
  crates/fireside-tui/README.md → docs/src/content/docs/spec/appendix-engine-guidelines.md
- `Honesty Rules` --semantically_similar_to--> `Context7 Documentation Expert Agent`  [INFERRED] [semantically similar]
  .claude/skills/graphify/SKILL.md → .github/agents/context7.agent.md
- `Command / CommandHistory (undo-redo)` --semantically_similar_to--> `Core Runtime Guarantees`  [INFERRED] [semantically similar]
  crates/fireside-engine/README.md → docs/src/content/docs/spec/appendix-engine-guidelines.md
- `Layout enum (12 variants)` --semantically_similar_to--> `ViewMode enum (default, fullscreen)`  [INFERRED] [semantically similar]
  crates/fireside-core/README.md → docs/src/content/docs/spec/data-model.md
- `Transition enum (8 variants, core)` --semantically_similar_to--> `Transition enum (none, fade — protocol)`  [INFERRED] [semantically similar]
  crates/fireside-core/README.md → docs/src/content/docs/spec/data-model.md

## Import Cycles
- 1-file cycle: `crates/fireside-cli/src/main.rs -> crates/fireside-cli/src/main.rs`
- 1-file cycle: `crates/fireside-tui/src/app.rs -> crates/fireside-tui/src/app.rs`
- 1-file cycle: `crates/fireside-cli/tests/cli_e2e.rs -> crates/fireside-cli/tests/cli_e2e.rs`
- 1-file cycle: `crates/fireside-core/src/model/mod.rs -> crates/fireside-core/src/model/mod.rs`
- 1-file cycle: `crates/fireside-engine/src/session.rs -> crates/fireside-engine/src/session.rs`
- 1-file cycle: `crates/fireside-engine/src/validation.rs -> crates/fireside-engine/src/validation.rs`
- 1-file cycle: `crates/fireside-tui/src/lib.rs -> crates/fireside-tui/src/lib.rs`
- 1-file cycle: `crates/fireside-tui/src/render/mod.rs -> crates/fireside-tui/src/render/mod.rs`
- 1-file cycle: `crates/fireside-tui/src/render/blocks.rs -> crates/fireside-tui/src/render/blocks.rs`
- 1-file cycle: `crates/fireside-tui/src/render/markdown.rs -> crates/fireside-tui/src/render/markdown.rs`
- 1-file cycle: `crates/fireside-tui/src/theme.rs -> crates/fireside-tui/src/theme.rs`

## Hyperedges (group relationships)
- **Graphify Two-Track Extraction Pipeline** — graphify_skill_ast_extraction, graphify_skill_semantic_extraction, graphify_skill_extraction_cache, references_extraction_spec_subagent_prompt [EXTRACTED 1.00]
- **Fireside Engineering Constraint Set (MSRV, boundaries, TEA, errors)** — agents_rust_expert_agent_msrv_rule, agents_rust_expert_agent_crate_boundary_rules, agents_rust_expert_agent_tea_invariant, agents_rust_expert_agent_error_stratification, instructions_rust_best_practices_instructions_rust_best_practices, adr_skill_adr_skill, workflows_rust_msrv_job [EXTRACTED 1.00]
- **Fireside CI Quality Gates** — workflows_rust_rust_ci, workflows_audit_security_audit, workflows_docs_docs_ci, workflows_models_protocol_ci [EXTRACTED 1.00]
- **Four Traversal Operations (Next, Choose, Goto, Back)** — spec_traversal_next, spec_traversal_choose, spec_traversal_goto, spec_traversal_back, spec_traversal_history_invariants [EXTRACTED 1.00]
- **Rust Reference Implementation Layering (core -> engine -> tui -> cli)** — fireside_core_readme_fireside_core, fireside_engine_readme_fireside_engine, fireside_tui_readme_fireside_tui, fireside_cli_readme_fireside_cli [EXTRACTED 1.00]
- **Fireside Document Data Model** — spec_data_model_graph, spec_data_model_node, spec_data_model_contentblock, spec_data_model_traversal, spec_data_model_branchpoint, spec_data_model_branchoption, spec_data_model_nodeid [EXTRACTED 1.00]

## Communities (100 total, 15 thin omitted)

### Community 0 - "TUI App Helpers"
Cohesion: 0.22
Nodes (8): ADR-004: Presenter-first rewrite against protocol 0.1.0, Consequences, Context, Decision, Negative or Trade-offs, Neutral / Follow-up, Positive, Status

### Community 1 - "CLI Commands"
Cohesion: 0.15
Nodes (25): Command, Graph, Option, PathBuf, Result, Command, PathBuf, Path (+17 more)

### Community 2 - "Block Rendering Types"
Cohesion: 0.25
Nodes (9): CLI Event Loop, Terminal Lifecycle, Action enum (~35 variants), App struct (TUI state), AppMode (Presenting/Editing/GotoNode/Quitting), Markdown Rendering, Rendering Pipeline, Syntax Highlighting (+1 more)

### Community 3 - "Agent & Skill Governance"
Cohesion: 0.10
Nodes (27): Graphify Slash Command Trigger, AST Structural Extraction, Community Detection and Labeling, Token Cost Tracking, Semantic Extraction Cache, Fast Path Query on Existing Graph, Graphify Skill Pipeline, Semantic Extraction via Parallel Subagents (+19 more)

### Community 4 - "TUI App Core"
Cohesion: 0.09
Nodes (21): KeyCode, Option, Self, String, ViewMode, App, Graph, Result (+13 more)

### Community 7 - "Editor Navigation"
Cohesion: 0.26
Nodes (25): ContainerLayout, ContentBlock, Line, Option, String, Tokens, Vec, center() (+17 more)

### Community 9 - "Editor Interaction Tests"
Cohesion: 0.16
Nodes (34): Graph, HashSet, Node, Option, Result, Self, String, Vec (+26 more)

### Community 11 - "Theme & Content Types"
Cohesion: 0.38
Nodes (4): Self, Style, Default, Tokens

### Community 14 - "Traversal Engine"
Cohesion: 0.22
Nodes (8): Command / CommandHistory (undo-redo), Appendix B — Engine Guidelines, Container Rendering Guidance, Core Runtime Guarantees, Engine boundaries, Input and Error Strategy, Back Operation, History Invariants

### Community 16 - "Protocol Package"
Cohesion: 0.08
Nodes (24): author, default, dependencies, @typespec/compiler, @typespec/json-schema, @typespec/versioning, description, devDependencies (+16 more)

### Community 19 - "Docs Site Package"
Cohesion: 0.09
Nodes (21): dependencies, astro, astro-mermaid, @astrojs/starlight, mermaid, sharp, devDependencies, @astrojs/check (+13 more)

### Community 21 - "Presentation Session"
Cohesion: 0.10
Nodes (26): BranchPoint, Graph, HashSet, Node, NodeDefaults, NodeId, Option, Result (+18 more)

### Community 26 - "Keybindings Config"
Cohesion: 0.06
Nodes (32): Context7 Documentation Expert, Core Philosophy, Critical Operating Principles, 🚨 CRITICAL RULE - READ FIRST, Documentation Retrieval Strategy, Error Prevention Checklist, ✅ Every Response Should:, Example 1: Simple API Question (+24 more)

### Community 27 - "Window Chrome UI"
Cohesion: 0.06
Nodes (32): 10. Inappropriate Intimacy, 1. Long Method/Function, 2. Duplicated Code, 3. Large Class/Module, 4. Long Parameter List, 5. Feature Envy, 6. Primitive Obsession, 7. Magic Numbers/Strings (+24 more)

### Community 29 - "Graph Loading Tests"
Cohesion: 0.08
Nodes (23): 1. Executive Summary, 1. Executive Summary, 2. User Experience & Functionality, 2. User Stories, 3. AI System Architecture, 3. AI System Requirements (If Applicable), 4. Evaluation, 4. Technical Specifications (+15 more)

### Community 32 - "Docs Site Pages"
Cohesion: 0.08
Nodes (33): 404 Page, Docs Landing Page / Specification Map, Fireside Docs Site (Astro + Starlight), TypeSpec Emitter Config (tspconfig.yaml), Build, Fireside, Fireside Protocol, License (+25 more)

### Community 34 - "Graph Loader"
Cohesion: 0.09
Nodes (22): 10. Honest Caveats, 1. Current State Summary, 2. Spec Gaps and Drift, 3. Rust Implementation Progress by Crate, 4. CLI/TUI UX Assessment, 5. Context7 Verification (new since first draft), 6. Phased Roadmap (3–6 weeks, small PRs), 7. AI & Agentic Workflow Audit (`.github/` + `.claude/`) (+14 more)

### Community 36 - "Protocol TS Config"
Cohesion: 0.14
Nodes (13): compilerOptions, declaration, esModuleInterop, module, moduleResolution, outDir, rootDir, skipLibCheck (+5 more)

### Community 37 - "Protocol JS Validator"
Cohesion: 0.40
Nodes (13): checkDeadEndBranches(), checkNextBranchPointConflict(), checkReachability(), checkRequiredNodeIds(), checkSelfLoops(), checkTrivialCycles(), checkUniqueBranchKeys(), checkUniqueNodeIds() (+5 more)

### Community 38 - "Project Cheat Sheets"
Cohesion: 0.06
Nodes (43): Graphify Usage Rules, Copilot CLI Cheat Sheet, BranchPoint / BranchOption (core), `ContentBlock`, CoreError, Dependencies, Design Philosophy, Error Handling (+35 more)

### Community 40 - "Markdown Lint Config"
Cohesion: 0.18
Nodes (10): default, MD013, MD024, siblings_only, MD025, front_matter_title, level, MD041 (+2 more)

### Community 42 - "Hello Example Document"
Cohesion: 0.20
Nodes (9): author, date, defaults, transition, view-mode, description, fireside-version, nodes (+1 more)

### Community 43 - "Crate READMEs"
Cohesion: 0.29
Nodes (6): Dependencies, Design Philosophy, Error Handling, fireside-engine, Module Map, Testing

### Community 45 - "Graph & Command Docs"
Cohesion: 0.28
Nodes (9): Graph (runtime repr), GraphFile (wire repr), `Command` and `CommandHistory`, Key Types, Loader, `PresentationSession`, The `next` Priority Chain, `TraversalEngine` (+1 more)

### Community 46 - "Editor Command Tests"
Cohesion: 0.10
Nodes (20): Algorithm, Algorithm, Algorithm, Algorithm, Branch and Rejoin, Branch-point precedence, Branch return wiring, Conformance Checklist (+12 more)

### Community 47 - "Layout Rendering"
Cohesion: 0.20
Nodes (13): Line, Option, String, Style, Tokens, Vec, bold_fragment_carries_bold_style(), find_closer() (+5 more)

### Community 49 - "TypeSpec Linter Rules"
Cohesion: 0.31
Nodes (4): requireDocRule, useNodeIdScalarRule, $lib, $linter

### Community 52 - "Timeline UI"
Cohesion: 0.67
Nodes (3): Protocol Spec Drift Audit, Protocol TypeSpec CI, tsp-output Commit Verification

### Community 53 - "TEA Architecture Docs"
Cohesion: 0.20
Nodes (9): Building, Configuration Files, Dependencies, Design Philosophy, Event Loop Architecture, fireside-cli, iTerm2 Theme Import, iTerm2 Themes (+1 more)

### Community 55 - "Documentation Standards"
Cohesion: 0.15
Nodes (11): Behavioral Contract, Documentation Writer Agent, Quality Bar, Working Style, Formatting and Structure, Markdown Content Rules, Validation Requirements, No-Heredoc File Operations Rule (+3 more)

### Community 59 - "Validate Command"
Cohesion: 0.05
Nodes (41): ADR Format (Nygard-style), ADR Skill, Consequences section, Context section, Decision section, Example ADR, Numbering and Filing, Nygard ADR Format (+33 more)

### Community 60 - "Command History Tests"
Cohesion: 0.09
Nodes (22): For /graphify add and --watch, For /graphify query, For the commit hook and native CLAUDE.md integration, For --update and --cluster-only, /graphify, Interpreter guard for subcommands, Part A - Structural extraction for code files, Part B - Semantic extraction (parallel subagents) (+14 more)

### Community 61 - "Claude Hooks Settings"
Cohesion: 0.33
Nodes (5): hooks, PostToolUse, PreToolUse, permissions, allow

### Community 87 - "Model Module"
Cohesion: 0.07
Nodes (31): BranchOption, CoreError, BranchPoint, ContentBlock, Node, NodeDefaults, NodeId, Option (+23 more)

### Community 88 - "Render Module"
Cohesion: 0.16
Nodes (39): App, KeyCode, Line, Option, String, Tokens, Vec, ViewMode (+31 more)

### Community 92 - "Community 92"
Cohesion: 0.15
Nodes (12): `App` State, Application Modes, Dependencies, Design Philosophy: TEA (The Elm Architecture), Editor Mode, fireside-tui, Keybindings, Module Map (+4 more)

### Community 93 - "Community 93"
Cohesion: 0.20
Nodes (9): A useful diagram, For authors, For engine authors, For presenters, Good authoring habits, The short version, Three mental models, What problem it solves (+1 more)

### Community 96 - "Community 96"
Cohesion: 0.22
Nodes (8): Enforcement, Forbidden Patterns, MANDATORY: File Operation Override, Required Approach, Terminal IS Allowed For, Terminal is FORBIDDEN For, The Problem, The Rule

### Community 98 - "Community 98"
Cohesion: 0.22
Nodes (8): ContentBlock Validation Rules, Core Blocks, Error Severity Guidance, Failure Handling, Layer 1: Schema Validation, Layer 2: Semantic Checks, Recommended Checks, Required Checks

### Community 99 - "Community 99"
Cohesion: 0.25
Nodes (8): Commands, `fireside edit [path]`, `fireside fonts`, `fireside import-theme <file>`, `fireside new <name>`, `fireside open <dir>`, `fireside present <file>`, `fireside validate <file>`

### Community 100 - "Community 100"
Cohesion: 0.25
Nodes (7): graphify reference: extra exports and benchmark, Step 6b - Wiki (only if --wiki flag), Step 7 - Neo4j export (only if --neo4j or --neo4j-push flag), Step 7b - SVG export (only if --svg flag), Step 7c - GraphML export (only if --graphml flag), Step 7d - MCP server (only if --mcp flag), Step 8 - Token reduction benchmark (only if total_words > 5000)

### Community 101 - "Community 101"
Cohesion: 0.29
Nodes (6): Read the shape, Run it, Start with the graph, What to try next, What you will make, Why this structure works

### Community 102 - "Community 102"
Cohesion: 0.29
Nodes (6): CONTEXTUAL AWARENESS, Documentation Writer Prompt, OUTPUT EXPECTATIONS, PROMPT PURPOSE, WORKFLOW, YOUR TASK: The Four Document Types

### Community 103 - "Community 103"
Cohesion: 0.29
Nodes (6): ContentBlock Kinds, Core Types, Enums, Root Shape, Traversal Operations, Traversal Shapes

### Community 104 - "Community 104"
Cohesion: 0.29
Nodes (6): Canonical Format, Character Encoding, File Extensions, Media Type, Property and Enum Naming, Schema Relationship

### Community 105 - "Community 105"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 01 — Workspace manifest hygiene

### Community 106 - "Community 106"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 02 — Traversal string shorthand (D1) + hello.json smoke test

### Community 107 - "Community 107"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 03 — BranchOption: optional string key + description (D11)

### Community 108 - "Community 108"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 04 — Content blocks: image size, optional alt, list serialization (D13)

### Community 109 - "Community 109"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 05 — Transition unknown-value fallback (D10)

### Community 110 - "Community 110"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 06 — Remove `traversal.after` (D7)

### Community 111 - "Community 111"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 07 — Introduce `ViewMode` (D9, core only)

### Community 112 - "Community 112"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 08 — Require `Node.id` (D6)

### Community 113 - "Community 113"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 09 — Engine: explicit-edge traversal semantics (D2/D3/D4)

### Community 114 - "Community 114"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 10 — Engine: NodeId-based history (D5)

### Community 115 - "Community 115"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 11 — Validation: required checks + lint codes (D8)

### Community 116 - "Community 116"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 12 — Scaffold conformance

### Community 117 - "Community 117"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 13 — Round-trip fidelity in save_graph (D14)

### Community 118 - "Community 118"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 14 — `fireside validate` output parity

### Community 119 - "Community 119"
Cohesion: 0.29
Nodes (6): Acceptance, Gate (do this first, 5 minutes), Goal, Steps, Task 15 — Optional: JSON Schema Layer-1 validation spike, Verified API (Context7, jsonschema 0.40)

### Community 120 - "Community 120"
Cohesion: 0.29
Nodes (6): Acceptance, Do NOT, Goal, Ratatui mapping (APIs verified via Context7 against ratatui 0.30), Steps, Task 17 — TUI: ViewMode + container layouts via ratatui

### Community 121 - "Community 121"
Cohesion: 0.29
Nodes (6): Acceptance, Do NOT, Goal, Golden tests, Task 18 — TUI: traversal UX polish + golden tests, UX requirements (all styling via `DesignTokens`; reuse existing chrome/flash patterns)

### Community 122 - "Community 122"
Cohesion: 0.29
Nodes (6): Acceptance, ADRs to write, Do NOT, Goal, Steps, Task 19 — ADRs for protocol decisions

### Community 123 - "Community 123"
Cohesion: 0.29
Nodes (6): Acceptance, Background, Do NOT, Goal, Steps, Task 22 — AI workflow: consolidate engineering constraints

### Community 125 - "Community 125"
Cohesion: 0.33
Nodes (5): Content Structure, Fireside Docs, Local Development, Stack, Validate and Build

### Community 126 - "Community 126"
Cohesion: 0.33
Nodes (5): Canonical rules, Core expectations, Maintainability checklist, Planning rules, Rust Best Practices for Fireside

### Community 127 - "Community 127"
Cohesion: 0.33
Nodes (5): Design principles (apply to every TUI task), Drift guards (read before every task), Fireside Execution Plan, Rules for executing a task, Sequence and dependencies

### Community 128 - "Community 128"
Cohesion: 0.33
Nodes (5): For /graphify explain, For /graphify path, graphify reference: query, path, explain, Step 0 — Constrained query expansion (REQUIRED before traversal), Step 1 — Traversal

### Community 129 - "Community 129"
Cohesion: 0.33
Nodes (5): Acceptance, Do NOT, Goal, Steps, Task 16 — CI conformance job

### Community 130 - "Community 130"
Cohesion: 0.33
Nodes (5): Acceptance, Do NOT, Goal, Steps, Task 20 — Spec fixes S1–S4 + schema regeneration

### Community 131 - "Community 131"
Cohesion: 0.33
Nodes (5): Acceptance, Do NOT, Goal, Steps, Task 21 — Shared conformance fixtures + restore git hooks

### Community 132 - "Community 132"
Cohesion: 0.33
Nodes (5): Acceptance, Do NOT, Goal, Steps, Task 23 — AI workflow: fix stale agent/instruction files

### Community 133 - "Community 133"
Cohesion: 0.33
Nodes (5): Acceptance, Do NOT, Goal, Steps, Task 24 — AI workflow: automate graphify + permissions

### Community 135 - "Community 135"
Cohesion: 0.40
Nodes (4): Specification Map, Start Here, What Fireside Does Not Define, What Fireside Gives You

### Community 136 - "Community 136"
Cohesion: 0.40
Nodes (4): Canonical Terms, Conversational Layer, Traversal Verbs, Ubiquitous Language Notes

### Community 138 - "Community 138"
Cohesion: 0.50
Nodes (3): For /graphify add, For --watch, graphify reference: add a URL and watch a folder

### Community 139 - "Community 139"
Cohesion: 0.50
Nodes (3): For git commit hook, For native CLAUDE.md integration, graphify reference: commit hook and native CLAUDE.md integration

### Community 140 - "Community 140"
Cohesion: 0.50
Nodes (3): For --cluster-only, For --update (incremental re-extraction), graphify reference: incremental update and cluster-only

### Community 148 - "Community 148"
Cohesion: 0.22
Nodes (8): ADR-002: Retire node-level `Layout` in favor of `view-mode` + container layouts, Consequences, Context, Decision, Negative or Trade-offs, Neutral / Follow-up, Positive, Status

### Community 149 - "Community 149"
Cohesion: 0.22
Nodes (8): ADR-001: Remove `traversal.after`, Consequences, Context, Decision, Negative or Trade-offs, Neutral / Follow-up, Positive, Status

### Community 151 - "Community 151"
Cohesion: 0.20
Nodes (9): Claude Fable Planning Prompt, Goal, Important Constraints, Output Expectations, Part A — Audit the protocol spec and find gaps, Part B — Evaluate the Rust reference implementation and UX, Part C — Produce a concrete, phased plan, Repository Context (+1 more)

### Community 153 - "Community 153"
Cohesion: 0.25
Nodes (7): Build and Test Commands, Crate Boundary Rules, Error Handling Stratification, Fireside Engineering Constraints, Mandatory Idioms, MSRV, Source of Truth

### Community 160 - "Community 160"
Cohesion: 0.22
Nodes (8): ADR-003: Non-normative engine extras, Consequences, Context, Decision, Negative or Trade-offs, Neutral / Follow-up, Positive, Status

## Ambiguous Edges - Review These
- `Copilot CLI Cheat Sheet` → `Transition enum (8 variants, core)`  [AMBIGUOUS]
  COPILOT-CLI-CHEATSHEET.md · relation: references

## Knowledge Gaps
- **586 isolated node(s):** `allow`, `PreToolUse`, `PostToolUse`, `allow`, `install.sh script` (+581 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **15 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Copilot CLI Cheat Sheet` and `Transition enum (8 variants, core)`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **Why does `Session` connect `TUI App Core` to `Render Module`, `CLI Commands`?**
  _High betweenness centrality (0.012) - this node is a cross-community bridge._
- **Why does `EngineError` connect `Presentation Session` to `TUI App Core`?**
  _High betweenness centrality (0.006) - this node is a cross-community bridge._
- **What connects `allow`, `PreToolUse`, `PostToolUse` to the rest of the system?**
  _588 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `CLI Commands` be split into smaller, more focused modules?**
  _Cohesion score 0.14814814814814814 - nodes in this community are weakly interconnected._
- **Should `Agent & Skill Governance` be split into smaller, more focused modules?**
  _Cohesion score 0.09686609686609686 - nodes in this community are weakly interconnected._
- **Should `TUI App Core` be split into smaller, more focused modules?**
  _Cohesion score 0.09413067552602436 - nodes in this community are weakly interconnected._