//! The authoring editor's application state machine (spec 013,
//! `fireside edit`).
//!
//! TEA-style, matching the presenter's own contract (constitution IV,
//! generalized): `EditorApp::update` is the sole place `EditorApp` state
//! mutates. `fireside-tui` performs no file I/O anywhere in this
//! module — saving, drafts, and deck creation are `fireside-cli`'s job
//! (ADR-014); [`run`] takes an injected write-back sink and (optional)
//! text-art generator, the same "caller owns all I/O" contract
//! `fireside-tui::present_authoring` already gives the presenter.

pub(crate) mod forms;
pub(crate) mod hit;

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::execute;
use crossterm::tty::IsTty;
use fireside_engine::authoring::{self, AuthoringError, BlockPath, Op};
use fireside_engine::validate;
use ratatui::layout::Rect;

use fireside_core::{ContainerLayout, ContentBlock, Graph};

use crate::app::App as PresenterApp;
use crate::app::FlashKind;
use crate::error::TuiError;
use crate::{WriteBackError, render};

use forms::{CodeFocus, EditableField, FormState, PictureFocus, TextArtFocus};
use hit::{PickerRow, PickerTarget, PromptKind, SlideAction};

/// What's selected in the studio, if anything.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum Selection {
    #[default]
    None,
    Slide(String),
    Block(String, BlockPath),
}

/// A block or slide drag in progress (spec 013, `data-model.md`'s
/// `EditorApp::drag`). `Lifting` covers a press on a block that hasn't
/// moved yet (indistinguishable from a plain click until it does);
/// `Over` is an active drag currently resolved over a drop slot. Slide
/// drag (US3, T050) reuses this same state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum DragState {
    #[default]
    Idle,
    Lifting {
        node: String,
        path: BlockPath,
    },
    Over {
        node: String,
        path: BlockPath,
        to: usize,
    },
    /// A press on an outline row that hasn't moved yet (spec 013 US3,
    /// T050) — indistinguishable from a plain click until it does, exactly
    /// like `Lifting` for blocks.
    OutlineLifting {
        id: String,
    },
    /// An outline slide drag currently resolved over a drop candidate;
    /// `before: None` means "end of the run."
    OutlineOver {
        id: String,
        before: Option<String>,
    },
}

/// One undo/redo checkpoint: a full graph clone plus the selection at that
/// point, so undo restores view context too (design brief, "Never lose
/// work").
#[derive(Debug, Clone)]
pub(crate) struct HistorySnapshot {
    pub(crate) graph: Graph,
    pub(crate) selection: Selection,
}

/// How long a flash message stays on screen — same duration the presenter
/// itself uses (`app::FLASH_DURATION`), not shared as a symbol since the
/// two `App`/`EditorApp` types are otherwise independent.
const FLASH_DURATION: Duration = Duration::from_millis(3000);

/// A transient feedback message shown on the hint line — the editor's
/// equivalent of the presenter's footer flash (design brief principle 4:
/// every action produces immediate visible feedback).
#[derive(Debug, Clone)]
pub(crate) struct Flash {
    pub(crate) text: String,
    pub(crate) kind: FlashKind,
    /// A clickable follow-up action, if this flash offers one (T050's
    /// cross-branch-boundary refusal toast: "a way to perform the intended
    /// change correctly").
    pub(crate) action: Option<hit::FlashAction>,
    expires: Instant,
}

/// A message into the editor's state machine.
#[derive(Debug)]
pub(crate) enum Msg {
    /// A terminal event (key press, resize, mouse).
    Terminal(Event),
    /// The write-back sink's response to a `[ Save ]`/Ctrl+S.
    SaveResult(Result<(), String>),
    /// The CLI-injected art generator's response to "Generate from a
    /// phrase…" (T032).
    ArtGenerated(Result<String, String>),
}

/// What the open-time draft-vs-saved-file prompt shows (spec 013 US4,
/// FR-020, `contracts/cli-edit-command.md`'s "Behavior" section):
/// `fireside-cli` reads and compares the draft sidecar itself (this crate
/// never touches the filesystem or a clock) and hands over the recovered
/// graph plus two already-formatted, plain-language timestamps.
#[derive(Debug, Clone)]
pub struct DraftPrompt {
    pub draft: Graph,
    pub draft_touched: String,
    pub saved_touched: String,
}

/// All authoring-editor state (spec 013, `data-model.md`'s `EditorApp`
/// section).
#[derive(Debug)]
pub(crate) struct EditorApp {
    working_graph: Graph,
    saved_graph: Graph,
    selection: Selection,
    drag: DragState,
    open_form: Option<FormState>,
    history: Vec<HistorySnapshot>,
    redo: Vec<HistorySnapshot>,
    terminal_size: (u16, u16),
    status: Vec<fireside_engine::Diagnostic>,
    scroll: u16,
    /// The outline pane's own scroll offset (spec 013 E4, T068) — separate
    /// from the canvas's `scroll` so scrolling one pane never disturbs the
    /// other; clamped at read time by `hit::outline_scroll_offset`, the
    /// same pattern `scroll`/`canvas_layout` already use.
    outline_scroll: u16,
    hover: Option<hit::Target>,
    dirty_since_draft: bool,
    #[allow(dead_code)] // read by tests; a "draft saved Xs ago" indicator is future polish
    last_draft_write: Instant,
    showing_help: bool,
    /// The quit-with-unsaved-changes prompt (spec 013 US4, FR-019), open
    /// when `q` was pressed while [`Self::dirty`] was true.
    quit_prompt: bool,
    /// Set by the quit-prompt's `[ Save ]` chip: quit once the pending
    /// save this triggers comes back successful, never on a failed save.
    quit_after_save: bool,
    /// The open-time draft-vs-saved-file prompt (spec 013 US4, FR-020),
    /// gating the whole studio until resolved.
    draft_choice: Option<DraftPrompt>,
    present_requested: bool,
    pending_save: Option<Graph>,
    pending_art_request: Option<String>,
    flash: Option<Flash>,
    /// When this session opened — the first-run hint tour (spec 013 E4)
    /// rotates its three messages off this clock, read purely at render
    /// time (same pattern as [`Self::flash`]'s `Instant::now()` filter).
    opened_at: Instant,
    /// Set on the first successful save: the hint tour is dismissed
    /// forever after that (design brief E4) — an author who has already
    /// saved once doesn't need to keep being taught the basics.
    hint_tour_dismissed: bool,
    quit: bool,
}

/// Every distinct positive reveal step used by `content`, excluding the
/// block at `path` itself (spec 013 US3, T053's reveal-cycle ceiling) —
/// recurses into `Container` children like `Node::reveal_levels()` does,
/// so a step held only by a nested block still counts.
fn other_reveal_levels(content: &[ContentBlock], path: &[usize]) -> Vec<u32> {
    fn walk(blocks: &[ContentBlock], path: &[usize], out: &mut Vec<u32>) {
        for (i, block) in blocks.iter().enumerate() {
            let is_excluded = path.len() == 1 && path[0] == i;
            if !is_excluded
                && let Some(v) = block.reveal()
                && v > 0
            {
                out.push(v);
            }
            if let ContentBlock::Container { children, .. } = block {
                let child_path: &[usize] = if path.first() == Some(&i) {
                    &path[1..]
                } else {
                    &[]
                };
                walk(children, child_path, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(content, path, &mut out);
    out.sort_unstable();
    out.dedup();
    out
}

/// The node whose branch answer targets `id`, if any (spec 013 US3, T050's
/// cross-branch-boundary refusal toast's "take me there" link).
fn find_branch_predecessor(graph: &Graph, id: &str) -> Option<String> {
    graph
        .nodes
        .iter()
        .find(|n| {
            n.branch_point()
                .is_some_and(|bp| bp.options.iter().any(|o| o.target == id))
        })
        .map(|n| n.id.clone())
}

impl EditorApp {
    /// Opens a fresh editor session over `graph`, validating it up front
    /// for the status banner (spec FR-026) — a deck with diagnostics is
    /// never refused, only reported.
    #[must_use]
    pub(crate) fn new(graph: Graph) -> Self {
        let status = validate(&graph);
        let saved_graph = graph.clone();
        Self {
            working_graph: graph,
            saved_graph,
            selection: Selection::None,
            drag: DragState::Idle,
            open_form: None,
            history: Vec::new(),
            redo: Vec::new(),
            terminal_size: (80, 24),
            status,
            scroll: 0,
            outline_scroll: 0,
            hover: None,
            dirty_since_draft: false,
            last_draft_write: Instant::now(),
            showing_help: false,
            quit_prompt: false,
            quit_after_save: false,
            draft_choice: None,
            present_requested: false,
            pending_save: None,
            pending_art_request: None,
            flash: None,
            opened_at: Instant::now(),
            hint_tour_dismissed: false,
            quit: false,
        }
    }

    /// Opens a fresh editor session over `graph` (the deck file's own
    /// content), gated behind the open-time draft-vs-saved-file prompt
    /// (spec 013 US4, FR-020) — the studio itself doesn't draw until
    /// [`Self::resolve_draft_choice`] resolves it.
    #[must_use]
    pub(crate) fn new_with_draft(graph: Graph, prompt: DraftPrompt) -> Self {
        let mut app = Self::new(graph);
        app.draft_choice = Some(prompt);
        app
    }

    #[must_use]
    pub(crate) fn working_graph(&self) -> &Graph {
        &self.working_graph
    }

    #[must_use]
    pub(crate) fn selection(&self) -> &Selection {
        &self.selection
    }

    #[must_use]
    pub(crate) fn scroll(&self) -> u16 {
        self.scroll
    }

    #[must_use]
    pub(crate) fn outline_scroll(&self) -> u16 {
        self.outline_scroll
    }

    #[must_use]
    pub(crate) fn status(&self) -> &[fireside_engine::Diagnostic] {
        &self.status
    }

    #[must_use]
    pub(crate) fn showing_help(&self) -> bool {
        self.showing_help
    }

    #[must_use]
    pub(crate) fn hover(&self) -> Option<&hit::Target> {
        self.hover.as_ref()
    }

    #[must_use]
    pub(crate) fn open_form(&self) -> Option<&FormState> {
        self.open_form.as_ref()
    }

    /// The active flash message, if it has not expired.
    #[must_use]
    pub(crate) fn flash(&self) -> Option<&Flash> {
        self.flash.as_ref().filter(|f| f.expires > Instant::now())
    }

    /// When this session opened — `render::editor` reads this to pick the
    /// first-run hint tour's current message (spec 013 E4).
    #[must_use]
    pub(crate) fn opened_at(&self) -> Instant {
        self.opened_at
    }

    /// Whether the first-run hint tour has been dismissed by a successful
    /// save (spec 013 E4) — once true, the hint line settles on the tour's
    /// first, steadiest message instead of continuing to rotate.
    #[must_use]
    pub(crate) fn hint_tour_dismissed(&self) -> bool {
        self.hint_tour_dismissed
    }

    /// Whether `working_graph` has unsaved changes — the toolbar's `●` dot
    /// (spec FR-018).
    #[must_use]
    pub(crate) fn dirty(&self) -> bool {
        self.working_graph != self.saved_graph
    }

    #[must_use]
    #[allow(dead_code)] // read by tests; a visible undo-depth indicator is future polish
    pub(crate) fn history_len(&self) -> usize {
        self.history.len()
    }

    #[must_use]
    #[allow(dead_code)] // read by tests
    pub(crate) fn last_draft_write(&self) -> Instant {
        self.last_draft_write
    }

    /// Whether the quit-with-unsaved-changes prompt is open (spec 013 US4,
    /// FR-019) — read by `render::editor` to draw its overlay.
    #[must_use]
    pub(crate) fn quit_prompt(&self) -> bool {
        self.quit_prompt
    }

    /// The open-time draft-vs-saved-file prompt, if unresolved (spec 013
    /// US4, FR-020) — read by `render::editor` to draw its takeover screen
    /// in place of the studio.
    #[must_use]
    pub(crate) fn draft_choice(&self) -> Option<&DraftPrompt> {
        self.draft_choice.as_ref()
    }

    /// The block (or slide, once US3's outline drag lands) drag in
    /// progress, if any — read by `render::editor::canvas` to draw the
    /// dimmed ghost and drop indicator.
    #[must_use]
    pub(crate) fn drag(&self) -> &DragState {
        &self.drag
    }

    #[must_use]
    pub(crate) fn should_quit(&self) -> bool {
        self.quit
    }

    fn set_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
    }

    fn set_flash(&mut self, text: impl Into<String>, kind: FlashKind) {
        self.flash = Some(Flash {
            text: text.into(),
            kind,
            action: None,
            expires: Instant::now() + FLASH_DURATION,
        });
    }

    fn set_flash_with_action(
        &mut self,
        text: impl Into<String>,
        kind: FlashKind,
        action: hit::FlashAction,
    ) {
        self.flash = Some(Flash {
            text: text.into(),
            kind,
            action: Some(action),
            expires: Instant::now() + FLASH_DURATION,
        });
    }

    /// The node id the canvas is currently showing — what `[ ▶ Present ]`
    /// starts from.
    fn selected_node_id(&self) -> Option<String> {
        hit::selected_node(self).map(|n| n.id.clone())
    }

    /// Consumes a pending present request, if one is set, returning the
    /// slide to start from.
    fn take_present_request(&mut self) -> Option<String> {
        if std::mem::take(&mut self.present_requested) {
            self.selected_node_id()
        } else {
            None
        }
    }

    /// Consumes a pending save request, if one is set — the event loop
    /// hands it to the injected write-back sink and never leaves it
    /// unconsumed.
    fn take_pending_save(&mut self) -> Option<Graph> {
        self.pending_save.take()
    }

    /// Consumes a pending text-art generation request, if one is set.
    fn take_pending_art_request(&mut self) -> Option<String> {
        self.pending_art_request.take()
    }

    /// Consumes a pending draft-write request: `Some` whenever
    /// `working_graph` has changed (via [`Self::apply_op`]/
    /// [`Self::apply_direct`]) since the last draft write — checked once
    /// per event-loop tick, mirroring [`Self::take_pending_save`]'s
    /// pull-based contract (spec 013 US4, FR-020: "periodically... and on
    /// every structural op").
    fn take_pending_draft(&mut self) -> Option<Graph> {
        if std::mem::take(&mut self.dirty_since_draft) {
            self.last_draft_write = Instant::now();
            Some(self.working_graph.clone())
        } else {
            None
        }
    }

    /// `[`/`]`: selects the previous/next slide in outline order, wrapping
    /// — the keyboard-only counterpart to clicking an outline row (spec
    /// 013 US3: outline drag has no keyboard equivalent, per the design
    /// brief's "reorder slides" scope, but plain *selection* — the
    /// prerequisite for every other slide-level action — must still work
    /// without a mouse, per ADR-017's keyboard-complete posture). A no-op
    /// on an empty deck.
    fn select_adjacent_slide(&mut self, backward: bool) {
        let rows = authoring::outline_order(&self.working_graph);
        if rows.is_empty() {
            return;
        }
        let current = match &self.selection {
            Selection::Slide(id) | Selection::Block(id, _) => {
                rows.iter().position(|r| &r.node_id == id)
            }
            Selection::None => None,
        };
        let next = match (current, backward) {
            (None, false) => 0,
            (None, true) => rows.len() - 1,
            (Some(i), false) => (i + 1) % rows.len(),
            (Some(i), true) => (i + rows.len() - 1) % rows.len(),
        };
        self.selection = Selection::Slide(rows[next].node_id.clone());
        self.scroll = 0;
    }

    /// Tab/Shift+Tab: selects the next/previous top-level block on the
    /// canvas's current slide, wrapping — the keyboard-only counterpart to
    /// clicking a block (spec 013 US1 acceptance scenario 5: the same task
    /// must succeed mouse-only or keyboard-only). A no-op on an empty
    /// slide.
    fn select_adjacent_block(&mut self, backward: bool) {
        let Some(node) = hit::selected_node(self) else {
            return;
        };
        let count = node.content.len();
        if count == 0 {
            return;
        }
        let node_id = node.id.clone();
        let current = match &self.selection {
            Selection::Block(id, path) if *id == node_id && path.len() == 1 => Some(path[0]),
            _ => None,
        };
        let next = match (current, backward) {
            (None, false) => 0,
            (None, true) => count - 1,
            (Some(i), false) => (i + 1) % count,
            (Some(i), true) => (i + count - 1) % count,
        };
        self.selection = Selection::Block(node_id, vec![next]);
    }

    /// A click on the status banner (spec 013 E4, `contracts`'s
    /// `Target::StatusBanner`): jumps to the slide behind the most serious
    /// diagnostic that names one. `status` is already sorted errors-first
    /// (`validation::validate`), so the first diagnostic with a `node` is
    /// the right one to select. A no-op when every diagnostic is deck-wide
    /// (no single offending slide to jump to).
    fn jump_to_diagnostic(&mut self) {
        if let Some(id) = self.status.iter().find_map(|d| d.node.clone()) {
            self.selection = Selection::Slide(id);
            self.scroll = 0;
        }
    }

    // ─── Forms (spec 013, US1) ──────────────────────────────────────────

    /// Opens the currently selected block's edit form, or flashes that a
    /// divider has nothing to edit.
    fn open_form_for_selection(&mut self) {
        let Selection::Block(node, path) = self.selection.clone() else {
            return;
        };
        self.open_form_at(&node, &path);
    }

    fn open_form_at(&mut self, node: &str, path: &BlockPath) {
        let Some(node_ref) = self.working_graph.node(node) else {
            return;
        };
        let Some(block) = forms::block_at(&node_ref.content, path) else {
            return;
        };
        match forms::open(node, path.clone(), block) {
            Some(form) => self.open_form = Some(form),
            None => self.set_flash("Dividers have nothing to edit", FlashKind::Info),
        }
    }

    /// `[ Done ]`/Ctrl+S while a form is open: commits its staged content
    /// via `Op::EditBlock` (a no-op for `FormState::Container`, whose
    /// layout chip already commits immediately — T033), then closes it.
    fn commit_form(&mut self) {
        let Some(form) = &self.open_form else {
            return;
        };
        if matches!(form, FormState::Prompt { .. }) {
            self.commit_prompt();
            return;
        }
        if !form.can_commit() {
            self.set_flash(
                "That text art is too wide — shorten it or generate a new one",
                FlashKind::Error,
            );
            return;
        }
        let node = form.node().to_owned();
        let path = form.path().clone();
        if let Some(content) = form.build_content() {
            self.apply_op(Op::EditBlock {
                node,
                path,
                content,
            });
        }
        self.open_form = None;
    }

    /// Directly mutates `working_graph` outside `engine::authoring` (spec
    /// 013 US3, T054): deck-title rename and per-slide notes have no
    /// `Op` — they're metadata this feature's contract deliberately leaves
    /// out of the authoring-ops table — but still need undo, so this
    /// pushes history exactly like [`Self::apply_op`] does.
    fn apply_direct(&mut self, mutate: impl FnOnce(&mut Graph)) {
        self.push_history();
        mutate(&mut self.working_graph);
        self.redo.clear();
        self.dirty_since_draft = true;
    }

    /// `[ Done ]` on a direct-effect `Prompt` (`NewSlide`/`DeckTitle`/
    /// `Notes`) — `ChoicePrompt`/`NewAnswer` never reach here (their
    /// `[ Choose target → ]` chip routes to [`Self::begin_picker`]
    /// instead, per `FormState::prompt_commits_directly`).
    fn commit_prompt(&mut self) {
        let Some(FormState::Prompt { kind, fields, .. }) = self.open_form.clone() else {
            return;
        };
        match kind {
            PromptKind::NewSlide { after } => {
                let title = fields[0].text();
                if title.trim().is_empty() {
                    self.set_flash("Type a title first", FlashKind::Info);
                    return;
                }
                if self.apply_op(Op::AddSlide {
                    after: after.clone(),
                    title,
                }) && let Some(idx) = self.working_graph.nodes.iter().position(|n| n.id == after)
                    && let Some(new_node) = self.working_graph.nodes.get(idx + 1)
                {
                    self.selection = Selection::Slide(new_node.id.clone());
                }
            }
            PromptKind::DeckTitle => {
                let title = fields[0].text();
                self.apply_direct(|g| {
                    g.title = (!title.trim().is_empty()).then_some(title);
                });
            }
            PromptKind::Notes { node } => {
                let notes = fields[0].text();
                self.apply_direct(|g| {
                    if let Some(n) = g.nodes.iter_mut().find(|n| n.id == node) {
                        n.speaker_notes = (!notes.trim().is_empty()).then_some(notes);
                    }
                });
            }
            PromptKind::ChoicePrompt { .. } | PromptKind::NewAnswer { .. } => return,
        }
        self.open_form = None;
    }

    /// `[ Cancel ]`/Esc while a form is open: discards it with no op
    /// applied.
    fn cancel_form(&mut self) {
        self.open_form = None;
    }

    fn on_form_chip(&mut self, kind: hit::FormChipKind) {
        match kind {
            hit::FormChipKind::Done => self.commit_form(),
            hit::FormChipKind::Cancel => self.cancel_form(),
            hit::FormChipKind::ConvertToTextArt => self.convert_picture_to_text_art(),
            hit::FormChipKind::GenerateFromPhrase => self.request_art_generation(),
            hit::FormChipKind::CycleLayout => self.cycle_container_layout(),
            // Handled by `on_click` before it ever reaches here (needs the
            // `BlockKind` payload); kept so this match stays exhaustive.
            hit::FormChipKind::PaletteCard(kind) => self.add_block_from_palette(kind),
            hit::FormChipKind::ChooseTarget => self.begin_picker(),
            hit::FormChipKind::PickerRow(idx) => self.commit_picker_row(idx),
            hit::FormChipKind::PickerEnding => self.commit_picker_ending(),
            hit::FormChipKind::PickerNewSlide => self.commit_picker_new_slide(),
        }
    }

    // ─── Structure: slides, wiring, choices, reveal (spec 013, US3) ─────

    /// Every slide, by title — the picker's row list (spec 013, T051).
    fn picker_rows(&self) -> Vec<PickerRow> {
        self.working_graph
            .nodes
            .iter()
            .map(|n| PickerRow {
                id: n.id.clone(),
                title: n.title.clone().unwrap_or_else(|| n.id.clone()),
            })
            .collect()
    }

    /// `[ Choose target → ]` on a `ChoicePrompt`/`NewAnswer` prompt: reads
    /// the typed fields and hands off to `FormState::SlidePicker`.
    fn begin_picker(&mut self) {
        let Some(FormState::Prompt { kind, fields, .. }) = &self.open_form else {
            return;
        };
        let target = match kind.clone() {
            PromptKind::ChoicePrompt { node } => {
                let prompt = fields[0].text();
                let label = fields[1].text();
                if label.trim().is_empty() {
                    self.set_flash("Type the first answer's label", FlashKind::Info);
                    return;
                }
                PickerTarget::FirstAnswer {
                    node,
                    prompt: (!prompt.trim().is_empty()).then_some(prompt),
                    label,
                }
            }
            PromptKind::NewAnswer { node } => {
                let label = fields[0].text();
                let key = fields[1].text();
                if label.trim().is_empty() {
                    self.set_flash("Type the answer's label", FlashKind::Info);
                    return;
                }
                PickerTarget::NewAnswer {
                    node,
                    label,
                    key: (!key.trim().is_empty()).then_some(key),
                }
            }
            PromptKind::NewSlide { .. } | PromptKind::DeckTitle | PromptKind::Notes { .. } => {
                return;
            }
        };
        let rows = self.picker_rows();
        self.open_form = Some(FormState::SlidePicker { target, rows });
    }

    /// Applies `target`'s op with `chosen` as its slide, then closes the
    /// picker and re-selects the slide the wiring/choice belongs to.
    fn commit_picker_target(&mut self, target: PickerTarget, chosen: String) {
        let node = target.origin().to_owned();
        let applied = match target {
            PickerTarget::Next { node } => self.apply_op(Op::SetNext {
                id: node,
                target: chosen,
            }),
            PickerTarget::FirstAnswer {
                node,
                prompt,
                label,
            } => self.apply_op(Op::TurnIntoChoice {
                id: node,
                prompt,
                first_label: label,
                first_target: chosen,
            }),
            PickerTarget::NewAnswer { node, label, key } => self.apply_op(Op::AddAnswer {
                id: node,
                label,
                key,
                target: chosen,
            }),
            PickerTarget::RetargetAnswer { node, index } => self.apply_op(Op::RetargetAnswer {
                id: node,
                index,
                target: chosen,
            }),
        };
        if applied {
            self.selection = Selection::Slide(node);
        }
        self.open_form = None;
    }

    fn commit_picker_row(&mut self, idx: usize) {
        let Some(FormState::SlidePicker { target, rows }) = self.open_form.clone() else {
            return;
        };
        let Some(row) = rows.get(idx) else {
            return;
        };
        self.commit_picker_target(target, row.id.clone());
    }

    /// The picker's "nothing — an ending" row — `PickerTarget::Next` only
    /// (per `hit::form_chip_defs`, the only kind that ever shows it).
    fn commit_picker_ending(&mut self) {
        let Some(FormState::SlidePicker {
            target: PickerTarget::Next { node },
            ..
        }) = self.open_form.clone()
        else {
            return;
        };
        if self.apply_op(Op::ClearNext { id: node.clone() }) {
            self.selection = Selection::Slide(node);
        }
        self.open_form = None;
    }

    /// The picker's "a new slide…" row: creates a placeholder slide right
    /// after the wiring's origin, then wires the picker's target to it —
    /// no typed identifier anywhere (spec 013 US3, T051).
    fn commit_picker_new_slide(&mut self) {
        let Some(FormState::SlidePicker { target, .. }) = self.open_form.clone() else {
            return;
        };
        let origin = target.origin().to_owned();
        let existing: Vec<String> = self
            .working_graph
            .nodes
            .iter()
            .map(|n| n.id.clone())
            .collect();
        let new_id = authoring::slug("New slide", &existing);
        if !self.apply_op(Op::AddSlide {
            after: origin,
            title: "New slide".to_owned(),
        }) {
            return;
        }
        self.commit_picker_target(target, new_id);
    }

    /// An open `FormState::SlidePicker`'s keyboard-only row selection
    /// (spec 013 US3, ADR-017's keyboard-complete posture): digits 1-9
    /// pick a row by position, `n` picks "a new slide…", `e` picks
    /// "nothing — an ending" (`PickerTarget::Next` only, mirroring
    /// `form_chip_defs`'s rule for when that row exists at all). Returns
    /// whether the key was one of these, so the caller skips routing it to
    /// a (nonexistent) focused field.
    fn on_picker_key(&mut self, target: &PickerTarget, rows: &[PickerRow], code: KeyCode) -> bool {
        match code {
            KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                let idx = c.to_digit(10).expect("ascii digit") as usize - 1;
                if idx < rows.len() {
                    self.commit_picker_row(idx);
                }
                true
            }
            KeyCode::Char('n') => {
                self.commit_picker_new_slide();
                true
            }
            KeyCode::Char('e') if matches!(target, PickerTarget::Next { .. }) => {
                self.commit_picker_ending();
                true
            }
            _ => false,
        }
    }

    /// `[ Reveal: … ▾ ]`: cycles the selected block's reveal step
    /// none → 1 → 2 → … → none, auto-compacting via `Op::SetRevealStep`
    /// (spec 013 US3, T053).
    fn cycle_reveal_step(&mut self, node: String, path: BlockPath) {
        let Some(node_ref) = self.working_graph.node(&node) else {
            return;
        };
        let Some(block) = forms::block_at(&node_ref.content, &path) else {
            return;
        };
        let current = block.reveal().filter(|&v| v > 0);
        // Steps in use by every *other* block, excluding this one's own
        // current value — the ceiling this block can advance to is one
        // past that count (join an existing step, or start the next one),
        // so a lone block still cycles none → 1 → none rather than
        // getting stuck (spec 013 US3, T053).
        let other_levels = other_reveal_levels(&node_ref.content, &path);
        let next = match current {
            None => Some(1),
            Some(n) if n as usize <= other_levels.len() => Some(n + 1),
            Some(_) => None,
        };
        self.apply_op(Op::SetRevealStep {
            node,
            path,
            step: next,
        });
    }

    /// The toolbar chip / outline `＋ new slide` row (spec 013 US3, T049):
    /// opens the title prompt, wiring the new slide after the currently
    /// selected slide (or the deck's last slide when nothing is selected).
    fn open_new_slide_prompt(&mut self) {
        let after = match &self.selection {
            Selection::Slide(id) | Selection::Block(id, _) => id.clone(),
            Selection::None => match self.working_graph.nodes.last() {
                Some(n) => n.id.clone(),
                None => return,
            },
        };
        self.open_form = Some(FormState::Prompt {
            kind: PromptKind::NewSlide { after },
            fields: vec![EditableField::single_line(Vec::new(), "")],
            focus: 0,
        });
    }

    fn open_rename_deck_prompt(&mut self) {
        let title = self.working_graph.title.clone().unwrap_or_default();
        self.open_form = Some(FormState::Prompt {
            kind: PromptKind::DeckTitle,
            fields: vec![EditableField::single_line(Vec::new(), &title)],
            focus: 0,
        });
    }

    fn open_notes_prompt(&mut self, node: String) {
        let notes = self
            .working_graph
            .node(&node)
            .and_then(|n| n.speaker_notes.clone())
            .unwrap_or_default();
        self.open_form = Some(FormState::Prompt {
            kind: PromptKind::Notes { node },
            fields: vec![EditableField::from_text(
                Vec::new(),
                forms::EditableKind::Text,
                &notes,
            )],
            focus: 0,
        });
    }

    fn open_choice_prompt(&mut self, node: String) {
        self.open_form = Some(FormState::Prompt {
            kind: PromptKind::ChoicePrompt { node },
            fields: vec![
                EditableField::single_line(Vec::new(), ""),
                EditableField::single_line(Vec::new(), ""),
            ],
            focus: 1,
        });
    }

    fn open_new_answer_prompt(&mut self, node: String) {
        self.open_form = Some(FormState::Prompt {
            kind: PromptKind::NewAnswer { node },
            fields: vec![
                EditableField::single_line(Vec::new(), ""),
                EditableField::single_line(Vec::new(), ""),
            ],
            focus: 0,
        });
    }

    fn on_slide_chip(&mut self, node: String, action: SlideAction) {
        match action {
            SlideAction::Duplicate => {
                let Some(idx) = self.working_graph.nodes.iter().position(|n| n.id == node) else {
                    return;
                };
                if self.apply_op(Op::DuplicateSlide { id: node })
                    && let Some(dup) = self.working_graph.nodes.get(idx + 1)
                {
                    self.selection = Selection::Slide(dup.id.clone());
                }
            }
            SlideAction::Delete => {
                if self.apply_op(Op::DeleteSlide { id: node }) {
                    self.selection = Selection::None;
                    self.set_flash(
                        "Deleted \u{2014} press \u{21b6} Undo to bring it back",
                        FlashKind::Info,
                    );
                }
            }
            SlideAction::TurnIntoChoice => self.open_choice_prompt(node),
            SlideAction::TurnBackIntoSlide => {
                if self.apply_op(Op::TurnBackIntoSlide { id: node.clone() }) {
                    self.selection = Selection::Slide(node);
                }
            }
            SlideAction::AddAnswer => self.open_new_answer_prompt(node),
            SlideAction::RemoveAnswer => {
                let Some(bp) = self
                    .working_graph
                    .node(&node)
                    .and_then(|n| n.branch_point())
                else {
                    return;
                };
                let index = bp.options.len().saturating_sub(1);
                if self.apply_op(Op::RemoveAnswer {
                    id: node.clone(),
                    index,
                }) {
                    self.selection = Selection::Slide(node);
                    self.set_flash(
                        "Removed \u{2014} press \u{21b6} Undo to bring it back",
                        FlashKind::Info,
                    );
                }
            }
            SlideAction::Notes => self.open_notes_prompt(node),
        }
    }

    /// Outline slide drag across a branch boundary (spec 013 US3, T050):
    /// the flash carries a `[ take me there ]`-style link, resolved by
    /// finding the branch point whose answer targets `bad_id`, if any.
    fn attempt_reorder(&mut self, id: String, before: Option<String>) {
        match authoring::apply(
            &self.working_graph,
            &Op::ReorderSlide {
                id: id.clone(),
                before,
            },
        ) {
            Ok(next) => {
                self.push_history();
                self.working_graph = next;
                self.redo.clear();
                self.selection = Selection::Slide(id);
            }
            Err(AuthoringError::CrossesBranchBoundary(bad_id)) => {
                let title = self
                    .working_graph
                    .node(&bad_id)
                    .and_then(|n| n.title.clone())
                    .unwrap_or_else(|| bad_id.clone());
                let text = format!(
                    "\"{title}\" is reached only through a branch answer \u{2014} change the answer's target instead"
                );
                match find_branch_predecessor(&self.working_graph, &bad_id) {
                    Some(pred) => self.set_flash_with_action(
                        text,
                        FlashKind::Error,
                        hit::FlashAction::SelectSlide(pred),
                    ),
                    None => self.set_flash(text, FlashKind::Error),
                }
            }
            Err(err) => self.set_flash(err.to_string(), FlashKind::Error),
        }
    }

    fn focus_form_field(&mut self, slot: hit::FieldSlot) {
        let Some(form) = &mut self.open_form else {
            return;
        };
        match (form, slot) {
            (FormState::Code { focus, .. }, hit::FieldSlot::Language) => {
                *focus = CodeFocus::Language
            }
            (FormState::Code { focus, .. }, hit::FieldSlot::Source) => *focus = CodeFocus::Source,
            (FormState::Picture { focus, .. }, hit::FieldSlot::Src) => *focus = PictureFocus::Src,
            (FormState::Picture { focus, .. }, hit::FieldSlot::Alt) => *focus = PictureFocus::Alt,
            (FormState::TextArt { focus, .. }, hit::FieldSlot::Art) => *focus = TextArtFocus::Art,
            (FormState::TextArt { focus, .. }, hit::FieldSlot::Alt) => *focus = TextArtFocus::Alt,
            _ => {}
        }
    }

    /// The picture form's `[ Convert to text art ]` chip (T031): swaps the
    /// block to an `AsciiArt` kind in one `EditBlock` (the engine layer
    /// places no same-kind constraint on `EditBlock` — see
    /// `authoring::edit_block`), keeping the description, then reopens the
    /// form on the same path so the author can paste or generate the art
    /// immediately, matching "a new block starts with placeholder content
    /// and opens for editing immediately."
    fn convert_picture_to_text_art(&mut self) {
        let Some(FormState::Picture {
            node, path, alt, ..
        }) = &self.open_form
        else {
            return;
        };
        let node = node.clone();
        let path = path.clone();
        let alt_text = alt.text();
        let content = ContentBlock::AsciiArt {
            reveal: None,
            art: String::new(),
            alt: (!alt_text.trim().is_empty()).then_some(alt_text),
        };
        self.apply_op(Op::EditBlock {
            node: node.clone(),
            path: path.clone(),
            content,
        });
        self.open_form_at(&node, &path);
    }

    /// The container form's `[ Layout ▾ ]` chip (T033): cycles
    /// Stack → Columns → Center → Stack, applied immediately (not staged
    /// behind `[ Done ]`) since it is a single enum toggle, not free text —
    /// consistent with every other single-click structural change in this
    /// app.
    fn cycle_container_layout(&mut self) {
        let Some(FormState::Container {
            node, path, layout, ..
        }) = &self.open_form
        else {
            return;
        };
        let node = node.clone();
        let path = path.clone();
        let next = match layout {
            ContainerLayout::Stack => ContainerLayout::Columns,
            ContainerLayout::Columns => ContainerLayout::Center,
            ContainerLayout::Center => ContainerLayout::Stack,
        };
        let Some(node_ref) = self.working_graph.node(&node) else {
            return;
        };
        let Some(ContentBlock::Container { children, .. }) =
            forms::block_at(&node_ref.content, &path)
        else {
            return;
        };
        let content = ContentBlock::Container {
            reveal: None,
            children: children.clone(),
            layout: Some(next),
        };
        self.apply_op(Op::EditBlock {
            node: node.clone(),
            path: path.clone(),
            content,
        });
        self.open_form_at(&node, &path);
    }

    /// The text-art form's "Generate from a phrase…" chip (T032): treats
    /// the Art field's current text as the phrase, requesting a banner from
    /// the CLI-injected generator. `fireside-tui` cannot depend on
    /// `figlet-rs` itself (Constitution III) — see [`ArtGenerator`].
    fn request_art_generation(&mut self) {
        let Some(FormState::TextArt { art, .. }) = &self.open_form else {
            return;
        };
        let phrase = art.text();
        if phrase.trim().is_empty() {
            self.set_flash("Type a phrase in the Art field first", FlashKind::Info);
            return;
        }
        self.pending_art_request = Some(phrase);
    }

    fn on_art_generated(&mut self, result: Result<String, String>) {
        match result {
            Ok(art_text) => {
                if let Some(FormState::TextArt { art, .. }) = &mut self.open_form {
                    art.buffer = forms::to_buffer(&art_text);
                    art.cursor = (0, 0);
                }
            }
            Err(message) => self.set_flash(message, FlashKind::Error),
        }
    }

    fn focused_field_mut(&mut self) -> Option<&mut EditableField> {
        match self.open_form.as_mut()? {
            FormState::Heading { field, .. }
            | FormState::Text { field, .. }
            | FormState::List { field, .. } => Some(field),
            FormState::Code {
                language,
                source,
                focus,
                ..
            } => Some(match focus {
                CodeFocus::Language => language,
                CodeFocus::Source => source,
            }),
            FormState::Picture {
                src, alt, focus, ..
            } => Some(match focus {
                PictureFocus::Src => src,
                PictureFocus::Alt => alt,
            }),
            FormState::TextArt {
                art, alt, focus, ..
            } => Some(match focus {
                TextArtFocus::Art => art,
                TextArtFocus::Alt => alt,
            }),
            FormState::Prompt { fields, focus, .. } => fields.get_mut(*focus),
            FormState::Container { .. }
            | FormState::AddPalette { .. }
            | FormState::SlidePicker { .. } => None,
        }
    }

    /// Whether the currently focused field is single-line (Enter never
    /// inserts a newline into one — see `forms::EditableField::single_line`).
    fn focused_field_is_single_line(&self) -> bool {
        match &self.open_form {
            Some(FormState::Code {
                focus: CodeFocus::Language,
                ..
            })
            | Some(FormState::Picture { .. })
            | Some(FormState::TextArt {
                focus: TextArtFocus::Alt,
                ..
            }) => true,
            // Every `Prompt` field is single-line except `Notes`, which is
            // free text (spec 013 US3, T054).
            Some(FormState::Prompt {
                kind: PromptKind::Notes { .. },
                ..
            }) => false,
            Some(FormState::Prompt { .. }) => true,
            _ => false,
        }
    }

    /// Tab/Shift+Tab while a form is open: swaps focus between a
    /// multi-field form's two fields, or cycles the container form's
    /// layout (its only "field").
    fn form_cycle_field(&mut self) {
        if matches!(self.open_form, Some(FormState::Container { .. })) {
            self.cycle_container_layout();
            return;
        }
        let Some(form) = &mut self.open_form else {
            return;
        };
        match form {
            FormState::Code { focus, .. } => {
                *focus = match focus {
                    CodeFocus::Language => CodeFocus::Source,
                    CodeFocus::Source => CodeFocus::Language,
                };
            }
            FormState::Picture { focus, .. } => {
                *focus = match focus {
                    PictureFocus::Src => PictureFocus::Alt,
                    PictureFocus::Alt => PictureFocus::Src,
                };
            }
            FormState::TextArt { focus, .. } => {
                *focus = match focus {
                    TextArtFocus::Art => TextArtFocus::Alt,
                    TextArtFocus::Alt => TextArtFocus::Art,
                };
            }
            FormState::Prompt { fields, focus, .. } if fields.len() > 1 => {
                *focus = (*focus + 1) % fields.len();
            }
            _ => {}
        }
    }

    // ─── Ops, undo, save (spec 013, US1) ────────────────────────────────

    /// Applies `op`, pushing the pre-op state onto `history` first (capped
    /// at 100, per spec FR-016) and clearing `redo` — the sole path any op
    /// reaches `working_graph` through. A precondition failure flashes the
    /// error rather than touching `working_graph` (`authoring::apply` is
    /// atomic: `Err` never partially mutates). Returns whether the op
    /// applied, so callers that only want to react on success (delete's
    /// flash, a drag's selection follow-up) don't have to duplicate the
    /// match.
    fn apply_op(&mut self, op: Op) -> bool {
        match authoring::apply(&self.working_graph, &op) {
            Ok(next) => {
                self.push_history();
                self.working_graph = next;
                self.redo.clear();
                self.dirty_since_draft = true;
                true
            }
            Err(err) => {
                self.set_flash(err.to_string(), FlashKind::Error);
                false
            }
        }
    }

    // ─── Add / delete / reorder blocks (spec 013, US2) ──────────────────

    /// Opens the add-block palette (spec 013 T042), targeting position
    /// `at` within `path` (empty = the slide's top-level content).
    fn open_add_palette(&mut self, node: String, path: BlockPath, at: usize) {
        self.open_form = Some(FormState::AddPalette { node, path, at });
    }

    /// A palette card was chosen: inserts a placeholder block of `kind`
    /// via `Op::AddBlock`, then opens *its* edit form immediately — "the
    /// new block MUST start with placeholder content and open for editing
    /// immediately" (spec FR-007).
    fn add_block_from_palette(&mut self, kind: authoring::BlockKind) {
        let Some(FormState::AddPalette { node, path, at }) = self.open_form.clone() else {
            return;
        };
        self.open_form = None;
        if self.apply_op(Op::AddBlock {
            node: node.clone(),
            path: path.clone(),
            kind,
            at,
        }) {
            let mut new_path = path;
            new_path.push(at);
            self.selection = Selection::Block(node.clone(), new_path.clone());
            self.open_form_at(&node, &new_path);
        }
    }

    /// `[ Delete ]`: removes the block via `Op::DeleteBlock`, reindexes or
    /// clears a selection whose position the deletion shifted, and
    /// flashes a reversible, word-labeled notice rather than a blocking
    /// dialog (spec US2 acceptance scenario 2, FR-008/FR-017) — the
    /// toolbar's `[ ↶ Undo ]` chip/`u` key is the actual undo path.
    fn delete_block(&mut self, node: String, path: BlockPath) {
        let deleted_index = path.last().copied().unwrap_or(0);
        let deleted_parent = path[..path.len().saturating_sub(1)].to_vec();
        if self.apply_op(Op::DeleteBlock {
            node: node.clone(),
            path,
        }) {
            if let Selection::Block(sel_node, sel_path) = &mut self.selection
                && *sel_node == node
                && sel_path.len() == deleted_parent.len() + 1
                && sel_path[..deleted_parent.len()] == deleted_parent[..]
            {
                let sel_index = sel_path[deleted_parent.len()];
                match sel_index.cmp(&deleted_index) {
                    std::cmp::Ordering::Equal => self.selection = Selection::None,
                    std::cmp::Ordering::Greater => sel_path[deleted_parent.len()] -= 1,
                    std::cmp::Ordering::Less => {}
                }
            }
            self.set_flash(
                "Deleted \u{2014} press \u{21b6} Undo to bring it back",
                FlashKind::Info,
            );
        }
    }

    fn push_history(&mut self) {
        self.history.push(HistorySnapshot {
            graph: self.working_graph.clone(),
            selection: self.selection.clone(),
        });
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }

    /// `[ ↶ Undo ]`/`u`/`U`: restores the most recent pre-op snapshot,
    /// including the selection at that point, and closes any open form
    /// (its staged content no longer corresponds to anything on screen).
    fn undo(&mut self) {
        let Some(snapshot) = self.history.pop() else {
            self.set_flash("Nothing to undo", FlashKind::Info);
            return;
        };
        self.redo.push(HistorySnapshot {
            graph: self.working_graph.clone(),
            selection: self.selection.clone(),
        });
        self.working_graph = snapshot.graph;
        self.selection = snapshot.selection;
        self.open_form = None;
    }

    /// `[ Save ]`/Ctrl+S: commits an open form first (so "save" always
    /// saves what's on screen), then hands `working_graph` to the event
    /// loop as a pending save if there is anything unsaved.
    fn request_save(&mut self) {
        if self.open_form.is_some() {
            self.commit_form();
        }
        if !self.dirty() {
            self.set_flash("Nothing to save", FlashKind::Info);
            return;
        }
        self.pending_save = Some(self.working_graph.clone());
    }

    fn on_save_result(&mut self, result: Result<(), String>) {
        match result {
            Ok(()) => {
                self.saved_graph = self.working_graph.clone();
                self.dirty_since_draft = false;
                self.hint_tour_dismissed = true;
                self.set_flash("Saved", FlashKind::Info);
                // The quit-prompt's `[ Save ]` chip (spec 013 US4, FR-019):
                // only a *successful* save finishes the quit — a failure
                // leaves the author in the editor to retry or choose
                // differently, never losing the edit.
                if std::mem::take(&mut self.quit_after_save) {
                    self.quit = true;
                }
            }
            Err(message) => {
                self.quit_after_save = false;
                self.set_flash(message, FlashKind::Error);
            }
        }
    }

    // ─── Input ───────────────────────────────────────────────────────────

    /// Apply one message. The sole mutation point (constitution IV).
    pub(crate) fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Terminal(Event::Resize(w, h)) => {
                self.set_terminal_size(w, h);
                self.hover = None;
            }
            Msg::Terminal(Event::Key(key)) => self.on_key(key),
            Msg::Terminal(Event::Mouse(mouse)) => self.on_mouse(mouse),
            Msg::Terminal(_) => {}
            Msg::SaveResult(result) => self.on_save_result(result),
            Msg::ArtGenerated(result) => self.on_art_generated(result),
        }
    }

    // ─── Quit / drafts (spec 013, US4) ──────────────────────────────────

    /// `q`: quits immediately if there is nothing unsaved, otherwise opens
    /// the `[ Save ] [ Discard ] [ Keep editing ]` prompt (spec FR-019) —
    /// unsaved work is never silently discarded.
    fn request_quit(&mut self) {
        if self.dirty() {
            self.quit_prompt = true;
        } else {
            self.quit = true;
        }
    }

    fn on_quit_prompt_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('s' | 'S') => self.quit_prompt_save(),
            KeyCode::Char('d' | 'D') => {
                self.quit_prompt = false;
                self.quit = true;
            }
            KeyCode::Char('k' | 'K') | KeyCode::Esc => self.quit_prompt = false,
            _ => {}
        }
    }

    fn on_quit_prompt_click(&mut self, col: u16, row: u16) {
        let (w, h) = self.terminal_size;
        match hit::quit_prompt_hit(Rect::new(0, 0, w, h), col, row) {
            Some(hit::QuitAction::Save) => self.quit_prompt_save(),
            Some(hit::QuitAction::Discard) => {
                self.quit_prompt = false;
                self.quit = true;
            }
            Some(hit::QuitAction::KeepEditing) => self.quit_prompt = false,
            None => {}
        }
    }

    /// The quit-prompt's `[ Save ]` chip: stages the save exactly like
    /// Ctrl+S, then finishes the quit once [`Self::on_save_result`] sees
    /// it succeed.
    fn quit_prompt_save(&mut self) {
        self.quit_prompt = false;
        self.quit_after_save = true;
        self.request_save();
    }

    fn on_draft_choice_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('r' | 'R') => self.resolve_draft_choice(true),
            KeyCode::Char('o' | 'O') => self.resolve_draft_choice(false),
            _ => {}
        }
    }

    fn on_draft_choice_click(&mut self, col: u16, row: u16) {
        let (w, h) = self.terminal_size;
        if let Some(action) = hit::draft_choice_hit(Rect::new(0, 0, w, h), col, row) {
            self.resolve_draft_choice(action == hit::DraftAction::RestoreDraft);
        }
    }

    /// Resolves the open-time draft-vs-saved-file prompt (spec 013 US4,
    /// FR-020): `true` swaps the recovered draft in as `working_graph`
    /// (immediately dirty against the already-loaded saved file, exactly
    /// like any other unsaved edit — [`Self::dirty`] needs no special
    /// case); `false` keeps the saved file already loaded and simply
    /// declines the offer.
    fn resolve_draft_choice(&mut self, use_draft: bool) {
        let Some(choice) = self.draft_choice.take() else {
            return;
        };
        if use_draft {
            self.working_graph = choice.draft;
            self.status = validate(&self.working_graph);
        }
    }

    fn on_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if self.draft_choice.is_some() {
            self.on_draft_choice_key(key);
            return;
        }
        if self.quit_prompt {
            self.on_quit_prompt_key(key);
            return;
        }
        if self.showing_help {
            self.showing_help = false;
            return;
        }
        if self.open_form.is_some() {
            self.on_form_key(key);
            return;
        }
        match key.code {
            KeyCode::Char('q') => self.request_quit(),
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.request_save();
            }
            KeyCode::Esc => {
                if self.drag != DragState::Idle {
                    // A drag cancels without applying anything — nothing
                    // was ever written to `working_graph` until release,
                    // so "cancel" is just discarding the in-progress
                    // target (design brief: "the block returns").
                    self.drag = DragState::Idle;
                } else if self.selection != Selection::None {
                    self.selection = Selection::None;
                }
            }
            KeyCode::Char('?') => self.showing_help = true,
            KeyCode::Char('p' | 'P') => self.present_requested = true,
            KeyCode::Char('u' | 'U') => self.undo(),
            KeyCode::Enter => self.open_form_for_selection(),
            KeyCode::Tab => self.select_adjacent_block(false),
            KeyCode::BackTab => self.select_adjacent_block(true),
            KeyCode::Char(']') => self.select_adjacent_slide(false),
            KeyCode::Char('[') => self.select_adjacent_slide(true),
            KeyCode::Char('n') => self.open_new_slide_prompt(),
            KeyCode::Char('r') => self.on_reveal_key(),
            KeyCode::Char('c') => self.on_choice_key(),
            KeyCode::Char('a') => self.on_add_answer_key(),
            KeyCode::Char('g') => self.on_goes_to_key(),
            KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
            _ => {}
        }
    }

    /// `r`: the selected block's keyboard equivalent of the `[ Reveal ]`
    /// chip (spec 013 US3, ADR-017's keyboard-complete posture) — a no-op
    /// unless a block is selected.
    fn on_reveal_key(&mut self) {
        if let Selection::Block(node, path) = self.selection.clone() {
            self.cycle_reveal_step(node, path);
        }
    }

    /// `c`: the selected slide's keyboard equivalent of
    /// `[ Turn into a choice ]`/`[ Turn back into a normal slide ]` — a
    /// no-op unless a slide (not a block) is selected.
    fn on_choice_key(&mut self) {
        let Selection::Slide(id) = self.selection.clone() else {
            return;
        };
        let Some(node) = self.working_graph.node(&id) else {
            return;
        };
        if node.branch_point().is_some() {
            self.on_slide_chip(id, SlideAction::TurnBackIntoSlide);
        } else {
            self.open_choice_prompt(id);
        }
    }

    /// `a`: the selected branch-point slide's keyboard equivalent of
    /// `[ + Add answer ]` — a no-op unless the selected slide is already a
    /// branch point.
    fn on_add_answer_key(&mut self) {
        let Selection::Slide(id) = self.selection.clone() else {
            return;
        };
        let Some(node) = self.working_graph.node(&id) else {
            return;
        };
        if node.branch_point().is_some() {
            self.open_new_answer_prompt(id);
        }
    }

    /// `g`: the selected non-branch slide's keyboard equivalent of the
    /// "Goes to" strip's `[ change ]` chip.
    fn on_goes_to_key(&mut self) {
        let Selection::Slide(id) = self.selection.clone() else {
            return;
        };
        let Some(node) = self.working_graph.node(&id) else {
            return;
        };
        if node.branch_point().is_none() {
            let rows = self.picker_rows();
            self.open_form = Some(FormState::SlidePicker {
                target: PickerTarget::Next { node: id },
                rows,
            });
        }
    }

    /// Keys while a block's edit form is open: Esc cancels, Ctrl+S
    /// commits, Tab/Shift+Tab swaps field focus, everything else routes to
    /// the focused field's text buffer.
    fn on_form_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.cancel_form();
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            // `ChoicePrompt`/`NewAnswer` have no direct-commit `[ Done ]`
            // — their keyboard equivalent of clicking `[ Choose target →
            // ]` is Ctrl+S too, once the required field(s) are filled in.
            let routes_to_picker = matches!(&self.open_form, Some(form) if matches!(form, FormState::Prompt { .. }) && !form.prompt_commits_directly());
            if routes_to_picker {
                self.begin_picker();
            } else {
                self.commit_form();
            }
            return;
        }
        if matches!(key.code, KeyCode::Tab | KeyCode::BackTab) {
            self.form_cycle_field();
            return;
        }
        if let Some(FormState::SlidePicker { target, rows }) = self.open_form.clone()
            && self.on_picker_key(&target, &rows, key.code)
        {
            return;
        }
        let single_line = self.focused_field_is_single_line();
        let Some(field) = self.focused_field_mut() else {
            return;
        };
        match key.code {
            KeyCode::Char(c) => field.insert_char(c),
            KeyCode::Enter if !single_line => field.newline(),
            KeyCode::Backspace => field.backspace(),
            KeyCode::Delete => field.delete(),
            KeyCode::Left => field.move_left(),
            KeyCode::Right => field.move_right(),
            KeyCode::Up => {
                field.move_up();
            }
            KeyCode::Down => {
                field.move_down();
            }
            _ => {}
        }
    }

    fn on_mouse(&mut self, event: MouseEvent) {
        if self.draft_choice.is_some() {
            if let MouseEventKind::Down(MouseButton::Left) = event.kind {
                self.on_draft_choice_click(event.column, event.row);
            }
            return;
        }
        if self.quit_prompt {
            if let MouseEventKind::Down(MouseButton::Left) = event.kind {
                self.on_quit_prompt_click(event.column, event.row);
            }
            return;
        }
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => self.on_click(event.column, event.row),
            MouseEventKind::Drag(MouseButton::Left) => {
                self.on_drag_move(event.column, event.row);
            }
            MouseEventKind::Up(MouseButton::Left) => self.on_release(),
            MouseEventKind::Moved => {
                let (w, h) = self.terminal_size;
                self.hover = hit::hit(self, Rect::new(0, 0, w, h), event.column, event.row);
            }
            MouseEventKind::ScrollDown => self.scroll_at(event.column, event.row, true),
            MouseEventKind::ScrollUp => self.scroll_at(event.column, event.row, false),
            _ => {}
        }
    }

    /// Mouse-wheel scroll (spec 013 E4, T068): scrolls whichever pane the
    /// pointer sits over — the outline has its own offset from the canvas
    /// (`Self::outline_scroll`), so scrolling to find an off-screen slide
    /// in the outline never disturbs where the canvas is scrolled to, and
    /// vice versa. Off both panes (toolbar, status, hint), scrolls the
    /// canvas, matching this method's pre-existing behavior.
    fn scroll_at(&mut self, col: u16, row: u16, down: bool) {
        let (w, h) = self.terminal_size;
        let areas = hit::editor_areas(Rect::new(0, 0, w, h));
        let target = if hit::rect_contains(areas.outline, col, row) {
            &mut self.outline_scroll
        } else {
            &mut self.scroll
        };
        if down {
            *target = target.saturating_add(1);
        } else {
            *target = target.saturating_sub(1);
        }
    }

    /// A pointer move while the left button is held (crossterm reports
    /// this as `Drag`, distinct from button-less `Moved`): re-resolves
    /// the drop candidate for the block being lifted, auto-scrolling the
    /// canvas near its top/bottom edge (spec FR-009's "auto-scroll near
    /// canvas edges", design brief). A no-op unless a block drag is
    /// actually in progress.
    fn on_drag_move(&mut self, col: u16, row: u16) {
        match &self.drag {
            DragState::Lifting { .. } | DragState::Over { .. } => {
                let (node, path) = match &self.drag {
                    DragState::Lifting { node, path } | DragState::Over { node, path, .. } => {
                        (node.clone(), path.clone())
                    }
                    _ => unreachable!(),
                };
                let (w, h) = self.terminal_size;
                let areas = hit::editor_areas(Rect::new(0, 0, w, h));
                if row <= areas.canvas.y {
                    self.scroll = self.scroll.saturating_sub(1);
                } else if row.saturating_add(1) >= areas.canvas.bottom() {
                    self.scroll = self.scroll.saturating_add(1);
                }
                if let Some(to) = hit::resolve_drop_slot(self, &node, areas.canvas, col, row) {
                    self.drag = DragState::Over { node, path, to };
                }
            }
            DragState::OutlineLifting { .. } | DragState::OutlineOver { .. } => {
                let id = match &self.drag {
                    DragState::OutlineLifting { id } | DragState::OutlineOver { id, .. } => {
                        id.clone()
                    }
                    _ => unreachable!(),
                };
                let (w, h) = self.terminal_size;
                let areas = hit::editor_areas(Rect::new(0, 0, w, h));
                if row <= areas.outline.y {
                    self.outline_scroll = self.outline_scroll.saturating_sub(1);
                } else if row.saturating_add(1) >= areas.outline.bottom() {
                    self.outline_scroll = self.outline_scroll.saturating_add(1);
                }
                if let Some(before) = hit::resolve_outline_drop(self, areas.outline, col, row) {
                    self.drag = DragState::OutlineOver { id, before };
                }
            }
            DragState::Idle => {}
        }
    }

    /// Left-button release: commits the drag's last-resolved slot via
    /// `Op::MoveBlock`, converting the insertion-slot position (measured
    /// in the array's pre-removal order) into the post-removal index the
    /// engine's `MoveBlock::to` expects. A release with no resolved slot
    /// (the pointer never left the origin block, or left the canvas
    /// entirely) is a no-op — indistinguishable from, and as harmless as,
    /// a plain click.
    fn on_release(&mut self) {
        let drag = std::mem::replace(&mut self.drag, DragState::Idle);
        match drag {
            DragState::Over { node, path, to } => {
                let Some(&origin) = path.last() else { return };
                let move_to = if to <= origin { to } else { to - 1 };
                if move_to != origin
                    && self.apply_op(Op::MoveBlock {
                        node: node.clone(),
                        path: path.clone(),
                        to: move_to,
                    })
                {
                    self.selection = Selection::Block(node, vec![move_to]);
                }
            }
            DragState::OutlineOver { id, before } => {
                if before.as_deref() != Some(id.as_str()) {
                    self.attempt_reorder(id, before);
                }
            }
            DragState::Lifting { .. } | DragState::OutlineLifting { .. } | DragState::Idle => {}
        }
    }

    fn on_click(&mut self, col: u16, row: u16) {
        let (w, h) = self.terminal_size;
        match hit::hit(self, Rect::new(0, 0, w, h), col, row) {
            Some(hit::Target::OutlineRow(id)) => {
                self.selection = Selection::Slide(id.clone());
                self.scroll = 0;
                self.drag = DragState::OutlineLifting { id };
            }
            Some(hit::Target::Block(node_id, path)) => {
                self.selection = Selection::Block(node_id.clone(), path.clone());
                self.drag = DragState::Lifting {
                    node: node_id,
                    path,
                };
            }
            Some(hit::Target::InsertionSlot(node, path, at)) => {
                self.open_add_palette(node, path, at);
            }
            Some(hit::Target::BlockChip(node, path, hit::BlockAction::Edit)) => {
                self.open_form_at(&node, &path);
            }
            Some(hit::Target::BlockChip(node, path, hit::BlockAction::AddBelow)) => {
                let mut parent = path;
                let at = parent.pop().map_or(0, |i| i + 1);
                self.open_add_palette(node, parent, at);
            }
            Some(hit::Target::BlockChip(node, path, hit::BlockAction::Delete)) => {
                self.delete_block(node, path);
            }
            Some(hit::Target::BlockChip(node, path, hit::BlockAction::Reveal)) => {
                self.cycle_reveal_step(node, path);
            }
            Some(hit::Target::ToolbarChip(hit::ToolbarAction::Present)) => {
                self.present_requested = true;
            }
            Some(hit::Target::ToolbarChip(hit::ToolbarAction::Help)) => {
                self.showing_help = true;
            }
            Some(hit::Target::ToolbarChip(hit::ToolbarAction::Save)) => {
                self.request_save();
            }
            Some(hit::Target::ToolbarChip(hit::ToolbarAction::Undo)) => {
                self.undo();
            }
            Some(hit::Target::ToolbarChip(hit::ToolbarAction::AddSlide))
            | Some(hit::Target::OutlineNewSlide) => {
                self.open_new_slide_prompt();
            }
            Some(hit::Target::ToolbarTitle) => self.open_rename_deck_prompt(),
            Some(hit::Target::GoesToChip(node)) => {
                let rows = self.picker_rows();
                self.open_form = Some(FormState::SlidePicker {
                    target: PickerTarget::Next { node },
                    rows,
                });
            }
            Some(hit::Target::SlideChip(node, action)) => self.on_slide_chip(node, action),
            Some(hit::Target::AnswerChip(node, index)) => {
                let rows = self.picker_rows();
                self.open_form = Some(FormState::SlidePicker {
                    target: PickerTarget::RetargetAnswer { node, index },
                    rows,
                });
            }
            Some(hit::Target::FlashAction(hit::FlashAction::SelectSlide(id))) => {
                self.selection = Selection::Slide(id);
                self.flash = None;
            }
            Some(hit::Target::FormChip(hit::FormChipKind::PaletteCard(kind))) => {
                self.add_block_from_palette(kind);
            }
            Some(hit::Target::FormChip(kind)) => self.on_form_chip(kind),
            Some(hit::Target::FormField(slot)) => self.focus_form_field(slot),
            Some(hit::Target::StatusBanner) => self.jump_to_diagnostic(),
            // `MoveUp`/`MoveDown` chips aren't wired — block drag (US2)
            // already covers reordering (see `hit::BlockAction`'s doc).
            Some(hit::Target::BlockChip(..)) => {}
            None => {
                // A form, while open, occupies the whole hit-testing
                // surface (`hit::form_hit`) — a click outside it is
                // absorbed, not a "click elsewhere deselects" (the
                // selection underneath the form must survive it).
                if self.open_form.is_none() {
                    self.selection = Selection::None;
                }
            }
        }
    }
}

/// A write-back sink for the editor's `[ Save ]`/Ctrl+S: called with the
/// working graph when a save is requested. `fireside-tui` never touches
/// the filesystem itself — the caller (`fireside-cli::edit`) owns all I/O
/// and atomicity (spec FR-022).
pub type EditorWriteBackSink<'a> = &'a mut dyn FnMut(&Graph) -> Result<(), WriteBackError>;

/// The text-art form's "Generate from a phrase…" callback (spec 013,
/// T032): `fireside-tui` cannot depend on `figlet-rs` (Constitution III
/// scopes it to `fireside-cli`), so the CLI injects this exactly like
/// [`EditorWriteBackSink`] — called with the phrase typed into the form's
/// Art field, returning the rendered banner or a human-readable reason it
/// couldn't be rendered. `None` when the caller has no generator to offer
/// (the chip still shows; using it reports "not available").
pub type ArtGenerator<'a> = &'a mut dyn FnMut(&str) -> Result<String, String>;

/// The draft sidecar's write sink (spec 013 US4, T060): called with the
/// working graph whenever there has been a structural change since the
/// last draft write. Best-effort and infallible from the editor's point of
/// view, exactly like the presenter's own live session-state write
/// (`fireside-cli::session::write`) — an autosave failure must never
/// interrupt authoring.
pub type DraftSink<'a> = &'a mut dyn FnMut(&Graph);

/// Opens the full-screen authoring studio (spec 013) over `graph`: sets up
/// the terminal, runs the editor's own event loop, and always restores the
/// terminal, even on error — the same contract [`crate::present`] gives
/// the presenter. `draft`, if given, opens the studio behind the
/// draft-vs-saved-file prompt (spec 013 US4, FR-020) instead of drawing it
/// directly.
///
/// # Errors
///
/// Returns [`TuiError::NotATty`] outside an interactive terminal and
/// [`TuiError::Io`] for terminal failures.
pub fn run(
    graph: Graph,
    draft: Option<DraftPrompt>,
    sink: EditorWriteBackSink<'_>,
    draft_sink: DraftSink<'_>,
    art_generator: Option<ArtGenerator<'_>>,
) -> Result<(), TuiError> {
    if !io::stdout().is_tty() || !io::stdin().is_tty() {
        return Err(TuiError::NotATty);
    }
    let mut app = match draft {
        Some(prompt) => EditorApp::new_with_draft(graph, prompt),
        None => EditorApp::new(graph),
    };
    let mut terminal = ratatui::try_init()?;
    // Mouse capture is enabled once for the whole editor session — both
    // the studio's own loop and the in-process presenter loop `present_now`
    // enters share it, per research.md §6.
    let _ = execute!(io::stdout(), EnableMouseCapture);
    let result = editor_event_loop(&mut terminal, &mut app, sink, draft_sink, art_generator);
    let _ = execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

fn editor_event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut EditorApp,
    sink: EditorWriteBackSink<'_>,
    draft_sink: DraftSink<'_>,
    mut art_generator: Option<ArtGenerator<'_>>,
) -> Result<(), TuiError> {
    if let Ok(size) = terminal.size() {
        app.set_terminal_size(size.width, size.height);
    }
    while !app.should_quit() {
        if let Some(graph) = app.take_pending_save() {
            let result = sink(&graph).map_err(|err| err.to_string());
            app.update(Msg::SaveResult(result));
        }
        if let Some(graph) = app.take_pending_draft() {
            draft_sink(&graph);
        }
        if let Some(phrase) = app.take_pending_art_request() {
            let result = match &mut art_generator {
                Some(generator) => generator(&phrase),
                None => Err("Text-art generation isn't available in this build".to_owned()),
            };
            app.update(Msg::ArtGenerated(result));
        }
        terminal.draw(|frame| render::draw_editor(frame, app))?;
        if event::poll(Duration::from_millis(250))? {
            app.update(Msg::Terminal(event::read()?));
        }
        if let Some(start) = app.take_present_request() {
            present_now(terminal, app.working_graph(), Some(&start))?;
            if let Ok(size) = terminal.size() {
                app.set_terminal_size(size.width, size.height);
            }
        }
    }
    Ok(())
}

/// `[ ▶ Present ]`: runs the existing presenter event loop in-process
/// against the *same*, already-initialized terminal — no process spawn, no
/// second `try_init` (research.md §6). The embedded run never touches
/// resume state or the live session-state file: a no-op reload source, an
/// `Unavailable`-reporting write-back sink, and no-op position/tick sinks.
/// Control falls back to the editor loop on quit, which repaints.
fn present_now(
    terminal: &mut ratatui::DefaultTerminal,
    working_graph: &Graph,
    start_node: Option<&str>,
) -> Result<(), TuiError> {
    let mut session = fireside_engine::Session::new(working_graph.clone())?;
    if let Some(id) = start_node {
        session.goto(id);
    }
    let mut presenter = PresenterApp::new(session).without_sink();
    crate::event_loop(
        terminal,
        &mut presenter,
        &mut || None,
        &mut |_| Err(WriteBackError::Unavailable),
        &mut |_| {},
        &mut |_| {},
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fireside_core::ContentBlock;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    const FIXTURE: &str = r#"{"nodes":[
        {"id":"a","title":"Welcome","traversal":"b","content":[
            {"kind":"heading","level":1,"text":"Hello"},
            {"kind":"text","body":"World"}
        ]},
        {"id":"b","title":"The end","content":[{"kind":"text","body":"Done"}]}
    ]}"#;

    /// A slide with one block of every kind (spec 013 T038's "each of the
    /// 8 block kinds").
    const ALL_KINDS: &str = r#"{"nodes":[
        {"id":"a","title":"Everything","content":[
            {"kind":"heading","level":2,"text":"A heading"},
            {"kind":"text","body":"Some text"},
            {"kind":"code","language":"rust","source":"fn main() {}"},
            {"kind":"list","items":["one","two"]},
            {"kind":"image","src":"pic.png","alt":"a picture"},
            {"kind":"divider"},
            {"kind":"container","layout":"columns","children":[
                {"kind":"text","body":"left"}
            ]},
            {"kind":"ascii-art","art":"x-art"}
        ]}
    ]}"#;

    fn app() -> EditorApp {
        let mut app = EditorApp::new(Graph::from_json(FIXTURE).expect("fixture parses"));
        app.set_terminal_size(100, 30);
        app
    }

    fn all_kinds_app() -> EditorApp {
        let mut app = EditorApp::new(Graph::from_json(ALL_KINDS).expect("fixture parses"));
        app.set_terminal_size(100, 40);
        app
    }

    /// A 3-slide linear deck (spec 013 US3 fixtures): a → b → c.
    const LINEAR3: &str = r#"{"nodes":[
        {"id":"a","title":"Intro","traversal":"b","content":[{"kind":"text","body":"one"}]},
        {"id":"b","title":"Middle","traversal":"c","content":[{"kind":"text","body":"two"}]},
        {"id":"c","title":"End","content":[{"kind":"text","body":"three"}]}
    ]}"#;

    /// A slide with a branch point to two others (spec 013 US3 fixtures).
    const BRANCH: &str = r#"{"nodes":[
        {"id":"a","title":"Start","content":[{"kind":"text","body":"pick"}],
         "traversal":{"branch-point":{"options":[
            {"label":"To B","target":"b"},
            {"label":"To C","target":"c"}
         ]}}},
        {"id":"b","title":"B slide","content":[{"kind":"text","body":"b"}]},
        {"id":"c","title":"C slide","content":[{"kind":"text","body":"c"}]}
    ]}"#;

    fn linear3_app() -> EditorApp {
        let mut app = EditorApp::new(Graph::from_json(LINEAR3).expect("fixture parses"));
        app.set_terminal_size(100, 30);
        app
    }

    fn branch_app() -> EditorApp {
        let mut app = EditorApp::new(Graph::from_json(BRANCH).expect("fixture parses"));
        app.set_terminal_size(100, 30);
        app
    }

    /// A linear deck of `n` slides at a `height` short enough that the
    /// outline pane can't show them all at once — for T068's outline
    /// scroll/auto-scroll tests.
    fn many_slides_app(n: usize, height: u16) -> EditorApp {
        let nodes: Vec<String> = (0..n)
            .map(|i| {
                let next = if i + 1 < n {
                    format!(r#","traversal":"s{}""#, i + 1)
                } else {
                    String::new()
                };
                format!(
                    r#"{{"id":"s{i}","title":"Slide {i}"{next},"content":[{{"kind":"text","body":"x"}}]}}"#
                )
            })
            .collect();
        let json = format!(r#"{{"nodes":[{}]}}"#, nodes.join(","));
        let mut app = EditorApp::new(Graph::from_json(&json).expect("fixture parses"));
        app.set_terminal_size(100, height);
        app
    }

    /// A single slide whose `traversal` points nowhere — the fixture for
    /// T067's status-banner jump-to-diagnostic test.
    const DANGLING_TARGET: &str = r#"{"nodes":[
        {"id":"a","title":"Start","traversal":"missing","content":[{"kind":"text","body":"hi"}]}
    ]}"#;

    fn dangling_target_app() -> EditorApp {
        let mut app = EditorApp::new(Graph::from_json(DANGLING_TARGET).expect("fixture parses"));
        app.set_terminal_size(100, 30);
        app
    }

    fn press(app: &mut EditorApp, code: KeyCode) {
        app.update(Msg::Terminal(Event::Key(KeyEvent::from(code))));
    }

    fn type_text(app: &mut EditorApp, text: &str) {
        for c in text.chars() {
            press(app, KeyCode::Char(c));
        }
    }

    fn press_with(app: &mut EditorApp, code: KeyCode, modifiers: KeyModifiers) {
        app.update(Msg::Terminal(Event::Key(KeyEvent::new(code, modifiers))));
    }

    fn click(app: &mut EditorApp, col: u16, row: u16) {
        app.update(Msg::Terminal(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::empty(),
        })));
    }

    fn move_to(app: &mut EditorApp, col: u16, row: u16) {
        app.update(Msg::Terminal(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::empty(),
        })));
    }

    fn drag_to(app: &mut EditorApp, col: u16, row: u16) {
        app.update(Msg::Terminal(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::empty(),
        })));
    }

    fn release(app: &mut EditorApp, col: u16, row: u16) {
        app.update(Msg::Terminal(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::empty(),
        })));
    }

    fn draw(app: &EditorApp, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("backend");
        terminal
            .draw(|frame| render::draw_editor(frame, app))
            .expect("draw");
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>()
    }

    fn select_block(app: &mut EditorApp, node: &str, index: usize) {
        app.selection = Selection::Block(node.to_owned(), vec![index]);
    }

    #[test]
    fn opens_read_only_showing_the_entry_slide() {
        let app = app();
        assert_eq!(hit::selected_node(&app).map(|n| n.id.as_str()), Some("a"));
        assert!(!app.dirty());
        assert_eq!(app.history_len(), 0);
        assert_eq!(app.drag(), &DragState::Idle);
        assert!(app.open_form().is_none());
    }

    #[test]
    fn clicking_an_outline_row_selects_that_slide() {
        let mut app = app();
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        click(&mut app, areas.outline.x, areas.outline.y + 1);
        assert_eq!(app.selection(), &Selection::Slide("b".to_owned()));
    }

    #[test]
    fn clicking_a_block_selects_it() {
        let mut app = app();
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        // The first content line inside the card is well within the
        // heading block's extent for this small fixture.
        let target = hit::hit(
            &app,
            Rect::new(0, 0, 100, 30),
            areas.canvas.x + 40,
            areas.canvas.y + 4,
        );
        if let Some(hit::Target::Block(node, path)) = target {
            click(&mut app, areas.canvas.x + 40, areas.canvas.y + 4);
            assert_eq!(app.selection(), &Selection::Block(node, path));
        }
    }

    #[test]
    fn hover_tracks_the_pointer_without_selecting() {
        let mut app = app();
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        move_to(&mut app, areas.outline.x, areas.outline.y);
        assert_eq!(app.selection(), &Selection::None, "hover must not select");
        assert!(app.hover().is_some());
    }

    #[test]
    fn wheel_scrolls_the_canvas() {
        let mut app = app();
        assert_eq!(app.scroll(), 0);
        app.update(Msg::Terminal(Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: crossterm::event::KeyModifiers::empty(),
        })));
        assert_eq!(app.scroll(), 1);
    }

    #[test]
    fn arrow_keys_scroll_too() {
        let mut app = app();
        press(&mut app, KeyCode::Down);
        assert_eq!(app.scroll(), 1);
        press(&mut app, KeyCode::Up);
        assert_eq!(app.scroll(), 0);
    }

    /// Spec 013 E4, T068: wheel scroll over the outline advances the
    /// outline's own offset, not the canvas's — the two panes scroll
    /// independently.
    #[test]
    fn wheel_over_the_outline_scrolls_the_outline_not_the_canvas() {
        let mut app = many_slides_app(30, hit::MIN_HEIGHT);
        let areas = hit::editor_areas(Rect::new(0, 0, 100, hit::MIN_HEIGHT));
        app.update(Msg::Terminal(Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: areas.outline.x,
            row: areas.outline.y,
            modifiers: crossterm::event::KeyModifiers::empty(),
        })));
        assert_eq!(app.outline_scroll(), 1);
        assert_eq!(app.scroll(), 0);
    }

    /// Spec 013 E4, T068: dragging an outline row to the pane's bottom
    /// edge auto-scrolls it, the same way a block drag already
    /// auto-scrolls the canvas near its edges — without this, a slide
    /// couldn't be dragged past whatever fits on screen in a large deck.
    #[test]
    fn dragging_an_outline_row_to_the_bottom_edge_auto_scrolls_it() {
        let mut app = many_slides_app(30, hit::MIN_HEIGHT);
        let areas = hit::editor_areas(Rect::new(0, 0, 100, hit::MIN_HEIGHT));
        click(&mut app, areas.outline.x, areas.outline.y);
        assert!(matches!(app.drag(), DragState::OutlineLifting { .. }));
        drag_to(&mut app, areas.outline.x, areas.outline.bottom() - 1);
        assert_eq!(app.outline_scroll(), 1);
    }

    /// Spec 013 E4, T067: clicking the status banner jumps to the slide
    /// behind its diagnostic (`contracts`'s `Target::StatusBanner`).
    #[test]
    fn clicking_the_status_banner_jumps_to_the_offending_slide() {
        let mut app = dangling_target_app();
        assert_eq!(app.selection(), &Selection::None);
        assert!(!app.status().is_empty(), "the dangling target is flagged");
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        click(&mut app, areas.status.x, areas.status.y);
        assert_eq!(app.selection(), &Selection::Slide("a".to_owned()));
    }

    /// Spec 013 E4, T066: a fresh session starts with the first-run hint
    /// tour un-dismissed and showing its (steady, click-to-select)
    /// message at rest.
    #[test]
    fn hint_tour_starts_undismissed_and_visible() {
        let app = app();
        assert!(!app.hint_tour_dismissed());
        let screen = draw(&app, 100, 30);
        assert!(screen.contains("Click a slide or block to select"));
    }

    /// Spec 013 E4, T066: the first successful save dismisses the hint
    /// tour forever — it settles on the tour's first, steadiest message.
    #[test]
    fn hint_tour_is_dismissed_forever_after_the_first_save() {
        let mut app = app();
        app.update(Msg::SaveResult(Ok(())));
        assert!(app.hint_tour_dismissed());
        // The save's own "Saved" flash takes priority over the hint line
        // while it's up (`draw_hint`'s priority order) — clear it to see
        // what the hint line settles on underneath.
        app.flash = None;
        let screen = draw(&app, 100, 30);
        assert!(screen.contains("Click a slide or block to select"));
    }

    #[test]
    fn question_mark_opens_help_and_any_key_closes_it() {
        let mut app = app();
        press(&mut app, KeyCode::Char('?'));
        assert!(app.showing_help());
        press(&mut app, KeyCode::Char('x'));
        assert!(!app.showing_help());
    }

    #[test]
    fn esc_deselects_before_quitting() {
        let mut app = app();
        app.selection = Selection::Slide("b".to_owned());
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.selection(), &Selection::None);
        assert!(!app.should_quit());
    }

    #[test]
    fn q_quits() {
        let mut app = app();
        press(&mut app, KeyCode::Char('q'));
        assert!(app.should_quit());
    }

    /// Spec 013 US4, FR-019/acceptance scenario 2: `q` with unsaved
    /// changes must ask first, never quit or discard silently.
    #[test]
    fn q_with_unsaved_changes_opens_the_quit_prompt_instead_of_quitting() {
        let mut app = app();
        assert!(app.apply_op(Op::RetitleSlide {
            id: "a".to_owned(),
            title: "Edited".to_owned(),
        }));
        assert!(app.dirty());

        press(&mut app, KeyCode::Char('q'));
        assert!(!app.should_quit(), "q must not quit immediately when dirty");
        assert!(app.quit_prompt());
    }

    #[test]
    fn quit_prompt_keep_editing_dismisses_without_losing_the_edit() {
        let mut app = app();
        app.apply_op(Op::RetitleSlide {
            id: "a".to_owned(),
            title: "Edited".to_owned(),
        });
        press(&mut app, KeyCode::Char('q'));
        press(&mut app, KeyCode::Char('k'));
        assert!(!app.quit_prompt());
        assert!(!app.should_quit());
        assert!(app.dirty(), "the edit is still there, unsaved");
    }

    #[test]
    fn quit_prompt_discard_quits_without_saving() {
        let mut app = app();
        app.apply_op(Op::RetitleSlide {
            id: "a".to_owned(),
            title: "Edited".to_owned(),
        });
        press(&mut app, KeyCode::Char('q'));
        press(&mut app, KeyCode::Char('d'));
        assert!(app.should_quit());
        assert!(app.dirty(), "discard quits without writing back");
    }

    #[test]
    fn quit_prompt_save_saves_then_quits_once_the_save_succeeds() {
        let mut app = app();
        app.apply_op(Op::RetitleSlide {
            id: "a".to_owned(),
            title: "Edited".to_owned(),
        });
        press(&mut app, KeyCode::Char('q'));
        press(&mut app, KeyCode::Char('s'));
        assert!(!app.quit_prompt());
        assert!(
            !app.should_quit(),
            "the quit finishes only once the save result comes back"
        );
        app.update(Msg::SaveResult(Ok(())));
        assert!(app.should_quit());
        assert!(!app.dirty());
    }

    #[test]
    fn quit_prompt_save_failure_keeps_editing_rather_than_quitting() {
        let mut app = app();
        app.apply_op(Op::RetitleSlide {
            id: "a".to_owned(),
            title: "Edited".to_owned(),
        });
        press(&mut app, KeyCode::Char('q'));
        press(&mut app, KeyCode::Char('s'));
        app.update(Msg::SaveResult(Err("disk full".to_owned())));
        assert!(
            !app.should_quit(),
            "a failed save must never lose the edit by quitting anyway"
        );
        assert!(app.dirty());
    }

    #[test]
    fn quit_prompt_chips_are_clickable() {
        let mut app = app();
        app.apply_op(Op::RetitleSlide {
            id: "a".to_owned(),
            title: "Edited".to_owned(),
        });
        press(&mut app, KeyCode::Char('q'));
        let area = Rect::new(0, 0, 100, 30);
        let (_, discard_rect) = hit::quit_prompt_chip_rects(area)
            .into_iter()
            .find(|(a, _)| *a == hit::QuitAction::Discard)
            .expect("discard chip rect");
        click(&mut app, discard_rect.x, discard_rect.y);
        assert!(app.should_quit());
    }

    /// Spec 013 US4, FR-020/contract "Behavior" section: the draft-choice
    /// prompt takes over the whole screen rather than drawing on top of
    /// the studio.
    #[test]
    fn draft_choice_gates_the_studio_until_resolved() {
        let saved = Graph::from_json(FIXTURE).expect("fixture parses");
        let mut draft = saved.clone();
        draft.nodes[0].title = Some("Recovered title".to_owned());
        let mut app = EditorApp::new_with_draft(
            saved,
            DraftPrompt {
                draft,
                draft_touched: "2 minutes ago".to_owned(),
                saved_touched: "an hour ago".to_owned(),
            },
        );
        app.set_terminal_size(100, 30);
        assert!(app.draft_choice().is_some());
        let screen = draw(&app, 100, 30);
        assert!(screen.contains("Recovered unsaved changes"));
        assert!(screen.contains("2 minutes ago"));
        assert!(screen.contains("an hour ago"));
        assert!(
            !screen.contains("Click a slide or block to select"),
            "the studio's hint line must not draw underneath the draft prompt"
        );
    }

    #[test]
    fn draft_choice_restore_swaps_in_the_draft_and_leaves_it_dirty() {
        let saved = Graph::from_json(FIXTURE).expect("fixture parses");
        let mut draft = saved.clone();
        draft.nodes[0].title = Some("Recovered title".to_owned());
        let draft_clone = draft.clone();
        let mut app = EditorApp::new_with_draft(
            saved,
            DraftPrompt {
                draft,
                draft_touched: "just now".to_owned(),
                saved_touched: "an hour ago".to_owned(),
            },
        );
        app.set_terminal_size(100, 30);
        press(&mut app, KeyCode::Char('r'));
        assert!(app.draft_choice().is_none());
        assert_eq!(app.working_graph(), &draft_clone);
        assert!(
            app.dirty(),
            "restoring a differing draft is an unsaved change"
        );
    }

    #[test]
    fn draft_choice_open_saved_keeps_the_saved_file() {
        let saved = Graph::from_json(FIXTURE).expect("fixture parses");
        let saved_clone = saved.clone();
        let mut draft = saved.clone();
        draft.nodes[0].title = Some("Recovered title".to_owned());
        let mut app = EditorApp::new_with_draft(
            saved,
            DraftPrompt {
                draft,
                draft_touched: "just now".to_owned(),
                saved_touched: "an hour ago".to_owned(),
            },
        );
        app.set_terminal_size(100, 30);
        press(&mut app, KeyCode::Char('o'));
        assert!(app.draft_choice().is_none());
        assert_eq!(app.working_graph(), &saved_clone);
        assert!(!app.dirty());
    }

    #[test]
    fn draft_choice_chips_are_clickable() {
        let saved = Graph::from_json(FIXTURE).expect("fixture parses");
        let saved_clone = saved.clone();
        let draft = saved.clone();
        let mut app = EditorApp::new_with_draft(
            saved,
            DraftPrompt {
                draft,
                draft_touched: "just now".to_owned(),
                saved_touched: "an hour ago".to_owned(),
            },
        );
        app.set_terminal_size(100, 30);
        let area = Rect::new(0, 0, 100, 30);
        let (_, rect) = hit::draft_choice_chip_rects(area)
            .into_iter()
            .find(|(a, _)| *a == hit::DraftAction::OpenSaved)
            .expect("open-saved chip rect");
        click(&mut app, rect.x, rect.y);
        assert!(app.draft_choice().is_none());
        assert_eq!(app.working_graph(), &saved_clone);
    }

    /// Spec 013 US4, T060: a structural op marks the draft dirty, and the
    /// event loop's pull (`take_pending_draft`) consumes it exactly once.
    #[test]
    fn structural_ops_queue_exactly_one_pending_draft_write_each() {
        let mut app = app();
        assert_eq!(app.take_pending_draft(), None, "nothing pending yet");
        app.apply_op(Op::RetitleSlide {
            id: "a".to_owned(),
            title: "Edited".to_owned(),
        });
        let pending = app.take_pending_draft();
        assert_eq!(pending.as_ref(), Some(app.working_graph()));
        assert_eq!(
            app.take_pending_draft(),
            None,
            "already consumed, no repeat write for the same op"
        );
    }

    #[test]
    fn resize_updates_terminal_size_and_clears_hover() {
        let mut app = app();
        move_to(&mut app, 5, 5);
        app.update(Msg::Terminal(Event::Resize(90, 28)));
        assert_eq!(app.terminal_size, (90, 28));
        assert!(app.hover().is_none());
    }

    // `present_now` drives the real blocking presenter event loop (reading
    // actual terminal events via `crossterm::event`), so it cannot run
    // under `TestBackend` the way drawing can — that would hang waiting
    // for real input. This test instead pins the state-machine half of
    // "present and return": the request is captured with the right start
    // slide, consumed exactly once, and leaves the editor's own state
    // untouched. The full embedded-terminal path is proven by the tmux
    // smoke test (T026), in a real terminal.
    #[test]
    fn present_request_captures_the_selected_slide_and_is_consumed_once() {
        let mut app = app();
        app.selection = Selection::Slide("b".to_owned());
        press(&mut app, KeyCode::Char('p'));
        assert_eq!(app.take_present_request().as_deref(), Some("b"));
        assert_eq!(
            app.take_present_request(),
            None,
            "a present request must not fire twice"
        );
        // Back in the editor: selection and the working graph are
        // untouched by having requested a present.
        assert_eq!(app.selection(), &Selection::Slide("b".to_owned()));
        assert!(!app.dirty());
    }

    #[test]
    fn read_only_studio_renders_at_100x30() {
        let app = app();
        let screen = draw(&app, 100, 30);
        assert!(
            screen.contains('W'),
            "expected the deck title/content on screen"
        );
    }

    #[test]
    fn below_minimum_size_shows_the_resize_guard_not_the_studio() {
        let app = app();
        let screen = draw(&app, 44, 14);
        assert!(screen.contains("bigger"));
    }

    // ─── US1: select → edit → save → undo (T038) ────────────────────────

    #[test]
    fn selecting_a_divider_offers_no_edit_form() {
        let mut app = all_kinds_app();
        select_block(&mut app, "a", 5); // the divider
        press(&mut app, KeyCode::Enter);
        assert!(app.open_form().is_none());
        assert_eq!(
            app.flash().map(|f| f.text.as_str()),
            Some("Dividers have nothing to edit")
        );
    }

    #[test]
    fn tab_selects_blocks_without_the_mouse_and_wraps() {
        let mut app = all_kinds_app();
        // Nothing selected yet: Tab lands on the first block.
        press(&mut app, KeyCode::Tab);
        assert_eq!(app.selection(), &Selection::Block("a".to_owned(), vec![0]));
        press(&mut app, KeyCode::Tab);
        assert_eq!(app.selection(), &Selection::Block("a".to_owned(), vec![1]));
        press(&mut app, KeyCode::BackTab);
        assert_eq!(app.selection(), &Selection::Block("a".to_owned(), vec![0]));
        // Wraps from the first block back to the last with Shift+Tab.
        press(&mut app, KeyCode::BackTab);
        assert_eq!(app.selection(), &Selection::Block("a".to_owned(), vec![7]));
    }

    #[test]
    fn keyboard_only_select_edit_save_matches_the_mouse_path() {
        let mut app = app();
        press(&mut app, KeyCode::Tab); // selects block 0 (the heading)
        press(&mut app, KeyCode::Tab); // selects block 1 (the text block)
        assert_eq!(app.selection(), &Selection::Block("a".to_owned(), vec![1]));
        press(&mut app, KeyCode::Enter);
        assert!(matches!(app.open_form(), Some(FormState::Text { .. })));
        press(&mut app, KeyCode::Char('!'));
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert!(app.open_form().is_none());
        assert_eq!(
            app.working_graph().node("a").unwrap().content[1],
            ContentBlock::Text {
                reveal: None,
                body: "!World".to_owned(),
            }
        );
    }

    #[test]
    fn heading_select_edit_save_undo_round_trips_via_keyboard() {
        let mut app = app();
        select_block(&mut app, "a", 0);
        press(&mut app, KeyCode::Enter);
        let Some(FormState::Heading { field, .. }) = app.open_form() else {
            panic!("heading form open: {:?}", app.open_form());
        };
        assert_eq!(field.buffer, vec!["Hello".to_owned()]);
        // Move to end of "Hello" and append " there".
        for _ in 0..5 {
            press(&mut app, KeyCode::Right);
        }
        for c in " there".chars() {
            press(&mut app, KeyCode::Char(c));
        }
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert!(
            app.open_form().is_none(),
            "Ctrl+S commits and closes the form"
        );
        let node = app.working_graph().node("a").expect("node a");
        assert_eq!(
            node.content[0],
            ContentBlock::Heading {
                reveal: None,
                level: 1,
                text: "Hello there".to_owned(),
            }
        );
        assert!(app.dirty());

        app.undo();
        let node = app.working_graph().node("a").expect("node a");
        assert_eq!(
            node.content[0],
            ContentBlock::Heading {
                reveal: None,
                level: 1,
                text: "Hello".to_owned(),
            },
            "undo restores the exact prior wording"
        );
    }

    /// Spec 013 US4, FR-016/acceptance scenario 3: at least the 100 most
    /// recent actions must each undo in order, restoring the exact prior
    /// state. 100 ops exactly fits the cap (`push_history`'s `> 100`
    /// eviction never triggers), so every one of them stays undoable.
    #[test]
    fn undo_reverses_one_hundred_sequential_actions_in_exact_order() {
        let mut app = app();
        let mut snapshots = vec![app.working_graph().clone()];
        for i in 0..100 {
            assert!(app.apply_op(Op::EditBlock {
                node: "a".to_owned(),
                path: vec![1],
                content: ContentBlock::Text {
                    reveal: None,
                    body: format!("Body {i}"),
                },
            }));
            snapshots.push(app.working_graph().clone());
        }
        assert_eq!(app.history_len(), 100);

        for i in (0..100).rev() {
            app.undo();
            assert_eq!(
                app.working_graph(),
                &snapshots[i],
                "undo did not restore the exact state after action {i}"
            );
        }
        app.undo();
        assert_eq!(
            app.working_graph(),
            &snapshots[0],
            "an extra undo past the beginning is a no-op, not a further change"
        );
    }

    /// The 100-action cap (spec FR-016: "at least the 100 most recent") —
    /// a 101st action evicts the oldest snapshot, so undo can restore
    /// everything back to the state after the first action, but no
    /// further.
    #[test]
    fn history_caps_at_one_hundred_evicting_the_oldest_snapshot() {
        let mut app = app();
        let after_first = {
            assert!(app.apply_op(Op::EditBlock {
                node: "a".to_owned(),
                path: vec![1],
                content: ContentBlock::Text {
                    reveal: None,
                    body: "Body 0".to_owned(),
                },
            }));
            app.working_graph().clone()
        };
        for i in 1..101 {
            assert!(app.apply_op(Op::EditBlock {
                node: "a".to_owned(),
                path: vec![1],
                content: ContentBlock::Text {
                    reveal: None,
                    body: format!("Body {i}"),
                },
            }));
        }
        assert_eq!(app.history_len(), 100, "history never grows past the cap");

        for _ in 0..100 {
            app.undo();
        }
        assert_eq!(
            app.working_graph(),
            &after_first,
            "100 undos land on the state right after the first action, the oldest still retained"
        );
    }

    #[test]
    fn text_select_edit_via_mouse_click_edit_chip_and_done_chip() {
        let mut app = app();
        select_block(&mut app, "a", 1); // the "World" text block
        let area = Rect::new(0, 0, 100, 30);
        let areas = hit::editor_areas(area);
        // The hint line now shows the block's [ Edit ] chip.
        let target = hit::hit(&app, area, areas.hint.x + 2, areas.hint.y);
        assert!(
            matches!(
                target,
                Some(hit::Target::BlockChip(_, _, hit::BlockAction::Edit))
            ),
            "expected the hint line's Edit chip, got {target:?}"
        );
        click(&mut app, areas.hint.x + 2, areas.hint.y);
        let Some(FormState::Text { field, .. }) = app.open_form() else {
            panic!("text form open: {:?}", app.open_form());
        };
        assert_eq!(field.buffer, vec!["World".to_owned()]);

        // Locate and click the [ Done ] chip inside the form overlay.
        let layout = hit::form_layout(app.open_form().expect("form open"), area);
        let (_, _, done_rect) = layout
            .chips
            .iter()
            .find(|(kind, _, _)| *kind == hit::FormChipKind::Done)
            .expect("a Done chip exists");
        click(&mut app, done_rect.x, done_rect.y);
        assert!(
            app.open_form().is_none(),
            "clicking Done commits and closes the form"
        );
        assert_eq!(
            app.working_graph().node("a").unwrap().content[1],
            ContentBlock::Text {
                reveal: None,
                body: "World".to_owned(),
            },
            "unedited text round-trips unchanged"
        );
    }

    #[test]
    fn cancel_discards_the_in_progress_edit() {
        let mut app = app();
        select_block(&mut app, "a", 1);
        press(&mut app, KeyCode::Enter);
        press(&mut app, KeyCode::Char('X'));
        press(&mut app, KeyCode::Esc);
        assert!(app.open_form().is_none());
        assert_eq!(
            app.working_graph().node("a").unwrap().content[1],
            ContentBlock::Text {
                reveal: None,
                body: "World".to_owned(),
            },
            "Esc must discard, never commit"
        );
        assert!(!app.dirty());
    }

    #[test]
    fn code_form_edits_language_and_source_across_both_fields() {
        let mut app = all_kinds_app();
        select_block(&mut app, "a", 2); // the code block
        press(&mut app, KeyCode::Enter);
        assert!(matches!(
            app.open_form(),
            Some(FormState::Code {
                focus: CodeFocus::Source,
                ..
            })
        ));
        press(&mut app, KeyCode::Tab); // focus starts on Source; move to Language
        {
            let Some(FormState::Code { language, .. }) = &mut app.open_form else {
                panic!("code form open");
            };
            language.buffer = vec!["python".to_owned()];
        }
        let Some(FormState::Code { source, .. }) = app.open_form() else {
            panic!("code form open");
        };
        assert_eq!(
            source.buffer,
            vec!["fn main() {}".to_owned()],
            "editing Language must not touch Source"
        );
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        let ContentBlock::Code {
            language, source, ..
        } = &app.working_graph().node("a").unwrap().content[2]
        else {
            panic!("still a code block");
        };
        assert_eq!(language.as_deref(), Some("python"));
        assert_eq!(source, "fn main() {}");
    }

    #[test]
    fn list_form_edits_items_one_per_line_and_drops_blanks() {
        let mut app = all_kinds_app();
        select_block(&mut app, "a", 3); // the list block: ["one", "two"]
        press(&mut app, KeyCode::Enter);
        {
            let Some(FormState::List { field, .. }) = &mut app.open_form else {
                panic!("list form open");
            };
            field.buffer = vec![
                "one".to_owned(),
                String::new(),
                "two".to_owned(),
                "three".to_owned(),
            ];
        }
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        let ContentBlock::List { items, .. } = &app.working_graph().node("a").unwrap().content[3]
        else {
            panic!("still a list block");
        };
        assert_eq!(
            items,
            &vec!["one".to_owned(), "two".to_owned(), "three".to_owned()],
            "blank lines are dropped, the rest kept in order"
        );
    }

    #[test]
    fn picture_convert_to_text_art_swaps_the_block_kind_and_reopens_its_form() {
        let mut app = all_kinds_app();
        select_block(&mut app, "a", 4); // the image block
        press(&mut app, KeyCode::Enter);
        assert!(matches!(app.open_form(), Some(FormState::Picture { .. })));
        let area = Rect::new(0, 0, 100, 40);
        let layout = hit::form_layout(app.open_form().expect("form open"), area);
        let (_, _, convert_rect) = layout
            .chips
            .iter()
            .find(|(kind, _, _)| *kind == hit::FormChipKind::ConvertToTextArt)
            .expect("a Convert chip exists");
        app.set_terminal_size(100, 40);
        click(&mut app, convert_rect.x, convert_rect.y);
        assert!(
            matches!(app.open_form(), Some(FormState::TextArt { .. })),
            "converting reopens the same block as a text-art form: {:?}",
            app.open_form()
        );
        let ContentBlock::AsciiArt { alt, .. } = &app.working_graph().node("a").unwrap().content[4]
        else {
            panic!("block 4 is now text art");
        };
        assert_eq!(alt.as_deref(), Some("a picture"));
    }

    #[test]
    fn text_art_generate_from_phrase_replaces_the_art_buffer() {
        let mut app = all_kinds_app();
        select_block(&mut app, "a", 7); // the ascii-art block
        press(&mut app, KeyCode::Enter);
        let Some(FormState::TextArt { art, .. }) = app.open_form() else {
            panic!("text art form open");
        };
        assert_eq!(art.buffer, vec!["x-art".to_owned()]);
        // Simulate the CLI-injected generator's round trip directly (the
        // real callback threading is exercised by the CLI e2e/tmux layers).
        app.update(Msg::ArtGenerated(Ok("BIG\nART".to_owned())));
        let Some(FormState::TextArt { art, .. }) = app.open_form() else {
            panic!("text art form still open");
        };
        assert_eq!(art.buffer, vec!["BIG".to_owned(), "ART".to_owned()]);
    }

    #[test]
    fn text_art_too_wide_refuses_to_commit() {
        let mut app = all_kinds_app();
        select_block(&mut app, "a", 7);
        press(&mut app, KeyCode::Enter);
        {
            let Some(FormState::TextArt { art, .. }) = &mut app.open_form else {
                panic!("text art form open");
            };
            art.buffer[0] = "x".repeat(forms::MAX_ART_WIDTH + 1);
        }
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert!(
            app.open_form().is_some(),
            "an oversized art body must not commit"
        );
        assert!(app.flash().is_some());
    }

    #[test]
    fn container_layout_cycle_commits_immediately_and_is_undoable() {
        let mut app = all_kinds_app();
        select_block(&mut app, "a", 6); // the container block
        press(&mut app, KeyCode::Enter);
        assert!(matches!(app.open_form(), Some(FormState::Container { .. })));
        let ContentBlock::Container { layout, .. } =
            &app.working_graph().node("a").unwrap().content[6]
        else {
            panic!("still a container");
        };
        assert_eq!(*layout, Some(ContainerLayout::Columns));

        press(&mut app, KeyCode::Tab); // cycles Columns -> Center, commits immediately
        let ContentBlock::Container { layout, .. } =
            &app.working_graph().node("a").unwrap().content[6]
        else {
            panic!("still a container");
        };
        assert_eq!(*layout, Some(ContainerLayout::Center));
        assert!(app.dirty());

        app.undo();
        let ContentBlock::Container { layout, .. } =
            &app.working_graph().node("a").unwrap().content[6]
        else {
            panic!("still a container");
        };
        assert_eq!(
            *layout,
            Some(ContainerLayout::Columns),
            "undo reverses the layout cycle too"
        );
    }

    #[test]
    fn request_save_round_trips_through_save_result_and_clears_dirty() {
        let mut app = app();
        select_block(&mut app, "a", 1);
        press(&mut app, KeyCode::Enter);
        press(&mut app, KeyCode::Char('!'));
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL); // commits the form
        assert!(app.dirty());

        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL); // Ctrl+S with no form open: save
        let pending = app.take_pending_save().expect("a save was requested");
        assert!(
            app.take_pending_save().is_none(),
            "a save request is consumed once"
        );

        app.update(Msg::SaveResult(Ok(())));
        assert!(!app.dirty(), "a successful save clears the dirty indicator");
        assert_eq!(
            app.flash().map(|f| f.text.clone()),
            Some("Saved".to_owned())
        );
        let _ = pending;
    }

    #[test]
    fn save_failure_keeps_the_edit_and_flashes_the_reason() {
        let mut app = app();
        select_block(&mut app, "a", 1);
        press(&mut app, KeyCode::Enter);
        press(&mut app, KeyCode::Char('!'));
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL); // commits the form
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL); // requests the save
        let _ = app.take_pending_save();
        app.update(Msg::SaveResult(Err("disk full".to_owned())));
        assert!(app.dirty(), "a failed save must not be treated as saved");
        assert_eq!(
            app.flash().map(|f| f.text.clone()),
            Some("disk full".to_owned())
        );
    }

    // ─── US2: add / delete / drag-reorder blocks (T047) ─────────────────

    #[test]
    fn add_block_via_palette_inserts_a_placeholder_and_opens_its_form() {
        let mut app = app();
        select_block(&mut app, "a", 0); // the heading
        let area = Rect::new(0, 0, 100, 30);
        let areas = hit::editor_areas(area);
        let chips = hit::selected_block_chips(&app);
        let (_, add_below_rect) = hit::chip_rects(areas.hint, &chips)
            .into_iter()
            .find(|(a, _)| *a == hit::BlockAction::AddBelow)
            .expect("an Add below chip exists");
        click(&mut app, add_below_rect.x, add_below_rect.y);
        assert!(matches!(
            app.open_form(),
            Some(FormState::AddPalette { .. })
        ));

        let layout = hit::form_layout(app.open_form().expect("palette open"), area);
        let (_, _, card_rect) = layout
            .chips
            .iter()
            .find(|(kind, _, _)| {
                matches!(kind, hit::FormChipKind::PaletteCard(k) if *k == authoring::BlockKind::Text)
            })
            .expect("a Text card exists");
        click(&mut app, card_rect.x, card_rect.y);

        // Placeholder content is inserted right after the selected block
        // (position 1) and its own form opens immediately (spec FR-007).
        assert!(
            matches!(app.open_form(), Some(FormState::Text { .. })),
            "the new block's form opens immediately: {:?}",
            app.open_form()
        );
        let node = app.working_graph().node("a").expect("node a");
        assert_eq!(node.content.len(), 3);
        assert_eq!(
            node.content[1],
            ContentBlock::Text {
                reveal: None,
                body: "New text".to_owned(),
            }
        );
        assert_eq!(app.selection(), &Selection::Block("a".to_owned(), vec![1]));
    }

    #[test]
    fn every_palette_card_inserts_its_own_block_kind() {
        type KindCheck = fn(&ContentBlock) -> bool;
        let cases: [(authoring::BlockKind, KindCheck); 8] = [
            (authoring::BlockKind::Heading, |b| {
                matches!(b, ContentBlock::Heading { .. })
            }),
            (authoring::BlockKind::Text, |b| {
                matches!(b, ContentBlock::Text { .. })
            }),
            (authoring::BlockKind::Code, |b| {
                matches!(b, ContentBlock::Code { .. })
            }),
            (authoring::BlockKind::List, |b| {
                matches!(b, ContentBlock::List { .. })
            }),
            (authoring::BlockKind::Image, |b| {
                matches!(b, ContentBlock::Image { .. })
            }),
            (authoring::BlockKind::Divider, |b| {
                matches!(b, ContentBlock::Divider { .. })
            }),
            (authoring::BlockKind::Container, |b| {
                matches!(b, ContentBlock::Container { .. })
            }),
            (authoring::BlockKind::AsciiArt, |b| {
                matches!(b, ContentBlock::AsciiArt { .. })
            }),
        ];
        let area = Rect::new(0, 0, 100, 30);
        let areas = hit::editor_areas(area);
        for (kind, is_expected_kind) in cases {
            let mut app = app();
            select_block(&mut app, "a", 0);
            let chips = hit::selected_block_chips(&app);
            let (_, add_below_rect) = hit::chip_rects(areas.hint, &chips)
                .into_iter()
                .find(|(a, _)| *a == hit::BlockAction::AddBelow)
                .expect("an Add below chip exists");
            click(&mut app, add_below_rect.x, add_below_rect.y);
            let layout = hit::form_layout(app.open_form().expect("palette open"), area);
            let (_, _, card_rect) = layout
                .chips
                .iter()
                .find(|(k, _, _)| matches!(k, hit::FormChipKind::PaletteCard(k) if *k == kind))
                .unwrap_or_else(|| panic!("a card exists for {kind:?}"));
            click(&mut app, card_rect.x, card_rect.y);

            let node = app.working_graph().node("a").expect("node a");
            assert!(
                is_expected_kind(&node.content[1]),
                "{kind:?}'s card inserted the wrong block kind: {:?}",
                node.content[1]
            );
            // Every kind but Divider opens its own form immediately.
            if kind != authoring::BlockKind::Divider {
                assert!(
                    app.open_form().is_some(),
                    "{kind:?}'s new block should open its own form"
                );
            }
        }
    }

    #[test]
    fn cancel_chip_closes_the_palette_without_adding_anything() {
        let mut app = app();
        select_block(&mut app, "a", 0);
        let area = Rect::new(0, 0, 100, 30);
        let areas = hit::editor_areas(area);
        let chips = hit::selected_block_chips(&app);
        let (_, add_below_rect) = hit::chip_rects(areas.hint, &chips)
            .into_iter()
            .find(|(a, _)| *a == hit::BlockAction::AddBelow)
            .expect("an Add below chip exists");
        click(&mut app, add_below_rect.x, add_below_rect.y);
        let layout = hit::form_layout(app.open_form().expect("palette open"), area);
        let (_, _, cancel_rect) = layout
            .chips
            .iter()
            .find(|(kind, _, _)| *kind == hit::FormChipKind::Cancel)
            .expect("a Cancel chip exists");
        click(&mut app, cancel_rect.x, cancel_rect.y);
        assert!(app.open_form().is_none());
        assert_eq!(app.working_graph().node("a").unwrap().content.len(), 2);
        assert!(!app.dirty());
    }

    #[test]
    fn delete_chip_removes_the_block_flashes_and_is_undoable() {
        let mut app = app();
        select_block(&mut app, "a", 1); // the text block
        let area = Rect::new(0, 0, 100, 30);
        let areas = hit::editor_areas(area);
        let chips = hit::selected_block_chips(&app);
        let (_, delete_rect) = hit::chip_rects(areas.hint, &chips)
            .into_iter()
            .find(|(a, _)| *a == hit::BlockAction::Delete)
            .expect("a Delete chip exists");
        click(&mut app, delete_rect.x, delete_rect.y);

        assert_eq!(app.working_graph().node("a").unwrap().content.len(), 1);
        assert_eq!(app.selection(), &Selection::None);
        assert!(
            app.flash().is_some_and(|f| f.text.starts_with("Deleted")),
            "expected a reversible 'Deleted' notice, got {:?}",
            app.flash()
        );

        app.undo();
        let node = app.working_graph().node("a").expect("node a");
        assert_eq!(node.content.len(), 2, "undo restores the deleted block");
        assert_eq!(
            node.content[1],
            ContentBlock::Text {
                reveal: None,
                body: "World".to_owned(),
            }
        );
    }

    #[test]
    fn drag_reorders_blocks_mouse_only() {
        let mut app = app();
        let area = Rect::new(0, 0, 100, 30);
        let areas = hit::editor_areas(area);
        let layout = hit::canvas_layout(&app, areas.canvas).expect("canvas has layout");
        let (start0, _) = layout.block_extents[0];
        let (_, end1) = layout.block_extents[1];

        // Press on block 0 (the heading): selects it and arms the drag.
        click(&mut app, layout.inner.x, layout.inner.y + start0 as u16);
        assert_eq!(app.selection(), &Selection::Block("a".to_owned(), vec![0]));
        assert_eq!(
            app.drag(),
            &DragState::Lifting {
                node: "a".to_owned(),
                path: vec![0],
            }
        );

        // Drag into the bottom half of block 1 — "insert after block 1".
        let drop_row = layout.inner.y + (end1 - 1) as u16;
        drag_to(&mut app, layout.inner.x, drop_row);
        assert_eq!(
            app.drag(),
            &DragState::Over {
                node: "a".to_owned(),
                path: vec![0],
                to: 2,
            }
        );

        release(&mut app, layout.inner.x, drop_row);
        assert_eq!(app.drag(), &DragState::Idle);
        let node = app.working_graph().node("a").expect("node a");
        assert_eq!(
            node.content[0],
            ContentBlock::Text {
                reveal: None,
                body: "World".to_owned(),
            },
            "the text block is now first"
        );
        assert_eq!(
            node.content[1],
            ContentBlock::Heading {
                reveal: None,
                level: 1,
                text: "Hello".to_owned(),
            },
            "the dragged heading is now last"
        );
        assert_eq!(app.selection(), &Selection::Block("a".to_owned(), vec![1]));

        app.undo();
        let node = app.working_graph().node("a").expect("node a");
        assert_eq!(
            node.content[0],
            ContentBlock::Heading {
                reveal: None,
                level: 1,
                text: "Hello".to_owned(),
            },
            "undo restores the original order"
        );
    }

    #[test]
    fn esc_cancels_a_drag_leaving_the_block_where_it_was() {
        let mut app = app();
        let area = Rect::new(0, 0, 100, 30);
        let areas = hit::editor_areas(area);
        let layout = hit::canvas_layout(&app, areas.canvas).expect("canvas has layout");

        click(&mut app, layout.inner.x, layout.inner.y);
        assert_ne!(app.drag(), &DragState::Idle);
        let (_, end1) = layout.block_extents[1];
        drag_to(&mut app, layout.inner.x, layout.inner.y + (end1 - 1) as u16);
        assert!(matches!(app.drag(), DragState::Over { .. }));

        press(&mut app, KeyCode::Esc);
        assert_eq!(app.drag(), &DragState::Idle, "Esc cancels the drag");
        let node = app.working_graph().node("a").expect("node a");
        assert_eq!(
            node.content[0],
            ContentBlock::Heading {
                reveal: None,
                level: 1,
                text: "Hello".to_owned(),
            },
            "cancelling a drag makes no change"
        );
        assert!(!app.dirty());
    }

    #[test]
    fn a_press_and_release_with_no_movement_is_a_plain_click() {
        let mut app = app();
        let area = Rect::new(0, 0, 100, 30);
        let areas = hit::editor_areas(area);
        let layout = hit::canvas_layout(&app, areas.canvas).expect("canvas has layout");
        click(&mut app, layout.inner.x, layout.inner.y);
        assert_eq!(app.selection(), &Selection::Block("a".to_owned(), vec![0]));
        release(&mut app, layout.inner.x, layout.inner.y);
        assert_eq!(app.drag(), &DragState::Idle);
        assert!(
            !app.dirty(),
            "a click with no drag movement changes nothing"
        );
    }

    #[test]
    fn empty_slide_shows_and_resolves_the_add_first_block_target() {
        const EMPTY: &str = r#"{"nodes":[{"id":"a","title":"Blank","content":[]}]}"#;
        let mut app = EditorApp::new(Graph::from_json(EMPTY).expect("fixture parses"));
        app.set_terminal_size(100, 30);
        let screen = draw(&app, 100, 30);
        assert!(screen.contains("Add your first block"));

        let area = Rect::new(0, 0, 100, 30);
        let areas = hit::editor_areas(area);
        click(&mut app, areas.canvas.x + 2, areas.canvas.y + 2);
        assert!(matches!(
            app.open_form(),
            Some(FormState::AddPalette { .. })
        ));
    }

    // ─── US3: restructure the deck (spec 013, T056) ──────────────────────

    fn click_form_chip(app: &mut EditorApp, kind: hit::FormChipKind) {
        let form = app.open_form().expect("a form is open").clone();
        let layout = hit::form_layout(&form, Rect::new(0, 0, 100, 30));
        let (_, _, rect) = layout
            .chips
            .iter()
            .find(|(k, _, _)| *k == kind)
            .unwrap_or_else(|| panic!("chip {kind:?} exists"));
        click(app, rect.x, rect.y);
    }

    fn click_picker_row(app: &mut EditorApp, title: &str) {
        let Some(FormState::SlidePicker { rows, .. }) = app.open_form() else {
            panic!("a slide picker is open");
        };
        let idx = rows
            .iter()
            .position(|r| r.title == title)
            .unwrap_or_else(|| panic!("a picker row titled {title:?} exists"));
        click_form_chip(app, hit::FormChipKind::PickerRow(idx));
    }

    fn click_outline_row(app: &mut EditorApp, row: u16) {
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        click(app, areas.outline.x, areas.outline.y + row);
    }

    fn drag_outline_row(app: &mut EditorApp, from_row: u16, to_row: u16) {
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        click(app, areas.outline.x, areas.outline.y + from_row);
        drag_to(app, areas.outline.x, areas.outline.y + to_row);
        release(app, areas.outline.x, areas.outline.y + to_row);
    }

    fn click_slide_chip(app: &mut EditorApp, action: hit::SlideAction) {
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        let chips = hit::selected_slide_chips(app);
        let (_, rect) = hit::chip_rects(areas.hint, &chips)
            .into_iter()
            .find(|(a, _)| *a == action)
            .unwrap_or_else(|| panic!("slide chip {action:?} exists"));
        click(app, rect.x, rect.y);
    }

    fn click_block_reveal_chip(app: &mut EditorApp) {
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 40));
        let chips = hit::selected_block_chips(app);
        let (_, rect) = hit::chip_rects(areas.hint, &chips)
            .into_iter()
            .find(|(a, _)| *a == hit::BlockAction::Reveal)
            .expect("a Reveal chip exists");
        click(app, rect.x, rect.y);
    }

    /// `[`/`]` select the previous/next outline slide, wrapping — the
    /// keyboard-only counterpart to clicking an outline row (spec 013
    /// US3, ADR-017's keyboard-complete posture).
    #[test]
    fn bracket_keys_select_the_adjacent_slide_and_wrap() {
        let mut app = linear3_app();
        press(&mut app, KeyCode::Char(']'));
        assert_eq!(app.selection(), &Selection::Slide("a".to_owned()));
        press(&mut app, KeyCode::Char(']'));
        assert_eq!(app.selection(), &Selection::Slide("b".to_owned()));
        press(&mut app, KeyCode::Char('['));
        assert_eq!(app.selection(), &Selection::Slide("a".to_owned()));
        press(&mut app, KeyCode::Char('['));
        assert_eq!(
            app.selection(),
            &Selection::Slide("c".to_owned()),
            "wraps backward past the first slide to the last"
        );
    }

    #[test]
    fn outline_new_slide_row_prompts_a_title_and_wires_it_after_the_last_slide() {
        let mut app = linear3_app();
        click_outline_row(&mut app, 3); // the permanent "+ new slide" row
        assert!(matches!(
            app.open_form(),
            Some(FormState::Prompt {
                kind: PromptKind::NewSlide { .. },
                ..
            })
        ));
        type_text(&mut app, "Bonus");
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert!(app.open_form().is_none());
        assert_eq!(app.working_graph().nodes.len(), 4);
        assert_eq!(
            app.working_graph().node("c").unwrap().next_target(),
            Some("bonus"),
            "the new slide is wired after the deck's last slide, which had no next"
        );
        assert_eq!(app.selection(), &Selection::Slide("bonus".to_owned()));
    }

    #[test]
    fn slide_duplicate_and_delete_round_trip() {
        let mut app = linear3_app();
        app.selection = Selection::Slide("b".to_owned());
        click_slide_chip(&mut app, hit::SlideAction::Duplicate);
        assert_eq!(app.working_graph().nodes.len(), 4);
        assert_eq!(app.selection(), &Selection::Slide("middle".to_owned()));

        click_slide_chip(&mut app, hit::SlideAction::Delete);
        assert_eq!(app.working_graph().nodes.len(), 3);
        assert_eq!(app.selection(), &Selection::None);
        assert!(app.working_graph().node("middle").is_none());
    }

    /// Acceptance scenario 2: turn a slide into a branch point with two
    /// named answers, chosen by title from a picker — never a typed id.
    #[test]
    fn turn_into_a_choice_and_add_a_second_answer_via_the_picker() {
        let mut app = linear3_app();
        app.selection = Selection::Slide("a".to_owned());
        click_slide_chip(&mut app, hit::SlideAction::TurnIntoChoice);
        assert!(matches!(
            app.open_form(),
            Some(FormState::Prompt {
                kind: PromptKind::ChoicePrompt { .. },
                ..
            })
        ));
        type_text(&mut app, "Go to the end");
        click_form_chip(&mut app, hit::FormChipKind::ChooseTarget);
        assert!(matches!(
            app.open_form(),
            Some(FormState::SlidePicker { .. })
        ));
        click_picker_row(&mut app, "End");
        assert!(app.open_form().is_none());
        let bp = app
            .working_graph()
            .node("a")
            .unwrap()
            .branch_point()
            .expect("a is now a branch point");
        assert_eq!(bp.options.len(), 1);
        assert_eq!(bp.options[0].label, "Go to the end");
        assert_eq!(bp.options[0].target, "c");
        assert_eq!(app.selection(), &Selection::Slide("a".to_owned()));

        click_slide_chip(&mut app, hit::SlideAction::AddAnswer);
        type_text(&mut app, "Stay in the middle");
        click_form_chip(&mut app, hit::FormChipKind::ChooseTarget);
        click_picker_row(&mut app, "Middle");
        let bp = app
            .working_graph()
            .node("a")
            .unwrap()
            .branch_point()
            .unwrap();
        assert_eq!(bp.options.len(), 2);
        assert_eq!(bp.options[1].label, "Stay in the middle");
        assert_eq!(bp.options[1].target, "b");

        click_slide_chip(&mut app, hit::SlideAction::RemoveAnswer);
        let bp = app
            .working_graph()
            .node("a")
            .unwrap()
            .branch_point()
            .unwrap();
        assert_eq!(bp.options.len(), 1, "removes the last answer");
        assert_eq!(bp.options[0].label, "Go to the end");
        assert!(
            !hit::selected_slide_chips(&app)
                .iter()
                .any(|(a, _)| *a == hit::SlideAction::RemoveAnswer),
            "the chip disappears once only one answer remains"
        );

        click_slide_chip(&mut app, hit::SlideAction::TurnBackIntoSlide);
        assert_eq!(
            app.working_graph().node("a").unwrap().next_target(),
            Some("c"),
            "turning back keeps the first answer's target"
        );
    }

    /// The same build as the mouse-driven test above, but entirely via
    /// the keyboard — `n`/`c`/`a`/`g`/`r` plus a picker's digit-row
    /// shortcuts (spec 013 US3, ADR-017's keyboard-complete posture).
    #[test]
    fn keyboard_only_new_slide_choice_and_reveal_match_the_mouse_path() {
        let mut app = linear3_app();
        press(&mut app, KeyCode::Char(']')); // selects "a"
        press(&mut app, KeyCode::Char('n')); // new-slide prompt, after "a"
        type_text(&mut app, "Bonus");
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert_eq!(app.selection(), &Selection::Slide("bonus".to_owned()));
        assert_eq!(app.working_graph().nodes.len(), 4);

        app.selection = Selection::Slide("a".to_owned());
        press(&mut app, KeyCode::Char('c')); // turn into a choice
        type_text(&mut app, "Go to the end");
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL); // -> Choose target picker
        assert!(matches!(
            app.open_form(),
            Some(FormState::SlidePicker { .. })
        ));
        let end_row = {
            let Some(FormState::SlidePicker { rows, .. }) = app.open_form() else {
                panic!("picker open");
            };
            rows.iter().position(|r| r.title == "End").unwrap() + 1 // 1-based digit
        };
        press(
            &mut app,
            KeyCode::Char(char::from_digit(end_row as u32, 10).unwrap()),
        );
        assert!(app.open_form().is_none());
        let bp = app
            .working_graph()
            .node("a")
            .unwrap()
            .branch_point()
            .unwrap();
        assert_eq!(bp.options[0].target, "c");

        app.selection = Selection::Slide("a".to_owned());
        press(&mut app, KeyCode::Char('a')); // add another answer
        type_text(&mut app, "Bonus round");
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        let bonus_row = {
            let Some(FormState::SlidePicker { rows, .. }) = app.open_form() else {
                panic!("picker open");
            };
            rows.iter().position(|r| r.title == "Bonus").unwrap() + 1
        };
        press(
            &mut app,
            KeyCode::Char(char::from_digit(bonus_row as u32, 10).unwrap()),
        );
        let bp = app
            .working_graph()
            .node("a")
            .unwrap()
            .branch_point()
            .unwrap();
        assert_eq!(bp.options.len(), 2);
        assert_eq!(bp.options[1].target, "bonus");

        // `g` on a non-branch slide rewires "goes to" via the keyboard too.
        app.selection = Selection::Slide("b".to_owned());
        press(&mut app, KeyCode::Char('g'));
        assert!(matches!(
            app.open_form(),
            Some(FormState::SlidePicker { .. })
        ));
        press(&mut app, KeyCode::Char('n')); // "a new slide…" row
        assert_eq!(
            app.working_graph().node("b").unwrap().next_target(),
            Some("new-slide")
        );

        select_block(&mut app, "a", 0); // "a"'s original content block, untouched by turning it into a choice
        press(&mut app, KeyCode::Char('r'));
        assert_eq!(
            app.working_graph().node("a").unwrap().content[0].reveal(),
            Some(1)
        );
    }

    /// Acceptance scenario 3 (happy path): dragging a slide within a
    /// straight run reorders it and the wiring follows.
    #[test]
    fn outline_drag_reorders_a_linear_run() {
        let mut app = linear3_app();
        drag_outline_row(&mut app, 2, 1); // drag "c" (row 2) to before "b" (row 1)
        assert_eq!(
            app.working_graph().node("a").unwrap().next_target(),
            Some("c")
        );
        assert_eq!(
            app.working_graph().node("c").unwrap().next_target(),
            Some("b")
        );
        assert!(app.working_graph().node("b").unwrap().is_terminal());
        assert_eq!(app.selection(), &Selection::Slide("c".to_owned()));
    }

    /// Acceptance scenario 3 (refusal): dragging a branch-answer's target
    /// across the branch boundary is refused with an explanation and a
    /// clickable way to go fix it at the source instead.
    #[test]
    fn outline_drag_across_a_branch_boundary_refuses_with_a_jump_link() {
        let mut app = branch_app();
        drag_outline_row(&mut app, 1, 2); // drag "b" (row 1) to before "c" (row 2)
        assert_eq!(
            app.working_graph(),
            &Graph::from_json(BRANCH).unwrap(),
            "a refused reorder changes nothing"
        );
        let flash = app.flash().expect("a refusal flash is shown");
        assert!(flash.text.contains("reached only through a branch answer"));
        assert_eq!(
            flash.action,
            Some(hit::FlashAction::SelectSlide("a".to_owned()))
        );

        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        click(&mut app, areas.hint.x, areas.hint.y);
        assert_eq!(app.selection(), &Selection::Slide("a".to_owned()));
        assert!(app.flash().is_none());
    }

    /// Acceptance scenario 4: reveal steps stay consecutive from one
    /// regardless of the order they were assigned in.
    #[test]
    fn reveal_chip_cycles_through_consecutive_steps() {
        let mut app = all_kinds_app();
        select_block(&mut app, "a", 0);
        click_block_reveal_chip(&mut app);
        assert_eq!(
            app.working_graph().node("a").unwrap().content[0].reveal(),
            Some(1)
        );

        select_block(&mut app, "a", 1);
        click_block_reveal_chip(&mut app);
        assert_eq!(
            app.working_graph().node("a").unwrap().content[1].reveal(),
            Some(1),
            "a block with no prior step joins the first existing one"
        );
        click_block_reveal_chip(&mut app);
        assert_eq!(
            app.working_graph().node("a").unwrap().content[1].reveal(),
            Some(2),
            "cycling again starts a new, later step"
        );
        click_block_reveal_chip(&mut app);
        assert_eq!(
            app.working_graph().node("a").unwrap().content[1].reveal(),
            None,
            "cycling past the last step returns to none"
        );
        assert_eq!(
            app.working_graph().node("a").unwrap().reveal_levels(),
            vec![1]
        );
    }

    /// Acceptance scenario 1: choosing the next slide by name from a
    /// picker, including switching to "nothing — an ending."
    #[test]
    fn goes_to_picker_rewires_next_and_can_clear_it_to_an_ending() {
        let mut app = linear3_app();
        app.selection = Selection::Slide("a".to_owned());
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        let node = app.working_graph().node("a").unwrap();
        let text = hit::wiring_summary(app.working_graph(), node);
        let rect = hit::wiring_change_rect(areas.wiring, text.chars().count() as u16);
        click(&mut app, rect.x, rect.y);
        assert!(matches!(
            app.open_form(),
            Some(FormState::SlidePicker { .. })
        ));
        click_picker_row(&mut app, "End");
        assert_eq!(
            app.working_graph().node("a").unwrap().next_target(),
            Some("c")
        );

        app.selection = Selection::Slide("a".to_owned());
        let node = app.working_graph().node("a").unwrap();
        let text = hit::wiring_summary(app.working_graph(), node);
        let rect = hit::wiring_change_rect(areas.wiring, text.chars().count() as u16);
        click(&mut app, rect.x, rect.y);
        click_form_chip(&mut app, hit::FormChipKind::PickerEnding);
        assert!(app.working_graph().node("a").unwrap().is_terminal());
    }

    #[test]
    fn answer_chip_retargets_an_existing_branch_answer() {
        let mut app = branch_app();
        app.selection = Selection::Slide("a".to_owned());
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        let spans =
            hit::wiring_answer_spans(app.working_graph(), app.working_graph().node("a").unwrap());
        let first = &spans[0];
        click(&mut app, areas.wiring.x + first.start, areas.wiring.y);
        assert!(matches!(
            app.open_form(),
            Some(FormState::SlidePicker { .. })
        ));
        click_picker_row(&mut app, "C slide");
        let bp = app
            .working_graph()
            .node("a")
            .unwrap()
            .branch_point()
            .unwrap();
        assert_eq!(bp.options[0].target, "c");
    }

    #[test]
    fn toolbar_title_rename_and_slide_notes_round_trip() {
        let mut app = linear3_app();
        let areas = hit::editor_areas(Rect::new(0, 0, 100, 30));
        click(&mut app, areas.toolbar.x, areas.toolbar.y);
        assert!(matches!(
            app.open_form(),
            Some(FormState::Prompt {
                kind: PromptKind::DeckTitle,
                ..
            })
        ));
        type_text(&mut app, "My Great Talk");
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert_eq!(app.working_graph().title.as_deref(), Some("My Great Talk"));

        app.selection = Selection::Slide("a".to_owned());
        click_slide_chip(&mut app, hit::SlideAction::Notes);
        type_text(&mut app, "Smile and slow down");
        press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert_eq!(
            app.working_graph()
                .node("a")
                .unwrap()
                .speaker_notes
                .as_deref(),
            Some("Smile and slow down")
        );
    }
}
