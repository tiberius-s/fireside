use std::path::Path;

use anyhow::{Context, Result, bail};
use ratatui::style::Color;

use fireside_tui::design::iterm2::Iterm2Scheme;
use fireside_tui::design::tokens::DesignTokens;
use fireside_tui::design::vscode::VscodeScheme;

/// Import a terminal color scheme as a Fireside theme.
///
/// Accepts both iTerm2 `.itermcolors` plist files and VS Code JSON files
/// from the [iTerm2-Color-Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes)
/// `vscode/` directory. The format is auto-detected from the file extension.
pub fn import_iterm2_theme(file: &Path, name: Option<&str>) -> Result<()> {
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");

    let (tokens, default_name): (DesignTokens, String) = match ext {
        "itermcolors" => {
            let scheme = Iterm2Scheme::load(file).context("loading iTerm2 color scheme")?;
            let n = scheme.name.clone();
            (scheme.to_tokens(), n)
        }
        "json" => {
            let scheme = VscodeScheme::load(file).context("loading VS Code color scheme")?;
            let n = scheme.name.clone();
            (scheme.to_tokens(), n)
        }
        other => bail!("unsupported theme format '.{other}'; expected .itermcolors or .json"),
    };

    let theme_name = name.unwrap_or(&default_name);
    write_theme_file(file, theme_name, &tokens)
}

/// Serialise design tokens to a Fireside ThemeFile JSON and write to
/// `~/.config/fireside/themes/<name>.json`.
fn write_theme_file(source: &Path, theme_name: &str, tokens: &DesignTokens) -> Result<()> {
    let to_hex = |c: Color| -> String {
        match c {
            Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
            _ => String::from("#000000"),
        }
    };

    // All 19 ThemeFile fields — keeps the exported JSON in sync with the
    // expanded Theme struct from the previous session.
    let json_content = serde_json::json!({
        "source": source.display().to_string(),
        // Core surface colors
        "background":   to_hex(tokens.background),
        "surface":      to_hex(tokens.surface),
        // Foreground
        "foreground":   to_hex(tokens.on_background),
        "on_surface":   to_hex(tokens.on_surface),
        // Typography scale
        "heading_h1":   to_hex(tokens.heading_h1),
        "heading_h2":   to_hex(tokens.heading_h2),
        "heading_h3":   to_hex(tokens.heading_h3),
        // Code block
        "code_background": to_hex(tokens.code_bg),
        "code_foreground": to_hex(tokens.code_fg),
        // Semantic accents
        "accent":       to_hex(tokens.accent),
        "error":        to_hex(tokens.error),
        "success":      to_hex(tokens.success),
        // Borders
        "border_active":   to_hex(tokens.border_active),
        "border_inactive": to_hex(tokens.border_inactive),
        // Toolbar / footer
        "toolbar_bg":   to_hex(tokens.toolbar_bg),
        "toolbar_fg":   to_hex(tokens.toolbar_fg),
        // Misc
        "block_quote":  to_hex(tokens.quote),
        "code_border":  to_hex(tokens.border_inactive),
        "footer":       to_hex(tokens.footer)
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
