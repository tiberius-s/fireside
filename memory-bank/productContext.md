# Fireside — Product Context

## Why Fireside Exists

Fireside is a **portable format for branching presentations and lessons**.
It defines a JSON-based directed graph that any conforming engine can render,
while the reference implementation targets the terminal.

Developers frequently present at meetups, conferences, and internal team
meetings. Many prefer tools that integrate with their workflow — text editors,
version control, and terminals. Existing tools like presenterm provide basic
terminal presentation capability, but lack interactive branching, visual flair,
and a portable specification.

## Problems It Solves

1. **Linear presentations are limiting** — Talks often need audience-driven
   navigation, detours, or optional deep-dives. Fireside enables branch points
   where the presenter can choose different routes through the material.
2. **No portable format** — Existing terminal presentation tools use
   tool-specific formats. Fireside defines a spec-first, implementation-independent
   graph format that any runtime with JSON parsing can consume.
3. **Terminal presentations look plain** — While terminal UIs are functional,
   they lack visual engagement. Fireside adds retro visual effects (ASCII art,
   pixel art, animated transitions) that are both distinctive and terminal-native.
4. **Context switching** — Developers leave their terminal to use GUI
   presentation tools. Fireside keeps everything in the terminal.
5. **Theme fragmentation** — Users want to use their terminal color scheme
   seamlessly. Fireside imports iTerm2 color schemes directly.

## 8 Canonical User Journeys

The specification documents 8 canonical user journeys:

1. **Linear walkthrough** — Author creates a straight-line presentation; engine
   advances through nodes sequentially via Next.
2. **Choose-your-own-adventure** — Audience votes at branch points; presenter
   selects an option via Choose.
3. **Training course with quiz** — Lesson nodes lead to quiz branch points;
   correct/incorrect paths diverge and rejoin.
4. **Conference talk with deep-dives** — Main track has optional branch points
   to detailed examples; presenter can skip or explore.
5. **Team onboarding** — Multi-day onboarding graph with per-role branches;
   each hire gets a tailored traversal.
6. **Product demo** — Sales demo with feature branches; presenter adapts to
   audience interest.
7. **Interactive workshop** — Hands-on exercises interspersed with instruction;
   branch points let participants choose difficulty.
8. **Portfolio presentation** — Non-linear portfolio where the viewer navigates
   project showcases via Goto operations.

## How It Should Work

1. Author a Fireside graph as JSON (with `$schema` for IDE autocompletion)
2. Run `fireside present graph.json` to start a terminal session
3. Navigate with keyboard (arrow keys, vim bindings)
4. At branch points, choose an option and continue
5. Import themes from iTerm2 color schemes; fonts restricted to monospace

## User Experience Goals

- **Zero friction** — Launch directly into a presentation
- **Structured format** — JSON with schema validation for IDE autocompletion
- **Responsive** — Instant traversal, smooth transitions
- **Discoverable** — `?` for help, intuitive keybindings
- **Customizable** — iTerm2 themes, monospace fonts, per-node directives
- **Portable** — Spec-defined format, not locked to one implementation
