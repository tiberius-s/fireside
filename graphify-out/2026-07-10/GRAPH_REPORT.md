# Graph Report - fireside  (2026-07-10)

## Corpus Check
- 82 files · ~42,684 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 931 nodes · 1389 edges · 68 communities (52 shown, 16 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 33 edges (avg confidence: 0.84)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `8fdb7efd`
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
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Theme & Content Types|Theme & Content Types]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Traversal Engine|Traversal Engine]]
- [[_COMMUNITY_Protocol Package|Protocol Package]]
- [[_COMMUNITY_Docs Site Package|Docs Site Package]]
- [[_COMMUNITY_Presentation Session|Presentation Session]]
- [[_COMMUNITY_Keybindings Config|Keybindings Config]]
- [[_COMMUNITY_Window Chrome UI|Window Chrome UI]]
- [[_COMMUNITY_Graph Loading Tests|Graph Loading Tests]]
- [[_COMMUNITY_Docs Site Pages|Docs Site Pages]]
- [[_COMMUNITY_Protocol TS Config|Protocol TS Config]]
- [[_COMMUNITY_Protocol JS Validator|Protocol JS Validator]]
- [[_COMMUNITY_Project Cheat Sheets|Project Cheat Sheets]]
- [[_COMMUNITY_Markdown Lint Config|Markdown Lint Config]]
- [[_COMMUNITY_Hello Example Document|Hello Example Document]]
- [[_COMMUNITY_Editor Command Tests|Editor Command Tests]]
- [[_COMMUNITY_Layout Rendering|Layout Rendering]]
- [[_COMMUNITY_TypeSpec Linter Rules|TypeSpec Linter Rules]]
- [[_COMMUNITY_Timeline UI|Timeline UI]]
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
- [[_COMMUNITY_Community 93|Community 93]]
- [[_COMMUNITY_Community 96|Community 96]]
- [[_COMMUNITY_Community 98|Community 98]]
- [[_COMMUNITY_Community 100|Community 100]]
- [[_COMMUNITY_Community 101|Community 101]]
- [[_COMMUNITY_Community 102|Community 102]]
- [[_COMMUNITY_Community 103|Community 103]]
- [[_COMMUNITY_Community 104|Community 104]]
- [[_COMMUNITY_Community 125|Community 125]]
- [[_COMMUNITY_Community 126|Community 126]]
- [[_COMMUNITY_Community 128|Community 128]]
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
- [[_COMMUNITY_Community 153|Community 153]]
- [[_COMMUNITY_Community 163|Community 163]]

## God Nodes (most connected - your core abstractions)
1. `App` - 30 edges
2. `app()` - 23 edges
3. `Session` - 20 edges
4. `render_block()` - 20 edges
5. `Diagnostic` - 19 edges
6. `press()` - 18 edges
7. `screen()` - 18 edges
8. `Graphify Skill Pipeline` - 17 edges
9. `Node` - 14 edges
10. `hello_session()` - 14 edges

## Surprising Connections (you probably didn't know these)
- `Command / CommandHistory (undo-redo)` --semantically_similar_to--> `Core Runtime Guarantees`  [INFERRED] [semantically similar]
  crates/fireside-engine/README.md → docs/src/content/docs/spec/appendix-engine-guidelines.md
- `Honesty Rules` --semantically_similar_to--> `Context7 Documentation Expert Agent`  [INFERRED] [semantically similar]
  .claude/skills/graphify/SKILL.md → .github/agents/context7.agent.md
- `TEA (The Elm Architecture) Discipline` --semantically_similar_to--> `Core Runtime Guarantees`  [INFERRED] [semantically similar]
  crates/fireside-tui/README.md → docs/src/content/docs/spec/appendix-engine-guidelines.md
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
- 1-file cycle: `crates/fireside-tui/src/render/syntax.rs -> crates/fireside-tui/src/render/syntax.rs`
- 1-file cycle: `crates/fireside-tui/src/theme.rs -> crates/fireside-tui/src/theme.rs`

## Hyperedges (group relationships)
- **Graphify Two-Track Extraction Pipeline** — graphify_skill_ast_extraction, graphify_skill_semantic_extraction, graphify_skill_extraction_cache, references_extraction_spec_subagent_prompt [EXTRACTED 1.00]
- **Fireside Engineering Constraint Set (MSRV, boundaries, TEA, errors)** — agents_rust_expert_agent_msrv_rule, agents_rust_expert_agent_crate_boundary_rules, agents_rust_expert_agent_tea_invariant, agents_rust_expert_agent_error_stratification, instructions_rust_best_practices_instructions_rust_best_practices, adr_skill_adr_skill, workflows_rust_msrv_job [EXTRACTED 1.00]
- **Fireside CI Quality Gates** — workflows_rust_rust_ci, workflows_audit_security_audit, workflows_docs_docs_ci, workflows_models_protocol_ci [EXTRACTED 1.00]
- **Four Traversal Operations (Next, Choose, Goto, Back)** — spec_traversal_next, spec_traversal_choose, spec_traversal_goto, spec_traversal_back, spec_traversal_history_invariants [EXTRACTED 1.00]
- **Rust Reference Implementation Layering (core -> engine -> tui -> cli)** — fireside_core_readme_fireside_core, fireside_engine_readme_fireside_engine, fireside_tui_readme_fireside_tui, fireside_cli_readme_fireside_cli [EXTRACTED 1.00]
- **Fireside Document Data Model** — spec_data_model_graph, spec_data_model_node, spec_data_model_contentblock, spec_data_model_traversal, spec_data_model_branchpoint, spec_data_model_branchoption, spec_data_model_nodeid [EXTRACTED 1.00]

## Communities (68 total, 16 thin omitted)

### Community 0 - "TUI App Helpers"
Cohesion: 0.23
Nodes (12): Option, Span, Style, Tokens, Vec, highlight(), row_text_reassembles_the_source_exactly(), rust_keywords_strings_and_comments_get_distinct_styles() (+4 more)

### Community 1 - "CLI Commands"
Cohesion: 0.15
Nodes (25): Command, Graph, Option, PathBuf, Result, Command, PathBuf, Path (+17 more)

### Community 2 - "Block Rendering Types"
Cohesion: 0.22
Nodes (8): ADR-001: Remove `traversal.after`, Consequences, Context, Decision, Negative or Trade-offs, Neutral / Follow-up, Positive, Status

### Community 3 - "Agent & Skill Governance"
Cohesion: 0.10
Nodes (27): Graphify Slash Command Trigger, AST Structural Extraction, Community Detection and Labeling, Token Cost Tracking, Semantic Extraction Cache, Fast Path Query on Existing Graph, Graphify Skill Pipeline, Semantic Extraction via Parallel Subagents (+19 more)

### Community 4 - "TUI App Core"
Cohesion: 0.09
Nodes (22): KeyCode, Option, Self, String, ViewMode, App, Graph, Result (+14 more)

### Community 7 - "Editor Navigation"
Cohesion: 0.22
Nodes (30): ContainerLayout, ContentBlock, Line, Option, Span, String, Tokens, Vec (+22 more)

### Community 9 - "Editor Interaction Tests"
Cohesion: 0.16
Nodes (34): Graph, HashSet, Node, Option, Result, Self, String, Vec (+26 more)

### Community 10 - "Community 10"
Cohesion: 0.22
Nodes (8): ADR-002: Retire node-level `Layout` in favor of `view-mode` + container layouts, Consequences, Context, Decision, Negative or Trade-offs, Neutral / Follow-up, Positive, Status

### Community 11 - "Theme & Content Types"
Cohesion: 0.38
Nodes (4): Self, Style, Default, Tokens

### Community 12 - "Community 12"
Cohesion: 0.22
Nodes (8): ADR-003: Non-normative engine extras, Consequences, Context, Decision, Negative or Trade-offs, Neutral / Follow-up, Positive, Status

### Community 13 - "Community 13"
Cohesion: 0.22
Nodes (8): ADR-004: Presenter-first rewrite against protocol 0.1.0, Consequences, Context, Decision, Negative or Trade-offs, Neutral / Follow-up, Positive, Status

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
Cohesion: 0.06
Nodes (39): 404 Page, Docs Landing Page / Specification Map, Fireside Docs Site (Astro + Starlight), fireside-cli, CoreError, fireside-core, fireside-engine, fireside-tui (+31 more)

### Community 36 - "Protocol TS Config"
Cohesion: 0.14
Nodes (13): compilerOptions, declaration, esModuleInterop, module, moduleResolution, outDir, rootDir, skipLibCheck (+5 more)

### Community 37 - "Protocol JS Validator"
Cohesion: 0.40
Nodes (13): checkDeadEndBranches(), checkNextBranchPointConflict(), checkReachability(), checkRequiredNodeIds(), checkSelfLoops(), checkTrivialCycles(), checkUniqueBranchKeys(), checkUniqueNodeIds() (+5 more)

### Community 38 - "Project Cheat Sheets"
Cohesion: 0.10
Nodes (29): Graphify Usage Rules, Copilot CLI Cheat Sheet, BranchPoint / BranchOption (core), Graph (runtime repr), GraphFile (wire repr), Layout enum (12 variants), Transition enum (8 variants, core), Traversal struct (core) (+21 more)

### Community 40 - "Markdown Lint Config"
Cohesion: 0.18
Nodes (10): default, MD013, MD024, siblings_only, MD025, front_matter_title, level, MD041 (+2 more)

### Community 42 - "Hello Example Document"
Cohesion: 0.20
Nodes (9): author, date, defaults, transition, view-mode, description, fireside-version, nodes (+1 more)

### Community 46 - "Editor Command Tests"
Cohesion: 0.06
Nodes (33): CLI Event Loop, Command / CommandHistory (undo-redo), Action enum (~35 variants), App struct (TUI state), AppMode (Presenting/Editing/GotoNode/Quitting), TEA (The Elm Architecture) Discipline, Appendix B — Engine Guidelines, Container Rendering Guidance (+25 more)

### Community 47 - "Layout Rendering"
Cohesion: 0.20
Nodes (13): Line, Option, String, Style, Tokens, Vec, bold_fragment_carries_bold_style(), find_closer() (+5 more)

### Community 49 - "TypeSpec Linter Rules"
Cohesion: 0.31
Nodes (4): requireDocRule, useNodeIdScalarRule, $lib, $linter

### Community 52 - "Timeline UI"
Cohesion: 0.67
Nodes (3): Protocol Spec Drift Audit, Protocol TypeSpec CI, tsp-output Commit Verification

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
Cohesion: 0.14
Nodes (50): App, KeyCode, Line, Option, String, Tokens, Vec, ViewMode (+42 more)

### Community 93 - "Community 93"
Cohesion: 0.20
Nodes (9): A useful diagram, For authors, For engine authors, For presenters, Good authoring habits, The short version, Three mental models, What problem it solves (+1 more)

### Community 96 - "Community 96"
Cohesion: 0.22
Nodes (8): Enforcement, Forbidden Patterns, MANDATORY: File Operation Override, Required Approach, Terminal IS Allowed For, Terminal is FORBIDDEN For, The Problem, The Rule

### Community 98 - "Community 98"
Cohesion: 0.22
Nodes (8): ContentBlock Validation Rules, Core Blocks, Error Severity Guidance, Failure Handling, Layer 1: Schema Validation, Layer 2: Semantic Checks, Recommended Checks, Required Checks

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

### Community 125 - "Community 125"
Cohesion: 0.33
Nodes (5): Content Structure, Fireside Docs, Local Development, Stack, Validate and Build

### Community 126 - "Community 126"
Cohesion: 0.33
Nodes (5): Canonical rules, Core expectations, Maintainability checklist, Planning rules, Rust Best Practices for Fireside

### Community 128 - "Community 128"
Cohesion: 0.33
Nodes (5): For /graphify explain, For /graphify path, graphify reference: query, path, explain, Step 0 — Constrained query expansion (REQUIRED before traversal), Step 1 — Traversal

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

### Community 153 - "Community 153"
Cohesion: 0.20
Nodes (9): Build and Test Commands, Crate Boundary Rules, Error Handling Stratification, Fireside Engineering Constraints, Mandatory Idioms, MSRV, Product North Star, Source of Truth (+1 more)

## Ambiguous Edges - Review These
- `Copilot CLI Cheat Sheet` → `Transition enum (8 variants, core)`  [AMBIGUOUS]
  COPILOT-CLI-CHEATSHEET.md · relation: references

## Knowledge Gaps
- **410 isolated node(s):** `allow`, `PreToolUse`, `PostToolUse`, `allow`, `install.sh script` (+405 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **16 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Copilot CLI Cheat Sheet` and `Transition enum (8 variants, core)`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **Why does `Session` connect `TUI App Core` to `Render Module`, `CLI Commands`, `Presentation Session`?**
  _High betweenness centrality (0.023) - this node is a cross-community bridge._
- **What connects `allow`, `PreToolUse`, `PostToolUse` to the rest of the system?**
  _412 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `CLI Commands` be split into smaller, more focused modules?**
  _Cohesion score 0.14814814814814814 - nodes in this community are weakly interconnected._
- **Should `Agent & Skill Governance` be split into smaller, more focused modules?**
  _Cohesion score 0.09686609686609686 - nodes in this community are weakly interconnected._
- **Should `TUI App Core` be split into smaller, more focused modules?**
  _Cohesion score 0.08695652173913043 - nodes in this community are weakly interconnected._
- **Should `Protocol Package` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._