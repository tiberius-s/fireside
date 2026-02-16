//! Fireside entry point.
//!
//! Parses CLI arguments and dispatches to command handlers.

use std::io;
use std::path::Path;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

use commands::fonts::list_fonts;
use commands::project::run_project;
use commands::scaffold::{scaffold_presentation, scaffold_project};
use commands::session::{run_editor, run_presentation};
use commands::theme::import_iterm2_theme;
use commands::validate::run_validate;

/// Fireside — a portable format for branching presentations and lessons.
#[derive(Debug, Parser)]
#[command(name = "fireside", version, about, long_about = None)]
struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    command: Option<Command>,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Present a Fireside graph in the terminal.
    Present {
        /// Path to the JSON graph file.
        file: std::path::PathBuf,

        /// Theme to use (overrides document metadata). Can be a name or path to .itermcolors/.toml.
        #[arg(short, long)]
        theme: Option<String>,

        /// Start at a specific node number (1-indexed).
        #[arg(short = 's', long, default_value = "1")]
        start: usize,
    },

    /// Open a Fireside project directory.
    Open {
        /// Path to the project directory (must contain fireside.yml).
        dir: std::path::PathBuf,

        /// Theme override.
        #[arg(short, long)]
        theme: Option<String>,
    },

    /// Open the node editor for a file or project.
    Edit {
        /// Path to a JSON file or project directory.
        path: Option<std::path::PathBuf>,
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
        dir: std::path::PathBuf,
    },

    /// Validate a Fireside graph for structural integrity.
    Validate {
        /// Path to the JSON graph file to validate.
        file: std::path::PathBuf,
    },

    /// List installed monospace fonts.
    Fonts,

    /// Import an iTerm2 color scheme (.itermcolors) as a Fireside theme.
    ImportTheme {
        /// Path to the .itermcolors file.
        file: std::path::PathBuf,

        /// Name for the imported theme.
        #[arg(short, long)]
        name: Option<String>,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .with_writer(io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Present { file, theme, start }) => {
            run_presentation(&file, theme.as_deref(), start)?;
        }
        Some(Command::Open { dir, theme }) => {
            run_project(&dir, theme.as_deref())?;
        }
        Some(Command::Edit { path }) => {
            let target = path.as_deref().unwrap_or(Path::new("."));
            run_editor(target)?;
        }
        Some(Command::New { name, project, dir }) => {
            if project {
                scaffold_project(&name, &dir)?;
            } else {
                scaffold_presentation(&name, &dir)?;
            }
        }
        Some(Command::Validate { file }) => {
            run_validate(&file)?;
        }
        Some(Command::Fonts) => {
            list_fonts();
        }
        Some(Command::ImportTheme { file, name }) => {
            import_iterm2_theme(&file, name.as_deref())?;
        }
        None => {
            println!("Fireside — Portable Branching Presentations");
            println!();
            println!("Usage:");
            println!("  fireside present <file.json>   Present a graph in the terminal");
            println!("  fireside open <dir>            Open a project directory");
            println!("  fireside edit [path]           Edit nodes in the TUI editor");
            println!("  fireside new <name>            Scaffold a new presentation");
            println!("  fireside new <name> -p         Scaffold a new project directory");
            println!("  fireside validate <file.json>  Validate a graph file");
            println!("  fireside fonts                 List installed monospace fonts");
            println!("  fireside import-theme <file>   Import an iTerm2 color scheme");
            println!();
            println!("Run `fireside --help` for full options.");
        }
    }

    Ok(())
}
