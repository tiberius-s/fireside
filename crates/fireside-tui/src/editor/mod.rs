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
use fireside_engine::authoring::{self, BlockPath, Op};
use fireside_engine::validate;
use ratatui::layout::Rect;

use fireside_core::{ContainerLayout, ContentBlock, Graph};

use crate::app::App as PresenterApp;
use crate::app::FlashKind;
use crate::error::TuiError;
use crate::{WriteBackError, render};

use forms::{CodeFocus, EditableField, FormState, PictureFocus, TextArtFocus};

/// What's selected in the studio, if anything.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum Selection {
    #[default]
    None,
    Slide(String),
    Block(String, BlockPath),
}

/// A block or slide drag in progress. Only `Idle` exists until US2/US3
/// (T044/T050) add the lifting/hovering states drag-and-drop needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum DragState {
    #[default]
    Idle,
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

/// All authoring-editor state (spec 013, `data-model.md`'s `EditorApp`
/// section).
#[derive(Debug)]
pub(crate) struct EditorApp {
    working_graph: Graph,
    saved_graph: Graph,
    selection: Selection,
    #[allow(dead_code)] // read once drag-and-drop (US2, T044) lands
    drag: DragState,
    open_form: Option<FormState>,
    history: Vec<HistorySnapshot>,
    redo: Vec<HistorySnapshot>,
    terminal_size: (u16, u16),
    status: Vec<fireside_engine::Diagnostic>,
    scroll: u16,
    hover: Option<hit::Target>,
    #[allow(dead_code)] // read once draft autosave (US4, T060) lands
    dirty_since_draft: bool,
    #[allow(dead_code)] // read once draft autosave (US4, T060) lands
    last_draft_write: Instant,
    showing_help: bool,
    present_requested: bool,
    pending_save: Option<Graph>,
    pending_art_request: Option<String>,
    flash: Option<Flash>,
    quit: bool,
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
            hover: None,
            dirty_since_draft: false,
            last_draft_write: Instant::now(),
            showing_help: false,
            present_requested: false,
            pending_save: None,
            pending_art_request: None,
            flash: None,
            quit: false,
        }
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

    /// Forward-declared accessor: drag-and-drop (US2, T044) is the first
    /// thing that reads this outside tests.
    #[must_use]
    #[allow(dead_code)]
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
            FormState::Container { .. } => None,
        }
    }

    /// Whether the currently focused field is single-line (Enter never
    /// inserts a newline into one — see `forms::EditableField::single_line`).
    fn focused_field_is_single_line(&self) -> bool {
        matches!(
            &self.open_form,
            Some(FormState::Code {
                focus: CodeFocus::Language,
                ..
            }) | Some(FormState::Picture { .. })
                | Some(FormState::TextArt {
                    focus: TextArtFocus::Alt,
                    ..
                })
        )
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
            _ => {}
        }
    }

    // ─── Ops, undo, save (spec 013, US1) ────────────────────────────────

    /// Applies `op`, pushing the pre-op state onto `history` first (capped
    /// at 100, per spec FR-016) and clearing `redo` — the sole path any op
    /// reaches `working_graph` through. A precondition failure flashes the
    /// error rather than touching `working_graph` (`authoring::apply` is
    /// atomic: `Err` never partially mutates).
    fn apply_op(&mut self, op: Op) {
        match authoring::apply(&self.working_graph, &op) {
            Ok(next) => {
                self.push_history();
                self.working_graph = next;
                self.redo.clear();
            }
            Err(err) => self.set_flash(err.to_string(), FlashKind::Error),
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
                self.set_flash("Saved", FlashKind::Info);
            }
            Err(message) => self.set_flash(message, FlashKind::Error),
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

    fn on_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
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
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.request_save();
            }
            KeyCode::Esc => {
                if self.selection != Selection::None {
                    self.selection = Selection::None;
                }
            }
            KeyCode::Char('?') => self.showing_help = true,
            KeyCode::Char('p' | 'P') => self.present_requested = true,
            KeyCode::Char('u' | 'U') => self.undo(),
            KeyCode::Enter => self.open_form_for_selection(),
            KeyCode::Tab => self.select_adjacent_block(false),
            KeyCode::BackTab => self.select_adjacent_block(true),
            KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
            _ => {}
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
            self.commit_form();
            return;
        }
        if matches!(key.code, KeyCode::Tab | KeyCode::BackTab) {
            self.form_cycle_field();
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
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => self.on_click(event.column, event.row),
            MouseEventKind::Moved => {
                let (w, h) = self.terminal_size;
                self.hover = hit::hit(self, Rect::new(0, 0, w, h), event.column, event.row);
            }
            MouseEventKind::ScrollDown => self.scroll = self.scroll.saturating_add(1),
            MouseEventKind::ScrollUp => self.scroll = self.scroll.saturating_sub(1),
            _ => {}
        }
    }

    fn on_click(&mut self, col: u16, row: u16) {
        let (w, h) = self.terminal_size;
        match hit::hit(self, Rect::new(0, 0, w, h), col, row) {
            Some(hit::Target::OutlineRow(id)) => {
                self.selection = Selection::Slide(id);
                self.scroll = 0;
            }
            Some(hit::Target::Block(node_id, path)) => {
                self.selection = Selection::Block(node_id, path);
            }
            Some(hit::Target::BlockChip(node, path, hit::BlockAction::Edit)) => {
                self.open_form_at(&node, &path);
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
            Some(hit::Target::FormChip(kind)) => self.on_form_chip(kind),
            Some(hit::Target::FormField(slot)) => self.focus_form_field(slot),
            // Add-slide isn't wired until US3 — the chip resolves but is a
            // no-op for now (design brief: "no mutations" beyond a wave's
            // own scope).
            Some(
                hit::Target::BlockChip(..)
                | hit::Target::ToolbarChip(hit::ToolbarAction::AddSlide)
                | hit::Target::OutlineNewSlide
                | hit::Target::InsertionSlot(..)
                | hit::Target::GoesToChip(_)
                | hit::Target::StatusBanner,
            ) => {}
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

/// Opens the full-screen authoring studio (spec 013) over `graph`: sets up
/// the terminal, runs the editor's own event loop, and always restores the
/// terminal, even on error — the same contract [`crate::present`] gives
/// the presenter.
///
/// # Errors
///
/// Returns [`TuiError::NotATty`] outside an interactive terminal and
/// [`TuiError::Io`] for terminal failures.
pub fn run(
    graph: Graph,
    sink: EditorWriteBackSink<'_>,
    art_generator: Option<ArtGenerator<'_>>,
) -> Result<(), TuiError> {
    if !io::stdout().is_tty() || !io::stdin().is_tty() {
        return Err(TuiError::NotATty);
    }
    let mut app = EditorApp::new(graph);
    let mut terminal = ratatui::try_init()?;
    // Mouse capture is enabled once for the whole editor session — both
    // the studio's own loop and the in-process presenter loop `present_now`
    // enters share it, per research.md §6.
    let _ = execute!(io::stdout(), EnableMouseCapture);
    let result = editor_event_loop(&mut terminal, &mut app, sink, art_generator);
    let _ = execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

fn editor_event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut EditorApp,
    sink: EditorWriteBackSink<'_>,
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

    fn press(app: &mut EditorApp, code: KeyCode) {
        app.update(Msg::Terminal(Event::Key(KeyEvent::from(code))));
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
}
