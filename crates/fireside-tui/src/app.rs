//! Application state machine and main update loop.
//!
//! Implements the TEA (Model-View-Update) pattern: the `App` struct holds
//! all state, `update()` processes actions, and `view()` renders the UI.
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

use fireside_core::model::content::{ContentBlock, ListItem};
use fireside_core::model::graph::Graph;
use fireside_core::model::layout::Layout;
use fireside_core::model::transition::Transition;
use fireside_engine::validation::validate_content_block;
use fireside_engine::{Command, PresentationSession, save_graph};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorPaneFocus {
    NodeList,
    NodeDetail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorInlineTarget {
    BlockField { block_index: usize },
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
            Action::GotoConfirm => {
                if let AppMode::GotoNode { ref buffer } = self.mode
                    && let Ok(num) = buffer.parse::<usize>()
                {
                    // User enters 1-based, we use 0-based
                    let idx = num.saturating_sub(1);
                    let from_index = self.session.current_node_index();
                    let _ = self.session.traversal.goto(idx, &self.session.graph);
                    let next_index = self.session.current_node_index();
                    if next_index != from_index {
                        self.record_navigation(next_index, false);
                    }
                    self.start_transition_if_needed(from_index);
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

                if let Some(action) = map_key_to_action(key, &self.mode) {
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

    fn add_node_after_selected(&mut self) {
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
            self.editor_selected_node =
                (base_index + 1).min(self.session.graph.nodes.len().saturating_sub(1));
            self.editor_selected_block = 0;
            self.sync_editor_list_viewport();
            let _ = self
                .session
                .traversal
                .goto(self.editor_selected_node, &self.session.graph);
            self.editor_status = Some("Added node".to_string());
        }
    }

    fn remove_selected_node(&mut self) {
        let idx = self.editor_selected_node;
        let Some(node_id) = self.session.graph.nodes.get(idx).and_then(|n| n.id.clone()) else {
            return;
        };

        let command = Command::RemoveNode { node_id };
        if self.session.execute_command(command).is_ok() {
            self.sync_editor_selection_bounds();
            let _ = self
                .session
                .traversal
                .goto(self.editor_selected_node, &self.session.graph);
            self.editor_status = Some("Removed node".to_string());
        }
    }

    fn editor_select_next(&mut self) {
        let max = self.session.graph.nodes.len().saturating_sub(1);
        self.editor_selected_node = (self.editor_selected_node + 1).min(max);
        self.editor_selected_block = 0;
        self.sync_editor_list_viewport();
        let _ = self
            .session
            .traversal
            .goto(self.editor_selected_node, &self.session.graph);
    }

    fn editor_select_prev(&mut self) {
        self.editor_selected_node = self.editor_selected_node.saturating_sub(1);
        self.editor_selected_block = 0;
        self.sync_editor_list_viewport();
        let _ = self
            .session
            .traversal
            .goto(self.editor_selected_node, &self.session.graph);
    }

    fn editor_select_next_block(&mut self) {
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
    }

    fn editor_select_prev_block(&mut self) {
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
    }

    fn move_selected_block(&mut self, forward: bool) {
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
        }
    }

    fn editor_page_down(&mut self) {
        let page = self.editor_list_visible_rows().max(1);
        let max = self.session.graph.nodes.len().saturating_sub(1);
        self.editor_selected_node = (self.editor_selected_node + page).min(max);
        self.editor_selected_block = 0;
        self.sync_editor_list_viewport();
        let _ = self
            .session
            .traversal
            .goto(self.editor_selected_node, &self.session.graph);
    }

    fn editor_page_up(&mut self) {
        let page = self.editor_list_visible_rows().max(1);
        self.editor_selected_node = self.editor_selected_node.saturating_sub(page);
        self.editor_selected_block = 0;
        self.sync_editor_list_viewport();
        let _ = self
            .session
            .traversal
            .goto(self.editor_selected_node, &self.session.graph);
    }

    fn editor_jump_top(&mut self) {
        self.editor_selected_node = 0;
        self.editor_selected_block = 0;
        self.sync_editor_list_viewport();
        let _ = self
            .session
            .traversal
            .goto(self.editor_selected_node, &self.session.graph);
    }

    fn editor_jump_bottom(&mut self) {
        self.editor_selected_node = self.session.graph.nodes.len().saturating_sub(1);
        self.editor_selected_block = 0;
        self.sync_editor_list_viewport();
        let _ = self
            .session
            .traversal
            .goto(self.editor_selected_node, &self.session.graph);
    }

    fn start_editor_node_search(&mut self) {
        self.editor_index_jump_input = None;
        self.editor_search_input = Some(String::new());
        self.editor_focus = EditorPaneFocus::NodeList;
        self.editor_status = Some("Node search: type text, Enter to jump, Esc to cancel".into());
    }

    fn start_editor_index_jump(&mut self) {
        self.editor_search_input = None;
        self.editor_index_jump_input = Some(String::new());
        self.editor_focus = EditorPaneFocus::NodeList;
        self.editor_status =
            Some("Jump to index: type number, Enter to jump, Esc to cancel".into());
    }

    fn handle_editor_search_key(&mut self, code: KeyCode) -> bool {
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

    fn handle_editor_index_jump_key(&mut self, code: KeyCode) -> bool {
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
            self.editor_selected_node = idx;
            self.editor_selected_block = 0;
            self.sync_editor_list_viewport();
            let _ = self.session.traversal.goto(idx, &self.session.graph);
            self.editor_status = Some(format!("Node search matched #{}", idx + 1));
        } else {
            self.editor_status = Some(format!("No node id matches '{trimmed}'"));
        }
    }

    fn jump_editor_search_hit(&mut self, forward: bool) {
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
            self.editor_selected_node = idx;
            self.editor_selected_block = 0;
            self.sync_editor_list_viewport();
            let _ = self.session.traversal.goto(idx, &self.session.graph);
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
        self.editor_selected_node = idx;
        self.editor_selected_block = 0;
        self.sync_editor_list_viewport();
        let _ = self.session.traversal.goto(idx, &self.session.graph);
        self.editor_status = Some(format!("Jumped to node #{}", idx + 1));
    }

    fn selected_node_block_count(&self) -> usize {
        self.session
            .graph
            .nodes
            .get(self.editor_selected_node)
            .map_or(0, |node| node.content.len())
    }

    fn selected_block_with_index(&self) -> Option<(usize, &ContentBlock)> {
        let node = self.session.graph.nodes.get(self.editor_selected_node)?;
        let count = node.content.len();
        if count == 0 {
            return None;
        }

        let index = self.editor_selected_block.min(count - 1);
        node.content.get(index).map(|block| (index, block))
    }

    fn sync_editor_block_selection_bounds(&mut self) {
        let max = self.selected_node_block_count().saturating_sub(1);
        self.editor_selected_block = self.editor_selected_block.min(max);
    }

    fn sync_editor_selection_bounds(&mut self) {
        let max = self.session.graph.nodes.len().saturating_sub(1);
        self.editor_selected_node = self.editor_selected_node.min(max);
        self.editor_graph_selected_node = self.editor_graph_selected_node.min(max);
        self.sync_editor_block_selection_bounds();
        self.sync_editor_list_viewport();
        self.sync_editor_graph_viewport();
    }

    fn sync_editor_list_viewport(&mut self) {
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

    fn sync_editor_graph_viewport(&mut self) {
        let total = self.session.graph.nodes.len();
        if total == 0 {
            self.editor_graph_scroll_offset = 0;
            return;
        }

        let root = Rect::new(0, 0, self.terminal_size.0, self.terminal_size.1);
        let sections = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
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
                .constraints([Constraint::Min(1), Constraint::Length(3)])
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
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(root);
        let body = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(sections[0]);

        body[0].height.saturating_sub(2) as usize
    }

    fn editor_graph_visible_rows(&self) -> usize {
        let root = Rect::new(0, 0, self.terminal_size.0, self.terminal_size.1);
        let sections = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(root);

        graph_overlay_page_span(
            sections[0],
            &self.session,
            self.editor_graph_selected_node,
            self.editor_graph_scroll_offset,
        )
    }

    fn toggle_editor_graph_view(&mut self) {
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

    fn handle_graph_overlay_key(&mut self, code: KeyCode) -> bool {
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
        self.editor_selected_node = idx;
        self.editor_selected_block = 0;
        self.sync_editor_list_viewport();
        let _ = self.session.traversal.goto(idx, &self.session.graph);
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

    fn start_selected_block_edit(&mut self) {
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

    fn start_inline_edit(&mut self, target: EditorInlineTarget, seed: String) {
        let idx = self.editor_selected_node;
        if self.session.graph.nodes.get(idx).is_none() {
            return;
        }

        self.editor_text_input = Some(seed);
        self.editor_inline_target = Some(target);
        self.editor_focus = EditorPaneFocus::NodeDetail;
    }

    fn handle_inline_edit_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
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

                let updated = match existing {
                    ContentBlock::Heading { level, .. } => ContentBlock::Heading { level, text },
                    ContentBlock::Text { .. } => ContentBlock::Text { body: text },
                    ContentBlock::Code {
                        language,
                        highlight_lines,
                        show_line_numbers,
                        ..
                    } => ContentBlock::Code {
                        language,
                        source: text,
                        highlight_lines,
                        show_line_numbers,
                    },
                    ContentBlock::List { ordered, mut items } => {
                        if let Some(first) = items.first_mut() {
                            first.text = text;
                        } else {
                            items.push(ListItem {
                                text,
                                children: Vec::new(),
                            });
                        }
                        ContentBlock::List { ordered, items }
                    }
                    ContentBlock::Image { alt, caption, .. } => ContentBlock::Image {
                        src: text,
                        alt,
                        caption,
                    },
                    ContentBlock::Divider => ContentBlock::Divider,
                    ContentBlock::Container { children, .. } => {
                        let layout = if text.trim().is_empty() {
                            None
                        } else {
                            Some(text)
                        };
                        ContentBlock::Container { layout, children }
                    }
                    ContentBlock::Extension {
                        fallback, payload, ..
                    } => ContentBlock::Extension {
                        extension_type: text,
                        fallback,
                        payload,
                    },
                };

                let command = Command::UpdateBlock {
                    node_id,
                    block_index,
                    block: updated,
                };
                if self.session.execute_command(command).is_ok() {
                    self.editor_status = Some(format!("Updated block #{}", block_index + 1));
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

    fn save_editor_graph(&mut self) {
        if self.save_graph_to_target() {
            self.pending_exit_action = None;
        }
    }

    fn save_graph_to_target(&mut self) -> bool {
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

    fn cycle_layout(&mut self, forward: bool) {
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

    fn cycle_transition(&mut self, forward: bool) {
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

    fn open_layout_picker(&mut self) {
        let selected = self
            .editor_last_layout_picker
            .min(layout_variants().len().saturating_sub(1));

        self.editor_picker = Some(EditorPickerOverlay::Layout { selected });
        self.editor_status = Some("Layout picker: arrows or 1-9/0 + Enter".to_string());
    }

    fn open_transition_picker(&mut self) {
        let selected = self
            .editor_last_transition_picker
            .min(transition_variants().len().saturating_sub(1));

        self.editor_picker = Some(EditorPickerOverlay::Transition { selected });
        self.editor_status = Some("Transition picker: arrows or 1-9/0 + Enter".to_string());
    }

    fn open_block_type_picker(&mut self) {
        let selected = self
            .editor_last_block_picker
            .min(block_type_variants().len().saturating_sub(1));

        self.editor_picker = Some(EditorPickerOverlay::BlockType { selected });
        self.editor_status = Some("Block picker: arrows or 1-9/0 + Enter".to_string());
    }

    fn handle_picker_key(&mut self, code: KeyCode) -> bool {
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

    fn adjust_picker_selection(&mut self, max_index: usize, forward: bool) {
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

    fn set_picker_selection(&mut self, selected: usize) {
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

    fn apply_picker_selection(&mut self) {
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

    fn request_exit_action(&mut self, action: PendingExitAction) {
        if self.session.dirty {
            self.pending_exit_action = Some(action);
            if self.mode == AppMode::Editing {
                self.editor_status =
                    Some("Unsaved changes: y=yes n=no s=save-first Esc=cancel".to_string());
            }
            self.set_flash(
                "Unsaved changes: y=yes n=no s=save-first Esc=cancel",
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
            KeyCode::Char('y') | KeyCode::Char('Y') => {
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
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
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

impl App {
    fn load_editor_preferences(&mut self) {
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

    fn persist_editor_preferences(&self) {
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

    fn handle_editor_mouse_click(&mut self, column: u16, row: u16) {
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
                    self.editor_selected_node = index;
                    self.editor_selected_block = 0;
                    self.sync_editor_list_viewport();
                    let _ = self.session.traversal.goto(index, &self.session.graph);
                }
            }
            return;
        }

        if point_in_rect(column, row, detail_area) {
            self.editor_focus = EditorPaneFocus::NodeDetail;
            self.persist_editor_preferences();
        }
    }

    fn handle_mouse_drag(&mut self, column: u16, row: u16) {
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

    fn handle_editor_mouse_scroll(&mut self, direction: MouseScrollDirection) {
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

    fn handle_picker_mouse_click(
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

    fn handle_graph_overlay_mouse_click(
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
            self.editor_selected_node = idx;
            self.editor_selected_block = 0;
            self.sync_editor_list_viewport();
            let _ = self.session.traversal.goto(idx, &self.session.graph);
            self.editor_graph_overlay = false;
            self.editor_status = Some(format!("Graph jump: node #{}", idx + 1));
        }

        true
    }
}

fn search_tokens(query: &str) -> Vec<String> {
    let tokens = query
        .split_whitespace()
        .map(|token| token.to_ascii_lowercase())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    if tokens.is_empty() {
        vec![query.to_ascii_lowercase()]
    } else {
        tokens
    }
}

fn score_node_id_match(candidate: &str, tokens: &[String]) -> Option<usize> {
    let mut total_score = 0usize;
    for token in tokens {
        let component = if candidate == token {
            0
        } else if candidate.starts_with(token) {
            1
        } else if candidate.contains(token) {
            2
        } else if is_subsequence(candidate, token) {
            3
        } else {
            return None;
        };

        total_score += component;
    }

    Some(total_score)
}

fn is_subsequence(candidate: &str, needle: &str) -> bool {
    let mut needle_chars = needle.chars();
    let mut current = needle_chars.next();

    for ch in candidate.chars() {
        if Some(ch) == current {
            current = needle_chars.next();
            if current.is_none() {
                return true;
            }
        }
    }

    false
}

fn next_search_hit_from(
    session: &PresentationSession,
    tokens: &[String],
    current: usize,
) -> Option<usize> {
    let total = session.graph.nodes.len();
    if total == 0 {
        return None;
    }

    for step in 1..=total {
        let idx = (current + step) % total;
        let id = session.graph.nodes[idx].id.as_deref().unwrap_or("");
        if score_node_id_match(&id.to_ascii_lowercase(), tokens).is_some() {
            return Some(idx);
        }
    }

    None
}

fn prev_search_hit_from(
    session: &PresentationSession,
    tokens: &[String],
    current: usize,
) -> Option<usize> {
    let total = session.graph.nodes.len();
    if total == 0 {
        return None;
    }

    for step in 1..=total {
        let idx = (current + total - (step % total)) % total;
        let id = session.graph.nodes[idx].id.as_deref().unwrap_or("");
        if score_node_id_match(&id.to_ascii_lowercase(), tokens).is_some() {
            return Some(idx);
        }
    }

    None
}

fn layout_variants() -> &'static [Layout] {
    &[
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
    ]
}

fn transition_variants() -> &'static [Transition] {
    &[
        Transition::None,
        Transition::Fade,
        Transition::SlideLeft,
        Transition::SlideRight,
        Transition::Wipe,
        Transition::Dissolve,
        Transition::Matrix,
        Transition::Typewriter,
    ]
}

fn block_type_variants() -> &'static [(&'static str, ContentBlock)] {
    static VARIANTS: std::sync::LazyLock<Vec<(&'static str, ContentBlock)>> =
        std::sync::LazyLock::new(|| {
            vec![
                (
                    "Heading",
                    ContentBlock::Heading {
                        level: 1,
                        text: "New heading".to_string(),
                    },
                ),
                (
                    "Text",
                    ContentBlock::Text {
                        body: "New text block".to_string(),
                    },
                ),
                (
                    "Code",
                    ContentBlock::Code {
                        language: Some("text".to_string()),
                        source: String::new(),
                        highlight_lines: vec![],
                        show_line_numbers: false,
                    },
                ),
                (
                    "List",
                    ContentBlock::List {
                        ordered: false,
                        items: vec![ListItem {
                            text: "New list item".to_string(),
                            children: vec![],
                        }],
                    },
                ),
                (
                    "Image",
                    ContentBlock::Image {
                        src: String::new(),
                        alt: String::new(),
                        caption: None,
                    },
                ),
                ("Divider", ContentBlock::Divider),
                (
                    "Container",
                    ContentBlock::Container {
                        layout: None,
                        children: vec![ContentBlock::Text {
                            body: "Container child".to_string(),
                        }],
                    },
                ),
                (
                    "Extension",
                    ContentBlock::Extension {
                        extension_type: "custom.unknown".to_string(),
                        fallback: Some(Box::new(ContentBlock::Text {
                            body: "Extension fallback".to_string(),
                        })),
                        payload: serde_json::Value::Object(serde_json::Map::new()),
                    },
                ),
            ]
        });
    VARIANTS.as_slice()
}

fn bump_index(current: usize, max_index: usize, forward: bool) -> usize {
    if forward {
        (current + 1).min(max_index)
    } else {
        current.saturating_sub(1)
    }
}

fn digit_to_index(code: KeyCode) -> Option<usize> {
    match code {
        KeyCode::Char(ch @ '1'..='9') => ch.to_digit(10).map(|d| d as usize - 1),
        KeyCode::Char('0') => Some(9),
        _ => None,
    }
}

fn picker_row_span(overlay: EditorPickerOverlay) -> usize {
    match overlay {
        EditorPickerOverlay::BlockType { .. } => 2,
        EditorPickerOverlay::Layout { .. } | EditorPickerOverlay::Transition { .. } => 1,
    }
}

fn is_editor_actionable_warning(message: &str) -> bool {
    [
        "heading text is empty",
        "text body is empty",
        "code source is empty",
        "list has no items",
        "list item #1 is empty",
        "image src is empty",
        "extension type is empty",
    ]
    .iter()
    .any(|pattern| message.contains(pattern))
}

fn point_in_rect(column: u16, row: u16, rect: Rect) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

fn centered_popup(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let vertical = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_pct) / 2),
            Constraint::Percentage(height_pct),
            Constraint::Percentage((100 - height_pct) / 2),
        ])
        .split(area);

    let horizontal = RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_pct) / 2),
            Constraint::Percentage(width_pct),
            Constraint::Percentage((100 - width_pct) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use fireside_core::model::branch::{BranchOption, BranchPoint};
    use fireside_core::model::graph::{Graph, GraphFile};
    use fireside_core::model::node::Node;
    use fireside_core::model::traversal::Traversal;
    use fireside_engine::PresentationSession;

    use super::{App, Theme};
    use crate::event::Action;

    fn graph_with_ids(ids: &[&str]) -> Graph {
        let file = GraphFile {
            title: None,
            fireside_version: None,
            author: None,
            date: None,
            description: None,
            version: None,
            tags: Vec::new(),
            theme: None,
            font: None,
            defaults: None,
            extensions: Vec::new(),
            nodes: ids
                .iter()
                .map(|id| Node {
                    id: Some((*id).to_string()),
                    title: None,
                    tags: Vec::new(),
                    duration: None,
                    layout: None,
                    transition: None,
                    speaker_notes: None,
                    traversal: None,
                    content: Vec::new(),
                })
                .collect(),
        };

        Graph::from_file(file).expect("graph should be valid")
    }

    fn branch_graph() -> Graph {
        let mut start = Node {
            id: Some("start".to_string()),
            title: None,
            tags: Vec::new(),
            duration: None,
            layout: None,
            transition: None,
            speaker_notes: None,
            traversal: None,
            content: Vec::new(),
        };
        start.traversal = Some(Traversal {
            next: None,
            after: None,
            branch_point: Some(BranchPoint {
                id: Some("branch-0".to_string()),
                prompt: Some("Choose path".to_string()),
                options: vec![
                    BranchOption {
                        label: "Path A".to_string(),
                        key: '1',
                        target: "path-a".to_string(),
                    },
                    BranchOption {
                        label: "Path B".to_string(),
                        key: '2',
                        target: "path-b".to_string(),
                    },
                ],
            }),
        });

        let path_a = Node {
            id: Some("path-a".to_string()),
            title: None,
            tags: Vec::new(),
            duration: None,
            layout: None,
            transition: None,
            speaker_notes: None,
            traversal: None,
            content: Vec::new(),
        };
        let path_b = Node {
            id: Some("path-b".to_string()),
            title: None,
            tags: Vec::new(),
            duration: None,
            layout: None,
            transition: None,
            speaker_notes: None,
            traversal: None,
            content: Vec::new(),
        };

        Graph::from_file(GraphFile {
            title: None,
            fireside_version: None,
            author: None,
            date: None,
            description: None,
            version: None,
            tags: Vec::new(),
            theme: None,
            font: None,
            defaults: None,
            extensions: Vec::new(),
            nodes: vec![start, path_a, path_b],
        })
        .expect("branch graph should be valid")
    }

    #[test]
    fn reload_graph_preserves_current_node_by_id() {
        let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 1);
        let mut app = App::new(session, Theme::default());

        app.reload_graph(graph_with_ids(&["x", "b", "y"]));

        assert_eq!(app.session.current_node_index(), 1);
        assert_eq!(app.session.current_node().id.as_deref(), Some("b"));
    }

    #[test]
    fn reload_graph_falls_back_to_clamped_index_when_id_missing() {
        let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 2);
        let mut app = App::new(session, Theme::default());

        app.reload_graph(graph_with_ids(&["a", "b"]));

        assert_eq!(app.session.current_node_index(), 1);
        assert_eq!(app.session.current_node().id.as_deref(), Some("b"));
    }

    #[test]
    fn hot_reload_is_only_enabled_in_presenter_mode() {
        let session = PresentationSession::new(graph_with_ids(&["a", "b"]), 0);
        let mut app = App::new(session, Theme::default());

        assert!(app.can_hot_reload());
        app.enter_edit_mode();
        assert!(!app.can_hot_reload());
    }

    #[test]
    fn presenter_branch_overlay_up_down_enter_chooses_focused_option() {
        let session = PresentationSession::new(branch_graph(), 0);
        let mut app = App::new(session, Theme::default());

        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)));
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));

        assert_eq!(app.session.current_node().id.as_deref(), Some("path-a"));
        assert_eq!(app.branch_focused_option, 0);
    }

    #[test]
    fn graph_overlay_toggle_tracks_editor_selection() {
        let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 1);
        let mut app = App::new(session, Theme::default());

        app.enter_edit_mode();
        app.update(Action::EditorToggleGraphView);

        assert!(app.editor_graph_overlay);
        assert_eq!(app.editor_graph_selected_node, 1);

        app.update(Action::EditorToggleGraphView);

        assert!(!app.editor_graph_overlay);
    }

    #[test]
    fn graph_overlay_keyboard_enter_jumps_to_selected_node() {
        let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
        let mut app = App::new(session, Theme::default());

        app.enter_edit_mode();
        app.update(Action::EditorToggleGraphView);
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));

        assert!(!app.editor_graph_overlay);
        assert_eq!(app.editor_selected_node, 1);
        assert_eq!(app.session.current_node_index(), 1);
    }

    #[test]
    fn graph_overlay_paged_navigation_advances_selection() {
        let ids = (1..=24)
            .map(|index| format!("node-{index}"))
            .collect::<Vec<_>>();
        let id_refs = ids.iter().map(String::as_str).collect::<Vec<_>>();

        let session = PresentationSession::new(graph_with_ids(&id_refs), 0);
        let mut app = App::new(session, Theme::default());
        app.terminal_size = (100, 28);

        app.enter_edit_mode();
        app.update(Action::EditorToggleGraphView);
        let page = app.editor_graph_visible_rows().max(1);

        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::PageDown,
            KeyModifiers::NONE,
        )));
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));

        assert_eq!(app.editor_selected_node, page.min(23));
    }

    #[test]
    fn graph_overlay_home_end_navigation_hits_bounds() {
        let ids = (1..=12)
            .map(|index| format!("node-{index}"))
            .collect::<Vec<_>>();
        let id_refs = ids.iter().map(String::as_str).collect::<Vec<_>>();

        let session = PresentationSession::new(graph_with_ids(&id_refs), 5);
        let mut app = App::new(session, Theme::default());
        app.enter_edit_mode();
        app.update(Action::EditorToggleGraphView);

        app.handle_event(Event::Key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)));
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        assert_eq!(app.editor_selected_node, 11);

        app.update(Action::EditorToggleGraphView);
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)));
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        assert_eq!(app.editor_selected_node, 0);
    }

    #[test]
    fn graph_overlay_present_shortcut_switches_to_presenting_mode() {
        let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
        let mut app = App::new(session, Theme::default());

        app.enter_edit_mode();
        app.update(Action::EditorToggleGraphView);
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('p'),
            KeyModifiers::NONE,
        )));

        assert_eq!(app.mode, super::AppMode::Presenting);
        assert!(!app.editor_graph_overlay);
        assert_eq!(app.session.current_node_index(), 1);
        assert_eq!(app.editor_selected_node, 1);
    }

    #[test]
    fn presenter_enter_edit_mode_sets_breadcrumb_status() {
        let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 2);
        let mut app = App::new(session, Theme::default());

        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('e'),
            KeyModifiers::NONE,
        )));

        assert_eq!(app.mode, super::AppMode::Editing);
        assert_eq!(app.editor_selected_node, 2);
        let status = app.editor_status.as_deref().unwrap_or_default();
        assert!(status.contains("Presenter"));
        assert!(status.contains("node #3"));
    }

    #[test]
    fn help_overlay_scroll_keys_adjust_offset() {
        let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
        let mut app = App::new(session, Theme::default());
        app.terminal_size = (80, 24);

        app.update(Action::ToggleHelp);
        let start = app.help_scroll_offset;

        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::PageDown,
            KeyModifiers::NONE,
        )));
        assert!(app.help_scroll_offset >= start);

        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
        assert!(app.help_scroll_offset >= start);

        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)));
        assert_eq!(app.help_scroll_offset, 0);
    }

    #[test]
    fn help_overlay_section_jump_key_moves_scroll() {
        let session = PresentationSession::new(graph_with_ids(&["a", "b", "c"]), 0);
        let mut app = App::new(session, Theme::default());
        app.terminal_size = (80, 24);

        app.update(Action::ToggleHelp);
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('4'),
            KeyModifiers::NONE,
        )));

        assert!(app.help_scroll_offset > 0);
    }
}
