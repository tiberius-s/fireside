//! Slideways entry point.
//!
//! Parses CLI arguments, initializes the terminal, loads the slide deck,
//! and runs the main event loop. Supports standalone TUI launch (no args),
//! project directories, and single-file presentation.

use std::io;
use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event;
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use slideways::app::App;
use slideways::cli::{Cli, Command};
use slideways::config;
use slideways::design::fonts;
use slideways::design::iterm2::Iterm2Scheme;
use slideways::loader;
use slideways::project::Project;

use ratatui::style::Color;

fn main() -> Result<()> {
    // Initialize logging
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
        Some(Command::Fonts) => {
            list_fonts();
        }
        Some(Command::ImportTheme { file, name }) => {
            import_iterm2_theme(&file, name.as_deref())?;
        }
        None => {
            // No subcommand: launch dashboard/editor TUI
            println!("Slideways — Terminal Presentation Tool");
            println!();
            println!("Usage:");
            println!("  slideways present <file.json>  Present a slide deck");
            println!("  slideways open <dir>           Open a project directory");
            println!("  slideways edit [path]          Edit slides in the TUI editor");
            println!("  slideways new <name>           Scaffold a new presentation");
            println!("  slideways new <name> -p        Scaffold a new project directory");
            println!("  slideways fonts                List installed monospace fonts");
            println!("  slideways import-theme <file>  Import an iTerm2 color scheme");
            println!();
            println!("Run `slideways --help` for full options.");
        }
    }

    Ok(())
}

/// Run the interactive presentation.
fn run_presentation(file: &Path, theme_name: Option<&str>, start_slide: usize) -> Result<()> {
    // Load the slide deck
    let deck = loader::load_deck(file).context("loading slide deck")?;

    // Resolve theme: CLI flag > frontmatter > default
    let effective_theme = theme_name.or(deck.metadata.theme.as_deref());
    let theme = config::resolve_theme(effective_theme);

    // Create app state (convert 1-based start to 0-based)
    let mut app = App::new(deck, theme, start_slide.saturating_sub(1));

    // Initialize terminal
    enable_raw_mode().context("enabling raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("entering alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("creating terminal")?;

    // Main event loop
    let result = run_event_loop(&mut terminal, &mut app);

    // Restore terminal (always, even if the loop errored)
    disable_raw_mode().context("disabling raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).context("leaving alternate screen")?;
    terminal.show_cursor().context("showing cursor")?;

    result
}

/// The main event loop implementing the TEA pattern.
fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        // View: render the current state
        terminal
            .draw(|frame| app.view(frame))
            .context("drawing frame")?;

        // Check for quit
        if app.should_quit() {
            break;
        }

        // Poll for events with timeout (allows Tick actions for animations)
        if event::poll(std::time::Duration::from_millis(250)).context("polling events")? {
            let ev = event::read().context("reading event")?;
            app.handle_event(ev);
        }
    }

    Ok(())
}

/// Scaffold a new presentation file from a template.
fn scaffold_presentation(name: &str, dir: &Path) -> Result<()> {
    let filename = if name.ends_with(".json") {
        name.to_owned()
    } else {
        format!("{name}.json")
    };

    let path = dir.join(&filename);

    if path.exists() {
        anyhow::bail!("File already exists: {}", path.display());
    }

    let date = chrono_free_date();
    let template = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/slideways/slideways/main/schemas/presentation.schema.json",
        "title": name,
        "author": "Your Name",
        "date": date,
        "theme": "default",
        "defaults": {
            "layout": "top",
            "transition": "fade"
        },
        "slides": [
            {
                "id": "title",
                "layout": "center",
                "content": [
                    { "type": "heading", "level": 1, "text": name },
                    { "type": "text", "body": "Your presentation starts here" }
                ]
            },
            {
                "content": [
                    { "type": "heading", "level": 2, "text": "Slide 2" },
                    {
                        "type": "list",
                        "ordered": false,
                        "items": [
                            { "text": "First point" },
                            { "text": "Second point" },
                            { "text": "Third point" }
                        ]
                    }
                ]
            },
            {
                "content": [
                    { "type": "heading", "level": 2, "text": "Code Example" },
                    {
                        "type": "code",
                        "language": "rust",
                        "source": "fn main() {\n    println!(\"Hello from Slideways!\");\n}"
                    }
                ]
            },
            {
                "layout": "center",
                "content": [
                    { "type": "heading", "level": 2, "text": "Thank You" },
                    { "type": "text", "body": "Questions?" }
                ]
            }
        ]
    });

    std::fs::create_dir_all(dir).context("creating output directory")?;
    let json_str = serde_json::to_string_pretty(&template).context("serializing template")?;
    std::fs::write(&path, json_str).context("writing presentation file")?;

    println!("Created new presentation: {}", path.display());
    Ok(())
}

/// Get today's date as a string without requiring the chrono crate.
fn chrono_free_date() -> String {
    // Simple date from system time — YYYY-MM-DD format
    // This avoids adding chrono as a dependency for one use case
    let now = std::time::SystemTime::now();
    let since_epoch = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let days = since_epoch.as_secs() / 86400;

    // Approximate date calculation (good enough for scaffolding)
    let year = 1970 + (days / 365);
    let remaining = days % 365;
    let month = remaining / 30 + 1;
    let day = remaining % 30 + 1;

    format!("{year:04}-{month:02}-{day:02}")
}

/// Open and present a project directory.
fn run_project(dir: &Path, theme_name: Option<&str>) -> Result<()> {
    let project = Project::load(dir).context("loading project")?;

    let paths = project.slide_paths();
    let entry = paths.first().context("project has no slide files")?;

    run_presentation(entry, theme_name, 1)
}

/// Open the TUI editor for a file or project.
fn run_editor(_target: &Path) -> Result<()> {
    anyhow::bail!("The slide editor is not yet implemented");
}

/// Scaffold a new project directory with slideways.yml and an initial presentation.
fn scaffold_project(name: &str, dir: &Path) -> Result<()> {
    let project_dir = dir.join(name);

    if project_dir.exists() {
        anyhow::bail!("Directory already exists: {}", project_dir.display());
    }

    std::fs::create_dir_all(&project_dir).context("creating project directory")?;

    // Create slideways.yml
    let config = format!(
        "# Slideways project configuration\n\
         name: {name}\n\
         slides:\n\
         - slides/main.json\n\
         theme: default\n"
    );
    std::fs::write(project_dir.join("slideways.yml"), config).context("writing project config")?;

    // Create slides directory and initial presentation
    let slides_dir = project_dir.join("slides");
    std::fs::create_dir_all(&slides_dir).context("creating slides directory")?;
    scaffold_presentation("main", &slides_dir)?;

    // Create themes directory
    std::fs::create_dir_all(project_dir.join("themes")).context("creating themes directory")?;

    println!("Created new project: {}", project_dir.display());
    Ok(())
}

/// List installed monospace fonts.
fn list_fonts() {
    let discovered = fonts::list_monospace_fonts();
    if discovered.is_empty() {
        println!("No monospace fonts detected.");
    } else {
        println!("Installed monospace fonts:");
        for font in &discovered {
            println!("  {}", font.family);
        }
    }
}

/// Import an iTerm2 color scheme as a Slideways theme.
fn import_iterm2_theme(file: &Path, name: Option<&str>) -> Result<()> {
    let scheme = Iterm2Scheme::load(file).context("loading iTerm2 color scheme")?;
    let theme_name = name.unwrap_or(&scheme.name);
    let tokens = scheme.to_tokens();

    // Convert token colors to hex strings for the TOML theme file
    let to_hex = |c: Color| -> String {
        match c {
            Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
            _ => String::from("#000000"),
        }
    };

    let toml_content = format!(
        "# Slideways theme imported from: {source}\n\
         background = \"{bg}\"\n\
         foreground = \"{fg}\"\n\
         heading_h1 = \"{h1}\"\n\
         heading_h2 = \"{h2}\"\n\
         heading_h3 = \"{h3}\"\n\
         code_background = \"{code_bg}\"\n\
         code_foreground = \"{code_fg}\"\n\
         code_border = \"{border}\"\n\
         block_quote = \"{quote}\"\n\
         footer = \"{footer}\"\n",
        source = file.display(),
        bg = to_hex(tokens.background),
        fg = to_hex(tokens.on_background),
        h1 = to_hex(tokens.heading_h1),
        h2 = to_hex(tokens.heading_h2),
        h3 = to_hex(tokens.heading_h3),
        code_bg = to_hex(tokens.code_bg),
        code_fg = to_hex(tokens.code_fg),
        border = to_hex(tokens.border_inactive),
        quote = to_hex(tokens.quote),
        footer = to_hex(tokens.footer),
    );

    // Write to ~/.config/slideways/themes/
    let themes_dir = std::env::var_os("HOME")
        .map(|h| Path::new(&h).join(".config/slideways/themes"))
        .unwrap_or_else(|| Path::new("themes").to_path_buf());
    std::fs::create_dir_all(&themes_dir).context("creating themes directory")?;

    let theme_path = themes_dir.join(format!("{theme_name}.toml"));
    std::fs::write(&theme_path, toml_content).context("writing theme file")?;

    println!("Imported theme '{theme_name}' to {}", theme_path.display());
    Ok(())
}
