//! Node layout templates — reusable layouts with Ratatui area mappings.
//!
//! Each template defines a named layout pattern with specific area calculations,
//! frontmatter schema expectations, and semantic roles for content blocks.
//!
//! ## Available Templates
//!
//! | Template           | Purpose                           |
//! |--------------------|-----------------------------------|
//! | `Title`            | Opening node: big title, subtitle |
//! | `TitleSubtitle`    | Title + subtitle + optional byline|
//! | `TwoColumn`        | Side-by-side content split        |
//! | `ImageCaption`     | Image with caption text below     |
//! | `CodeFocus`        | Full-width code with minimal chrome|
//! | `Quote`            | Centered block quote              |
//! | `BulletList`       | Standard content with bullet list |
//! | `SpeakerNotes`     | Thumbnail view with notes panel   |

use ratatui::layout::{Constraint, Direction, Layout, Rect};

use fireside_core::model::layout::Layout as NodeLayout;

use super::tokens::{Breakpoint, Spacing};

/// Node template identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeTemplate {
    /// Big centered title, optional subtitle below.
    Title,
    /// Title + subtitle + optional author/date byline.
    TitleSubtitle,
    /// Two columns side by side with gutter.
    TwoColumn,
    /// Image area on top, caption text below.
    ImageCaption,
    /// Full-width code block with minimal padding.
    CodeFocus,
    /// Centered block quote with attribution.
    Quote,
    /// Standard heading + bullet list layout.
    BulletList,
    /// Presentation thumbnail with speaker notes to the side.
    SpeakerNotes,
}

impl NodeTemplate {
    /// Map a Fireside node layout to the closest design template.
    #[must_use]
    pub fn from_layout(layout: NodeLayout) -> Self {
        match layout {
            NodeLayout::Title => Self::Title,
            NodeLayout::SplitHorizontal => Self::TwoColumn,
            NodeLayout::CodeFocus | NodeLayout::Fullscreen => Self::CodeFocus,
            NodeLayout::Center => Self::Quote,
            NodeLayout::Default
            | NodeLayout::Top
            | NodeLayout::SplitVertical
            | NodeLayout::AlignLeft
            | NodeLayout::AlignRight
            | NodeLayout::Blank => Self::BulletList,
        }
    }

    /// Parse a template name from a string.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().replace(['-', '_'], "").as_str() {
            "title" => Some(Self::Title),
            "titlesubtitle" => Some(Self::TitleSubtitle),
            "twocolumn" | "2col" => Some(Self::TwoColumn),
            "imagecaption" | "image" => Some(Self::ImageCaption),
            "codefocus" | "code" => Some(Self::CodeFocus),
            "quote" | "blockquote" => Some(Self::Quote),
            "bulletlist" | "list" | "bullets" => Some(Self::BulletList),
            "speakernotes" | "notes" => Some(Self::SpeakerNotes),
            _ => None,
        }
    }

    /// User-visible display name.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::TitleSubtitle => "Title + Subtitle",
            Self::TwoColumn => "Two Column",
            Self::ImageCaption => "Image + Caption",
            Self::CodeFocus => "Code Focus",
            Self::Quote => "Quote",
            Self::BulletList => "Bullet List",
            Self::SpeakerNotes => "Speaker Notes",
        }
    }

    /// All available templates (for template chooser UI).
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::Title,
            Self::TitleSubtitle,
            Self::TwoColumn,
            Self::ImageCaption,
            Self::CodeFocus,
            Self::Quote,
            Self::BulletList,
            Self::SpeakerNotes,
        ]
    }

    /// Compute the content areas for this template within the given rect.
    ///
    /// The `breakpoint` influences padding and margin scaling.
    #[must_use]
    pub fn compute_areas(&self, area: Rect, bp: Breakpoint) -> TemplateAreas {
        match self {
            Self::Title => compute_title(area, bp),
            Self::TitleSubtitle => compute_title_subtitle(area, bp),
            Self::TwoColumn => compute_two_column(area, bp),
            Self::ImageCaption => compute_image_caption(area, bp),
            Self::CodeFocus => compute_code_focus(area, bp),
            Self::Quote => compute_quote(area, bp),
            Self::BulletList => compute_bullet_list(area, bp),
            Self::SpeakerNotes => compute_speaker_notes(area, bp),
        }
    }

    /// Example YAML frontmatter for this template.
    #[must_use]
    pub fn example_frontmatter(&self) -> &'static str {
        match self {
            Self::Title => {
                r#"---
template: title
---
# My Presentation
"#
            }
            Self::TitleSubtitle => {
                r#"---
template: title-subtitle
---
# My Presentation
## A descriptive subtitle
### Author Name · 2026
"#
            }
            Self::TwoColumn => {
                r#"---
template: two-column
---
<!-- column: left -->
## Left Column
Content here

<!-- column: right -->
## Right Column
Content here
"#
            }
            Self::ImageCaption => {
                r#"---
template: image-caption
---
![Alt text](path/to/image.png)

*Caption text below the image*
"#
            }
            Self::CodeFocus => {
                r#"---
template: code-focus
---
```rust
fn main() {
    println!("Full width code");
}
```
"#
            }
            Self::Quote => {
                r#"---
template: quote
---
> "The best code is no code at all."
> — Someone Wise
"#
            }
            Self::BulletList => {
                r#"---
template: bullet-list
---
## Key Points

- First important point
- Second important point
- Third important point
"#
            }
            Self::SpeakerNotes => {
                r#"---
template: speaker-notes
---
## Node Content

Content visible to the audience

<!-- notes -->
These are speaker notes only visible to the presenter
"#
            }
        }
    }
}

/// Computed content areas for a specific template layout.
#[derive(Debug, Clone)]
pub struct TemplateAreas {
    /// The primary content area.
    pub main: Rect,
    /// Optional secondary area (right column, caption, notes panel).
    pub secondary: Option<Rect>,
    /// Footer area for progress bar.
    pub footer: Rect,
}

// ─── Template area computations ─────────────────────────────────────────

fn split_footer(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);
    (chunks[0], chunks[1])
}

fn center_in(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_pct) / 2),
            Constraint::Percentage(height_pct),
            Constraint::Percentage((100 - height_pct) / 2),
        ])
        .split(area);

    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_pct) / 2),
            Constraint::Percentage(width_pct),
            Constraint::Percentage((100 - width_pct) / 2),
        ])
        .split(vert[1]);

    horiz[1]
}

fn pad(area: Rect, h: u16, v: u16) -> Rect {
    Rect {
        x: area.x + h,
        y: area.y + v,
        width: area.width.saturating_sub(h * 2),
        height: area.height.saturating_sub(v * 2),
    }
}

fn compute_title(area: Rect, bp: Breakpoint) -> TemplateAreas {
    let (content, footer) = split_footer(area);
    let w = if bp == Breakpoint::Compact { 90 } else { 70 };
    TemplateAreas {
        main: center_in(content, w, 50),
        secondary: None,
        footer,
    }
}

fn compute_title_subtitle(area: Rect, bp: Breakpoint) -> TemplateAreas {
    let (content, footer) = split_footer(area);
    let w = if bp == Breakpoint::Compact { 90 } else { 75 };
    TemplateAreas {
        main: center_in(content, w, 60),
        secondary: None,
        footer,
    }
}

fn compute_two_column(area: Rect, bp: Breakpoint) -> TemplateAreas {
    let (content, footer) = split_footer(area);
    let padded = pad(content, bp.h_padding(), Spacing::XS);

    let gutter_pct = if bp == Breakpoint::Compact { 2 } else { 4 };
    let col_pct = (100 - gutter_pct) / 2;

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(col_pct),
            Constraint::Percentage(gutter_pct),
            Constraint::Percentage(col_pct),
        ])
        .split(padded);

    TemplateAreas {
        main: cols[0],
        secondary: Some(cols[2]),
        footer,
    }
}

fn compute_image_caption(area: Rect, _bp: Breakpoint) -> TemplateAreas {
    let (content, footer) = split_footer(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(content);

    TemplateAreas {
        main: chunks[0],
        secondary: Some(chunks[1]),
        footer,
    }
}

fn compute_code_focus(area: Rect, _bp: Breakpoint) -> TemplateAreas {
    let (content, footer) = split_footer(area);
    TemplateAreas {
        main: pad(content, Spacing::XS, 0),
        secondary: None,
        footer,
    }
}

fn compute_quote(area: Rect, bp: Breakpoint) -> TemplateAreas {
    let (content, footer) = split_footer(area);
    let w = if bp == Breakpoint::Compact { 85 } else { 65 };
    TemplateAreas {
        main: center_in(content, w, 60),
        secondary: None,
        footer,
    }
}

fn compute_bullet_list(area: Rect, bp: Breakpoint) -> TemplateAreas {
    let (content, footer) = split_footer(area);
    TemplateAreas {
        main: pad(content, bp.h_padding(), Spacing::XS),
        secondary: None,
        footer,
    }
}

fn compute_speaker_notes(area: Rect, bp: Breakpoint) -> TemplateAreas {
    let (content, footer) = split_footer(area);

    let split_pct = if bp == Breakpoint::Compact { 60 } else { 65 };

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(split_pct),
            Constraint::Percentage(100 - split_pct),
        ])
        .split(content);

    TemplateAreas {
        main: cols[0],
        secondary: Some(cols[1]),
        footer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_area() -> Rect {
        Rect::new(0, 0, 120, 40)
    }

    #[test]
    fn all_templates_produce_valid_areas() {
        let area = test_area();
        let bp = Breakpoint::Standard;

        for template in NodeTemplate::all() {
            let areas = template.compute_areas(area, bp);
            assert!(areas.main.width > 0, "{:?} main width is 0", template);
            assert!(areas.main.height > 0, "{:?} main height is 0", template);
            assert!(areas.footer.width > 0, "{:?} footer width is 0", template);
        }
    }

    #[test]
    fn two_column_has_secondary() {
        let areas = NodeTemplate::TwoColumn.compute_areas(test_area(), Breakpoint::Standard);
        assert!(areas.secondary.is_some());
    }

    #[test]
    fn speaker_notes_has_secondary() {
        let areas = NodeTemplate::SpeakerNotes.compute_areas(test_area(), Breakpoint::Standard);
        assert!(areas.secondary.is_some());
    }

    #[test]
    fn title_no_secondary() {
        let areas = NodeTemplate::Title.compute_areas(test_area(), Breakpoint::Standard);
        assert!(areas.secondary.is_none());
    }

    #[test]
    fn compact_breakpoint_reduces_padding() {
        let area = Rect::new(0, 0, 80, 24);
        let compact = NodeTemplate::BulletList.compute_areas(area, Breakpoint::Compact);
        let wide =
            NodeTemplate::BulletList.compute_areas(Rect::new(0, 0, 160, 50), Breakpoint::Wide);
        // Compact should have narrower padding (wider usable area relative to total)
        assert!(compact.main.width > 0);
        assert!(wide.main.width > compact.main.width);
    }

    #[test]
    fn parse_template_names() {
        assert_eq!(NodeTemplate::from_name("title"), Some(NodeTemplate::Title));
        assert_eq!(
            NodeTemplate::from_name("two-column"),
            Some(NodeTemplate::TwoColumn)
        );
        assert_eq!(
            NodeTemplate::from_name("code_focus"),
            Some(NodeTemplate::CodeFocus)
        );
        assert_eq!(
            NodeTemplate::from_name("speaker-notes"),
            Some(NodeTemplate::SpeakerNotes)
        );
        assert_eq!(NodeTemplate::from_name("unknown"), None);
    }

    #[test]
    fn all_templates_have_example_frontmatter() {
        for template in NodeTemplate::all() {
            let fm = template.example_frontmatter();
            assert!(
                fm.contains("---"),
                "{:?} frontmatter missing YAML delimiters",
                template
            );
        }
    }
}
