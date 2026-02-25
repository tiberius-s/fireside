use super::*;

use crate::render::blocks::render_block;
use crate::theme::Theme;

impl App {
    pub(super) fn add_node_after_selected(&mut self) {
        let base_index = self.editor_selected_node;
        let mut suffix = self.session.graph.nodes.len() + 1;
        let new_node_id = loop {
            let candidate = format!("node-{suffix}");
            if self.session.graph.index_of(&candidate).is_none() {
                break candidate;
            }
            suffix += 1;
        };

        let command = Command::AddNode {
            node_id: new_node_id,
            after_index: Some(base_index),
        };

        if self.session.execute_command(command).is_ok() {
            self.select_editor_node(base_index + 1);
            self.editor_status = Some("Added node".to_string());
        }
    }

    pub(super) fn remove_selected_node(&mut self) {
        let idx = self.editor_selected_node;
        let Some(node_id) = self.session.graph.nodes.get(idx).and_then(|n| n.id.clone()) else {
            return;
        };

        let command = Command::RemoveNode { node_id };
        if self.session.execute_command(command).is_ok() {
            self.sync_editor_selection_bounds();
            self.select_editor_node(self.editor_selected_node);
            self.editor_status = Some("Removed node".to_string());
        }
    }

    pub(super) fn editor_select_next(&mut self) {
        let max = self.session.graph.nodes.len().saturating_sub(1);
        self.select_editor_node((self.editor_selected_node + 1).min(max));
    }

    pub(super) fn editor_select_prev(&mut self) {
        self.select_editor_node(self.editor_selected_node.saturating_sub(1));
    }

    pub(super) fn editor_select_next_block(&mut self) {
        let count = self.selected_node_block_count();
        if count == 0 {
            self.editor_status = Some("Selected node has no content blocks".to_string());
            return;
        }

        self.editor_selected_block = (self.editor_selected_block + 1).min(count - 1);
        self.editor_status = Some(format!(
            "Selected block #{}",
            self.editor_selected_block + 1
        ));
        self.scroll_detail_to_selected_block();
    }

    pub(super) fn editor_select_prev_block(&mut self) {
        let count = self.selected_node_block_count();
        if count == 0 {
            self.editor_status = Some("Selected node has no content blocks".to_string());
            return;
        }

        self.editor_selected_block = self.editor_selected_block.saturating_sub(1);
        self.editor_status = Some(format!(
            "Selected block #{}",
            self.editor_selected_block + 1
        ));
        self.scroll_detail_to_selected_block();
    }

    /// Delete the currently selected content block from the selected node.
    pub(super) fn remove_selected_block(&mut self) {
        let count = self.selected_node_block_count();
        if count == 0 {
            self.editor_status = Some("No content blocks to delete".to_string());
            return;
        }

        let block_index = self.editor_selected_block.min(count - 1);
        let node_index = self.editor_selected_node;
        let node_id = match self.session.ensure_node_id(node_index) {
            Ok(id) => id,
            Err(_) => return,
        };

        let command = Command::RemoveBlock {
            node_id,
            block_index,
        };

        if self.session.execute_command(command).is_ok() {
            // Keep the selection within bounds after removal.
            let new_count = self.selected_node_block_count();
            if new_count > 0 {
                self.editor_selected_block = block_index.min(new_count - 1);
            } else {
                self.editor_selected_block = 0;
            }
            self.editor_status = Some(format!("Deleted block #{}", block_index + 1));
            self.scroll_detail_to_selected_block();
        }
    }

    pub(super) fn move_selected_block(&mut self, forward: bool) {
        let count = self.selected_node_block_count();
        if count < 2 {
            self.editor_status = Some("Need at least two blocks to reorder".to_string());
            return;
        }

        let from_index = self.editor_selected_block.min(count - 1);
        let to_index = if forward {
            (from_index + 1).min(count - 1)
        } else {
            from_index.saturating_sub(1)
        };

        if from_index == to_index {
            return;
        }

        let node_index = self.editor_selected_node;
        let node_id = match self.session.ensure_node_id(node_index) {
            Ok(id) => id,
            Err(_) => return,
        };

        let command = Command::MoveBlock {
            node_id,
            from_index,
            to_index,
        };

        if self.session.execute_command(command).is_ok() {
            self.editor_selected_block = to_index;
            self.editor_status = Some(format!("Moved block to #{}", to_index + 1));
            self.scroll_detail_to_selected_block();
        }
    }

    pub(super) fn editor_page_down(&mut self) {
        let page = self.editor_list_visible_rows().max(1);
        let max = self.session.graph.nodes.len().saturating_sub(1);
        self.select_editor_node((self.editor_selected_node + page).min(max));
    }

    pub(super) fn editor_page_up(&mut self) {
        let page = self.editor_list_visible_rows().max(1);
        self.select_editor_node(self.editor_selected_node.saturating_sub(page));
    }

    pub(super) fn editor_jump_top(&mut self) {
        self.select_editor_node(0);
    }

    pub(super) fn editor_jump_bottom(&mut self) {
        self.select_editor_node(self.session.graph.nodes.len().saturating_sub(1));
    }

    pub(super) fn start_editor_node_search(&mut self) {
        self.editor_index_jump_input = None;
        self.editor_search_input = Some(String::new());
        self.editor_focus = EditorPaneFocus::NodeList;
        self.editor_status = Some("Node search: type text, Enter to jump, Esc to cancel".into());
    }

    pub(super) fn start_editor_index_jump(&mut self) {
        self.editor_search_input = None;
        self.editor_index_jump_input = Some(String::new());
        self.editor_focus = EditorPaneFocus::NodeList;
        self.editor_status =
            Some("Jump to index: type number, Enter to jump, Esc to cancel".into());
    }

    pub(super) fn handle_editor_search_key(&mut self, code: KeyCode) -> bool {
        let Some(buffer) = self.editor_search_input.as_mut() else {
            return false;
        };

        match code {
            KeyCode::Esc => {
                self.editor_search_input = None;
                self.editor_status = Some("Node search cancelled".to_string());
                true
            }
            KeyCode::Enter => {
                self.commit_editor_search();
                true
            }
            KeyCode::Backspace => {
                buffer.pop();
                true
            }
            KeyCode::Char(ch) => {
                buffer.push(ch);
                true
            }
            _ => true,
        }
    }

    pub(super) fn handle_editor_index_jump_key(&mut self, code: KeyCode) -> bool {
        let Some(buffer) = self.editor_index_jump_input.as_mut() else {
            return false;
        };

        match code {
            KeyCode::Esc => {
                self.editor_index_jump_input = None;
                self.editor_status = Some("Index jump cancelled".to_string());
                true
            }
            KeyCode::Enter => {
                self.commit_editor_index_jump();
                true
            }
            KeyCode::Backspace => {
                buffer.pop();
                true
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                buffer.push(ch);
                true
            }
            _ => true,
        }
    }

    fn commit_editor_search(&mut self) {
        let query = self.editor_search_input.take().unwrap_or_default();
        let trimmed = query.trim();
        if trimmed.is_empty() {
            self.editor_status = Some("Node search empty".to_string());
            return;
        }
        self.editor_search_query = Some(trimmed.to_string());

        let tokens = search_tokens(trimmed);
        let total = self.session.graph.nodes.len();
        if total == 0 {
            self.editor_status = Some("Node search failed: no nodes".to_string());
            return;
        }

        let current = self.editor_selected_node;
        let found = self
            .session
            .graph
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                let id = node.id.as_deref().unwrap_or("");
                let id_lower = id.to_ascii_lowercase();
                let score = score_node_id_match(&id_lower, &tokens)?;
                let distance = if idx >= current {
                    idx - current
                } else {
                    total - (current - idx)
                };
                Some((score, distance, idx))
            })
            .min_by_key(|&(score, distance, idx)| (score, distance, idx))
            .map(|(_, _, idx)| idx);

        if let Some(idx) = found {
            self.select_editor_node(idx);
            self.editor_status = Some(format!("Node search matched #{}", idx + 1));
        } else {
            self.editor_status = Some(format!("No node id matches '{trimmed}'"));
        }
    }

    pub(super) fn jump_editor_search_hit(&mut self, forward: bool) {
        let Some(query) = self.editor_search_query.as_deref() else {
            self.editor_status = Some("No prior search query".to_string());
            return;
        };

        let total = self.session.graph.nodes.len();
        if total == 0 {
            self.editor_status = Some("No nodes available".to_string());
            return;
        }

        let tokens = search_tokens(query);
        let current = self.editor_selected_node;
        let next = if forward {
            next_search_hit_from(&self.session, &tokens, current)
        } else {
            prev_search_hit_from(&self.session, &tokens, current)
        };

        if let Some(idx) = next {
            self.select_editor_node(idx);
            let direction = if forward { "next" } else { "previous" };
            self.editor_status = Some(format!("Search {direction} hit: #{}", idx + 1));
        } else {
            self.editor_status = Some(format!("No node id matches '{query}'"));
        }
    }

    fn commit_editor_index_jump(&mut self) {
        let query = self.editor_index_jump_input.take().unwrap_or_default();
        let trimmed = query.trim();
        if trimmed.is_empty() {
            self.editor_status = Some("Index jump empty".to_string());
            return;
        }

        let Ok(parsed) = trimmed.parse::<usize>() else {
            self.editor_status = Some(format!("Invalid index '{trimmed}'"));
            return;
        };

        let total = self.session.graph.nodes.len();
        if total == 0 {
            self.editor_status = Some("Index jump failed: no nodes".to_string());
            return;
        }

        if parsed == 0 || parsed > total {
            self.editor_status = Some(format!("Index out of range: 1..{total}"));
            return;
        }

        let idx = parsed - 1;
        self.select_editor_node(idx);
        self.editor_status = Some(format!("Jumped to node #{}", idx + 1));
    }

    pub(super) fn select_editor_node(&mut self, index: usize) {
        let total = self.session.graph.nodes.len();
        if total == 0 {
            return;
        }

        let clamped = index.min(total - 1);
        self.editor_selected_node = clamped;
        self.editor_selected_block = 0;
        // Reset the WYSIWYG preview scroll when switching nodes.
        self.editor_detail_scroll_offset = 0;
        self.sync_editor_list_viewport();
        let _ = self
            .session
            .traversal
            .goto(self.editor_selected_node, &self.session.graph);
    }

    pub(super) fn selected_node_block_count(&self) -> usize {
        self.session
            .graph
            .nodes
            .get(self.editor_selected_node)
            .map_or(0, |node| node.content.len())
    }

    pub(super) fn selected_block_with_index(&self) -> Option<(usize, &ContentBlock)> {
        let node = self.session.graph.nodes.get(self.editor_selected_node)?;
        let count = node.content.len();
        if count == 0 {
            return None;
        }

        let index = self.editor_selected_block.min(count - 1);
        node.content.get(index).map(|block| (index, block))
    }

    pub(super) fn sync_editor_block_selection_bounds(&mut self) {
        let max = self.selected_node_block_count().saturating_sub(1);
        self.editor_selected_block = self.editor_selected_block.min(max);
    }

    pub(super) fn sync_editor_selection_bounds(&mut self) {
        let max = self.session.graph.nodes.len().saturating_sub(1);
        self.editor_selected_node = self.editor_selected_node.min(max);
        self.editor_graph_selected_node = self.editor_graph_selected_node.min(max);
        self.sync_editor_block_selection_bounds();
        self.sync_editor_list_viewport();
        self.sync_editor_graph_viewport();
    }

    pub(super) fn sync_editor_list_viewport(&mut self) {
        let total = self.session.graph.nodes.len();
        if total == 0 {
            self.editor_list_scroll_offset = 0;
            return;
        }

        let visible_rows = self.editor_list_visible_rows().max(1);
        let max_offset = total.saturating_sub(visible_rows);
        self.editor_list_scroll_offset = self.editor_list_scroll_offset.min(max_offset);

        if self.editor_selected_node < self.editor_list_scroll_offset {
            self.editor_list_scroll_offset = self.editor_selected_node;
            return;
        }

        let end = self.editor_list_scroll_offset + visible_rows;
        if self.editor_selected_node >= end {
            self.editor_list_scroll_offset = self.editor_selected_node + 1 - visible_rows;
        }
    }

    pub(super) fn sync_editor_graph_viewport(&mut self) {
        let total = self.session.graph.nodes.len();
        if total == 0 {
            self.editor_graph_scroll_offset = 0;
            return;
        }

        let root = Rect::new(0, 0, self.terminal_size.0, self.terminal_size.1);
        // Length(2) mirrors render_editor's status-bar row constraint.
        let sections = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(root);

        let window = graph_overlay_window(
            sections[0],
            &self.session,
            self.editor_graph_selected_node,
            self.editor_graph_scroll_offset,
        );

        if self.editor_graph_selected_node < window.start
            || self.editor_graph_selected_node >= window.end
        {
            self.editor_graph_scroll_offset = self.editor_graph_selected_node;
        } else {
            self.editor_graph_scroll_offset = window.start;
        }
    }

    fn editor_list_visible_rows(&self) -> usize {
        if self.terminal_size.0 <= 80 {
            if !self.editor_node_list_visible {
                return 0;
            }
            let root = Rect::new(0, 0, self.terminal_size.0, self.terminal_size.1);
            let sections = RatatuiLayout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(2)])
                .split(root);
            let body = RatatuiLayout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
                .split(sections[0]);
            return body[0].height.saturating_sub(2) as usize;
        }

        let root = Rect::new(0, 0, self.terminal_size.0, self.terminal_size.1);
        let sections = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(root);
        let body = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(sections[0]);

        body[0].height.saturating_sub(2) as usize
    }

    pub(super) fn editor_graph_visible_rows(&self) -> usize {
        let root = Rect::new(0, 0, self.terminal_size.0, self.terminal_size.1);
        let sections = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(root);

        graph_overlay_page_span(
            sections[0],
            &self.session,
            self.editor_graph_selected_node,
            self.editor_graph_scroll_offset,
        )
    }

    pub(super) fn toggle_editor_graph_view(&mut self) {
        self.editor_graph_overlay = !self.editor_graph_overlay;
        if self.editor_graph_overlay {
            self.editor_picker = None;
            self.editor_graph_selected_node = self.editor_selected_node;
            self.sync_editor_graph_viewport();
            self.editor_status = Some("Graph view opened (Enter jumps to node)".to_string());
        } else {
            self.editor_status = Some("Graph view closed".to_string());
        }
    }

    pub(super) fn handle_graph_overlay_key(&mut self, code: KeyCode) -> bool {
        if !self.editor_graph_overlay {
            return false;
        }

        match code {
            KeyCode::Esc | KeyCode::Char('v') => {
                self.editor_graph_overlay = false;
                self.editor_status = Some("Graph view closed".to_string());
                true
            }
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Left | KeyCode::Char('h') => {
                self.editor_graph_selected_node = self.editor_graph_selected_node.saturating_sub(1);
                self.sync_editor_graph_viewport();
                true
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Right | KeyCode::Char('l') => {
                let max = self.session.graph.nodes.len().saturating_sub(1);
                self.editor_graph_selected_node = (self.editor_graph_selected_node + 1).min(max);
                self.sync_editor_graph_viewport();
                true
            }
            KeyCode::PageUp => {
                let page = self.editor_graph_visible_rows().max(1);
                self.editor_graph_selected_node =
                    self.editor_graph_selected_node.saturating_sub(page);
                self.sync_editor_graph_viewport();
                true
            }
            KeyCode::PageDown => {
                let page = self.editor_graph_visible_rows().max(1);
                let max = self.session.graph.nodes.len().saturating_sub(1);
                self.editor_graph_selected_node = (self.editor_graph_selected_node + page).min(max);
                self.sync_editor_graph_viewport();
                true
            }
            KeyCode::Home => {
                self.editor_graph_selected_node = 0;
                self.sync_editor_graph_viewport();
                true
            }
            KeyCode::End => {
                self.editor_graph_selected_node = self.session.graph.nodes.len().saturating_sub(1);
                self.sync_editor_graph_viewport();
                true
            }
            KeyCode::Enter => {
                self.apply_graph_overlay_selection(false);
                true
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.apply_graph_overlay_selection(true);
                true
            }
            _ => true,
        }
    }

    fn apply_graph_overlay_selection(&mut self, enter_present_mode: bool) {
        let idx = self
            .editor_graph_selected_node
            .min(self.session.graph.nodes.len().saturating_sub(1));
        self.select_editor_node(idx);
        self.editor_graph_overlay = false;

        if enter_present_mode {
            self.mode = AppMode::Presenting;
            self.editor_text_input = None;
            self.editor_inline_target = None;
            self.editor_picker = None;
            self.editor_search_input = None;
            self.editor_index_jump_input = None;
            self.pending_exit_action = None;
            self.persist_editor_preferences();
            self.editor_status = Some(format!("Presenter jump: node #{}", idx + 1));
        } else {
            self.editor_status = Some(format!("Graph jump: node #{}", idx + 1));
        }
    }

    /// Scroll the WYSIWYG detail pane so the currently selected block is visible.
    ///
    /// Uses `render_block` to count rendered lines for all preceding blocks, then
    /// sets `editor_detail_scroll_offset` so the selected block's header appears near
    /// the top of the pane.  This is called whenever `editor_selected_block` changes.
    pub(super) fn scroll_detail_to_selected_block(&mut self) {
        let block_index = self.editor_selected_block;
        if block_index == 0 {
            // First block is always visible without scrolling.
            self.editor_detail_scroll_offset = 0;
            return;
        }

        let Some(node) = self.session.graph.nodes.get(self.editor_selected_node) else {
            return;
        };

        // The detail pane renders a fixed preamble before the block list (editor.rs):
        //   node title, blank, METADATA header, layout row, transition row,
        //   id row, blocks count row, blank, SLIDE PREVIEW header  →  9 lines.
        const PREAMBLE: usize = 9;

        // Approximate detail pane width (70 % of terminal, minus gutter/border).
        let approx_width = ((self.terminal_size.0 as usize * 70) / 100)
            .saturating_sub(5)
            .max(40) as u16;

        let theme = Theme::default();
        // Sum header row (1) + rendered lines + separator row (1) for each preceding block.
        let preceding_lines: usize = node.content[..block_index]
            .iter()
            .map(|block| 1 + render_block(block, &theme, approx_width).len() + 1)
            .sum();

        // Leave one line of context above the selected block header.
        self.editor_detail_scroll_offset = (PREAMBLE + preceding_lines).saturating_sub(1);
    }
}
