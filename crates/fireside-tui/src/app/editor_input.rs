use super::*;

impl App {
    pub(super) fn load_editor_preferences(&mut self) {
        let prefs = load_editor_ui_prefs();
        self.editor_focus = match prefs.last_focus.as_str() {
            "node-detail" => EditorPaneFocus::NodeDetail,
            _ => EditorPaneFocus::NodeList,
        };
        self.editor_last_layout_picker = prefs.last_layout_picker;
        self.editor_last_transition_picker = prefs.last_transition_picker;
        self.editor_list_scroll_offset = prefs.last_list_scroll_offset;
        self.sync_editor_list_viewport();
    }

    pub(super) fn persist_editor_preferences(&self) {
        let prefs = EditorUiPrefs {
            last_focus: match self.editor_focus {
                EditorPaneFocus::NodeList => "node-list".to_string(),
                EditorPaneFocus::NodeDetail => "node-detail".to_string(),
            },
            last_layout_picker: self.editor_last_layout_picker,
            last_transition_picker: self.editor_last_transition_picker,
            last_list_scroll_offset: self.editor_list_scroll_offset,
        };
        let _ = save_editor_ui_prefs(&prefs);
    }

    pub(super) fn handle_editor_mouse_click(&mut self, column: u16, row: u16) {
        let root = Rect::new(0, 0, self.terminal_size.0, self.terminal_size.1);
        let sections = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(root);
        let compact = self.terminal_size.0 <= 80;
        let (list_area, detail_area) = if compact {
            if self.editor_node_list_visible {
                let v = RatatuiLayout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
                    .split(sections[0]);
                (Some(v[0]), v[1])
            } else {
                (None, sections[0])
            }
        } else {
            let body = RatatuiLayout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(sections[0]);
            (Some(body[0]), body[1])
        };

        if self.editor_graph_overlay {
            if self.handle_graph_overlay_mouse_click(column, row, sections[0]) {
                return;
            }
            return;
        }

        if let Some(overlay) = self.editor_picker
            && self.handle_picker_mouse_click(column, row, overlay, sections[0])
        {
            return;
        }

        if let Some(list_area) = list_area
            && point_in_rect(column, row, list_area)
        {
            self.editor_focus = EditorPaneFocus::NodeList;
            self.persist_editor_preferences();

            let row_start = list_area.y.saturating_add(1);
            if row >= row_start {
                let index = self.editor_list_scroll_offset + row.saturating_sub(row_start) as usize;
                if index < self.session.graph.nodes.len() {
                    self.select_editor_node(index);
                }
            }
            return;
        }

        if point_in_rect(column, row, detail_area) {
            self.editor_focus = EditorPaneFocus::NodeDetail;
            self.persist_editor_preferences();
        }
    }

    pub(super) fn handle_mouse_drag(&mut self, column: u16, row: u16) {
        if self.mode != AppMode::Editing {
            return;
        }

        let Some(overlay) = self.editor_picker else {
            return;
        };

        let root = Rect::new(0, 0, self.terminal_size.0, self.terminal_size.1);
        let sections = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(root);

        let popup = centered_popup(sections[0], 55, 65);
        if !point_in_rect(column, row, popup) {
            return;
        }

        let inner = Rect {
            x: popup.x.saturating_add(1),
            y: popup.y.saturating_add(1),
            width: popup.width.saturating_sub(2),
            height: popup.height.saturating_sub(2),
        };
        if row < inner.y {
            return;
        }

        let option_count = match overlay {
            EditorPickerOverlay::Layout { .. } => layout_variants().len(),
            EditorPickerOverlay::Transition { .. } => transition_variants().len(),
            EditorPickerOverlay::BlockType { .. } => block_type_variants().len(),
        };
        let row_span = picker_row_span(overlay);
        let index = (row.saturating_sub(inner.y) as usize) / row_span;
        if index < option_count {
            self.set_picker_selection(index);
            self.editor_status = Some(format!("Picker preview: option {}", index + 1));
        }
    }

    pub(super) fn handle_editor_mouse_scroll(&mut self, direction: MouseScrollDirection) {
        if self.editor_text_input.is_some() {
            return;
        }

        if self.editor_search_input.is_some() {
            return;
        }

        if self.editor_index_jump_input.is_some() {
            return;
        }

        if self.editor_graph_overlay {
            match direction {
                MouseScrollDirection::Up => {
                    self.editor_graph_selected_node =
                        self.editor_graph_selected_node.saturating_sub(1);
                }
                MouseScrollDirection::Down => {
                    let max = self.session.graph.nodes.len().saturating_sub(1);
                    self.editor_graph_selected_node =
                        (self.editor_graph_selected_node + 1).min(max);
                }
            }
            self.sync_editor_graph_viewport();
            return;
        }

        if let Some(overlay) = self.editor_picker {
            let max_index = match overlay {
                EditorPickerOverlay::Layout { .. } => layout_variants().len().saturating_sub(1),
                EditorPickerOverlay::Transition { .. } => {
                    transition_variants().len().saturating_sub(1)
                }
                EditorPickerOverlay::BlockType { .. } => {
                    block_type_variants().len().saturating_sub(1)
                }
            };

            self.adjust_picker_selection(
                max_index,
                matches!(direction, MouseScrollDirection::Down),
            );
            return;
        }

        if self.editor_focus != EditorPaneFocus::NodeList {
            return;
        }

        match direction {
            MouseScrollDirection::Up => self.editor_select_prev(),
            MouseScrollDirection::Down => self.editor_select_next(),
        }
    }

    pub(super) fn handle_picker_mouse_click(
        &mut self,
        column: u16,
        row: u16,
        overlay: EditorPickerOverlay,
        content_area: Rect,
    ) -> bool {
        let popup = centered_popup(content_area, 55, 65);
        if !point_in_rect(column, row, popup) {
            self.editor_picker = None;
            self.editor_status = Some("Picker cancelled".to_string());
            return true;
        }

        let inner = Rect {
            x: popup.x.saturating_add(1),
            y: popup.y.saturating_add(1),
            width: popup.width.saturating_sub(2),
            height: popup.height.saturating_sub(2),
        };

        if row < inner.y {
            return true;
        }

        let option_count = match overlay {
            EditorPickerOverlay::Layout { .. } => layout_variants().len(),
            EditorPickerOverlay::Transition { .. } => transition_variants().len(),
            EditorPickerOverlay::BlockType { .. } => block_type_variants().len(),
        };

        let row_span = picker_row_span(overlay);
        let index = (row.saturating_sub(inner.y) as usize) / row_span;
        if index < option_count {
            self.set_picker_selection(index);
            self.apply_picker_selection();
        }
        true
    }

    pub(super) fn handle_graph_overlay_mouse_click(
        &mut self,
        column: u16,
        row: u16,
        content_area: Rect,
    ) -> bool {
        let popup = graph_overlay_rect(content_area);
        if !point_in_rect(column, row, popup) {
            self.editor_graph_overlay = false;
            self.editor_status = Some("Graph view closed".to_string());
            return true;
        }

        let list_area = graph_overlay_list_panel_rect(content_area);

        if !point_in_rect(column, row, list_area) {
            return true;
        }

        let idx = graph_overlay_row_to_node(
            content_area,
            &self.session,
            self.editor_graph_selected_node,
            self.editor_graph_scroll_offset,
            row,
        );
        let Some(idx) = idx else {
            return true;
        };

        if idx < self.session.graph.nodes.len() {
            self.editor_graph_selected_node = idx;
            self.sync_editor_graph_viewport();
            self.select_editor_node(idx);
            self.editor_graph_overlay = false;
            self.editor_status = Some(format!("Graph jump: node #{}", idx + 1));
        }

        true
    }
}
