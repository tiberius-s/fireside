//! Command type definitions for graph mutation.
//!
//! Defines the [`Command`] enum whose variants represent every atomic
//! mutation that the editor can perform on a [`Graph`].  Commands are
//! value types — they carry all the data needed to apply *and* undo the
//! operation without holding any mutable references.

use fireside_core::model::content::ContentBlock;
use fireside_core::model::node::{Node, NodeId};

/// A command that mutates the graph within a session.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// Update the content blocks of a node.
    UpdateNodeContent {
        /// Target node ID.
        node_id: NodeId,
        /// New content blocks.
        content: Vec<ContentBlock>,
    },

    /// Update a specific content block in a node.
    UpdateBlock {
        /// Target node ID.
        node_id: NodeId,
        /// Zero-based block index.
        block_index: usize,
        /// New block value.
        block: ContentBlock,
    },

    /// Move a content block within a node.
    MoveBlock {
        /// Target node ID.
        node_id: NodeId,
        /// Source zero-based block index.
        from_index: usize,
        /// Destination zero-based block index.
        to_index: usize,
    },

    /// Add a new node to the graph.
    AddNode {
        /// The node ID for the new node.
        node_id: NodeId,
        /// Insert after this node index (None = append).
        after_index: Option<usize>,
    },

    /// Restore a previously removed node at an index.
    RestoreNode {
        /// Full node data to restore.
        node: Node,
        /// Index at which to restore.
        index: usize,
    },

    /// Remove a node from the graph.
    RemoveNode {
        /// The node ID to remove.
        node_id: NodeId,
    },

    /// Remove a specific content block from a node.
    RemoveBlock {
        /// Target node ID.
        node_id: NodeId,
        /// Zero-based index of the block to remove.
        block_index: usize,
    },

    /// Insert a content block into a node at a specific position.
    ///
    /// Used as the undo inverse of `RemoveBlock`.
    InsertBlock {
        /// Target node ID.
        node_id: NodeId,
        /// Zero-based insertion index.
        block_index: usize,
        /// The block to insert.
        block: ContentBlock,
    },

    /// Set the traversal next override for a node.
    SetTraversalNext {
        /// Source node ID.
        node_id: NodeId,
        /// Target node ID for the next override.
        target: NodeId,
    },

    /// Clear the traversal next override for a node.
    ClearTraversalNext {
        /// Node ID to clear.
        node_id: NodeId,
    },
}
