//! `fireside edit <deck>`: the full-screen authoring studio (spec 013).
//!
//! Scaffolded incrementally per `specs/013-authoring-editor/tasks.md` —
//! this module gains the opening-rules chain, the draft sidecar, and the
//! save/conflict-guard plumbing as the Foundational and User Story phases
//! land. Until then the command parses but refuses to run.

use std::path::Path;

use anyhow::{Result, bail};

/// Entry point for `fireside edit <file>`. Placeholder until
/// `specs/013-authoring-editor/tasks.md` T024 wires the opening-rules
/// chain and the editor event loop.
pub(crate) fn edit_deck(_file: &Path) -> Result<()> {
    bail!("fireside edit is under construction (spec 013) — not yet available")
}
