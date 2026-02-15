//! Layout variants that control how slide content is positioned.

use serde::{Deserialize, Serialize};

/// Determines how content is arranged within the slide area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    /// Content is centered both horizontally and vertically.
    Center,
    /// Content is aligned to the top with standard padding.
    #[default]
    Top,
    /// Content is split into two equal columns.
    TwoColumn,
    /// Title slide layout: large centered title, subtitle below.
    Title,
    /// Code-focused layout: maximized content area with minimal chrome.
    CodeFocus,
    /// Blank slide with no predefined layout.
    Blank,
}
