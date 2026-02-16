//! Fireside TUI — Ratatui-based terminal presentation and editing engine.
//!
//! This crate provides the terminal user interface for Fireside, implementing
//! both present mode and (future) edit mode. It depends on `fireside-core`
//! for protocol types and `fireside-engine` for traversal and session logic.
//!
//! # Architecture
//!
//! TEA (The Elm Architecture) pattern:
//! ```text
//! Event (crossterm) → Action (enum) → App::update(&mut self, action) → View (ratatui)
//! ```

pub mod app;
pub mod config;
pub mod design;
pub mod error;
pub mod event;
pub mod render;
pub mod theme;
pub mod ui;

pub use app::App;
pub use event::Action;
pub use theme::Theme;
