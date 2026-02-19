use std::path::Path;

use anyhow::{Context, Result};
use ratatui::style::Color;

use fireside_tui::design::iterm2::Iterm2Scheme;

/// Import an iTerm2 color scheme as a Fireside theme.
pub fn import_iterm2_theme(file: &Path, name: Option<&str>) -> Result<()> {
    let scheme = Iterm2Scheme::load(file).context("loading iTerm2 color scheme")?;
    let theme_name = name.unwrap_or(&scheme.name);
    let tokens = scheme.to_tokens();

    let to_hex = |c: Color| -> String {
        match c {
            Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
            _ => String::from("#000000"),
        }
    };

    let json_content = serde_json::json!({
        "source": file.display().to_string(),
        "background": to_hex(tokens.background),
        "foreground": to_hex(tokens.on_background),
        "heading_h1": to_hex(tokens.heading_h1),
        "heading_h2": to_hex(tokens.heading_h2),
        "heading_h3": to_hex(tokens.heading_h3),
        "code_background": to_hex(tokens.code_bg),
        "code_foreground": to_hex(tokens.code_fg),
        "code_border": to_hex(tokens.border_inactive),
        "block_quote": to_hex(tokens.quote),
        "footer": to_hex(tokens.footer)
    });

    let themes_dir = std::env::var_os("HOME")
        .map(|h| Path::new(&h).join(".config/fireside/themes"))
        .unwrap_or_else(|| Path::new("themes").to_path_buf());
    std::fs::create_dir_all(&themes_dir).context("creating themes directory")?;

    let theme_path = themes_dir.join(format!("{theme_name}.json"));
    let json_text =
        serde_json::to_string_pretty(&json_content).context("serializing theme json")?;
    std::fs::write(&theme_path, json_text).context("writing theme file")?;

    println!("Imported theme '{theme_name}' to {}", theme_path.display());
    Ok(())
}
