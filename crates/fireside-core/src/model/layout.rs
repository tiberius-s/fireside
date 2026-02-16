//! Layout variants that control how node content is positioned.
//!
//! Uses kebab-case serialization per the Fireside protocol.

use serde::{Deserialize, Serialize};

/// Determines how content is arranged within the node area.
///
/// Matches the 12 protocol layout modes. Additional engine-specific
/// layouts can be added as needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    /// Content with standard padding (default).
    #[default]
    Default,
    /// Content is centered both horizontally and vertically.
    Center,
    /// Content is aligned to the top with standard padding.
    Top,
    /// Content is split into two equal columns.
    #[serde(rename = "split-horizontal")]
    SplitHorizontal,
    /// Content is split into two rows.
    #[serde(rename = "split-vertical")]
    SplitVertical,
    /// Title slide layout: large centered title, subtitle below.
    Title,
    /// Code-focused layout: maximized content area with minimal chrome.
    #[serde(rename = "code-focus")]
    CodeFocus,
    /// Fullscreen content with no chrome.
    Fullscreen,
    /// Content aligned to the left.
    #[serde(rename = "align-left")]
    AlignLeft,
    /// Content aligned to the right.
    #[serde(rename = "align-right")]
    AlignRight,
    /// Blank node with no predefined layout.
    Blank,
}
