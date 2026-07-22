//! Per-block-kind edit-form state (spec 013, US1), plus [`EditableField`] —
//! promoted out of `app.rs` (`research.md` §2) so the presenter's
//! quick-edit modal and these forms share one text-buffer/cursor
//! primitive. Nothing here draws anything (`render::editor::forms` owns
//! that) or touches `EditorApp` (`editor::mod` owns opening/committing) —
//! this module is pure state and pure construction, matching
//! `engine::authoring`'s own "construction, not detection" discipline one
//! layer up: a [`FormState`] can only ever hold a shape [`FormState::build_content`]
//! can turn back into a valid [`ContentBlock`] of the same kind.

use fireside_core::{ContainerLayout, ContentBlock};
use fireside_engine::authoring::BlockPath;

// ─── EditableField (promoted from `app.rs`) ────────────────────────────────

/// Which editable text the field represents — carried only for the
/// presenter's quick-edit modal label (`render::overlays::draw_edit`); the
/// editor's own per-kind forms label fields their own way and treat this as
/// inert metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditableKind {
    /// A heading block at the given level (1-6).
    Heading(u8),
    /// A prose text block, a single-line field, or any other free text.
    Text,
    /// A list block's items, one per buffer row.
    List { ordered: bool },
}

/// One editable text buffer plus its in-progress cursor — the shared
/// primitive both the presenter's quick-edit modal and the authoring
/// editor's block forms edit through. Discarded entirely when its owner
/// (the quick-edit modal, or an open [`FormState`]) closes without saving.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditableField {
    pub(crate) path: BlockPath,
    pub(crate) kind: EditableKind,
    /// Multi-line buffer, initialized from the block's current content.
    pub(crate) buffer: Vec<String>,
    /// `buffer`'s value when the field opened, kept to detect unsaved
    /// changes without re-reading the source block.
    initial: Vec<String>,
    /// (row, column) into `buffer`, in characters (not bytes).
    pub(crate) cursor: (usize, usize),
}

impl EditableField {
    pub(crate) fn new(path: BlockPath, kind: EditableKind, buffer: Vec<String>) -> Self {
        let buffer = if buffer.is_empty() {
            vec![String::new()]
        } else {
            buffer
        };
        Self {
            path,
            kind,
            initial: buffer.clone(),
            buffer,
            cursor: (0, 0),
        }
    }

    /// A field seeded from one block of text, split on `\n` (heading/text
    /// bodies, and any other single free-text field).
    pub(crate) fn from_text(path: BlockPath, kind: EditableKind, text: &str) -> Self {
        Self::new(path, kind, to_buffer(text))
    }

    /// A one-row field (image path/description, code language, art
    /// description): `Enter` is never routed to [`EditableField::newline`]
    /// for these — the owning form's key handler treats `Enter` as
    /// "move on" instead, so the buffer never grows past one row in
    /// practice, but nothing here enforces that; it is a caller discipline.
    pub(crate) fn single_line(path: BlockPath, text: &str) -> Self {
        Self::new(path, EditableKind::Text, vec![text.to_owned()])
    }

    pub(crate) fn char_len(&self, row: usize) -> usize {
        self.buffer[row].chars().count()
    }

    fn byte_offset(&self, row: usize, col: usize) -> usize {
        self.buffer[row]
            .char_indices()
            .nth(col)
            .map_or(self.buffer[row].len(), |(b, _)| b)
    }

    pub(crate) fn insert_char(&mut self, c: char) {
        let (row, col) = self.cursor;
        let idx = self.byte_offset(row, col);
        self.buffer[row].insert(idx, c);
        self.cursor.1 += 1;
    }

    pub(crate) fn newline(&mut self) {
        let (row, col) = self.cursor;
        let idx = self.byte_offset(row, col);
        let rest = self.buffer[row].split_off(idx);
        self.buffer.insert(row + 1, rest);
        self.cursor = (row + 1, 0);
    }

    pub(crate) fn backspace(&mut self) {
        let (row, col) = self.cursor;
        if col > 0 {
            let start = self.byte_offset(row, col - 1);
            let end = self.byte_offset(row, col);
            self.buffer[row].replace_range(start..end, "");
            self.cursor.1 -= 1;
        } else if row > 0 {
            let line = self.buffer.remove(row);
            let prev_len = self.char_len(row - 1);
            self.buffer[row - 1].push_str(&line);
            self.cursor = (row - 1, prev_len);
        }
    }

    pub(crate) fn delete(&mut self) {
        let (row, col) = self.cursor;
        if col < self.char_len(row) {
            let start = self.byte_offset(row, col);
            let end = self.byte_offset(row, col + 1);
            self.buffer[row].replace_range(start..end, "");
        } else if row + 1 < self.buffer.len() {
            let next = self.buffer.remove(row + 1);
            self.buffer[row].push_str(&next);
        }
    }

    pub(crate) fn move_left(&mut self) {
        let (row, col) = self.cursor;
        if col > 0 {
            self.cursor.1 -= 1;
        } else if row > 0 {
            self.cursor = (row - 1, self.char_len(row - 1));
        }
    }

    pub(crate) fn move_right(&mut self) {
        let (row, col) = self.cursor;
        if col < self.char_len(row) {
            self.cursor.1 += 1;
        } else if row + 1 < self.buffer.len() {
            self.cursor = (row + 1, 0);
        }
    }

    /// Moves the cursor up a line; `false` at the first line means the
    /// caller should move focus to the previous field instead.
    pub(crate) fn move_up(&mut self) -> bool {
        let (row, col) = self.cursor;
        if row == 0 {
            return false;
        }
        self.cursor = (row - 1, col.min(self.char_len(row - 1)));
        true
    }

    /// Moves the cursor down a line; `false` at the last line means the
    /// caller should move focus to the next field instead.
    pub(crate) fn move_down(&mut self) -> bool {
        let (row, col) = self.cursor;
        if row + 1 >= self.buffer.len() {
            return false;
        }
        self.cursor = (row + 1, col.min(self.char_len(row + 1)));
        true
    }

    /// The buffer joined back into the single-string form the protocol
    /// stores (`Heading::text` / `Text::body`, or any other free-text
    /// field).
    pub(crate) fn text(&self) -> String {
        self.buffer.join("\n")
    }

    /// Whether the field has changed since it opened.
    pub(crate) fn dirty(&self) -> bool {
        self.buffer != self.initial
    }
}

/// Splits `text` into an [`EditableField`] buffer: one row per line, or a
/// single empty row for empty text (a buffer is never empty — cursor
/// math assumes at least one row).
pub(crate) fn to_buffer(text: &str) -> Vec<String> {
    if text.is_empty() {
        vec![String::new()]
    } else {
        text.split('\n').map(str::to_owned).collect()
    }
}

// ─── Block edit forms (spec 013, US1) ──────────────────────────────────────

/// The maximum column width accepted for a text-art block's `art` field,
/// mirroring `fireside-cli::art::DEFAULT_ART_WIDTH` — the same
/// threshold `ascii-art-too-wide` validates against. `fireside-tui` cannot
/// depend on `fireside-cli` (Constitution III), so this is the shared
/// number, not a shared symbol.
pub(crate) const MAX_ART_WIDTH: usize = 76;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CodeFocus {
    Language,
    Source,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PictureFocus {
    Src,
    Alt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextArtFocus {
    Art,
    Alt,
}

/// A plain-language, one-line summary of a container's child — shown
/// read-only inside the container form (spec 013 T033's "breadcrumb
/// navigation into children"; drilling in to *edit* a child is left to the
/// canvas's own block selection once US2 extends hit-testing past
/// top-level blocks, per `hit.rs`'s note on `canvas_hit`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChildSummary {
    pub(crate) label: String,
}

/// The form open for one block, keyed by the node and block it edits. Every
/// variant maps to exactly one of the eight authoring-facing block kinds
/// (`Divider` has no fields, so it has no form — selecting one offers
/// no `[ Edit ]` action at all).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FormState {
    Heading {
        node: String,
        path: BlockPath,
        field: EditableField,
    },
    Text {
        node: String,
        path: BlockPath,
        field: EditableField,
    },
    Code {
        node: String,
        path: BlockPath,
        language: EditableField,
        source: EditableField,
        focus: CodeFocus,
    },
    List {
        node: String,
        path: BlockPath,
        field: EditableField,
    },
    Picture {
        node: String,
        path: BlockPath,
        src: EditableField,
        alt: EditableField,
        focus: PictureFocus,
    },
    TextArt {
        node: String,
        path: BlockPath,
        art: EditableField,
        alt: EditableField,
        focus: TextArtFocus,
    },
    Container {
        node: String,
        path: BlockPath,
        layout: ContainerLayout,
        children: Vec<ChildSummary>,
    },
}

impl FormState {
    pub(crate) fn node(&self) -> &str {
        match self {
            Self::Heading { node, .. }
            | Self::Text { node, .. }
            | Self::Code { node, .. }
            | Self::List { node, .. }
            | Self::Picture { node, .. }
            | Self::TextArt { node, .. }
            | Self::Container { node, .. } => node,
        }
    }

    pub(crate) fn path(&self) -> &BlockPath {
        match self {
            Self::Heading { path, .. }
            | Self::Text { path, .. }
            | Self::Code { path, .. }
            | Self::List { path, .. }
            | Self::Picture { path, .. }
            | Self::TextArt { path, .. }
            | Self::Container { path, .. } => path,
        }
    }

    /// The block containing this one, if any — any block whose path
    /// has more than one index lives inside a `Container`, so the parent's
    /// path is simply this one's own path with its last index dropped.
    pub(crate) fn parent_container_path(&self) -> Option<BlockPath> {
        let path = self.path();
        (path.len() > 1).then(|| path[..path.len() - 1].to_vec())
    }

    /// A pasted/generated text-art body wider than [`MAX_ART_WIDTH`]
    /// columns on any line — checked before `[ Done ]` can commit
    /// (spec 013 T032).
    pub(crate) fn art_too_wide(&self) -> bool {
        match self {
            Self::TextArt { art, .. } => art
                .buffer
                .iter()
                .any(|line| line.chars().count() > MAX_ART_WIDTH),
            _ => false,
        }
    }

    /// Whether `[ Done ]` can commit right now — `false` only for an
    /// oversized text-art body, which must be shortened or regenerated
    /// first.
    pub(crate) fn can_commit(&self) -> bool {
        !self.art_too_wide()
    }

    /// The [`ContentBlock`] this form's current field values build, ready
    /// for [`fireside_engine::authoring::Op::EditBlock`]. `None` for
    /// [`FormState::Container`], whose layout chip commits immediately
    /// (spec 013 T033) rather than staging a change for `[ Done ]`.
    pub(crate) fn build_content(&self) -> Option<ContentBlock> {
        match self {
            Self::Heading { field, .. } => {
                let EditableKind::Heading(level) = field.kind else {
                    unreachable!("heading forms always carry EditableKind::Heading")
                };
                Some(ContentBlock::Heading {
                    reveal: None,
                    level,
                    text: field.text(),
                })
            }
            Self::Text { field, .. } => Some(ContentBlock::Text {
                reveal: None,
                body: field.text(),
            }),
            Self::Code {
                language, source, ..
            } => {
                let lang = language.text();
                Some(ContentBlock::Code {
                    reveal: None,
                    language: (!lang.trim().is_empty()).then_some(lang),
                    source: source.text(),
                    highlight_lines: None,
                    show_line_numbers: None,
                })
            }
            Self::List { field, .. } => {
                let EditableKind::List { ordered } = field.kind else {
                    unreachable!("list forms always carry EditableKind::List")
                };
                let items: Vec<String> = field
                    .buffer
                    .iter()
                    .filter(|line| !line.trim().is_empty())
                    .cloned()
                    .collect();
                Some(ContentBlock::List {
                    reveal: None,
                    ordered: Some(ordered),
                    items,
                })
            }
            Self::Picture { src, alt, .. } => {
                let alt_text = alt.text();
                Some(ContentBlock::Image {
                    reveal: None,
                    src: src.text(),
                    alt: (!alt_text.trim().is_empty()).then_some(alt_text),
                    caption: None,
                    width: None,
                    height: None,
                })
            }
            Self::TextArt { art, alt, .. } => {
                let alt_text = alt.text();
                Some(ContentBlock::AsciiArt {
                    reveal: None,
                    art: art.text(),
                    alt: (!alt_text.trim().is_empty()).then_some(alt_text),
                })
            }
            Self::Container { .. } => None,
        }
    }
}

/// The block at `path` within `blocks`, recursing into container children
/// — the read-side counterpart the editor uses to look up a selected
/// block before opening its form.
pub(crate) fn block_at<'a>(blocks: &'a [ContentBlock], path: &[usize]) -> Option<&'a ContentBlock> {
    let (&first, rest) = path.split_first()?;
    let block = blocks.get(first)?;
    if rest.is_empty() {
        Some(block)
    } else if let ContentBlock::Container { children, .. } = block {
        block_at(children, rest)
    } else {
        None
    }
}

/// One plain-language name per block kind (spec FR-006/FR-024) — never
/// the internal `ContentBlock` variant name or a raw `"kind"` string.
pub(crate) fn kind_label(block: &ContentBlock) -> &'static str {
    match block {
        ContentBlock::Heading { .. } => "heading",
        ContentBlock::Text { .. } => "text",
        ContentBlock::Code { .. } => "code",
        ContentBlock::List { .. } => "list",
        ContentBlock::Image { .. } => "picture",
        ContentBlock::Divider { .. } => "divider",
        ContentBlock::Container { .. } => "layout",
        ContentBlock::AsciiArt { .. } => "text art",
    }
}

fn child_summary(block: &ContentBlock) -> ChildSummary {
    let snippet = match block {
        ContentBlock::Heading { text, .. } => text.clone(),
        ContentBlock::Text { body, .. } => body.clone(),
        ContentBlock::Code { source, .. } => source.lines().next().unwrap_or_default().to_owned(),
        ContentBlock::List { items, .. } => items.first().cloned().unwrap_or_default(),
        ContentBlock::Image { alt, src, .. } => alt.clone().unwrap_or_else(|| src.clone()),
        ContentBlock::Divider { .. } => String::new(),
        ContentBlock::Container { children, .. } => {
            format!(
                "{} block{}",
                children.len(),
                if children.len() == 1 { "" } else { "s" }
            )
        }
        ContentBlock::AsciiArt { alt, .. } => alt.clone().unwrap_or_default(),
    };
    let label = if snippet.trim().is_empty() {
        kind_label(block).to_owned()
    } else {
        format!("{} — {}", kind_label(block), snippet.trim())
    };
    ChildSummary { label }
}

/// Opens the form for `block` at `path` on `node`, or `None` for a
/// `Divider`, which has nothing to edit (spec 013 T027-T033).
#[must_use]
pub(crate) fn open(node: &str, path: BlockPath, block: &ContentBlock) -> Option<FormState> {
    let node = node.to_owned();
    match block {
        ContentBlock::Heading { level, text, .. } => Some(FormState::Heading {
            field: EditableField::from_text(path.clone(), EditableKind::Heading(*level), text),
            node,
            path,
        }),
        ContentBlock::Text { body, .. } => Some(FormState::Text {
            field: EditableField::from_text(path.clone(), EditableKind::Text, body),
            node,
            path,
        }),
        ContentBlock::Code {
            language, source, ..
        } => Some(FormState::Code {
            language: EditableField::single_line(path.clone(), language.as_deref().unwrap_or("")),
            source: EditableField::from_text(path.clone(), EditableKind::Text, source),
            focus: CodeFocus::Source,
            node,
            path,
        }),
        ContentBlock::List { ordered, items, .. } => Some(FormState::List {
            field: EditableField::new(
                path.clone(),
                EditableKind::List {
                    ordered: ordered.unwrap_or(false),
                },
                if items.is_empty() {
                    vec![String::new()]
                } else {
                    items.clone()
                },
            ),
            node,
            path,
        }),
        ContentBlock::Image { src, alt, .. } => Some(FormState::Picture {
            src: EditableField::single_line(path.clone(), src),
            alt: EditableField::single_line(path.clone(), alt.as_deref().unwrap_or("")),
            focus: PictureFocus::Src,
            node,
            path,
        }),
        ContentBlock::AsciiArt { art, alt, .. } => Some(FormState::TextArt {
            art: EditableField::from_text(path.clone(), EditableKind::Text, art),
            alt: EditableField::single_line(path.clone(), alt.as_deref().unwrap_or("")),
            focus: TextArtFocus::Art,
            node,
            path,
        }),
        ContentBlock::Container {
            children, layout, ..
        } => Some(FormState::Container {
            layout: layout.unwrap_or_default(),
            children: children.iter().map(child_summary).collect(),
            node,
            path,
        }),
        ContentBlock::Divider { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(indices: &[usize]) -> BlockPath {
        indices.to_vec()
    }

    #[test]
    fn heading_form_round_trips_edited_text() {
        let block = ContentBlock::Heading {
            reveal: Some(2),
            level: 2,
            text: "Old title".to_owned(),
        };
        let Some(mut form) = open("a", path(&[0]), &block) else {
            panic!("heading has a form");
        };
        let FormState::Heading { field, .. } = &mut form else {
            panic!("heading form");
        };
        field.buffer[0] = "New title".to_owned();
        let content = form.build_content().expect("heading commits");
        assert_eq!(
            content,
            ContentBlock::Heading {
                reveal: None,
                level: 2,
                text: "New title".to_owned(),
            }
        );
    }

    #[test]
    fn divider_has_no_form() {
        let block = ContentBlock::Divider { reveal: None };
        assert!(open("a", path(&[0]), &block).is_none());
    }

    #[test]
    fn list_form_drops_blank_lines_on_commit() {
        let block = ContentBlock::List {
            reveal: None,
            ordered: Some(true),
            items: vec!["one".to_owned(), "two".to_owned()],
        };
        let Some(mut form) = open("a", path(&[0]), &block) else {
            panic!("list has a form");
        };
        let FormState::List { field, .. } = &mut form else {
            panic!("list form");
        };
        field.buffer = vec![
            "one".to_owned(),
            String::new(),
            "  ".to_owned(),
            "two".to_owned(),
        ];
        let ContentBlock::List { items, ordered, .. } = form.build_content().expect("list commits")
        else {
            panic!("list content");
        };
        assert_eq!(items, vec!["one".to_owned(), "two".to_owned()]);
        assert_eq!(ordered, Some(true));
    }

    #[test]
    fn code_form_treats_blank_language_as_absent() {
        let block = ContentBlock::Code {
            reveal: None,
            language: Some("rust".to_owned()),
            source: "fn main() {}".to_owned(),
            highlight_lines: None,
            show_line_numbers: None,
        };
        let Some(mut form) = open("a", path(&[0]), &block) else {
            panic!("code has a form");
        };
        let FormState::Code { language, .. } = &mut form else {
            panic!("code form");
        };
        language.buffer[0].clear();
        let ContentBlock::Code { language, .. } = form.build_content().expect("code commits")
        else {
            panic!("code content");
        };
        assert_eq!(language, None);
    }

    #[test]
    fn text_art_over_max_width_cannot_commit() {
        let block = ContentBlock::AsciiArt {
            reveal: None,
            art: "short".to_owned(),
            alt: None,
        };
        let Some(mut form) = open("a", path(&[0]), &block) else {
            panic!("text art has a form");
        };
        let FormState::TextArt { art, .. } = &mut form else {
            panic!("text art form");
        };
        art.buffer[0] = "x".repeat(MAX_ART_WIDTH + 1);
        assert!(!form.can_commit());
        let FormState::TextArt { art, .. } = &mut form else {
            panic!("text art form");
        };
        art.buffer[0] = "x".repeat(MAX_ART_WIDTH);
        assert!(form.can_commit());
    }

    #[test]
    fn container_form_reports_its_children_and_no_staged_content() {
        let block = ContentBlock::Container {
            reveal: None,
            layout: Some(ContainerLayout::Columns),
            children: vec![
                ContentBlock::Text {
                    reveal: None,
                    body: "left".to_owned(),
                },
                ContentBlock::Divider { reveal: None },
            ],
        };
        let Some(form) = open("a", path(&[0]), &block) else {
            panic!("container has a form");
        };
        let FormState::Container {
            children, layout, ..
        } = &form
        else {
            panic!("container form");
        };
        assert_eq!(children.len(), 2);
        assert_eq!(*layout, ContainerLayout::Columns);
        assert!(form.build_content().is_none());
    }

    #[test]
    fn nested_block_reports_its_container_as_parent() {
        let block = ContentBlock::Text {
            reveal: None,
            body: "nested".to_owned(),
        };
        let form = open("a", path(&[0, 1]), &block).expect("text has a form");
        assert_eq!(form.parent_container_path(), Some(path(&[0])));

        let top = open("a", path(&[0]), &block).expect("text has a form");
        assert_eq!(top.parent_container_path(), None);
    }

    #[test]
    fn block_at_recurses_into_containers() {
        let blocks = vec![ContentBlock::Container {
            reveal: None,
            layout: None,
            children: vec![ContentBlock::Text {
                reveal: None,
                body: "inner".to_owned(),
            }],
        }];
        let found = block_at(&blocks, &[0, 0]).expect("nested block resolves");
        assert!(matches!(found, ContentBlock::Text { body, .. } if body == "inner"));
        assert!(block_at(&blocks, &[0, 5]).is_none());
        assert!(block_at(&blocks, &[5]).is_none());
    }
}
