use crossterm::event::KeyEvent;

use crate::ui::textarea::TextArea;

use super::*;

impl App {
    pub(super) fn start_selected_block_edit(&mut self) {
        let Some((block_index, block)) = self.selected_block_with_index() else {
            self.editor_status = Some("No content blocks to edit".to_string());
            return;
        };

        let (seed, label, multiline) = match block {
            ContentBlock::Heading { text, .. } => (text.clone(), "Heading text", false),
            ContentBlock::Text { body } => (body.clone(), "Text body", true),
            ContentBlock::Code { source, .. } => (source.clone(), "Code source", true),
            // All list items — one per line; split back on commit.
            ContentBlock::List { items, .. } => (
                items
                    .iter()
                    .map(|i| i.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
                "List items  (one per line)",
                true,
            ),
            ContentBlock::Image { src, .. } => (src.clone(), "Image src", false),
            ContentBlock::Divider => {
                self.editor_status = Some("Divider block has no editable text field".to_string());
                return;
            }
            ContentBlock::Container { layout, .. } => (
                layout.clone().unwrap_or_default(),
                "Container layout",
                false,
            ),
            ContentBlock::Extension { extension_type, .. } => {
                (extension_type.clone(), "Extension type", false)
            }
        };

        self.start_inline_edit(
            EditorInlineTarget::BlockField { block_index },
            seed,
            multiline,
            label,
        );
        self.editor_status = Some(format!("Editing block #{}", block_index + 1));
    }

    pub(super) fn start_selected_block_metadata_edit(&mut self) {
        let Some((block_index, block)) = self.selected_block_with_index() else {
            self.editor_status = Some("No content blocks to edit".to_string());
            return;
        };

        let (seed, label) = match block {
            ContentBlock::Heading { level, .. } => (
                level.to_string(),
                "Heading level — enter 1 (largest) to 6 (smallest)",
            ),
            ContentBlock::Text { .. } => {
                self.editor_status = Some(
                    "Text blocks have no secondary field — use [i] to edit content".to_string(),
                );
                return;
            }
            ContentBlock::Code { language, .. } => (
                language.clone().unwrap_or_default(),
                "Code language — e.g. rust, python, js, ts, go, bash",
            ),
            ContentBlock::List { ordered, .. } => {
                let mode = if *ordered { "ordered" } else { "unordered" };
                (mode.to_string(), "List mode — type ordered or unordered")
            }
            ContentBlock::Image { alt, .. } => (alt.clone(), "Image alt text (accessibility)"),
            ContentBlock::Divider => {
                self.editor_status = Some(
                    "Divider blocks have no secondary field — press [x] to delete".to_string(),
                );
                return;
            }
            ContentBlock::Container { layout, .. } => (
                layout.clone().unwrap_or_default(),
                "Container layout — e.g. horizontal, vertical",
            ),
            ContentBlock::Extension { extension_type, .. } => {
                (extension_type.clone(), "Extension type identifier")
            }
        };

        self.start_inline_edit(
            EditorInlineTarget::BlockMetadataField { block_index },
            seed,
            false,
            label,
        );
        self.editor_status = Some(format!("{label}  (block #{})", block_index + 1));
    }

    pub(super) fn start_inline_edit(
        &mut self,
        target: EditorInlineTarget,
        seed: String,
        multiline: bool,
        label: &str,
    ) {
        let idx = self.editor_selected_node;
        if self.session.graph.nodes.get(idx).is_none() {
            return;
        }

        let textarea = TextArea::new(&seed, multiline, label);

        self.editor_textarea = Some(textarea);
        self.editor_textarea_multiline = multiline;
        self.editor_textarea_label = label.to_string();
        self.editor_inline_target = Some(target);
        self.editor_focus = EditorPaneFocus::NodeDetail;
    }

    /// Handle a key event while a textarea is active.
    ///
    /// Returns `true` when the key was consumed (preventing further dispatch).
    pub(super) fn handle_inline_edit_key(&mut self, key: KeyEvent) -> bool {
        if self.editor_textarea.is_none() {
            return false;
        }

        match key.code {
            KeyCode::Esc => {
                self.commit_inline_edit();
                true
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor_textarea = None;
                self.editor_inline_target = None;
                self.editor_status = Some("Edit cancelled".to_string());
                true
            }
            // Enter commits for single-line fields; inserts a newline for multiline.
            KeyCode::Enter if !self.editor_textarea_multiline => {
                self.commit_inline_edit();
                true
            }
            _ => {
                if let Some(ta) = self.editor_textarea.as_mut() {
                    ta.input(key);
                }
                true
            }
        }
    }

    fn commit_inline_edit(&mut self) {
        let Some(textarea) = self.editor_textarea.take() else {
            return;
        };
        let target = match self.editor_inline_target.take() {
            Some(target) => target,
            None => return,
        };

        // Join lines back, preserving newlines for multiline blocks.
        let text = textarea.text();
        let idx = self.editor_selected_node;
        match target {
            EditorInlineTarget::BlockField { block_index } => {
                let node_id = match self.session.ensure_node_id(idx) {
                    Ok(id) => id,
                    Err(_) => return,
                };

                let Some(existing) = self
                    .session
                    .graph
                    .nodes
                    .get(idx)
                    .and_then(|node| node.content.get(block_index))
                    .cloned()
                else {
                    self.editor_status = Some("Edit failed: block not found".to_string());
                    return;
                };

                let updated = update_block_from_inline_text(existing, text);

                let command = Command::UpdateBlock {
                    node_id,
                    block_index,
                    block: updated,
                };
                if self.session.execute_command(command).is_ok() {
                    self.editor_status = Some(format!("Updated block #{}", block_index + 1));
                }
            }
            EditorInlineTarget::BlockMetadataField { block_index } => {
                let node_id = match self.session.ensure_node_id(idx) {
                    Ok(id) => id,
                    Err(_) => return,
                };

                let Some(existing) = self
                    .session
                    .graph
                    .nodes
                    .get(idx)
                    .and_then(|node| node.content.get(block_index))
                    .cloned()
                else {
                    self.editor_status = Some("Edit failed: block not found".to_string());
                    return;
                };

                // Metadata fields are always single-line; take only first line.
                let first_line = text.lines().next().unwrap_or("").to_string();
                let updated = match update_block_metadata_from_inline_text(existing, first_line) {
                    Ok(block) => block,
                    Err(err) => {
                        self.editor_status = Some(err);
                        return;
                    }
                };

                let command = Command::UpdateBlock {
                    node_id,
                    block_index,
                    block: updated,
                };
                if self.session.execute_command(command).is_ok() {
                    self.editor_status =
                        Some(format!("Updated block #{} metadata", block_index + 1));
                }
            }
            EditorInlineTarget::SpeakerNotes => {
                if let Some(node) = self.session.graph.nodes.get_mut(idx) {
                    let trimmed = text.trim();
                    node.speaker_notes = if trimmed.is_empty() { None } else { Some(text) };
                    self.session.mark_dirty();
                    self.editor_status = Some("Speaker notes updated".to_string());
                }
            }
        }
    }

    pub(super) fn save_editor_graph(&mut self) {
        if self.save_graph_to_target() {
            self.pending_exit_action = None;
        }
    }

    pub(super) fn save_graph_to_target(&mut self) -> bool {
        let Some(path) = self.editor_target_path.as_ref() else {
            self.editor_status = Some("No save target configured".to_string());
            self.set_flash("No save target configured", FlashKind::Error);
            return false;
        };

        match save_graph(path, &self.session.graph) {
            Ok(()) => {
                self.session.mark_clean();
                self.editor_status = Some(format!("Saved {}", path.display()));
                self.set_flash(format!("Saved {}", path.display()), FlashKind::Success);
                true
            }
            Err(err) => {
                self.editor_status = Some(format!("Save failed: {err}"));
                self.set_flash(format!("Save failed: {err}"), FlashKind::Error);
                false
            }
        }
    }
}
