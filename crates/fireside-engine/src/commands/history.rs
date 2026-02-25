//! Undo/redo command history.
//!
//! [`CommandHistory`] maintains two stacks — *applied* and *undone* — and
//! routes all mutations through [`super::apply::apply_command`] so that
//! every change is automatically reversible.

use fireside_core::model::graph::Graph;

use crate::error::EngineError;

use super::apply::apply_command;
use super::types::Command;

/// Internal entry pairing a command with its pre-computed inverse.
#[derive(Debug, Clone)]
struct HistoryEntry {
    command: Command,
    inverse: Command,
}

/// Undo/redo stack for graph mutations.
///
/// All mutations should go through [`CommandHistory::apply_command`] so
/// that every change can be undone and redone without additional
/// bookkeeping at the call site.
#[derive(Debug)]
pub struct CommandHistory {
    /// Applied commands with their inverses (for undo).
    applied: Vec<HistoryEntry>,
    /// Undone commands with their inverses (for redo).
    undone: Vec<HistoryEntry>,
}

impl CommandHistory {
    /// Create an empty command history.
    #[must_use]
    pub fn new() -> Self {
        Self {
            applied: Vec::new(),
            undone: Vec::new(),
        }
    }

    /// Apply a command to the graph and record it for undo.
    ///
    /// Applying a new command clears the redo stack.
    ///
    /// # Errors
    ///
    /// Returns an `EngineError::CommandError` when the command is invalid
    /// for the current graph state.
    pub fn apply_command(
        &mut self,
        graph: &mut Graph,
        command: Command,
    ) -> Result<(), EngineError> {
        let inverse = apply_command(graph, &command)?;
        self.applied.push(HistoryEntry { command, inverse });
        self.undone.clear();
        Ok(())
    }

    /// Undo the most recent applied command.
    ///
    /// Returns `Ok(true)` if a command was undone, `Ok(false)` if there is
    /// nothing to undo.
    ///
    /// # Errors
    ///
    /// Returns an `EngineError` if applying the inverse command fails.
    pub fn undo(&mut self, graph: &mut Graph) -> Result<bool, EngineError> {
        let Some(entry) = self.applied.pop() else {
            return Ok(false);
        };

        apply_command(graph, &entry.inverse)?;
        self.undone.push(entry);
        Ok(true)
    }

    /// Redo the most recently undone command.
    ///
    /// Returns `Ok(true)` if a command was redone, `Ok(false)` if there is
    /// nothing to redo.
    ///
    /// # Errors
    ///
    /// Returns an `EngineError` if applying the command fails.
    pub fn redo(&mut self, graph: &mut Graph) -> Result<bool, EngineError> {
        let Some(entry) = self.undone.pop() else {
            return Ok(false);
        };

        apply_command(graph, &entry.command)?;
        self.applied.push(entry);
        Ok(true)
    }

    /// Returns `true` if there are commands to undo.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.applied.is_empty()
    }

    /// Returns `true` if there are commands to redo.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}
