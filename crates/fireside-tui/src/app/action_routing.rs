use super::*;

impl App {
    /// Process a single action, updating application state.
    pub fn update(&mut self, action: Action) {
        self.refresh_timed_state();
        self.needs_redraw = true;

        match action {
            Action::NextNode => {
                let from_index = self.session.current_node_index();
                self.session.traversal.next(&self.session.graph);
                let next_index = self.session.current_node_index();
                if next_index != from_index {
                    self.record_navigation(next_index, false);
                }
                self.start_transition_if_needed(from_index);
            }
            Action::PrevNode => {
                let from_index = self.session.current_node_index();
                self.session.traversal.back();
                let next_index = self.session.current_node_index();
                if next_index != from_index {
                    self.record_navigation(next_index, false);
                }
                self.start_transition_if_needed(from_index);
            }
            Action::GoToNode(idx) => {
                let from_index = self.session.current_node_index();
                let _ = self.session.traversal.goto(idx, &self.session.graph);
                let next_index = self.session.current_node_index();
                if next_index != from_index {
                    self.record_navigation(next_index, false);
                }
                self.start_transition_if_needed(from_index);
            }
            Action::ChooseBranch(key) => {
                let from_index = self.session.current_node_index();
                let _ = self.session.traversal.choose(key, &self.session.graph);
                self.branch_focused_option = 0;
                let next_index = self.session.current_node_index();
                if next_index != from_index {
                    self.record_navigation(next_index, true);
                }
                self.start_transition_if_needed(from_index);
            }
            Action::ToggleHelp => {
                self.show_help = !self.show_help;
                if self.show_help {
                    self.help_scroll_offset = 0;
                }
            }
            Action::ToggleSpeakerNotes => {
                self.show_speaker_notes = !self.show_speaker_notes;
            }
            Action::ToggleZenMode => {
                self.show_zen_mode = !self.show_zen_mode;
            }
            Action::ToggleTimeline => {
                self.show_timeline = !self.show_timeline;
            }
            Action::JumpToBranchPoint => {
                if self.mode != AppMode::Editing {
                    let current_index = self.session.current_node_index();
                    if let Some(target) =
                        self.nav_path
                            .iter()
                            .rev()
                            .map(|(index, _)| *index)
                            .find(|index| {
                                *index != current_index
                                    && self
                                        .session
                                        .graph
                                        .nodes
                                        .get(*index)
                                        .is_some_and(|node| node.branch_point().is_some())
                            })
                    {
                        let from_index = current_index;
                        let _ = self.session.traversal.goto(target, &self.session.graph);
                        let next_index = self.session.current_node_index();
                        if next_index != from_index {
                            self.record_navigation(next_index, false);
                        }
                        self.start_transition_if_needed(from_index);
                    }
                }
            }
            Action::EnterEditMode => {
                self.enter_edit_mode();
            }
            Action::ExitEditMode => {
                self.request_exit_action(PendingExitAction::ExitEditor);
            }
            Action::EditorAppendTextBlock => {
                if self.mode == AppMode::Editing {
                    self.open_block_type_picker();
                }
            }
            Action::EditorAddNode => {
                if self.mode == AppMode::Editing {
                    self.add_node_after_selected();
                }
            }
            Action::EditorRemoveNode => {
                if self.mode == AppMode::Editing {
                    self.remove_selected_node();
                }
            }
            Action::EditorSelectNextNode => {
                if self.mode == AppMode::Editing {
                    self.editor_select_next();
                }
            }
            Action::EditorSelectPrevNode => {
                if self.mode == AppMode::Editing {
                    self.editor_select_prev();
                }
            }
            Action::EditorPageDown => {
                if self.mode == AppMode::Editing {
                    self.editor_page_down();
                }
            }
            Action::EditorPageUp => {
                if self.mode == AppMode::Editing {
                    self.editor_page_up();
                }
            }
            Action::EditorJumpTop => {
                if self.mode == AppMode::Editing {
                    self.editor_jump_top();
                }
            }
            Action::EditorJumpBottom => {
                if self.mode == AppMode::Editing {
                    self.editor_jump_bottom();
                }
            }
            Action::EditorStartNodeSearch => {
                if self.mode == AppMode::Editing {
                    self.start_editor_node_search();
                }
            }
            Action::EditorSearchPrevHit => {
                if self.mode == AppMode::Editing {
                    self.jump_editor_search_hit(false);
                }
            }
            Action::EditorSearchNextHit => {
                if self.mode == AppMode::Editing {
                    self.jump_editor_search_hit(true);
                }
            }
            Action::EditorStartIndexJump => {
                if self.mode == AppMode::Editing {
                    self.start_editor_index_jump();
                }
            }
            Action::EditorToggleFocus => {
                if self.mode == AppMode::Editing {
                    self.editor_focus = match self.editor_focus {
                        EditorPaneFocus::NodeList => EditorPaneFocus::NodeDetail,
                        EditorPaneFocus::NodeDetail => EditorPaneFocus::NodeList,
                    };
                    self.persist_editor_preferences();
                }
            }
            Action::EditorStartInlineEdit => {
                if self.mode == AppMode::Editing {
                    self.start_selected_block_edit();
                }
            }
            Action::EditorStartInlineMetaEdit => {
                if self.mode == AppMode::Editing {
                    self.start_selected_block_metadata_edit();
                }
            }
            Action::EditorSelectNextBlock => {
                if self.mode == AppMode::Editing {
                    self.editor_select_next_block();
                }
            }
            Action::EditorSelectPrevBlock => {
                if self.mode == AppMode::Editing {
                    self.editor_select_prev_block();
                }
            }
            Action::EditorMoveBlockUp => {
                if self.mode == AppMode::Editing {
                    self.move_selected_block(false);
                }
            }
            Action::EditorMoveBlockDown => {
                if self.mode == AppMode::Editing {
                    self.move_selected_block(true);
                }
            }
            Action::EditorRemoveBlock => {
                if self.mode == AppMode::Editing {
                    self.remove_selected_block();
                }
            }
            // Scroll the WYSIWYG detail preview by one line.
            Action::EditorDetailScrollDown => {
                if self.mode == AppMode::Editing {
                    self.editor_detail_scroll_offset =
                        self.editor_detail_scroll_offset.saturating_add(1);
                }
            }
            Action::EditorDetailScrollUp => {
                if self.mode == AppMode::Editing {
                    self.editor_detail_scroll_offset =
                        self.editor_detail_scroll_offset.saturating_sub(1);
                }
            }
            Action::EditorStartNotesEdit => {
                if self.mode == AppMode::Editing {
                    let seed = self
                        .session
                        .graph
                        .nodes
                        .get(self.editor_selected_node)
                        .and_then(|node| node.speaker_notes.clone())
                        .unwrap_or_default();
                    self.start_inline_edit(EditorInlineTarget::SpeakerNotes, seed);
                    self.editor_status = Some("Editing speaker notes".to_string());
                }
            }
            Action::EditorCycleLayoutNext => {
                if self.mode == AppMode::Editing {
                    self.cycle_layout(true);
                }
            }
            Action::EditorCycleLayoutPrev => {
                if self.mode == AppMode::Editing {
                    self.cycle_layout(false);
                }
            }
            Action::EditorOpenLayoutPicker => {
                if self.mode == AppMode::Editing {
                    self.open_layout_picker();
                }
            }
            Action::EditorCycleTransitionNext => {
                if self.mode == AppMode::Editing {
                    self.cycle_transition(true);
                }
            }
            Action::EditorCycleTransitionPrev => {
                if self.mode == AppMode::Editing {
                    self.cycle_transition(false);
                }
            }
            Action::EditorOpenTransitionPicker => {
                if self.mode == AppMode::Editing {
                    self.open_transition_picker();
                }
            }
            Action::EditorSaveGraph => {
                if self.mode == AppMode::Editing {
                    self.save_editor_graph();
                }
            }
            Action::EditorToggleGraphView => {
                if self.mode == AppMode::Editing {
                    self.toggle_editor_graph_view();
                }
            }
            Action::EditorUndo => {
                if self.mode == AppMode::Editing && self.session.undo().unwrap_or(false) {
                    self.sync_editor_selection_bounds();
                }
            }
            Action::EditorRedo => {
                if self.mode == AppMode::Editing && self.session.redo().unwrap_or(false) {
                    self.sync_editor_selection_bounds();
                }
            }
            Action::Quit => {
                if self.session.dirty {
                    self.request_exit_action(PendingExitAction::QuitApp);
                } else {
                    self.mode = AppMode::Quitting;
                }
            }
            Action::EnterGotoMode => {
                self.mode = AppMode::GotoNode {
                    buffer: String::new(),
                };
            }
            Action::GotoDigit(digit) => {
                if let AppMode::GotoNode { ref mut buffer } = self.mode {
                    buffer.push_str(&digit.to_string());
                }
            }
            // Non-digit character appended to the goto buffer (for ID-prefix search).
            Action::GotoChar(c) => {
                if let AppMode::GotoNode { ref mut buffer } = self.mode {
                    buffer.push(c);
                }
            }
            // Backspace trims one character from the goto buffer.
            Action::GotoBackspace => {
                if let AppMode::GotoNode { ref mut buffer } = self.mode {
                    buffer.pop();
                }
            }
            Action::GotoConfirm => {
                if let AppMode::GotoNode { ref buffer } = self.mode {
                    let from_index = self.session.current_node_index();
                    // If the buffer is all-digits, treat it as a 1-based node index.
                    // Otherwise, find the first node whose ID starts with the typed text.
                    let target = if let Ok(num) = buffer.parse::<usize>() {
                        Some(num.saturating_sub(1))
                    } else {
                        let buf = buffer.clone();
                        self.session
                            .graph
                            .nodes
                            .iter()
                            .enumerate()
                            .find(|(_, n)| {
                                n.id.as_deref()
                                    .map(|id| id.starts_with(buf.as_str()))
                                    .unwrap_or(false)
                            })
                            .map(|(i, _)| i)
                    };
                    if let Some(idx) = target {
                        let _ = self.session.traversal.goto(idx, &self.session.graph);
                        let next_index = self.session.current_node_index();
                        if next_index != from_index {
                            self.record_navigation(next_index, false);
                        }
                        self.start_transition_if_needed(from_index);
                    }
                }
                self.mode = AppMode::Presenting;
            }
            Action::GotoCancel => {
                self.mode = AppMode::Presenting;
            }
            Action::Resize(width, height) => {
                self.terminal_size = (width, height);
                if self.mode == AppMode::Editing {
                    self.sync_editor_list_viewport();
                    self.sync_editor_graph_viewport();
                }
            }
            Action::MouseClick { column, row } => {
                self.handle_mouse_click(column, row);
            }
            Action::MouseDrag { column, row } => {
                self.handle_mouse_drag(column, row);
            }
            Action::MouseScroll(direction) => {
                self.handle_mouse_scroll(direction);
            }
            Action::Tick => {
                if let Some(mut transition) = self.active_transition {
                    transition.frame = transition.frame.saturating_add(1);
                    if transition.frame >= transition.total_frames {
                        self.active_transition = None;
                    } else {
                        self.active_transition = Some(transition);
                    }
                }
                self.refresh_timed_state();
            }
        }
    }

    /// Handle a crossterm event and map it to an action.
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if self.handle_pending_exit_key(key.code) {
                    self.needs_redraw = true;
                    return;
                }

                if self.mode == AppMode::Presenting
                    && !self.show_help
                    && self.handle_presenter_branch_keys(key.code)
                {
                    self.needs_redraw = true;
                    return;
                }

                if self.mode == AppMode::Editing
                    && key.code == KeyCode::Char('n')
                    && self.terminal_size.0 <= 80
                    && self.editor_text_input.is_none()
                    && self.editor_search_input.is_none()
                    && self.editor_index_jump_input.is_none()
                {
                    self.editor_node_list_visible = !self.editor_node_list_visible;
                    self.editor_status = Some(if self.editor_node_list_visible {
                        "Compact: node list visible".to_string()
                    } else {
                        "Compact: node list hidden".to_string()
                    });
                    self.needs_redraw = true;
                    return;
                }

                if self.show_help && self.handle_help_overlay_key(key.code) {
                    self.needs_redraw = true;
                    return;
                }

                if self.mode == AppMode::Editing && self.handle_graph_overlay_key(key.code) {
                    self.needs_redraw = true;
                    return;
                }

                if self.mode == AppMode::Editing && self.handle_picker_key(key.code) {
                    self.needs_redraw = true;
                    return;
                }

                if self.mode == AppMode::Editing
                    && self.handle_inline_edit_key(key.code, key.modifiers)
                {
                    self.needs_redraw = true;
                    return;
                }

                if self.mode == AppMode::Editing && self.handle_editor_search_key(key.code) {
                    self.needs_redraw = true;
                    return;
                }

                if self.mode == AppMode::Editing && self.handle_editor_index_jump_key(key.code) {
                    self.needs_redraw = true;
                    return;
                }

                if let Some(action) = map_key_to_action(key, &self.mode, self.editor_focus) {
                    self.update(action);
                }
            }
            Event::Resize(w, h) => {
                self.update(Action::Resize(w, h));
            }
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    self.update(Action::MouseClick {
                        column: mouse.column,
                        row: mouse.row,
                    });
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    self.update(Action::MouseDrag {
                        column: mouse.column,
                        row: mouse.row,
                    });
                }
                MouseEventKind::ScrollUp => {
                    self.update(Action::MouseScroll(MouseScrollDirection::Up));
                }
                MouseEventKind::ScrollDown => {
                    self.update(Action::MouseScroll(MouseScrollDirection::Down));
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_mouse_click(&mut self, column: u16, row: u16) {
        if self.mode == AppMode::Editing {
            self.handle_editor_mouse_click(column, row);
            return;
        }

        let (width, height) = self.terminal_size;

        let current = self.session.current_node_index();
        let node = &self.session.graph.nodes[current];

        if let Some(branch_point) = node.branch_point() {
            let root = Rect::new(0, 0, width, height);
            let popup = branch_overlay_rect(root, branch_point.options.len() as u16);

            let in_x = column >= popup.x && column < popup.x.saturating_add(popup.width);
            let in_y = row >= popup.y && row < popup.y.saturating_add(popup.height);

            if in_x && in_y {
                let has_prompt = branch_point
                    .prompt
                    .as_deref()
                    .is_some_and(|prompt| !prompt.trim().is_empty());
                let option_start = popup.y.saturating_add(if has_prompt { 3 } else { 1 });
                let option_idx = row.saturating_sub(option_start) as usize;
                let option_key = branch_point
                    .options
                    .get(option_idx)
                    .map(|option| option.key);
                if let Some(key) = option_key {
                    let from_index = self.session.current_node_index();
                    let _ = self.session.traversal.choose(key, &self.session.graph);
                    self.start_transition_if_needed(from_index);
                    return;
                }
            }
        }

        if column < width / 2 {
            let from_index = self.session.current_node_index();
            self.session.traversal.back();
            self.start_transition_if_needed(from_index);
        } else {
            let from_index = self.session.current_node_index();
            self.session.traversal.next(&self.session.graph);
            self.start_transition_if_needed(from_index);
        }
    }

    fn handle_help_overlay_key(&mut self, code: KeyCode) -> bool {
        if !self.show_help {
            return false;
        }

        let nav = self.help_overlay_navigation();
        let viewport = nav.viewport_rows.max(1);
        let max_scroll = nav.total_rows.saturating_sub(viewport);

        match code {
            KeyCode::Esc | KeyCode::Char('?') => {
                self.show_help = false;
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.help_scroll_offset = (self.help_scroll_offset + 1).min(max_scroll);
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.help_scroll_offset = self.help_scroll_offset.saturating_sub(1);
                true
            }
            KeyCode::PageDown => {
                self.help_scroll_offset = (self.help_scroll_offset + viewport).min(max_scroll);
                true
            }
            KeyCode::PageUp => {
                self.help_scroll_offset = self.help_scroll_offset.saturating_sub(viewport);
                true
            }
            KeyCode::Home => {
                self.help_scroll_offset = 0;
                true
            }
            KeyCode::End => {
                self.help_scroll_offset = max_scroll;
                true
            }
            KeyCode::Char(ch @ '1'..='4') => {
                let section_index = ch as usize - '1' as usize;
                if let Some(target) = nav.section_starts.get(section_index).copied() {
                    self.help_scroll_offset = target.min(max_scroll);
                }
                true
            }
            _ => true,
        }
    }

    fn help_overlay_navigation(&self) -> crate::ui::help::HelpNavigation {
        let root = Rect::new(0, 0, self.terminal_size.0, self.terminal_size.1);
        let mode = if self.mode == AppMode::Editing {
            HelpMode::Editing
        } else {
            HelpMode::Presenting
        };
        help_navigation(root, mode)
    }

    fn handle_mouse_scroll(&mut self, direction: MouseScrollDirection) {
        if self.mode == AppMode::Editing {
            self.handle_editor_mouse_scroll(direction);
            return;
        }

        match direction {
            MouseScrollDirection::Up => {
                let from_index = self.session.current_node_index();
                self.session.traversal.back();
                self.start_transition_if_needed(from_index);
            }
            MouseScrollDirection::Down => {
                let from_index = self.session.current_node_index();
                self.session.traversal.next(&self.session.graph);
                self.start_transition_if_needed(from_index);
            }
        }
    }

    fn handle_presenter_branch_keys(&mut self, code: KeyCode) -> bool {
        let Some(branch) = self.session.current_node().branch_point() else {
            self.branch_focused_option = 0;
            return false;
        };

        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.branch_focused_option = self.branch_focused_option.saturating_sub(1);
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = branch.options.len().saturating_sub(1);
                self.branch_focused_option = (self.branch_focused_option + 1).min(max);
                true
            }
            KeyCode::Enter => {
                if let Some(option) = branch.options.get(self.branch_focused_option) {
                    let from_index = self.session.current_node_index();
                    let _ = self
                        .session
                        .traversal
                        .choose(option.key, &self.session.graph);
                    self.branch_focused_option = 0;
                    self.start_transition_if_needed(from_index);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn start_transition_if_needed(&mut self, from_index: usize) {
        if self.mode != AppMode::Presenting {
            self.active_transition = None;
            return;
        }

        let to_index = self.session.current_node_index();
        if to_index == from_index {
            self.active_transition = None;
            return;
        }

        let kind = self.session.graph.nodes[to_index]
            .transition
            .unwrap_or(Transition::None);

        if kind == Transition::None {
            self.active_transition = None;
            return;
        }

        self.active_transition = Some(ActiveTransition {
            from_index,
            kind,
            frame: 0,
            total_frames: 7,
        });
    }
}
