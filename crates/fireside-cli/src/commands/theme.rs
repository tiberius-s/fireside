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

    let toml_content = format!(
        "# Fireside theme imported from: {source}\n\
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

    let themes_dir = std::env::var_os("HOME")
        .map(|h| Path::new(&h).join(".config/fireside/themes"))
        .unwrap_or_else(|| Path::new("themes").to_path_buf());
    std::fs::create_dir_all(&themes_dir).context("creating themes directory")?;

    let theme_path = themes_dir.join(format!("{theme_name}.toml"));
    std::fs::write(&theme_path, toml_content).context("writing theme file")?;

    println!("Imported theme '{theme_name}' to {}", theme_path.display());
    Ok(())
}
