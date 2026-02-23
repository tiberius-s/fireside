use super::*;

impl App {
    pub(super) fn cycle_layout(&mut self, forward: bool) {
        let idx = self.editor_selected_node;
        if let Some(node) = self.session.graph.nodes.get_mut(idx) {
            let variants = [
                Layout::Default,
                Layout::Center,
                Layout::Top,
                Layout::SplitHorizontal,
                Layout::SplitVertical,
                Layout::Title,
                Layout::CodeFocus,
                Layout::Fullscreen,
                Layout::AlignLeft,
                Layout::AlignRight,
                Layout::Blank,
            ];

            let current = node.layout.unwrap_or(Layout::Default);
            let pos = variants.iter().position(|v| *v == current).unwrap_or(0);
            let next = if forward {
                (pos + 1) % variants.len()
            } else {
                (pos + variants.len() - 1) % variants.len()
            };

            node.layout = Some(variants[next]);
            self.session.mark_dirty();
            self.editor_status = Some(format!("Layout set to {:?}", variants[next]));
        }
    }

    pub(super) fn cycle_transition(&mut self, forward: bool) {
        let idx = self.editor_selected_node;
        if let Some(node) = self.session.graph.nodes.get_mut(idx) {
            let variants = [
                Transition::None,
                Transition::Fade,
                Transition::SlideLeft,
                Transition::SlideRight,
                Transition::Wipe,
                Transition::Dissolve,
                Transition::Matrix,
                Transition::Typewriter,
            ];

            let current = node.transition.unwrap_or(Transition::None);
            let pos = variants.iter().position(|v| *v == current).unwrap_or(0);
            let next = if forward {
                (pos + 1) % variants.len()
            } else {
                (pos + variants.len() - 1) % variants.len()
            };

            node.transition = Some(variants[next]);
            self.session.mark_dirty();
            self.editor_status = Some(format!("Transition set to {:?}", variants[next]));
        }
    }

    pub(super) fn open_layout_picker(&mut self) {
        let selected = self
            .editor_last_layout_picker
            .min(layout_variants().len().saturating_sub(1));

        self.editor_picker = Some(EditorPickerOverlay::Layout { selected });
        self.editor_status = Some("Layout picker: arrows or 1-9/0 + Enter".to_string());
    }

    pub(super) fn open_transition_picker(&mut self) {
        let selected = self
            .editor_last_transition_picker
            .min(transition_variants().len().saturating_sub(1));

        self.editor_picker = Some(EditorPickerOverlay::Transition { selected });
        self.editor_status = Some("Transition picker: arrows or 1-9/0 + Enter".to_string());
    }

    pub(super) fn open_block_type_picker(&mut self) {
        let selected = self
            .editor_last_block_picker
            .min(block_type_variants().len().saturating_sub(1));

        self.editor_picker = Some(EditorPickerOverlay::BlockType { selected });
        self.editor_status = Some("Block picker: arrows or 1-9/0 + Enter".to_string());
    }

    pub(super) fn handle_picker_key(&mut self, code: KeyCode) -> bool {
        let Some(overlay) = self.editor_picker else {
            return false;
        };

        let max_index = match overlay {
            EditorPickerOverlay::Layout { .. } => layout_variants().len().saturating_sub(1),
            EditorPickerOverlay::Transition { .. } => transition_variants().len().saturating_sub(1),
            EditorPickerOverlay::BlockType { .. } => block_type_variants().len().saturating_sub(1),
        };

        match code {
            KeyCode::Esc => {
                self.editor_picker = None;
                self.editor_status = Some("Picker cancelled".to_string());
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.adjust_picker_selection(max_index, false);
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.adjust_picker_selection(max_index, true);
                true
            }
            KeyCode::Char('1'..='9') | KeyCode::Char('0') => {
                if let Some(index) = digit_to_index(code)
                    && index <= max_index
                {
                    self.set_picker_selection(index);
                }
                true
            }
            KeyCode::Enter => {
                self.apply_picker_selection();
                true
            }
            _ => true,
        }
    }

    pub(super) fn adjust_picker_selection(&mut self, max_index: usize, forward: bool) {
        self.editor_picker = self.editor_picker.map(|overlay| match overlay {
            EditorPickerOverlay::Layout { selected } => EditorPickerOverlay::Layout {
                selected: bump_index(selected, max_index, forward),
            },
            EditorPickerOverlay::Transition { selected } => EditorPickerOverlay::Transition {
                selected: bump_index(selected, max_index, forward),
            },
            EditorPickerOverlay::BlockType { selected } => EditorPickerOverlay::BlockType {
                selected: bump_index(selected, max_index, forward),
            },
        });
    }

    pub(super) fn set_picker_selection(&mut self, selected: usize) {
        self.editor_picker = self.editor_picker.map(|overlay| match overlay {
            EditorPickerOverlay::Layout { .. } => {
                self.editor_last_layout_picker = selected;
                EditorPickerOverlay::Layout { selected }
            }
            EditorPickerOverlay::Transition { .. } => {
                self.editor_last_transition_picker = selected;
                EditorPickerOverlay::Transition { selected }
            }
            EditorPickerOverlay::BlockType { .. } => {
                self.editor_last_block_picker = selected;
                EditorPickerOverlay::BlockType { selected }
            }
        });
        self.persist_editor_preferences();
    }

    pub(super) fn apply_picker_selection(&mut self) {
        let Some(overlay) = self.editor_picker.take() else {
            return;
        };

        let idx = self.editor_selected_node;
        match overlay {
            EditorPickerOverlay::Layout { selected } => {
                if let Some(node) = self.session.graph.nodes.get_mut(idx)
                    && let Some(layout) = layout_variants().get(selected).copied()
                {
                    self.editor_last_layout_picker = selected;
                    node.layout = Some(layout);
                    self.session.mark_dirty();
                    self.editor_status = Some(format!("Layout set to {:?}", layout));
                    self.persist_editor_preferences();
                }
            }
            EditorPickerOverlay::Transition { selected } => {
                if let Some(node) = self.session.graph.nodes.get_mut(idx)
                    && let Some(transition) = transition_variants().get(selected).copied()
                {
                    self.editor_last_transition_picker = selected;
                    node.transition = Some(transition);
                    self.session.mark_dirty();
                    self.editor_status = Some(format!("Transition set to {:?}", transition));
                    self.persist_editor_preferences();
                }
            }
            EditorPickerOverlay::BlockType { selected } => {
                self.editor_last_block_picker = selected;
                self.append_block_type(selected);
                self.persist_editor_preferences();
            }
        }
    }

    fn append_block_type(&mut self, selected: usize) {
        let idx = self.editor_selected_node;
        let node_id = match self.session.ensure_node_id(idx) {
            Ok(id) => id,
            Err(_) => return,
        };

        let Some((name, template)) = block_type_variants().get(selected) else {
            return;
        };

        let mut updated_content = self.session.graph.nodes[idx].content.clone();
        updated_content.push(template.clone());

        let command = Command::UpdateNodeContent {
            node_id,
            content: updated_content,
        };

        if self.session.execute_command(command).is_ok() {
            self.editor_selected_block = self.selected_node_block_count().saturating_sub(1);
            self.editor_status = Some(format!("Appended {name} block"));
            self.sync_editor_selection_bounds();
        }
    }
}
