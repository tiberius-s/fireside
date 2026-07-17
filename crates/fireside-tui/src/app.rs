//! The presenter application state machine.
//!
//! TEA-style: [`App::update`] is the **only** place state mutates. It
//! receives terminal events (keys, resizes) and applies them; rendering
//! reads the state and draws. Every keypress that cannot act produces a
//! flash message — the presenter is never left wondering whether a key
//! "worked".

use std::time::{Duration, Instant};

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use fireside_core::{ContentBlock, Graph, Node, Transition, ViewMode};
use fireside_engine::{Outcome, Session, Severity, validate};
use ratatui::layout::Rect;

use crate::render;

/// How long feedback messages stay on screen.
const FLASH_DURATION: Duration = Duration::from_millis(3000);

/// How long a slide's fade-in lasts: one dim beat, then full brightness.
const FADE_DURATION: Duration = Duration::from_millis(90);

/// A message into the state machine: terminal input, or a fresh read of
/// the deck source while presenting (live reload).
#[derive(Debug)]
pub enum Msg {
    /// A terminal event (key press, resize).
    Terminal(Event),
    /// The deck file changed on disk and was re-read: a new graph, or a
    /// human-readable message about why it could not be loaded.
    Reload(Result<Graph, String>),
    /// The write-back sink's response to a quick-edit save: success, or a
    /// human-readable message about why it could not be saved.
    SaveResult(Result<(), String>),
}

/// Which screen the presenter is looking at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    /// The slide itself.
    Present,
    /// The key reference overlay.
    Help,
    /// The map: every slide, visited markers, jump on Enter.
    Map {
        /// Index of the highlighted node.
        selected: usize,
    },
    /// The quick-edit modal: one editable field per heading/text block on
    /// the current node (ADR-005 — content-only, no structural edits).
    Edit {
        /// One entry per editable block found on the current node.
        fields: Vec<EditableField>,
        /// Index into `fields` of the block currently being typed into.
        focused: usize,
    },
}

/// Addresses one `ContentBlock` within a node's content tree: the sequence
/// of indices from the root `content` array down through any nested
/// `Container::children`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPath {
    indices: Vec<usize>,
}

/// Which kind of editable block an [`EditableField`] represents — carried
/// only for the modal's label, since heading and text edit identically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditableKind {
    /// A heading block at the given level (1-6).
    Heading(u8),
    /// A prose text block.
    Text,
}

/// One editable heading/text block in the quick-edit modal, plus its
/// in-progress edit buffer. Discarded entirely when the modal closes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditableField {
    path: BlockPath,
    pub kind: EditableKind,
    /// Multi-line buffer, initialized from the block's current content.
    pub buffer: Vec<String>,
    /// (row, column) into `buffer`, in characters (not bytes).
    pub cursor: (usize, usize),
}

impl EditableField {
    fn char_len(&self, row: usize) -> usize {
        self.buffer[row].chars().count()
    }

    fn byte_offset(&self, row: usize, col: usize) -> usize {
        self.buffer[row]
            .char_indices()
            .nth(col)
            .map_or(self.buffer[row].len(), |(b, _)| b)
    }

    fn insert_char(&mut self, c: char) {
        let (row, col) = self.cursor;
        let idx = self.byte_offset(row, col);
        self.buffer[row].insert(idx, c);
        self.cursor.1 += 1;
    }

    fn newline(&mut self) {
        let (row, col) = self.cursor;
        let idx = self.byte_offset(row, col);
        let rest = self.buffer[row].split_off(idx);
        self.buffer.insert(row + 1, rest);
        self.cursor = (row + 1, 0);
    }

    fn backspace(&mut self) {
        let (row, col) = self.cursor;
        if col > 0 {
            let start = self.byte_offset(row, col - 1);
            let end = self.byte_offset(row, col);
            self.buffer[row].replace_range(start..end, "");
            self.cursor.1 -= 1;
        } else if row > 0 {
            let line = self.buffer.remove(row);
            let prev_len = self.char_len(row - 1);
            self.buffer[row - 1].push_str(&line);
            self.cursor = (row - 1, prev_len);
        }
    }

    fn delete(&mut self) {
        let (row, col) = self.cursor;
        if col < self.char_len(row) {
            let start = self.byte_offset(row, col);
            let end = self.byte_offset(row, col + 1);
            self.buffer[row].replace_range(start..end, "");
        } else if row + 1 < self.buffer.len() {
            let next = self.buffer.remove(row + 1);
            self.buffer[row].push_str(&next);
        }
    }

    fn move_left(&mut self) {
        let (row, col) = self.cursor;
        if col > 0 {
            self.cursor.1 -= 1;
        } else if row > 0 {
            self.cursor = (row - 1, self.char_len(row - 1));
        }
    }

    fn move_right(&mut self) {
        let (row, col) = self.cursor;
        if col < self.char_len(row) {
            self.cursor.1 += 1;
        } else if row + 1 < self.buffer.len() {
            self.cursor = (row + 1, 0);
        }
    }

    /// Moves the cursor up a line; `false` at the first line means the
    /// caller should move focus to the previous field instead.
    fn move_up(&mut self) -> bool {
        let (row, col) = self.cursor;
        if row == 0 {
            return false;
        }
        self.cursor = (row - 1, col.min(self.char_len(row - 1)));
        true
    }

    /// Moves the cursor down a line; `false` at the last line means the
    /// caller should move focus to the next field instead.
    fn move_down(&mut self) -> bool {
        let (row, col) = self.cursor;
        if row + 1 >= self.buffer.len() {
            return false;
        }
        self.cursor = (row + 1, col.min(self.char_len(row + 1)));
        true
    }

    /// The buffer joined back into the single-string form the protocol
    /// stores (`Heading::text` / `Text::body`).
    fn text(&self) -> String {
        self.buffer.join("\n")
    }
}

/// Every heading/text block on `node`, in document order, including those
/// nested inside `Container` children — the set the quick-edit modal
/// offers (ADR-005: content-only, current node only).
pub(crate) fn editable_fields(node: &Node) -> Vec<EditableField> {
    let mut fields = Vec::new();
    collect_editable(&node.content, &mut Vec::new(), &mut fields);
    fields
}

fn collect_editable(blocks: &[ContentBlock], path: &mut Vec<usize>, out: &mut Vec<EditableField>) {
    for (i, block) in blocks.iter().enumerate() {
        path.push(i);
        match block {
            ContentBlock::Heading { level, text, .. } => out.push(EditableField {
                path: BlockPath {
                    indices: path.clone(),
                },
                kind: EditableKind::Heading(*level),
                buffer: to_buffer(text),
                cursor: (0, 0),
            }),
            ContentBlock::Text { body, .. } => out.push(EditableField {
                path: BlockPath {
                    indices: path.clone(),
                },
                kind: EditableKind::Text,
                buffer: to_buffer(body),
                cursor: (0, 0),
            }),
            ContentBlock::Container { children, .. } => collect_editable(children, path, out),
            _ => {}
        }
        path.pop();
    }
}

fn to_buffer(text: &str) -> Vec<String> {
    if text.is_empty() {
        vec![String::new()]
    } else {
        text.split('\n').map(str::to_owned).collect()
    }
}

/// The `ContentBlock` at `path` within `blocks`, recursing into container
/// children — the write-side counterpart to `collect_editable`'s addressing.
fn block_at_mut<'a>(
    blocks: &'a mut [ContentBlock],
    path: &[usize],
) -> Option<&'a mut ContentBlock> {
    let (&first, rest) = path.split_first()?;
    let block = blocks.get_mut(first)?;
    if rest.is_empty() {
        Some(block)
    } else if let ContentBlock::Container { children, .. } = block {
        block_at_mut(children, rest)
    } else {
        None
    }
}

/// The tone of a flash message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashKind {
    /// Neutral guidance.
    Info,
    /// Something was refused.
    Error,
}

/// A transient feedback message shown in the footer.
#[derive(Debug, Clone)]
pub struct Flash {
    /// The message text.
    pub text: String,
    /// Its tone.
    pub kind: FlashKind,
    expires: Instant,
}

/// All presenter state.
#[derive(Debug)]
pub struct App {
    session: Session,
    screen: Screen,
    branch_selected: usize,
    scroll: u16,
    view_override: Option<ViewMode>,
    show_notes: bool,
    show_timer: bool,
    started: Instant,
    flash: Option<Flash>,
    fade_started: Option<Instant>,
    viewport: (u16, u16),
    quit: bool,
    pending_save: Option<Graph>,
}

impl App {
    /// Create the app over a live session.
    #[must_use]
    pub fn new(session: Session) -> Self {
        Self {
            session,
            screen: Screen::Present,
            branch_selected: 0,
            scroll: 0,
            view_override: None,
            show_notes: false,
            show_timer: false,
            started: Instant::now(),
            flash: None,
            fade_started: None,
            viewport: (80, 24),
            quit: false,
            pending_save: None,
        }
    }

    /// The live session.
    #[must_use]
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// The active screen.
    #[must_use]
    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    /// Takes the graph produced by a quick-edit save, if one is pending —
    /// the event loop hands it to the write-back sink and never leaves it
    /// unconsumed, matching the `ReloadSource` poll pattern.
    #[must_use]
    pub(crate) fn take_pending_save(&mut self) -> Option<Graph> {
        self.pending_save.take()
    }

    /// Index of the highlighted branch option.
    #[must_use]
    pub fn branch_selected(&self) -> usize {
        self.branch_selected
    }

    /// Current content scroll offset in lines.
    #[must_use]
    pub fn scroll(&self) -> u16 {
        self.scroll
    }

    /// Whether the speaker-notes panel is open.
    #[must_use]
    pub fn show_notes(&self) -> bool {
        self.show_notes
    }

    /// Whether the elapsed timer is on screen.
    #[must_use]
    pub fn show_timer(&self) -> bool {
        self.show_timer
    }

    /// Time since the presentation started.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// The active flash message, if it has not expired.
    #[must_use]
    pub fn flash(&self) -> Option<&Flash> {
        self.flash.as_ref().filter(|f| f.expires > Instant::now())
    }

    /// Whether the event loop should exit.
    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.quit
    }

    /// Whether the current slide is inside its brief fade-in window. The
    /// renderer dims the slide while this holds; the event loop polls fast
    /// so the brighten lands on time.
    #[must_use]
    pub fn fading(&self) -> bool {
        self.fade_started
            .is_some_and(|started| started.elapsed() < FADE_DURATION)
    }

    /// The view mode in effect: the presenter's runtime toggle wins over the
    /// document (spec: the node-level value is a suggestion, not a
    /// constraint).
    #[must_use]
    pub fn view_mode(&self) -> ViewMode {
        self.view_override.unwrap_or_else(|| {
            self.session
                .current()
                .resolved_view_mode(self.session.defaults())
        })
    }

    /// Apply one message. The sole mutation point.
    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Terminal(Event::Resize(w, h)) => self.viewport = (w, h),
            Msg::Terminal(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                self.on_key(key);
            }
            Msg::Terminal(Event::Mouse(mouse)) => self.on_mouse(mouse),
            Msg::Terminal(_) => {}
            Msg::Reload(result) => self.on_reload(result),
            Msg::SaveResult(result) => self.on_save_result(result),
        }
    }

    /// Surfaces the write-back sink's outcome via the same flash mechanism
    /// every other keypress uses — a save is never a silent no-op. On
    /// success the modal closes (`save_edit` left it open pending this
    /// result). On failure the modal stays open with the presenter's edit
    /// intact — a conflict or I/O error means "not saved", not "abandon
    /// your edit"; the presenter can retry (Ctrl+S again) or cancel (Esc)
    /// once they've read why (FR-013).
    fn on_save_result(&mut self, result: Result<(), String>) {
        match result {
            Ok(()) => {
                self.screen = Screen::Present;
                self.set_flash("Saved", FlashKind::Info);
            }
            Err(message) => self.set_flash(&message, FlashKind::Error),
        }
    }

    /// Swap in a re-read deck without losing the presenter's place. A save
    /// that broke the deck never replaces the working one — the presenter
    /// keeps the old slides and a footer message says what happened.
    fn on_reload(&mut self, result: Result<Graph, String>) {
        let graph = match result {
            Ok(graph) => graph,
            Err(message) => {
                self.set_flash(&message, FlashKind::Error);
                return;
            }
        };
        let errors = validate(&graph)
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        if errors > 0 {
            let word = if errors == 1 { "problem" } else { "problems" };
            self.set_flash(
                &format!("Reload skipped — the saved deck has {errors} {word}; fix and save again"),
                FlashKind::Error,
            );
            return;
        }
        let here = self.session.current().id.clone();
        let Ok(mut session) = Session::new(graph) else {
            self.set_flash(
                "Reload skipped — the saved deck has no slides",
                FlashKind::Error,
            );
            return;
        };
        let survived = session.graph().node(&here).is_some();
        if survived && session.current().id != here {
            let _ = session.goto(&here);
        }
        self.session = session;
        self.scroll = 0;
        self.branch_selected = 0;
        self.fade_started = None;
        if survived {
            self.set_flash("Reloaded", FlashKind::Info);
        } else {
            self.set_flash(
                &format!("Reloaded — \"{here}\" is gone, back at the start"),
                FlashKind::Info,
            );
        }
    }

    fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        match &self.screen {
            Screen::Help => self.screen = Screen::Present,
            Screen::Map { selected } => {
                let selected = *selected;
                self.on_map_key(key.code, selected);
            }
            Screen::Present => self.on_present_key(key.code),
            Screen::Edit { .. } => self.on_edit_key(key),
        }
    }

    /// A mouse click, additive on top of every existing keyboard control
    /// (constitution Principle II: the footer stays the primary, taught
    /// contract). Only a left-button press is a "click" — other buttons and
    /// release/drag events are ignored. Hit-testing recomputes the exact
    /// same pure layout `render::draw` used for the last frame
    /// (`render::map_row_hit`/`branch_option_hit`), so a click can never
    /// land somewhere the screen doesn't actually show a target: clicking
    /// blank space, body text, or (since the branch menu itself is not
    /// drawn while reveal is pending) a branch option that hasn't appeared
    /// yet, is always a safe no-op.
    fn on_mouse(&mut self, event: MouseEvent) {
        if event.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        let (col, row) = (event.column, event.row);
        let (w, h) = self.viewport;
        let frame_area = Rect::new(0, 0, w, h);
        match &self.screen {
            Screen::Map { selected } => {
                let selected = *selected;
                if let Some(idx) = render::map_row_hit(self, frame_area, selected, col, row) {
                    let id = self.session.graph().nodes[idx].id.clone();
                    self.screen = Screen::Present;
                    if id != self.session.current().id {
                        let outcome = self.session.goto(&id);
                        self.apply(&outcome);
                    }
                }
            }
            Screen::Present
                if self.session.branch_point().is_some() && !self.session.has_pending_reveal() =>
            {
                if let Some(idx) = render::branch_option_hit(self, frame_area, col, row) {
                    let outcome = self.session.choose(idx);
                    self.apply(&outcome);
                }
            }
            _ => {}
        }
    }

    fn on_map_key(&mut self, code: KeyCode, selected: usize) {
        let count = self.session.graph().nodes.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.screen = Screen::Map {
                    selected: selected.saturating_sub(1),
                };
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.screen = Screen::Map {
                    selected: (selected + 1).min(count.saturating_sub(1)),
                };
            }
            KeyCode::Enter => {
                let id = self.session.graph().nodes[selected].id.clone();
                self.screen = Screen::Present;
                if id != self.session.current().id {
                    let outcome = self.session.goto(&id);
                    self.apply(&outcome);
                }
            }
            KeyCode::Esc | KeyCode::Char('m' | 'g' | 'q') => self.screen = Screen::Present,
            _ => {}
        }
    }

    fn on_present_key(&mut self, code: KeyCode) {
        let pending_reveal = self.session.has_pending_reveal();
        // While a node has reveal steps not yet shown, the branch menu is
        // not reachable at all — a presenter cannot skip ahead to a
        // choice by choosing early. What would otherwise be a
        // branch-selection keypress instead continues revealing.
        let at_branch = self.session.branch_point().is_some() && !pending_reveal;
        match code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('?' | 'h') => self.screen = Screen::Help,
            KeyCode::Char('m' | 'g') => {
                let current = self.session.current().id.clone();
                let selected = self
                    .session
                    .graph()
                    .nodes
                    .iter()
                    .position(|n| n.id == current)
                    .unwrap_or(0);
                self.screen = Screen::Map { selected };
            }
            KeyCode::Char('f') => {
                let next = match self.view_mode() {
                    ViewMode::Default => ViewMode::Fullscreen,
                    ViewMode::Fullscreen => ViewMode::Default,
                };
                self.view_override = Some(next);
                self.set_flash(
                    match next {
                        ViewMode::Fullscreen => "Fullscreen — press f to exit",
                        ViewMode::Default => "Standard view",
                    },
                    FlashKind::Info,
                );
            }
            KeyCode::Char('s') => {
                if self.session.current().speaker_notes.is_some() {
                    self.show_notes = !self.show_notes;
                } else {
                    self.set_flash("This slide has no speaker notes", FlashKind::Info);
                }
            }
            KeyCode::Char('t') => self.show_timer = !self.show_timer,
            KeyCode::Char('e') => self.open_edit(),
            _ if at_branch => self.on_branch_key(code),
            _ if pending_reveal => self.on_reveal_pending_key(code),
            _ => self.on_flow_key(code),
        }
    }

    /// Keys on a node with reveal steps still pending. Only the explicit
    /// "back" keys retreat; every other key — including ones that would
    /// normally choose a branch option — continues revealing, so a
    /// presenter reaching for the choice never hits a dead keypress
    /// (FR-007: attempting to choose early continues revealing).
    fn on_reveal_pending_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Left | KeyCode::Backspace | KeyCode::PageUp | KeyCode::Char('p') => {
                let outcome = self.session.back();
                self.apply(&outcome);
            }
            _ => {
                let outcome = self.session.next();
                self.apply(&outcome);
            }
        }
    }

    /// Opens the quick-edit modal on the current node's heading/text
    /// blocks, or flashes that there is nothing to edit (ADR-005 scope:
    /// content-only, current node only).
    fn open_edit(&mut self) {
        let fields = editable_fields(self.session.current());
        if fields.is_empty() {
            self.set_flash("This slide has no editable text", FlashKind::Info);
            return;
        }
        self.screen = Screen::Edit { fields, focused: 0 };
    }

    /// Keys while the quick-edit modal is open.
    fn on_edit_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.screen = Screen::Present;
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            self.save_edit();
            return;
        }
        let Screen::Edit { fields, focused } = &mut self.screen else {
            return;
        };
        match key.code {
            KeyCode::Char(c) => fields[*focused].insert_char(c),
            KeyCode::Enter => fields[*focused].newline(),
            KeyCode::Backspace => fields[*focused].backspace(),
            KeyCode::Delete => fields[*focused].delete(),
            KeyCode::Left => fields[*focused].move_left(),
            KeyCode::Right => fields[*focused].move_right(),
            KeyCode::Up if !fields[*focused].move_up() && *focused > 0 => *focused -= 1,
            KeyCode::Down if !fields[*focused].move_down() && *focused + 1 < fields.len() => {
                *focused += 1;
            }
            _ => {}
        }
    }

    /// Builds an edited graph from the modal's fields and hands it to the
    /// event loop as a pending save — `App` never touches the filesystem
    /// itself (crate boundary: `fireside-tui` has no file I/O). Leaves the
    /// modal open: `on_save_result` closes it on success and keeps it open
    /// with the edit intact on failure, so a conflict or I/O error is
    /// retryable rather than a silent loss of the presenter's edit.
    fn save_edit(&mut self) {
        let Screen::Edit { fields, .. } = &self.screen else {
            return;
        };
        let mut graph = self.session.graph().clone();
        let current_id = self.session.current().id.clone();
        if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == current_id) {
            for field in fields {
                if let Some(block) = block_at_mut(&mut node.content, &field.path.indices) {
                    match block {
                        ContentBlock::Heading { text, .. } => *text = field.text(),
                        ContentBlock::Text { body, .. } => *body = field.text(),
                        _ => {}
                    }
                }
            }
        }
        self.pending_save = Some(graph);
    }

    /// Keys while the current node presents a choice.
    fn on_branch_key(&mut self, code: KeyCode) {
        let count = self
            .session
            .branch_point()
            .map(|bp| bp.options.len())
            .unwrap_or(0);
        match code {
            KeyCode::Up | KeyCode::Char('k') if count > 0 => {
                self.branch_selected = (self.branch_selected + count - 1) % count;
            }
            KeyCode::Down | KeyCode::Char('j') if count > 0 => {
                self.branch_selected = (self.branch_selected + 1) % count;
            }
            KeyCode::Enter => {
                let outcome = self.session.choose(self.branch_selected);
                self.apply(&outcome);
            }
            KeyCode::Char(c @ '1'..='9') => {
                let idx = (c as usize) - ('1' as usize);
                if idx < count {
                    let outcome = self.session.choose(idx);
                    self.apply(&outcome);
                } else {
                    self.set_flash(&format!("There are only {count} choices"), FlashKind::Error);
                }
            }
            KeyCode::Char(' ') | KeyCode::Right | KeyCode::PageDown | KeyCode::Char('n') => {
                let outcome = self.session.next();
                self.apply(&outcome);
            }
            KeyCode::Left | KeyCode::Backspace | KeyCode::PageUp | KeyCode::Char('p') => {
                let outcome = self.session.back();
                self.apply(&outcome);
            }
            KeyCode::Char(c) if c.is_alphanumeric() => match self.option_for_key(c) {
                Some(idx) => {
                    let outcome = self.session.choose(idx);
                    self.apply(&outcome);
                }
                None => self.set_flash(&format!("No choice on key '{c}'"), FlashKind::Error),
            },
            _ => {}
        }
    }

    /// Keys on an ordinary (non-branch) node.
    fn on_flow_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(' ' | 'n') | KeyCode::Right | KeyCode::Enter | KeyCode::PageDown => {
                let outcome = self.session.next();
                self.apply(&outcome);
            }
            KeyCode::Left | KeyCode::Backspace | KeyCode::PageUp | KeyCode::Char('p') => {
                let outcome = self.session.back();
                self.apply(&outcome);
            }
            KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::Down => self.scroll = (self.scroll + 1).min(self.max_scroll()),
            _ => {}
        }
    }

    /// Match a typed character against the options' author-declared keys
    /// (first character, case-insensitive).
    fn option_for_key(&self, c: char) -> Option<usize> {
        let bp = self.session.branch_point()?;
        bp.options.iter().position(|opt| {
            opt.key
                .as_deref()
                .and_then(|k| k.chars().next())
                .is_some_and(|k| k.eq_ignore_ascii_case(&c))
        })
    }

    /// Turn a traversal outcome into presenter feedback.
    fn apply(&mut self, outcome: &Outcome) {
        match outcome {
            Outcome::Moved => {
                self.scroll = 0;
                self.branch_selected = 0;
                self.flash = None;
                let fades = self
                    .session
                    .current()
                    .resolved_transition(self.session.defaults())
                    == Transition::Fade;
                self.fade_started = fades.then(Instant::now);
            }
            Outcome::Revealed => {
                // The current node did not change — no fade, no
                // branch-selection reset, just clear any stale flash and
                // keep newly revealed content in view.
                self.scroll = 0;
                self.flash = None;
            }
            Outcome::BlockedByBranch => {
                self.set_flash(
                    "This slide asks for a choice — ↑↓ then Enter",
                    FlashKind::Info,
                );
            }
            Outcome::EndOfPath => {
                self.set_flash("End of this path — ← goes back", FlashKind::Info);
            }
            Outcome::HistoryEmpty => {
                self.set_flash("Already at the first slide", FlashKind::Info);
            }
            Outcome::InvalidChoice => {
                self.set_flash("That choice does not exist", FlashKind::Error);
            }
            Outcome::UnknownNode(id) => {
                self.set_flash(&format!("No slide is called \"{id}\""), FlashKind::Error);
            }
        }
    }

    fn set_flash(&mut self, text: &str, kind: FlashKind) {
        self.flash = Some(Flash {
            text: text.to_owned(),
            kind,
            expires: Instant::now() + FLASH_DURATION,
        });
    }

    /// The largest useful scroll offset for the current node at the current
    /// viewport, derived from the same line flow the renderer draws.
    fn max_scroll(&self) -> u16 {
        let (w, h) = self.viewport;
        render::max_scroll(self, w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One top-level heading (editable), one top-level code block (must be
    /// skipped), and a container whose children mix a divider (skipped)
    /// with a nested text block (editable, path `[2, 1]`).
    const FIXTURE: &str = r#"{
        "fireside-version": "0.1.0",
        "title": "fixture",
        "nodes": [
            {
                "id": "only",
                "content": [
                    { "kind": "heading", "level": 2, "text": "Top heading" },
                    { "kind": "code", "language": "text", "source": "skip me" },
                    { "kind": "container", "children": [
                        { "kind": "divider" },
                        { "kind": "text", "body": "Nested text" }
                    ]}
                ]
            }
        ]
    }"#;

    #[test]
    fn editable_fields_walks_containers_and_skips_non_text_blocks() {
        let graph = Graph::from_json(FIXTURE).expect("fixture parses");
        let node = &graph.nodes[0];
        let fields = editable_fields(node);
        assert_eq!(fields.len(), 2, "one heading + one nested text");
        assert_eq!(fields[0].path.indices, vec![0]);
        assert_eq!(fields[0].kind, EditableKind::Heading(2));
        assert_eq!(fields[0].buffer, vec!["Top heading".to_owned()]);
        assert_eq!(fields[1].path.indices, vec![2, 1]);
        assert_eq!(fields[1].kind, EditableKind::Text);
        assert_eq!(fields[1].buffer, vec!["Nested text".to_owned()]);
    }
}
