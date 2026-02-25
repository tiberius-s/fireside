//! Application state machine and main update loop.
//!
//! Implements the TEA (Model-View-Update) pattern: the `App` struct holds
//! all state, `update()` processes actions, and `view()` renders the UI.
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

use fireside_core::model::content::ContentBlock;
use fireside_core::model::graph::Graph;
use fireside_core::model::layout::Layout;
use fireside_core::model::transition::Transition;
use fireside_engine::validation::validate_content_block;
use fireside_engine::{Command, PresentationSession, save_graph};

use self::app_helpers::{
    block_type_variants, bump_index, centered_popup, digit_to_index, is_editor_actionable_warning,
    layout_variants, next_search_hit_from, picker_row_span, point_in_rect, prev_search_hit_from,
    score_node_id_match, search_tokens, transition_variants, update_block_from_inline_text,
    update_block_metadata_from_inline_text,
};

use crate::config::keybindings::map_key_to_action;
use crate::config::settings::{EditorUiPrefs, load_editor_ui_prefs, save_editor_ui_prefs};
use crate::event::{Action, MouseScrollDirection};
use crate::theme::Theme;
use crate::ui::branch::branch_overlay_rect;
use crate::ui::chrome::FlashKind;
use crate::ui::editor::{EditorViewState, render_editor};
use crate::ui::graph::{
    GraphOverlayViewState, graph_overlay_list_panel_rect, graph_overlay_page_span,
    graph_overlay_rect, graph_overlay_row_to_node, graph_overlay_window,
};
use crate::ui::help::{HelpMode, help_navigation};
use crate::ui::presenter::{PresenterTransition, PresenterViewState, render_presenter};

mod action_routing;
mod app_helpers;
mod editor_commands;
mod editor_input;
mod editor_navigation;
mod editor_picker;

#[cfg(test)]
mod app_tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorPaneFocus {
    NodeList,
    NodeDetail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorInlineTarget {
    BlockField { block_index: usize },
    BlockMetadataField { block_index: usize },
    SpeakerNotes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingExitAction {
    ExitEditor,
    QuitApp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorPickerOverlay {
    Layout { selected: usize },
    Transition { selected: usize },
    BlockType { selected: usize },
}

/// The current mode of the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    /// Normal presentation mode.
    Presenting,
    /// Interactive editing shell mode.
    Editing,
    /// Waiting for node number input.
    GotoNode {
        /// The digits entered so far.
        buffer: String,
    },
    /// Application is quitting.
    Quitting,
}

/// Main application state.
pub struct App {
    /// The presentation session (graph + traversal state).
    pub session: PresentationSession,
    /// Current application mode.
    pub mode: AppMode,
    /// Whether the help overlay is visible.
    pub show_help: bool,
    /// Whether speaker notes are visible.
    pub show_speaker_notes: bool,
    /// Top row offset for scrollable help overlay content.
    help_scroll_offset: usize,
    /// The active theme.
    pub theme: Theme,
    /// When the presentation started (for elapsed time display).
    pub start_time: Instant,
    /// Most recently known terminal size.
    pub terminal_size: (u16, u16),
    /// Selected node index for editing mode.
    pub editor_selected_node: usize,
    /// Selected content block index for editing mode.
    pub editor_selected_block: usize,
    /// Focused editor pane.
    pub editor_focus: EditorPaneFocus,
    /// Optional path where editor saves the current graph.
    pub editor_target_path: Option<PathBuf>,
    /// Active inline text input buffer.
    pub editor_text_input: Option<String>,
    /// Last editor status message.
    pub editor_status: Option<String>,
    /// Active inline edit target for current text buffer.
    editor_inline_target: Option<EditorInlineTarget>,
    /// Pending exit action requiring confirmation when dirty.
    pending_exit_action: Option<PendingExitAction>,
    /// Active metadata picker overlay.
    editor_picker: Option<EditorPickerOverlay>,
    /// Active node-id search input in editor mode.
    editor_search_input: Option<String>,
    /// Last committed node-id search query.
    editor_search_query: Option<String>,
    /// Active numeric index jump input in editor mode.
    editor_index_jump_input: Option<String>,
    /// Last layout picker index (persisted).
    editor_last_layout_picker: usize,
    /// Last transition picker index (persisted).
    editor_last_transition_picker: usize,
    /// Last block-type picker index (persisted).
    editor_last_block_picker: usize,
    /// Top row offset for virtualized node list rendering.
    editor_list_scroll_offset: usize,
    /// Whether the editor graph overlay is visible.
    editor_graph_overlay: bool,
    /// Selected node index in graph overlay.
    editor_graph_selected_node: usize,
    /// Top row offset for graph overlay list viewport.
    editor_graph_scroll_offset: usize,
    /// Top row offset for the detail pane (for scrolling rendered block content).
    editor_detail_scroll_offset: usize,
    /// Whether compact editor mode currently shows the node list overlay.
    editor_node_list_visible: bool,
    /// Focused branch option index in presenter mode branch overlays.
    branch_focused_option: usize,
    /// Active presenter transition animation.
    active_transition: Option<ActiveTransition>,
    /// Optional base directory for resolving relative content assets.
    document_base_dir: Option<PathBuf>,
    /// Whether presenter mode should render the progress footer.
    show_progress_bar: bool,
    /// Whether presenter mode should render elapsed timer in footer.
    show_elapsed_timer: bool,
    /// Whether presenter mode is in distraction-free mode.
    show_zen_mode: bool,
    /// Whether the presenter timeline strip is visible.
    show_timeline: bool,
    /// Optional target duration in seconds for pace guidance.
    target_duration_secs: Option<u64>,
    /// Recent visited nodes for timeline rendering.
    visited_nodes: Vec<usize>,
    /// Navigation path with branch-step markers for breadcrumb rendering.
    nav_path: Vec<(usize, bool)>,
    /// Whether the UI needs a redraw.
    needs_redraw: bool,
    /// Transient global flash message for presenter/editor chrome.
    flash_message: Option<(String, FlashKind, Instant)>,
    /// Timestamp when current unsaved period started.
    dirty_since: Option<Instant>,
    /// Last time an unsaved-data warning flash was emitted.
    last_dirty_flash_at: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ActiveTransition {
    from_index: usize,
    kind: Transition,
    frame: u8,
    total_frames: u8,
}

impl ActiveTransition {
    fn progress(self) -> f32 {
        if self.total_frames <= 1 {
            1.0
        } else {
            self.frame as f32 / (self.total_frames - 1) as f32
        }
    }
}

impl App {
    /// Create a new application with the given session and theme.
    #[must_use]
    pub fn new(session: PresentationSession, theme: Theme) -> Self {
        let start_index = session.current_node_index();
        Self {
            session,
            mode: AppMode::Presenting,
            show_help: false,
            show_speaker_notes: false,
            help_scroll_offset: 0,
            theme,
            start_time: Instant::now(),
            terminal_size: (120, 40),
            editor_selected_node: 0,
            editor_selected_block: 0,
            editor_focus: EditorPaneFocus::NodeList,
            editor_target_path: None,
            editor_text_input: None,
            editor_status: None,
            editor_inline_target: None,
            pending_exit_action: None,
            editor_picker: None,
            editor_search_input: None,
            editor_search_query: None,
            editor_index_jump_input: None,
            editor_last_layout_picker: 0,
            editor_last_transition_picker: 0,
            editor_last_block_picker: 0,
            editor_list_scroll_offset: 0,
            editor_graph_overlay: false,
            editor_graph_selected_node: 0,
            editor_graph_scroll_offset: 0,
            editor_detail_scroll_offset: 0,
            editor_node_list_visible: false,
            branch_focused_option: 0,
            active_transition: None,
            document_base_dir: None,
            show_progress_bar: true,
            show_elapsed_timer: true,
            show_zen_mode: false,
            show_timeline: true,
            target_duration_secs: None,
            visited_nodes: vec![start_index],
            nav_path: vec![(start_index, false)],
            needs_redraw: true,
            flash_message: None,
            dirty_since: None,
            last_dirty_flash_at: None,
        }
    }

    /// Enable or disable the presenter progress footer.
    pub fn set_show_progress_bar(&mut self, show: bool) {
        self.show_progress_bar = show;
    }

    /// Enable or disable elapsed timer text in the presenter footer.
    pub fn set_show_elapsed_timer(&mut self, show: bool) {
        self.show_elapsed_timer = show;
    }

    /// Set optional target duration for presenter pace guidance.
    pub fn set_target_duration_secs(&mut self, target_secs: Option<u64>) {
        self.target_duration_secs = target_secs;
    }

    /// Set the loaded document path used to resolve relative assets.
    pub fn set_document_path(&mut self, path: PathBuf) {
        self.document_base_dir = path.parent().map(std::path::Path::to_path_buf);
    }

    /// Reload session graph while preserving the current node when possible.
    pub fn reload_graph(&mut self, graph: Graph) {
        if graph.nodes.is_empty() {
            return;
        }

        let current_id = self.session.current_node().id.clone();
        let fallback_index = self
            .session
            .current_node_index()
            .min(graph.len().saturating_sub(1));
        let next_index = current_id
            .as_deref()
            .and_then(|id| graph.index_of(id))
            .unwrap_or(fallback_index);

        self.session = PresentationSession::new(graph, next_index);
        self.active_transition = None;
        self.visited_nodes.clear();
        self.nav_path.clear();
        self.record_navigation(next_index, false);

        if self.mode == AppMode::Editing {
            self.sync_editor_selection_bounds();
            self.sync_editor_list_viewport();
        }

        self.needs_redraw = true;
    }

    /// Returns true if a redraw is pending and clears the flag.
    pub fn take_needs_redraw(&mut self) -> bool {
        std::mem::take(&mut self.needs_redraw)
    }

    /// Returns `true` when presenter hot-reload is currently safe to apply.
    #[must_use]
    pub fn can_hot_reload(&self) -> bool {
        self.mode == AppMode::Presenting
    }

    /// Returns `true` if the application should quit.
    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.mode == AppMode::Quitting
    }

    /// Enter editing mode.
    pub fn enter_edit_mode(&mut self) {
        let previous_mode = self.mode.clone();
        self.mode = AppMode::Editing;
        self.editor_selected_node = self.session.current_node_index();
        self.sync_editor_block_selection_bounds();
        self.editor_graph_selected_node = self.editor_selected_node;
        self.load_editor_preferences();
        self.sync_editor_selection_bounds();
        self.sync_editor_list_viewport();
        self.sync_editor_graph_viewport();

        if previous_mode == AppMode::Presenting {
            self.editor_status = Some(format!(
                "Presenter → editor @ node #{}",
                self.editor_selected_node + 1
            ));
        }
    }

    /// Set the save target path used by editor save actions.
    pub fn set_editor_target_path(&mut self, path: PathBuf) {
        self.editor_target_path = Some(path);
    }

    /// Render the current application state to the terminal frame.
    pub fn view(&self, frame: &mut Frame) {
        let elapsed = self.start_time.elapsed().as_secs();
        match self.mode {
            AppMode::Editing => {
                let selected_block_index = self.selected_block_with_index().map(|(index, _)| index);
                let block_warning_messages = self
                    .selected_block_with_index()
                    .map(|(_, block)| {
                        validate_content_block(block)
                            .into_iter()
                            .map(|diag| diag.message)
                            .filter(|message| is_editor_actionable_warning(message))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                render_editor(
                    frame,
                    &self.session,
                    &self.theme,
                    self.show_help,
                    EditorViewState {
                        selected_index: self.editor_selected_node,
                        list_scroll_offset: self.editor_list_scroll_offset,
                        focus: self.editor_focus,
                        inline_text_input: self.editor_text_input.as_deref(),
                        selected_block_index,
                        block_warning_messages: &block_warning_messages,
                        search_input: self.editor_search_input.as_deref(),
                        index_jump_input: self.editor_index_jump_input.as_deref(),
                        status: self.editor_status.as_deref(),
                        pending_exit_confirmation: self.pending_exit_action.is_some(),
                        picker_overlay: self.editor_picker,
                        graph_overlay: self.editor_graph_overlay.then_some(GraphOverlayViewState {
                            selected_index: self.editor_graph_selected_node,
                            scroll_offset: self.editor_graph_scroll_offset,
                        }),
                        help_scroll_offset: self.help_scroll_offset,
                        detail_scroll_offset: self.editor_detail_scroll_offset,
                        node_list_visible: self.editor_node_list_visible,
                    },
                );
            }
            _ => {
                render_presenter(
                    frame,
                    &self.session,
                    &self.theme,
                    PresenterViewState {
                        show_help: self.show_help,
                        help_scroll_offset: self.help_scroll_offset,
                        show_speaker_notes: self.show_speaker_notes,
                        show_progress_bar: self.show_progress_bar && !self.show_zen_mode,
                        show_elapsed_timer: self.show_elapsed_timer,
                        show_chrome: !self.show_zen_mode,
                        show_timeline: self.show_timeline,
                        target_duration_secs: self.target_duration_secs,
                        visited_nodes: &self.visited_nodes,
                        nav_path: &self.nav_path,
                        content_base_dir: self.document_base_dir.as_deref(),
                        transition: self
                            .active_transition
                            .map(|transition| PresenterTransition {
                                kind: transition.kind,
                                progress: transition.progress(),
                                from_index: transition.from_index,
                            }),
                        elapsed_secs: elapsed,
                        goto_buffer: if let AppMode::GotoNode { ref buffer } = self.mode {
                            Some(buffer.as_str())
                        } else {
                            None
                        },
                        branch_focused_option: self.branch_focused_option,
                        flash_message: self.visible_flash(),
                        pending_exit_confirmation: self.pending_exit_action.is_some(),
                    },
                );
            }
        }
    }

    #[must_use]
    pub fn is_animating(&self) -> bool {
        self.active_transition.is_some()
    }

    #[must_use]
    pub fn needs_periodic_tick(&self) -> bool {
        self.is_animating() || self.visible_flash().is_some() || self.session.dirty
    }

    fn request_exit_action(&mut self, action: PendingExitAction) {
        if self.session.dirty {
            self.pending_exit_action = Some(action);
            if self.mode == AppMode::Editing {
                self.editor_status =
                    Some("Unsaved changes: s=save+exit  y=discard+exit  Esc=stay".to_string());
            }
            self.set_flash(
                "Unsaved changes: s=save+exit  y=discard+exit  Esc=stay",
                FlashKind::Warning,
            );
            return;
        }

        self.apply_exit_action(action);
    }

    fn handle_pending_exit_key(&mut self, code: KeyCode) -> bool {
        if self.pending_exit_action.is_none() {
            return false;
        }

        match code {
            KeyCode::Char('y')
            | KeyCode::Char('Y')
            | KeyCode::Char('n')
            | KeyCode::Char('N')
            | KeyCode::Char('d')
            | KeyCode::Char('D')
            | KeyCode::Char('q')
            | KeyCode::Char('Q')
            | KeyCode::Enter => {
                let action = self.pending_exit_action.take();
                if let Some(action) = action {
                    self.apply_exit_action(action);
                }
                true
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                let action = self.pending_exit_action;
                if self.save_graph_to_target() {
                    self.pending_exit_action = None;
                    if let Some(action) = action {
                        self.apply_exit_action(action);
                    }
                }
                true
            }
            KeyCode::Esc => {
                self.pending_exit_action = None;
                if self.mode == AppMode::Editing {
                    self.editor_status = Some("Stayed in editor".to_string());
                }
                self.set_flash("Quit cancelled", FlashKind::Info);
                true
            }
            _ => true,
        }
    }

    fn apply_exit_action(&mut self, action: PendingExitAction) {
        self.pending_exit_action = None;
        self.editor_picker = None;
        self.editor_graph_overlay = false;
        self.editor_search_input = None;
        self.editor_search_query = None;
        self.editor_index_jump_input = None;
        match action {
            PendingExitAction::ExitEditor => {
                self.mode = AppMode::Presenting;
                self.editor_text_input = None;
                self.editor_inline_target = None;
                self.persist_editor_preferences();
                let _ = self
                    .session
                    .traversal
                    .goto(self.editor_selected_node, &self.session.graph);
            }
            PendingExitAction::QuitApp => {
                self.mode = AppMode::Quitting;
            }
        }
    }

    fn set_flash(&mut self, message: impl Into<String>, kind: FlashKind) {
        self.flash_message = Some((message.into(), kind, Instant::now()));
        self.needs_redraw = true;
    }

    fn visible_flash(&self) -> Option<(&str, FlashKind)> {
        self.flash_message.as_ref().and_then(|(text, kind, at)| {
            if Instant::now().duration_since(*at) < Duration::from_secs(3) {
                Some((text.as_str(), *kind))
            } else {
                None
            }
        })
    }

    fn refresh_timed_state(&mut self) {
        let now = Instant::now();
        if let Some((_, _, shown_at)) = self.flash_message
            && now.duration_since(shown_at) >= Duration::from_secs(3)
        {
            self.flash_message = None;
            self.needs_redraw = true;
        }

        if self.session.dirty {
            if self.dirty_since.is_none() {
                self.dirty_since = Some(now);
                self.last_dirty_flash_at = None;
            }

            if let Some(since) = self.dirty_since
                && now.duration_since(since) >= Duration::from_secs(30)
            {
                let should_emit = self
                    .last_dirty_flash_at
                    .is_none_or(|last| now.duration_since(last) >= Duration::from_secs(30));
                if should_emit {
                    self.set_flash("Unsaved changes for over 30 seconds", FlashKind::Warning);
                    self.last_dirty_flash_at = Some(now);
                }
            }
        } else {
            self.dirty_since = None;
            self.last_dirty_flash_at = None;
        }
    }
}

impl App {
    fn record_navigation(&mut self, index: usize, via_branch: bool) {
        if self.visited_nodes.last().copied() != Some(index) {
            self.visited_nodes.push(index);
            if self.visited_nodes.len() > 128 {
                self.visited_nodes.remove(0);
            }
        }

        if self.nav_path.last().map(|(last, _)| *last) != Some(index) {
            self.nav_path.push((index, via_branch));
            if self.nav_path.len() > 128 {
                self.nav_path.remove(0);
            }
        }
    }
}
