//! CLI argument parsing using clap derive.
//!
//! Defines the `slideways` command-line interface with subcommands:
//! - `present` — load and present a slide deck
//! - `open` — open a Slideways project directory
//! - `edit` — open the slide editor for a file or project
//! - `new` — scaffold a new presentation

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Slideways — a terminal-based presentation tool with branching paths and retro visual effects.
#[derive(Debug, Parser)]
#[command(name = "slideways", version, about, long_about = None)]
pub struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Present a slide deck in the terminal.
    Present {
        /// Path to the JSON presentation file.
        file: PathBuf,

        /// Theme to use (overrides frontmatter). Can be a name or path to .itermcolors/.toml.
        #[arg(short, long)]
        theme: Option<String>,

        /// Start at a specific slide number (1-indexed).
        #[arg(short = 's', long, default_value = "1")]
        start: usize,
    },

    /// Open a Slideways project directory.
    Open {
        /// Path to the project directory (must contain slideways.yml).
        dir: PathBuf,

        /// Theme override.
        #[arg(short, long)]
        theme: Option<String>,
    },

    /// Open the slide editor for a file or project.
    Edit {
        /// Path to a markdown file or project directory.
        path: Option<PathBuf>,
    },

    /// Scaffold a new presentation or project.
    New {
        /// Name for the new presentation.
        name: String,

        /// Create a full project directory instead of a single file.
        #[arg(short, long)]
        project: bool,

        /// Directory to create the file/project in.
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },

    /// List installed monospace fonts.
    Fonts,

    /// Import an iTerm2 color scheme (.itermcolors) as a Slideways theme.
    ImportTheme {
        /// Path to the .itermcolors file.
        file: PathBuf,

        /// Name for the imported theme.
        #[arg(short, long)]
        name: Option<String>,
    },
}
