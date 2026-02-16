use fireside_tui::design::fonts;

/// List installed monospace fonts.
pub fn list_fonts() {
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
