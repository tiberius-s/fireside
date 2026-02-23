//! Rendering pipeline — converts content blocks to ratatui widgets.
//!
//! This module maps [`fireside_core::model::content::ContentBlock`]s into ratatui primitives for display
//! in the terminal.

pub mod blocks;
pub(crate) mod blocks_extension;
pub(crate) mod blocks_image;
pub mod code;
pub mod layout;
