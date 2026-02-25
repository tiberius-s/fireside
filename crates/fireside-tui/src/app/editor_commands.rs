use super::*;

impl App {
    pub(super) fn start_selected_block_edit(&mut self) {
        let Some((block_index, block)) = self.selected_block_with_index() else {
            self.editor_status = Some("No content blocks to edit".to_string());
            return;
        };

        let (seed, label) = match block {
            ContentBlock::Heading { text, .. } => (text.clone(), "Heading"),
            ContentBlock::Text { body } => (body.clone(), "Text"),
            ContentBlock::Code { source, .. } => (source.clone(), "Code"),
            ContentBlock::List { items, .. } => (
                items
                    .first()
                    .map(|item| item.text.clone())
                    .unwrap_or_default(),
                "List first item",
            ),
            ContentBlock::Image { src, .. } => (src.clone(), "Image src"),
            ContentBlock::Divider => {
                self.editor_status = Some("Divider block has no editable text field".to_string());
                return;
            }
            ContentBlock::Container { layout, .. } => {
                (layout.clone().unwrap_or_default(), "Container layout")
            }
            ContentBlock::Extension { extension_type, .. } => {
                (extension_type.clone(), "Extension type")
            }
        };

        self.start_inline_edit(EditorInlineTarget::BlockField { block_index }, seed);
        self.editor_status = Some(format!("Editing {label} (block #{})", block_index + 1));
    }

    pub(super) fn start_selected_block_metadata_edit(&mut self) {
        let Some((block_index, block)) = self.selected_block_with_index() else {
            self.editor_status = Some("No content blocks to edit".to_string());
            return;
        };

        let (seed, label) = match block {
            ContentBlock::Heading { level, .. } => (level.to_string(), "Heading level (1-6)"),
            ContentBlock::Text { .. } => {
                self.editor_status = Some("Text block has no secondary metadata field".to_string());
                return;
            }
            ContentBlock::Code { language, .. } => {
                (language.clone().unwrap_or_default(), "Code language")
            }
            ContentBlock::List { ordered, .. } => {
                let mode = if *ordered { "ordered" } else { "unordered" };
                (mode.to_string(), "List mode (ordered/unordered)")
            }
            ContentBlock::Image { alt, .. } => (alt.clone(), "Image alt text"),
            ContentBlock::Divider => {
                self.editor_status = Some("Divider has no secondary metadata field".to_string());
                return;
            }
            ContentBlock::Container { layout, .. } => {
                (layout.clone().unwrap_or_default(), "Container layout")
            }
            ContentBlock::Extension { extension_type, .. } => {
                (extension_type.clone(), "Extension type")
            }
        };

        self.start_inline_edit(EditorInlineTarget::BlockMetadataField { block_index }, seed);
        self.editor_status = Some(format!("Editing {label} (block #{})", block_index + 1));
    }

    pub(super) fn start_inline_edit(&mut self, target: EditorInlineTarget, seed: String) {
        let idx = self.editor_selected_node;
        if self.session.graph.nodes.get(idx).is_none() {
            return;
        }

        self.editor_text_input = Some(seed);
        self.editor_inline_target = Some(target);
        self.editor_focus = EditorPaneFocus::NodeDetail;
    }

    pub(super) fn handle_inline_edit_key(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> bool {
        let Some(buffer) = self.editor_text_input.as_mut() else {
            return false;
        };

        match code {
            KeyCode::Esc => {
                self.commit_inline_edit();
                true
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor_text_input = None;
                self.editor_inline_target = None;
                self.editor_status = Some("Inline edit cancelled".to_string());
                true
            }
            KeyCode::Enter => {
                self.commit_inline_edit();
                true
            }
            KeyCode::Backspace => {
                buffer.pop();
                true
            }
            KeyCode::Char(ch)
                if !modifiers.contains(KeyModifiers::CONTROL)
                    && !modifiers.contains(KeyModifiers::ALT) =>
            {
                buffer.push(ch);
                true
            }
            _ => true,
        }
    }

    fn commit_inline_edit(&mut self) {
        let Some(text) = self.editor_text_input.take() else {
            return;
        };
        let target = match self.editor_inline_target.take() {
            Some(target) => target,
            None => return,
        };

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
                    self.editor_status = Some("Inline edit failed: block not found".to_string());
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
                    self.editor_status = Some("Inline edit failed: block not found".to_string());
                    return;
                };

                let updated = match update_block_metadata_from_inline_text(existing, text) {
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
