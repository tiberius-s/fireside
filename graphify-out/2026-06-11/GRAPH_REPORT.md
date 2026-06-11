# Graph Report - .  (2026-06-10)

## Corpus Check
- 155 files · ~82,001 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1286 nodes · 2353 edges · 91 communities (73 shown, 18 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 189 edges (avg confidence: 0.81)
- Token cost: 199,510 input · 0 output

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
- [[_COMMUNITY_Action Routing|Action Routing]]
- [[_COMMUNITY_Theme & Content Types|Theme & Content Types]]
- [[_COMMUNITY_Core Graph Model|Core Graph Model]]
- [[_COMMUNITY_Settings & Config|Settings & Config]]
- [[_COMMUNITY_Traversal Engine|Traversal Engine]]
- [[_COMMUNITY_Presenter UI|Presenter UI]]
- [[_COMMUNITY_Protocol Package|Protocol Package]]
- [[_COMMUNITY_VS Code Theme Import|VS Code Theme Import]]
- [[_COMMUNITY_Textarea Widget|Textarea Widget]]
- [[_COMMUNITY_Docs Site Package|Docs Site Package]]
- [[_COMMUNITY_Node Model & Commands|Node Model & Commands]]
- [[_COMMUNITY_Presentation Session|Presentation Session]]
- [[_COMMUNITY_Graph Validation|Graph Validation]]
- [[_COMMUNITY_iTerm2 Theme Import|iTerm2 Theme Import]]
- [[_COMMUNITY_Transition Animations|Transition Animations]]
- [[_COMMUNITY_Help Overlay|Help Overlay]]
- [[_COMMUNITY_Keybindings Config|Keybindings Config]]
- [[_COMMUNITY_Window Chrome UI|Window Chrome UI]]
- [[_COMMUNITY_Command History|Command History]]
- [[_COMMUNITY_Graph Loading Tests|Graph Loading Tests]]
- [[_COMMUNITY_Extension Block Rendering|Extension Block Rendering]]
- [[_COMMUNITY_Progress Bar UI|Progress Bar UI]]
- [[_COMMUNITY_Docs Site Pages|Docs Site Pages]]
- [[_COMMUNITY_Code Block Rendering|Code Block Rendering]]
- [[_COMMUNITY_Graph Loader|Graph Loader]]
- [[_COMMUNITY_TUI Test Harness|TUI Test Harness]]
- [[_COMMUNITY_Protocol TS Config|Protocol TS Config]]
- [[_COMMUNITY_Protocol JS Validator|Protocol JS Validator]]
- [[_COMMUNITY_Project Cheat Sheets|Project Cheat Sheets]]
- [[_COMMUNITY_Content Roundtrip Tests|Content Roundtrip Tests]]
- [[_COMMUNITY_Markdown Lint Config|Markdown Lint Config]]
- [[_COMMUNITY_Font Management|Font Management]]
- [[_COMMUNITY_Hello Example Document|Hello Example Document]]
- [[_COMMUNITY_Crate READMEs|Crate READMEs]]
- [[_COMMUNITY_Branching Data Model|Branching Data Model]]
- [[_COMMUNITY_Graph & Command Docs|Graph & Command Docs]]
- [[_COMMUNITY_Editor Command Tests|Editor Command Tests]]
- [[_COMMUNITY_Layout Rendering|Layout Rendering]]
- [[_COMMUNITY_Breadcrumb UI|Breadcrumb UI]]
- [[_COMMUNITY_TypeSpec Linter Rules|TypeSpec Linter Rules]]
- [[_COMMUNITY_Branch Option Model|Branch Option Model]]
- [[_COMMUNITY_Timeline UI|Timeline UI]]
- [[_COMMUNITY_TEA Architecture Docs|TEA Architecture Docs]]
- [[_COMMUNITY_CLI E2E Tests|CLI E2E Tests]]
- [[_COMMUNITY_Documentation Standards|Documentation Standards]]
- [[_COMMUNITY_Content Block Docs|Content Block Docs]]
- [[_COMMUNITY_Traversal Model|Traversal Model]]
- [[_COMMUNITY_Text Block Rendering|Text Block Rendering]]
- [[_COMMUNITY_Validate Command|Validate Command]]
- [[_COMMUNITY_Command History Tests|Command History Tests]]
- [[_COMMUNITY_Claude Hooks Settings|Claude Hooks Settings]]
- [[_COMMUNITY_Claude Local Permissions|Claude Local Permissions]]
- [[_COMMUNITY_TUI Errors|TUI Errors]]
- [[_COMMUNITY_Dangling Ref Fixture|Dangling Ref Fixture]]
- [[_COMMUNITY_Duplicate ID Fixture|Duplicate ID Fixture]]
- [[_COMMUNITY_Empty Graph Fixture|Empty Graph Fixture]]
- [[_COMMUNITY_Branching Fixture|Branching Fixture]]
- [[_COMMUNITY_Linear Fixture|Linear Fixture]]
- [[_COMMUNITY_Event Actions|Event Actions]]
- [[_COMMUNITY_Command Types|Command Types]]
- [[_COMMUNITY_Core Errors|Core Errors]]
- [[_COMMUNITY_Engine Errors|Engine Errors]]
- [[_COMMUNITY_Docs TS Config|Docs TS Config]]
- [[_COMMUNITY_Git Hooks Install|Git Hooks Install]]
- [[_COMMUNITY_Layout Enum|Layout Enum]]
- [[_COMMUNITY_Transition Enum|Transition Enum]]
- [[_COMMUNITY_Docs Content Config|Docs Content Config]]
- [[_COMMUNITY_TypeSpec Diagnostics|TypeSpec Diagnostics]]

## God Nodes (most connected - your core abstractions)
1. `App` - 37 edges
2. `App` - 32 edges
3. `load_graph()` - 22 edges
4. `load_graph_from_str()` - 22 edges
5. `render_block_with_tokens()` - 19 edges
6. `render_editor()` - 19 edges
7. `render_presenter()` - 19 edges
8. `Graphify Skill Pipeline` - 17 edges
9. `render_node_content_with_base()` - 15 edges
10. `TextArea` - 15 edges

## Surprising Connections (you probably didn't know these)
- `TEA (The Elm Architecture) Discipline` --semantically_similar_to--> `Core Runtime Guarantees`  [INFERRED] [semantically similar]
  crates/fireside-tui/README.md → docs/src/content/docs/spec/appendix-engine-guidelines.md
- `Graphify Honesty Rules` --semantically_similar_to--> `Context7 Documentation Expert Agent`  [INFERRED] [semantically similar]
  .claude/skills/graphify/SKILL.md → .github/agents/context7.agent.md
- `Claude Fable Planning Prompt` --semantically_similar_to--> `Context7 Documentation Expert Agent`  [INFERRED] [semantically similar]
  .claude/prompts/claude-fable-plan.prompt.md → .github/agents/context7.agent.md
- `Layout enum (12 variants)` --semantically_similar_to--> `ViewMode enum (default, fullscreen)`  [INFERRED] [semantically similar]
  crates/fireside-core/README.md → docs/src/content/docs/spec/data-model.md
- `Transition enum (8 variants, core)` --semantically_similar_to--> `Transition enum (none, fade — protocol)`  [INFERRED] [semantically similar]
  crates/fireside-core/README.md → docs/src/content/docs/spec/data-model.md

## Import Cycles
- 1-file cycle: `crates/fireside-cli/src/commands/project.rs -> crates/fireside-cli/src/commands/project.rs`
- 1-file cycle: `crates/fireside-cli/src/commands/scaffold.rs -> crates/fireside-cli/src/commands/scaffold.rs`
- 1-file cycle: `crates/fireside-cli/src/commands/session.rs -> crates/fireside-cli/src/commands/session.rs`
- 1-file cycle: `crates/fireside-tui/tests/harness.rs -> crates/fireside-tui/tests/harness.rs`
- 1-file cycle: `crates/fireside-cli/src/commands/theme.rs -> crates/fireside-cli/src/commands/theme.rs`
- 1-file cycle: `crates/fireside-cli/src/commands/validate.rs -> crates/fireside-cli/src/commands/validate.rs`
- 1-file cycle: `crates/fireside-cli/src/main.rs -> crates/fireside-cli/src/main.rs`
- 1-file cycle: `crates/fireside-core/src/model/node.rs -> crates/fireside-core/src/model/node.rs`
- 1-file cycle: `crates/fireside-core/src/model/branch.rs -> crates/fireside-core/src/model/branch.rs`
- 1-file cycle: `crates/fireside-tui/src/theme.rs -> crates/fireside-tui/src/theme.rs`
- 1-file cycle: `crates/fireside-core/src/model/graph.rs -> crates/fireside-core/src/model/graph.rs`
- 1-file cycle: `crates/fireside-tui/src/app/app_tests/mod.rs -> crates/fireside-tui/src/app/app_tests/mod.rs`
- 1-file cycle: `crates/fireside-engine/src/commands/apply.rs -> crates/fireside-engine/src/commands/apply.rs`
- 1-file cycle: `crates/fireside-tui/src/ui/graph.rs -> crates/fireside-tui/src/ui/graph.rs`
- 1-file cycle: `crates/fireside-core/src/model/traversal.rs -> crates/fireside-core/src/model/traversal.rs`
- 1-file cycle: `crates/fireside-engine/src/commands/history.rs -> crates/fireside-engine/src/commands/history.rs`
- 1-file cycle: `crates/fireside-engine/src/session.rs -> crates/fireside-engine/src/session.rs`
- 1-file cycle: `crates/fireside-engine/src/loader.rs -> crates/fireside-engine/src/loader.rs`
- 1-file cycle: `crates/fireside-engine/src/traversal.rs -> crates/fireside-engine/src/traversal.rs`
- 1-file cycle: `crates/fireside-engine/src/validation.rs -> crates/fireside-engine/src/validation.rs`

## Hyperedges (group relationships)
- **Graphify Two-Track Extraction Pipeline** — graphify_skill_ast_extraction, graphify_skill_semantic_extraction, graphify_skill_extraction_cache, references_extraction_spec_subagent_prompt [EXTRACTED 1.00]
- **Fireside Engineering Constraint Set (MSRV, boundaries, TEA, errors)** — agents_rust_expert_agent_msrv_rule, agents_rust_expert_agent_crate_boundary_rules, agents_rust_expert_agent_tea_invariant, agents_rust_expert_agent_error_stratification, instructions_rust_best_practices_instructions_rust_best_practices, adr_skill_adr_skill, workflows_rust_msrv_job [EXTRACTED 1.00]
- **Fireside CI Quality Gates** — workflows_rust_rust_ci, workflows_audit_security_audit, workflows_docs_docs_ci, workflows_models_protocol_ci [EXTRACTED 1.00]
- **Four Traversal Operations (Next, Choose, Goto, Back)** — spec_traversal_next, spec_traversal_choose, spec_traversal_goto, spec_traversal_back, spec_traversal_history_invariants [EXTRACTED 1.00]
- **Rust Reference Implementation Layering (core -> engine -> tui -> cli)** — fireside_core_readme_fireside_core, fireside_engine_readme_fireside_engine, fireside_tui_readme_fireside_tui, fireside_cli_readme_fireside_cli [EXTRACTED 1.00]
- **Fireside Document Data Model** — spec_data_model_graph, spec_data_model_node, spec_data_model_contentblock, spec_data_model_traversal, spec_data_model_branchpoint, spec_data_model_branchoption, spec_data_model_nodeid [EXTRACTED 1.00]

## Communities (91 total, 18 thin omitted)

### Community 0 - "TUI App Helpers"
Cohesion: 0.05
Nodes (42): block_type_variants(), bump_index(), centered_popup(), digit_to_index(), is_subsequence(), layout_variants(), next_search_hit_from(), picker_row_span() (+34 more)

### Community 1 - "CLI Commands"
Cohesion: 0.07
Nodes (53): list_fonts(), ProjectConfig, resolve_project_entry(), resolve_project_entry_accepts_slides_when_nodes_missing(), resolve_project_entry_uses_nodes_over_slides(), run_project(), temp_dir(), scaffold_presentation() (+45 more)

### Community 2 - "Block Rendering Types"
Cohesion: 0.09
Nodes (55): DesignTokens, Line, Vec, DesignTokens, Line, Vec, DesignTokens, Line (+47 more)

### Community 3 - "Agent & Skill Governance"
Cohesion: 0.05
Nodes (50): Graphify Slash Command Trigger, ADR Skill, Nygard ADR Format, Context7 Documentation Expert Agent, Version Upgrade Checking Workflow, Verify-Before-Advising Context7 Rule, Fireside Crate Boundary Rules, Error Handling Stratification (+42 more)

### Community 4 - "TUI App Core"
Cohesion: 0.07
Nodes (24): ActiveTransition, is_editor_actionable_warning(), FlashKind, Frame, Graph, Into, KeyCode, Option (+16 more)

### Community 5 - "Editor Detail Pane"
Cohesion: 0.09
Nodes (40): ContentBlock, EditorPickerOverlay, Frame, Line, Node, NodeLayout, PresentationSession, Rect (+32 more)

### Community 6 - "Graph Overlay UI"
Cohesion: 0.14
Nodes (36): Color, Frame, Graph, Line, Node, Option, PresentationSession, Rect (+28 more)

### Community 7 - "Editor Navigation"
Cohesion: 0.12
Nodes (4): App, ContentBlock, KeyCode, Option

### Community 8 - "Design Templates"
Cohesion: 0.17
Nodes (26): Breakpoint, NodeLayout, Option, Rect, Self, all_templates_have_example_frontmatter(), all_templates_produce_valid_areas(), center_in() (+18 more)

### Community 9 - "Editor Interaction Tests"
Cohesion: 0.08
Nodes (12): editor_b_and_shift_b_select_blocks(), editor_m_enter_commits_heading_level_metadata(), editor_m_invalid_heading_level_sets_error_and_preserves_block(), editor_m_starts_metadata_edit_for_selected_block(), editor_mouse_click_selects_block_in_detail_pane(), editor_remove_block_deletes_selected_block(), branch_graph(), graph_with_content_blocks() (+4 more)

### Community 10 - "Action Routing"
Cohesion: 0.14
Nodes (19): App, Action, KeyCode, MouseScrollDirection, Frame, Graph, Node, Option (+11 more)

### Community 11 - "Theme & Content Types"
Cohesion: 0.08
Nodes (17): Result, Self, String, Vec, Color, Default, Option, Self (+9 more)

### Community 12 - "Core Graph Model"
Cohesion: 0.12
Nodes (22): GraphFile, HashMap, Layout, Node, NodeId, Option, Result, Self (+14 more)

### Community 13 - "Settings & Config"
Cohesion: 0.17
Nodes (23): Box, config_base_dir(), editor_prefs_path(), EditorUiPrefs, load_editor_ui_prefs(), load_settings(), load_settings_from_paths(), merge_settings_from_file() (+15 more)

### Community 14 - "Traversal Engine"
Cohesion: 0.17
Nodes (17): EngineError, Graph, Result, Self, back_pops_history(), choose_branch_option(), goto_out_of_bounds_errors(), goto_with_valid_index() (+9 more)

### Community 15 - "Presenter UI"
Cohesion: 0.27
Nodes (25): FlashKind, Frame, Layout, Node, Option, Path, PresentationSession, Rect (+17 more)

### Community 16 - "Protocol Package"
Cohesion: 0.08
Nodes (24): author, default, dependencies, @typespec/compiler, @typespec/json-schema, @typespec/versioning, description, devDependencies (+16 more)

### Community 17 - "VS Code Theme Import"
Cohesion: 0.15
Nodes (13): Color, DesignTokens, HashMap, Option, Path, Result, Self, String (+5 more)

### Community 18 - "Textarea Widget"
Cohesion: 0.16
Nodes (10): Frame, Into, KeyEvent, Rect, Self, String, Theme, Vec (+2 more)

### Community 19 - "Docs Site Package"
Cohesion: 0.09
Nodes (21): dependencies, astro, astro-mermaid, @astrojs/starlight, mermaid, sharp, devDependencies, @astrojs/check (+13 more)

### Community 20 - "Node Model & Commands"
Cohesion: 0.14
Nodes (15): apply_command(), BranchPoint, ContentBlock, Layout, NodeId, Option, String, Transition (+7 more)

### Community 21 - "Presentation Session"
Cohesion: 0.19
Nodes (9): Command, EngineError, Graph, Node, NodeId, Result, Self, PresentationSession (+1 more)

### Community 22 - "Graph Validation"
Cohesion: 0.19
Nodes (20): ContentBlock, EngineError, Graph, Node, Option, Result, String, Vec (+12 more)

### Community 23 - "iTerm2 Theme Import"
Cohesion: 0.17
Nodes (17): Color, DesignTokens, HashMap, Option, Path, PathBuf, Result, Self (+9 more)

### Community 24 - "Transition Animations"
Cohesion: 0.33
Nodes (20): Line, Option, Theme, Transition, Vec, StyledChar, blank_line(), clip_line_at() (+12 more)

### Community 25 - "Help Overlay"
Cohesion: 0.26
Nodes (18): Frame, Rect, Theme, Vec, build_help_rows(), centered_popup(), entry_active(), help_navigation() (+10 more)

### Community 26 - "Keybindings Config"
Cohesion: 0.21
Nodes (13): AppMode, editing_b_selects_next_block(), editing_m_starts_metadata_edit(), editing_shift_b_selects_prev_block(), map_edit_mode_key(), map_goto_mode_key(), map_key_to_action(), presenting_ctrl_h_toggles_timeline() (+5 more)

### Community 27 - "Window Chrome UI"
Cohesion: 0.23
Nodes (14): Color, Frame, Rect, Span, Theme, Vec, FlashKind, mode_badge_width() (+6 more)

### Community 28 - "Command History"
Cohesion: 0.23
Nodes (9): CommandHistory, HistoryEntry, Command, Default, EngineError, Graph, Result, Self (+1 more)

### Community 29 - "Graph Loading Tests"
Cohesion: 0.26
Nodes (14): PathBuf, PathBuf, load_graph(), hello_example_each_node_renders_non_empty_content_lines(), hello_example_image_success_and_fallback_render(), hello_example_loads_and_branches(), hello_example_transition_animation_ticks_to_completion(), hello_path() (+6 more)

### Community 30 - "Extension Block Rendering"
Cohesion: 0.32
Nodes (15): DesignTokens, Line, Option, String, Value, Vec, extract_mermaid_code(), fit_to_width() (+7 more)

### Community 31 - "Progress Bar UI"
Cohesion: 0.23
Nodes (12): Color, Frame, Option, PresentationSession, Rect, String, Theme, any_node_in_bucket_is_branch() (+4 more)

### Community 32 - "Docs Site Pages"
Cohesion: 0.19
Nodes (16): 404 Page, Docs Landing Page / Specification Map, Fireside Docs Site (Astro + Starlight), TypeSpec Emitter Config (tspconfig.yaml), Fireside Protocol, TypeSpec Domain Model (source of truth), Data Model Quick Reference, Domain Vocabulary (+8 more)

### Community 33 - "Code Block Rendering"
Cohesion: 0.22
Nodes (13): DesignTokens, Line, Option, Vec, Line, Option, String, Vec (+5 more)

### Community 34 - "Graph Loader"
Cohesion: 0.26
Nodes (12): Graph, GraphFile, Path, Result, empty_graph_returns_error(), graph_to_file(), graph_with_branching(), load_graph_from_str() (+4 more)

### Community 35 - "TUI Test Harness"
Cohesion: 0.20
Nodes (8): Action, App, Path, PathBuf, Self, String, AppHarness, hello_path()

### Community 36 - "Protocol TS Config"
Cohesion: 0.14
Nodes (13): compilerOptions, declaration, esModuleInterop, module, moduleResolution, outDir, rootDir, skipLibCheck (+5 more)

### Community 37 - "Protocol JS Validator"
Cohesion: 0.42
Nodes (12): checkDeadEndBranches(), checkNextBranchPointConflict(), checkReachability(), checkSelfLoops(), checkTrivialCycles(), checkUniqueBranchKeys(), checkUniqueNodeIds(), checkValidTargets() (+4 more)

### Community 38 - "Project Cheat Sheets"
Cohesion: 0.23
Nodes (12): Graphify Usage Rules, Copilot CLI Cheat Sheet, Layout enum (12 variants), Node (core type), Transition enum (8 variants, core), Getting Started Guide (First Fireside Graph), Graph (protocol type), Node (protocol type) (+4 more)

### Community 39 - "Content Roundtrip Tests"
Cohesion: 0.31
Nodes (9): ContentBlock, code_roundtrip(), container_roundtrip_with_children(), divider_roundtrip(), extension_roundtrip_with_nested_fallback(), heading_roundtrip(), image_roundtrip(), roundtrip() (+1 more)

### Community 40 - "Markdown Lint Config"
Cohesion: 0.18
Nodes (10): default, MD013, MD024, siblings_only, MD025, front_matter_title, level, MD041 (+2 more)

### Community 41 - "Font Management"
Cohesion: 0.38
Nodes (9): String, Vec, default_fonts_are_nonempty(), default_monospace_fonts(), list_monospace_detects_some_fonts(), list_monospace_fonts(), MonospaceFont, recommended_fonts() (+1 more)

### Community 42 - "Hello Example Document"
Cohesion: 0.20
Nodes (9): author, date, defaults, transition, view-mode, description, fireside-version, nodes (+1 more)

### Community 43 - "Crate READMEs"
Cohesion: 0.24
Nodes (10): fireside-cli crate, CoreError, fireside-core crate, fireside-engine crate, validate_graph, fireside-tui crate, Fireside Project Overview, Engine Conformance Contract (0.1.0) (+2 more)

### Community 44 - "Branching Data Model"
Cohesion: 0.24
Nodes (10): BranchPoint / BranchOption (core), Traversal struct (core), BranchOption (protocol type), BranchPoint (protocol type), NodeId Scalar, Traversal (protocol type), Branch-Point Gating, Choose Operation (+2 more)

### Community 45 - "Graph & Command Docs"
Cohesion: 0.22
Nodes (10): Graph (runtime repr), GraphFile (wire repr), Command / CommandHistory (undo-redo), Loader (load_graph / save_graph), PresentationSession, TraversalEngine, Appendix B — Engine Guidelines, Core Runtime Guarantees (+2 more)

### Community 46 - "Editor Command Tests"
Cohesion: 0.31
Nodes (8): CommandHistory, add_node_roundtrips_with_undo_redo(), graph_with_ids(), move_block_roundtrips_with_undo_redo(), remove_block_roundtrips_with_undo_redo(), update_block_roundtrips_with_undo_redo(), update_content_roundtrips_with_undo_redo(), Graph

### Community 47 - "Layout Rendering"
Cohesion: 0.53
Nodes (8): Layout, Rect, apply_layout(), center_rect(), compute_areas(), NodeAreas, pad_rect(), two_column_split()

### Community 48 - "Breadcrumb UI"
Cohesion: 0.39
Nodes (8): Frame, PresentationSession, Rect, String, Theme, NextInfo, node_short_label(), render_breadcrumb()

### Community 49 - "TypeSpec Linter Rules"
Cohesion: 0.31
Nodes (4): requireDocRule, useNodeIdScalarRule, $lib, $linter

### Community 51 - "Branch Option Model"
Cohesion: 0.32
Nodes (7): BranchOption, NodeId, Option, String, Vec, BranchOption, BranchPoint

### Community 52 - "Timeline UI"
Cohesion: 0.46
Nodes (7): Frame, PresentationSession, Rect, String, Theme, node_short_label(), render_timeline()

### Community 53 - "TEA Architecture Docs"
Cohesion: 0.29
Nodes (8): CLI Event Loop, Terminal Lifecycle Pattern, Action enum (~35 variants), App struct (TUI state), AppMode (Presenting/Editing/GotoNode/Quitting), iTerm2 Theme Import, TEA (The Elm Architecture) Discipline, Theme System

### Community 54 - "CLI E2E Tests"
Cohesion: 0.33
Nodes (3): PathBuf, repo_root(), validate_hello_exits_zero()

### Community 55 - "Documentation Standards"
Cohesion: 0.33
Nodes (6): Documentation Writer Agent, Markdown Content Rules, No-Heredoc File Operations Rule, Diataxis Framework, Documentation Writer Prompt Workflow, Docs CI and Pages Deploy

### Community 56 - "Content Block Docs"
Cohesion: 0.40
Nodes (6): ContentBlock enum (8 variants), ListItem custom deserialization, TUI Rendering Pipeline, Appendix C — Content Block Reference, ContainerBlock, ContentBlock Union (7 core kinds)

### Community 57 - "Traversal Model"
Cohesion: 0.60
Nodes (4): BranchPoint, NodeId, Option, Traversal

### Community 58 - "Text Block Rendering"
Cohesion: 0.50
Nodes (4): DesignTokens, Line, Vec, render_text()

### Community 59 - "Validate Command"
Cohesion: 0.67
Nodes (3): run_validate(), Path, Result

### Community 60 - "Command History Tests"
Cohesion: 0.67
Nodes (3): PathBuf, add_update_remove_undo_restores_original_graph_nodes(), fixture_path()

## Ambiguous Edges - Review These
- `Copilot CLI Cheat Sheet` → `Transition enum (8 variants, core)`  [AMBIGUOUS]
  COPILOT-CLI-CHEATSHEET.md · relation: references

## Knowledge Gaps
- **255 isolated node(s):** `PreToolUse`, `allow`, `install.sh script`, `default`, `MD013` (+250 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **18 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Copilot CLI Cheat Sheet` and `Transition enum (8 variants, core)`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **Why does `render_editor()` connect `Editor Detail Pane` to `Block Rendering Types`, `TUI App Core`, `Graph Overlay UI`, `Textarea Widget`, `Help Overlay`, `Window Chrome UI`?**
  _High betweenness centrality (0.119) - this node is a cross-community bridge._
- **Why does `Event` connect `Editor Interaction Tests` to `CLI Commands`, `TUI App Core`, `Action Routing`, `Textarea Widget`, `Keybindings Config`?**
  _High betweenness centrality (0.110) - this node is a cross-community bridge._
- **Why does `load_graph()` connect `Graph Loading Tests` to `CLI Commands`, `Graph Loader`, `TUI Test Harness`, `Validate Command`, `Command History Tests`?**
  _High betweenness centrality (0.076) - this node is a cross-community bridge._
- **Are the 16 inferred relationships involving `load_graph()` (e.g. with `run_editor()` and `run_event_loop()`) actually correct?**
  _`load_graph()` has 16 INFERRED edges - model-reasoned connections that need verification._
- **Are the 14 inferred relationships involving `load_graph_from_str()` (e.g. with `graph_with_ids()` and `move_block_roundtrips_with_undo_redo()`) actually correct?**
  _`load_graph_from_str()` has 14 INFERRED edges - model-reasoned connections that need verification._
- **Are the 4 inferred relationships involving `render_block_with_tokens()` (e.g. with `render_divider()` and `render_heading()`) actually correct?**
  _`render_block_with_tokens()` has 4 INFERRED edges - model-reasoned connections that need verification._