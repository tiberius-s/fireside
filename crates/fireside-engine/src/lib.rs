//! Fireside engine — the protocol's behavior, with no UI attached.
//!
//! Two responsibilities, both pure logic over `fireside-core` types:
//!
//! - [`validation`]: Layer-2 semantic checks (spec §4) with
//!   presenter-friendly diagnostics.
//! - [`session`]: the §3 traversal state machine. Every operation returns
//!   an [`Outcome`] so frontends can give feedback for every action.
//!
//! No file I/O, no rendering, no terminal — callers load text, this crate
//! gives them a validated, navigable presentation.

pub mod error;
pub mod session;
pub mod validation;

pub use error::EngineError;
pub use session::{Outcome, Session};
pub use validation::{Diagnostic, Severity, has_errors, validate};
