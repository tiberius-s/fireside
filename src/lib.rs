//! Slideways — a terminal-based presentation tool with branching paths and retro visual effects.
//!
//! This library crate re-exports the core modules for building and presenting
//! slide decks in the terminal using Ratatui. Presentations are JSON-native,
//! using structured schemas instead of markdown.

pub mod app;
pub mod cli;
pub mod config;
pub mod design;
pub mod error;
pub mod event;
pub mod loader;
pub mod model;
pub mod project;
pub mod render;
pub mod ui;
