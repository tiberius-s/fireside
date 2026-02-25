//! Rendering pipeline — converts content blocks to ratatui widgets.
//!
//! This module maps [`fireside_core::model::content::ContentBlock`]s into ratatui primitives for display
//! in the terminal.

pub mod blocks;
pub(crate) mod blocks_code;
pub(crate) mod blocks_divider;
pub(crate) mod blocks_extension;
pub(crate) mod blocks_heading;
pub(crate) mod blocks_image;
pub(crate) mod blocks_list;
pub(crate) mod blocks_text;
pub mod code;
pub mod layout;
