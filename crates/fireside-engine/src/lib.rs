//! Fireside Engine — document lifecycle, validation, traversal, and graph mutation.
//!
//! This crate owns the business logic layer between the protocol model
//! (`fireside-core`) and any frontend. It provides:
//!
//! - **Loading**: JSON document parsing into validated `Graph` instances
//! - **Validation**: graph integrity checks (dangling refs, duplicate IDs, etc.)
//! - **Traversal**: state machine implementing Next, Choose, Goto, Back operations
//! - **Session**: mutable presentation session with dirty tracking
//! - **Commands**: graph mutation API for editor support (with undo/redo)

pub mod commands;
pub mod error;
pub mod loader;
pub mod session;
pub mod traversal;
pub mod validation;

pub use error::EngineError;
pub use loader::{load_graph, load_graph_from_str};
pub use session::PresentationSession;
pub use traversal::TraversalEngine;
