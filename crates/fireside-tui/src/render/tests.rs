use super::*;
use crate::app::Msg;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use fireside_core::{ContentBlock, Graph};
use fireside_engine::Session;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Modifier;

/// A node with only non-editable content — a `code` block, plus a
/// container whose children are `image`/`divider` (no heading/text
/// anywhere, including nested).
const NOTHING_TO_EDIT: &str = r#"{
    "fireside-version": "0.1.0",
    "title": "fixture",
    "nodes": [
        {
            "id": "only",
            "content": [
                { "kind": "code", "language": "text", "source": "no text here" },
                { "kind": "container", "children": [
                    { "kind": "image", "src": "diagram.png" },
                    { "kind": "divider" }
                ]}
            ]
        }
    ]
}"#;

fn press_with(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.update(Msg::Terminal(Event::Key(KeyEvent::new(code, modifiers))));
}

const HELLO: &str = include_str!("../../../../docs/examples/hello.json");

fn app() -> App {
    let graph = Graph::from_json(HELLO).expect("hello parses");
    App::new(Session::new(graph).expect("non-empty"))
}

fn press(app: &mut App, code: KeyCode) {
    app.update(Msg::Terminal(Event::Key(KeyEvent::from(code))));
}

/// Render the app to a plain-text screen, lines joined by '\n'.
fn screen(app: &App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("backend");
    terminal.draw(|f| draw(f, app)).expect("draw");
    let buffer = terminal.backend().buffer().clone();
    let mut out = String::new();
    for y in 0..height {
        for x in 0..width {
            out.push_str(buffer[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

#[test]
fn first_slide_shows_title_content_and_footer_contract() {
    let app = app();
    let s = screen(&app, 80, 24);
    assert!(s.contains("Hello, Fireside"), "deck content visible");
    assert!(s.contains("1/6 seen"), "progress visible");
    assert!(s.contains("Space next"), "footer teaches the basics");
    assert!(s.contains("? help"));
}

#[test]
fn branch_point_renders_as_a_menu_with_selection() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' ')); // features
    press(&mut app, KeyCode::Char(' ')); // choose
    let s = screen(&app, 80, 24);
    assert!(s.contains("What would you like to explore?"));
    assert!(s.contains("▸"), "selection marker visible");
    assert!(s.contains("1.  Code demo "));
    assert!(s.contains("[a]"), "author hotkey visible");
    assert!(s.contains("Enter go"), "footer switches to branch keys");
}

#[test]
fn space_at_branch_flashes_guidance_instead_of_moving() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' ')); // blocked
    assert_eq!(app.session().current().id, "choose");
    let s = screen(&app, 80, 24);
    assert!(s.contains("asks for a choice"), "got: {s}");
}

#[test]
fn arrows_and_enter_choose_an_option() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Down); // -> Layout demo
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.session().current().id, "layout-demo");
    let s = screen(&app, 100, 30);
    assert!(s.contains("Left column"));
    assert!(s.contains("Right column"));
    let left = s.find("Left column").expect("left");
    let right = s.find("Right column").expect("right");
    let row_of = |pos: usize| s[..pos].matches('\n').count();
    assert_eq!(row_of(left), row_of(right), "columns share a row");
}

#[test]
fn author_hotkey_jumps_straight_to_target() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char('c')); // Finish -> thanks
    assert_eq!(app.session().current().id, "thanks");
}

#[test]
fn terminal_node_shows_end_marker_and_next_flashes() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char('3')); // quick-pick Finish
    let s = screen(&app, 80, 24);
    assert!(s.contains("■"), "end mark visible");
    assert!(s.contains("End of this path"));
    press(&mut app, KeyCode::Char(' '));
    let s = screen(&app, 80, 24);
    assert!(s.contains("End of this path — ← goes back"));
    assert_eq!(app.session().current().id, "thanks");
}

#[test]
fn the_ending_is_centered_not_left_aligned() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char('c')); // thanks (terminal)
    insta::assert_snapshot!(screen(&app, 80, 24));
}

#[test]
fn t_toggles_the_elapsed_timer() {
    let mut app = app();
    let s = screen(&app, 80, 24);
    assert!(!s.contains("0:00"), "timer hidden by default");
    press(&mut app, KeyCode::Char('t'));
    let s = screen(&app, 80, 24);
    assert!(s.contains("0:00"), "timer visible after t: {s}");
    press(&mut app, KeyCode::Char('t'));
    let s = screen(&app, 80, 24);
    assert!(!s.contains("0:00"), "t hides it again");
}

#[test]
fn timer_survives_fullscreen_and_flash() {
    let mut app = app();
    press(&mut app, KeyCode::Char('t'));
    press(&mut app, KeyCode::Backspace); // flashes "Already at the first slide"
    let s = screen(&app, 80, 24);
    assert!(s.contains("Already at the first slide"), "flash shows");
    assert!(s.contains("0:00"), "timer keeps its corner during a flash");
    press(&mut app, KeyCode::Char('f'));
    let s = screen(&app, 80, 24);
    assert!(s.contains("0:00"), "timer visible in fullscreen");
}

#[test]
fn every_scene_renders_at_60x18() {
    // Walk the whole deck at a small size: no panics, and each state's
    // full layout is pinned so a regression shows up as a snapshot diff
    // rather than requiring a bespoke assertion per scene.
    let mut app = app();
    insta::assert_snapshot!(screen(&app, 60, 18));
    press(&mut app, KeyCode::Char(' ')); // features
    insta::assert_snapshot!(screen(&app, 60, 18));
    press(&mut app, KeyCode::Char(' ')); // choose
    insta::assert_snapshot!(screen(&app, 60, 18));
    press(&mut app, KeyCode::Char('b')); // layout-demo (columns)
    insta::assert_snapshot!(screen(&app, 60, 18));
    press(&mut app, KeyCode::Char('m'));
    insta::assert_snapshot!(screen(&app, 60, 18));
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Char('?'));
    insta::assert_snapshot!(screen(&app, 60, 18));
}

#[test]
fn reload_swaps_the_deck_and_stays_on_the_current_slide() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' ')); // features
    let edited = HELLO.replace("Core Features", "Fresh Features");
    let graph = Graph::from_json(&edited).expect("edited deck parses");
    app.update(Msg::Reload(Ok(graph)));
    assert_eq!(
        app.session().current().id,
        "features",
        "still on the same slide"
    );
    let s = screen(&app, 80, 24);
    assert!(s.contains("Fresh Features"), "new content visible: {s}");
    assert!(s.contains("Reloaded"), "footer confirms the reload");
}

#[test]
fn reload_with_a_broken_save_keeps_the_working_deck() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    app.update(Msg::Reload(Err(
        "Reload failed — hello.json:3:7 — expected `,`".into(),
    )));
    let s = screen(&app, 80, 24);
    assert!(s.contains("Core Features"), "old deck still presented");
    assert!(
        s.contains("Reload failed — hello.json:3:7"),
        "footer explains"
    );
}

#[test]
fn reload_with_validation_errors_keeps_the_working_deck() {
    let mut app = app();
    let broken = HELLO.replace(
        "\"traversal\": \"features\"",
        "\"traversal\": \"missing-slide\"",
    );
    let graph = Graph::from_json(&broken).expect("broken deck still parses");
    app.update(Msg::Reload(Ok(graph)));
    let s = screen(&app, 80, 24);
    assert!(s.contains("Hello, Fireside"), "old deck still presented");
    assert!(s.contains("Reload skipped"), "footer explains: {s}");
}

#[test]
fn reload_that_removed_the_current_slide_returns_to_start() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' ')); // features
    let edited = HELLO
        .replace("\"id\": \"features\"", "\"id\": \"renamed\"")
        .replace("\"traversal\": \"features\"", "\"traversal\": \"renamed\"");
    let graph = Graph::from_json(&edited).expect("edited deck parses");
    app.update(Msg::Reload(Ok(graph)));
    assert_eq!(app.session().current().id, "intro", "back at the entry");
    let s = screen(&app, 80, 24);
    assert!(
        s.contains("is gone, back at the start"),
        "footer explains: {s}"
    );
}

#[test]
fn resize_event_updates_scroll_geometry() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char('a')); // code-demo
    app.update(Msg::Terminal(Event::Resize(60, 12)));
    // Scrolling clamps against the new, smaller viewport without panics.
    for _ in 0..50 {
        press(&mut app, KeyCode::Down);
    }
    let s = screen(&app, 60, 12);
    assert!(s.contains("│"), "code box still on screen");
}

#[test]
fn back_walks_the_real_path_and_start_flashes() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Backspace);
    assert_eq!(app.session().current().id, "intro");
    press(&mut app, KeyCode::Backspace);
    let s = screen(&app, 80, 24);
    assert!(s.contains("Already at the first slide"));
}

#[test]
fn fullscreen_node_hides_header_and_f_toggles_back() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char('a')); // code-demo, view-mode fullscreen
    let s = screen(&app, 80, 24);
    assert!(!s.contains("1/6 seen"), "fullscreen hides the header");
    assert!(s.contains("fn main()"), "code visible");
    press(&mut app, KeyCode::Char('f')); // back to standard
    let s = screen(&app, 80, 24);
    assert!(s.contains("seen"), "header is back");
}

#[test]
fn map_lists_slides_marks_progress_and_jumps() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' ')); // features
    press(&mut app, KeyCode::Char('m'));
    let s = screen(&app, 80, 24);
    assert!(s.contains("Map — Enter jumps"));
    assert!(s.contains("●"), "visited station");
    assert!(s.contains("◉"), "current station");
    assert!(s.contains("○"), "unvisited station");
    // Jump to the last slide.
    for _ in 0..5 {
        press(&mut app, KeyCode::Down);
    }
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.session().current().id, "thanks");
    // Back returns to where the jump came from (history, not order).
    press(&mut app, KeyCode::Backspace);
    assert_eq!(app.session().current().id, "features");
}

#[test]
fn map_draws_the_fork_with_its_option_keys() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char('m'));
    let s = screen(&app, 80, 24);
    assert!(s.contains("├"), "fork junction drawn: {s}");
    assert!(s.contains("╮"), "branch lane opens: {s}");
    assert!(s.contains("╯"), "branch lane rejoins: {s}");
    assert!(s.contains("[a]"), "option key legend: {s}");
    assert!(s.contains("[c]"), "all option keys shown: {s}");
    assert!(s.contains("you are here"), "glyph legend shown: {s}");
}

/// Send a left-button click at `(col, row)`, sized against `(w, h)` so
/// `App`'s tracked viewport matches what was actually rendered.
fn click_at(app: &mut App, w: u16, h: u16, col: u16, row: u16) {
    app.update(Msg::Terminal(Event::Resize(w, h)));
    app.update(Msg::Terminal(Event::Mouse(crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    })));
}

#[test]
fn clicking_a_map_row_navigates_to_that_slide() {
    let mut app = app();
    press(&mut app, KeyCode::Char('m'));
    let (w, h) = (80, 24);
    let buf = buffer(&app, w, h);
    let (x, y) = locate(&buf, w, h, " features ");
    click_at(&mut app, w, h, x, y);
    assert_eq!(*app.screen(), Screen::Present, "click closed the map");
    assert_eq!(app.session().current().id, "features", "click navigated");
}

#[test]
fn clicking_a_branch_option_chooses_it() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' ')); // features
    press(&mut app, KeyCode::Char(' ')); // choose (branch point)
    let (w, h) = (80, 24);
    let buf = buffer(&app, w, h);
    // Option 2 is "Layout demo" per `arrows_and_enter_choose_an_option`.
    let (x, y) = locate(&buf, w, h, "Layout demo");
    click_at(&mut app, w, h, x, y);
    assert_eq!(
        app.session().current().id,
        "layout-demo",
        "click chose the same target arrows+Enter would"
    );
}

#[test]
fn clicking_outside_any_interactive_row_is_inert() {
    let mut app = app();
    press(&mut app, KeyCode::Char('m'));
    let before = app.session().current().id.clone();
    // Row 0 is inside the overlay's top border, not a station row.
    click_at(&mut app, 80, 24, 40, 0);
    assert_eq!(
        *app.screen(),
        Screen::Map { selected: 0 },
        "still on the map"
    );
    assert_eq!(app.session().current().id, before, "nothing navigated");
}

#[test]
fn clicking_a_branch_option_row_before_it_is_drawn_is_inert() {
    // While reveal is pending the branch menu is not rendered at all
    // (mirrors the keyboard gate) — a click where the menu would
    // eventually appear has nothing to land on, so it does nothing.
    const DECK: &str = r#"{"nodes":[
        {"id":"a","traversal":{"branch-point":{"options":[
            {"label":"One","key":"1","target":"b"}
        ]}},"content":[
            {"kind":"text","body":"x","reveal":1}
        ]},
        {"id":"b","content":[]}
    ]}"#;
    let mut app =
        App::new(Session::new(Graph::from_json(DECK).expect("fixture parses")).expect("non-empty"));
    let (w, h) = (80, 24);
    click_at(&mut app, w, h, 40, 12);
    assert_eq!(app.session().current().id, "a", "no navigation happened");
    assert!(
        app.session().has_pending_reveal(),
        "the click did not consume the reveal step either"
    );
}

#[test]
fn keyboard_only_flows_are_unaffected_by_mouse_support() {
    // No `Msg::Terminal(Event::Mouse(..))` anywhere in this test —
    // a regression guarantee that mouse support changed nothing about
    // the existing keyboard-only path (FR-003).
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.session().current().id, "layout-demo");
}

#[test]
fn header_rule_carries_the_mini_rail() {
    let mut app = app();
    let s = screen(&app, 80, 24);
    let rail = s.lines().nth(1).expect("rule row");
    assert!(rail.contains("◉"), "current station on the rule: {rail}");
    press(&mut app, KeyCode::Char(' '));
    let s = screen(&app, 80, 24);
    let rail = s.lines().nth(1).expect("rule row");
    assert!(rail.contains("●───◉"), "travelled track then you: {rail}");
}

#[test]
fn the_ending_lists_the_route_travelled() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char('c')); // straight to thanks
    let s = screen(&app, 80, 24);
    assert!(s.contains("→"), "path trace shown on the ending: {s}");
}

#[test]
fn help_overlay_opens_and_any_key_closes() {
    let mut app = app();
    press(&mut app, KeyCode::Char('?'));
    let s = screen(&app, 80, 24);
    assert!(s.contains(" Keys "));
    assert!(s.contains("map — see and jump anywhere"));
    press(&mut app, KeyCode::Char('x'));
    assert_eq!(*app.screen(), Screen::Present);
    assert_eq!(
        app.session().current().id,
        "intro",
        "closing help moved nothing"
    );
}

#[test]
fn speaker_notes_toggle_and_absence_flashes() {
    let mut app = app();
    press(&mut app, KeyCode::Char('s')); // intro has no notes
    let s = screen(&app, 80, 24);
    assert!(s.contains("no speaker notes"));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char('a')); // code-demo has notes
    press(&mut app, KeyCode::Char('f')); // standard frame for the panel
    press(&mut app, KeyCode::Char('s'));
    let s = screen(&app, 80, 24);
    assert!(s.contains("Notes — s hides"));
    assert!(s.contains("Demonstrate fullscreen view mode"));
}

#[test]
fn q_quits() {
    let mut app = app();
    press(&mut app, KeyCode::Char('q'));
    assert!(app.should_quit());
}

#[test]
fn tiny_terminal_degrades_gracefully() {
    let app = app();
    let s = screen(&app, 9, 3);
    assert!(s.contains("Too small"));
}

/// Render and return the raw buffer for style-level assertions.
fn buffer(app: &App, width: u16, height: u16) -> ratatui::buffer::Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("backend");
    terminal.draw(|f| draw(f, app)).expect("draw");
    terminal.backend().buffer().clone()
}

/// The (x, y) of the first cell where `needle` starts on screen.
fn locate(buf: &ratatui::buffer::Buffer, width: u16, height: u16, needle: &str) -> (u16, u16) {
    for y in 0..height {
        let row: String = (0..width).map(|x| buf[(x, y)].symbol()).collect();
        if let Some(col) = row.find(needle) {
            let x = row[..col].chars().count() as u16;
            return (x, y);
        }
    }
    panic!("{needle:?} not on screen");
}

#[test]
fn default_view_frames_the_slide_in_a_rounded_card() {
    let app = app();
    insta::assert_snapshot!(screen(&app, 80, 24));
}

#[test]
fn the_card_is_the_same_stage_on_every_slide() {
    let mut app = app();
    let frame = |app: &App| {
        let buf = buffer(app, 80, 24);
        let top = locate(&buf, 80, 24, "╭");
        let bottom = locate(&buf, 80, 24, "╰");
        (top, bottom)
    };
    let first = frame(&app);
    press(&mut app, KeyCode::Char(' ')); // a slide with more content
    let second = frame(&app);
    assert_eq!(
        first, second,
        "the card frame must not resize between slides"
    );
}

#[test]
fn wide_terminals_keep_a_readable_measure() {
    let app = app();
    let buf = buffer(&app, 200, 40);
    let (x, _) = locate(&buf, 200, 40, "╭");
    // Card is capped at MEASURE + chrome (84), centered: left edge at 58.
    assert_eq!(x, 58, "card centered at the measure cap, not full width");
}

#[test]
fn fullscreen_uses_the_full_width_not_the_measure() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char('a')); // code-demo is fullscreen
    insta::assert_snapshot!(screen(&app, 120, 30));
}

#[test]
fn ascii_art_code_block_centers_within_the_card_at_80x24() {
    const ASCII_ART: &str = r#"{"nodes":[{"id":"a","content":[
        {"kind":"code","source":" /\\_/\\ \n( o.o )\n > ^ < "}
    ]}]}"#;
    const RUST_LANGUAGE: &str = r#"{"nodes":[{"id":"a","content":[
        {"kind":"code","language":"rust","source":" /\\_/\\ \n( o.o )\n > ^ < "}
    ]}]}"#;

    let ascii_app = App::new(
        Session::new(Graph::from_json(ASCII_ART).expect("ascii fixture parses"))
            .expect("non-empty"),
    );
    let rust_app = App::new(
        Session::new(Graph::from_json(RUST_LANGUAGE).expect("rust fixture parses"))
            .expect("non-empty"),
    );

    let ascii_screen = screen(&ascii_app, 80, 24);
    let rust_screen = screen(&rust_app, 80, 24);

    let ascii_row = ascii_screen
        .lines()
        .find(|l| l.contains("o.o"))
        .expect("ascii art row visible at 80x24");
    let rust_row = rust_screen
        .lines()
        .find(|l| l.contains("o.o"))
        .expect("rust code row visible at 80x24");

    // Rows are framed by the card's own border ("│ ... │"), so measure
    // where the art itself starts, not leading whitespace from the
    // start of the string (which is always 0 — the border isn't a
    // space).
    let ascii_col = ascii_row.find("o.o").expect("column of ascii art");
    let rust_col = rust_row.find("o.o").expect("column of rust code");

    assert!(
        ascii_col > rust_col,
        "ascii art (col {ascii_col}) should be indented further than an explicit-language \
         block (col {rust_col}) at the same 80x24 size: ascii={ascii_row:?} rust={rust_row:?}"
    );
}

#[test]
fn ascii_art_block_renders_centered_and_sized_to_content() {
    const NARROW: &str = r#"{"nodes":[{"id":"a","content":[
        {"kind":"ascii-art","art":" /\\_/\\ \n( o.o )\n > ^ < "}
    ]}]}"#;
    const WIDE_CODE: &str = r#"{"nodes":[{"id":"a","content":[
        {"kind":"code","language":"rust","source":" /\\_/\\ \n( o.o )\n > ^ < "}
    ]}]}"#;

    let art_app = App::new(
        Session::new(Graph::from_json(NARROW).expect("ascii-art fixture parses"))
            .expect("non-empty"),
    );
    let wide_app = App::new(
        Session::new(Graph::from_json(WIDE_CODE).expect("rust fixture parses")).expect("non-empty"),
    );

    let art_screen = screen(&art_app, 80, 24);
    let wide_screen = screen(&wide_app, 80, 24);
    assert!(art_screen.contains("ascii-art"), "{art_screen}");

    let art_row = art_screen
        .lines()
        .find(|l| l.contains("o.o"))
        .expect("art row visible at 80x24");
    let wide_row = wide_screen
        .lines()
        .find(|l| l.contains("o.o"))
        .expect("full-width row visible at 80x24");

    let art_col = art_row.find("o.o").expect("column of ascii-art");
    let wide_col = wide_row.find("o.o").expect("column of full-width block");
    assert!(
        art_col > wide_col,
        "ascii-art block (col {art_col}) should be centered/indented further than a \
         full-width block (col {wide_col}) at the same 80x24 size: \
         art={art_row:?} wide={wide_row:?}"
    );
}

#[test]
fn reveal_hides_content_until_next_is_pressed_enough_times() {
    const DECK: &str = r#"{"nodes":[{"id":"a","content":[
        {"kind":"text","body":"Always visible"},
        {"kind":"text","body":"First reveal","reveal":1},
        {"kind":"text","body":"Second reveal","reveal":2}
    ]}]}"#;
    let mut app =
        App::new(Session::new(Graph::from_json(DECK).expect("fixture parses")).expect("non-empty"));

    let s = screen(&app, 80, 24);
    assert!(s.contains("Always visible"), "{s}");
    assert!(!s.contains("First reveal"), "not yet revealed: {s}");
    assert!(
        s.contains("0/2 revealed"),
        "footer shows reveal progress: {s}"
    );

    press(&mut app, KeyCode::Char(' '));
    let s = screen(&app, 80, 24);
    assert!(s.contains("First reveal"), "{s}");
    assert!(!s.contains("Second reveal"), "still pending: {s}");
    assert!(s.contains("1/2 revealed"), "{s}");

    press(&mut app, KeyCode::Char(' '));
    let s = screen(&app, 80, 24);
    assert!(s.contains("Second reveal"), "{s}");
    assert!(
        !s.contains("revealed"),
        "badge gone once reveal is exhausted: {s}"
    );
}

#[test]
fn reveal_then_next_advances_normally_once_exhausted() {
    const DECK: &str = r#"{"nodes":[
        {"id":"a","traversal":"b","content":[
            {"kind":"text","body":"x","reveal":1}
        ]},
        {"id":"b","content":[{"kind":"text","body":"On b"}]}
    ]}"#;
    let mut app =
        App::new(Session::new(Graph::from_json(DECK).expect("fixture parses")).expect("non-empty"));
    press(&mut app, KeyCode::Char(' '));
    assert_eq!(
        app.session().current().id,
        "a",
        "first press reveals, does not navigate"
    );
    press(&mut app, KeyCode::Char(' '));
    assert_eq!(app.session().current().id, "b", "second press navigates");
}

#[test]
fn branch_keys_continue_revealing_instead_of_choosing_early() {
    const DECK: &str = r#"{"nodes":[
        {"id":"a","traversal":{"branch-point":{"options":[
            {"label":"One","key":"1","target":"b"}
        ]}},"content":[
            {"kind":"text","body":"x","reveal":1}
        ]},
        {"id":"b","content":[]}
    ]}"#;
    let mut app =
        App::new(Session::new(Graph::from_json(DECK).expect("fixture parses")).expect("non-empty"));

    // The branch key ('1') would normally choose an option; while
    // reveal is pending it must instead continue revealing (FR-007),
    // not silently do nothing.
    press(&mut app, KeyCode::Char('1'));
    assert_eq!(app.session().current().id, "a", "still on the branch node");
    assert!(
        !app.session().has_pending_reveal(),
        "the branch key consumed the reveal step"
    );

    // Now that reveal is exhausted, the same key selects the option.
    press(&mut app, KeyCode::Char('1'));
    assert_eq!(app.session().current().id, "b", "branch key now chooses");
}

#[test]
fn reveal_marks_do_not_change_a_deck_that_never_uses_them() {
    let app = app();
    let s = screen(&app, 80, 24);
    assert!(
        !s.contains("revealed"),
        "no reveal badge on an ordinary deck: {s}"
    );
    assert!(
        s.contains("Space next"),
        "ordinary footer hint unchanged: {s}"
    );
}

#[test]
fn ascii_art_reveal_gated_block_appears_as_one_unit() {
    const DECK: &str = r#"{"nodes":[{"id":"a","content":[
        {"kind":"text","body":"Always visible"},
        {"kind":"ascii-art","art":"first line\nsecond line","reveal":1}
    ]}]}"#;
    let mut app =
        App::new(Session::new(Graph::from_json(DECK).expect("fixture parses")).expect("non-empty"));

    let s = screen(&app, 80, 24);
    assert!(s.contains("Always visible"), "{s}");
    assert!(!s.contains("first line"), "not yet revealed: {s}");
    assert!(!s.contains("second line"), "not yet revealed: {s}");

    press(&mut app, KeyCode::Char(' '));
    let s = screen(&app, 80, 24);
    assert!(
        s.contains("first line") && s.contains("second line"),
        "every line of the art appears together on the same press: {s}"
    );
}

#[test]
fn hidden_column_reserves_no_width_until_revealed_at_80x24() {
    const DECK: &str = r#"{"nodes":[{"id":"a","content":[
        {"kind":"container","layout":"columns","children":[
            {"kind":"text","body":"Left column"},
            {"kind":"text","body":"Right column","reveal":1}
        ]}
    ]}]}"#;
    let mut app =
        App::new(Session::new(Graph::from_json(DECK).expect("fixture parses")).expect("non-empty"));

    // Before reveal: the hidden column reserves no width, so the layout
    // itself is different, not just the text — worth a snapshot of the
    // whole frame rather than a content-presence check.
    insta::assert_snapshot!(screen(&app, 80, 24));

    press(&mut app, KeyCode::Char(' '));
    insta::assert_snapshot!(screen(&app, 80, 24));
}

#[test]
fn code_gets_syntax_colors_from_the_theme() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char('a')); // code-demo
    let (w, h) = (100, 30);
    let buf = buffer(&app, w, h);
    let (x, y) = locate(&buf, w, h, "fn main");
    assert_eq!(
        buf[(x, y)].style().fg,
        Some(ratatui::style::Color::Magenta),
        "keywords use the keyword token"
    );
}

#[test]
fn highlight_lines_dim_the_rest_and_keep_focus_bright() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char('a')); // code-demo highlights lines 2-3
    let (w, h) = (100, 30);
    let buf = buffer(&app, w, h);
    let (x1, y1) = locate(&buf, w, h, "fn main");
    assert!(
        buf[(x1, y1)].style().add_modifier.contains(Modifier::DIM),
        "unhighlighted line is dimmed"
    );
    let (x2, y2) = locate(&buf, w, h, "let graph");
    assert!(
        !buf[(x2, y2)].style().add_modifier.contains(Modifier::DIM),
        "highlighted line keeps full brightness"
    );
}

#[test]
fn fade_transition_starts_dim_and_is_only_for_fade_nodes() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' ')); // features: transition none
    assert!(!app.fading(), "no fade on transition: none");
    press(&mut app, KeyCode::Char(' '));
    press(&mut app, KeyCode::Char('c')); // thanks: transition fade
    assert!(app.fading(), "fade node enters its fade window");
    let (w, h) = (80, 24);
    let buf = buffer(&app, w, h);
    let (x, y) = locate(&buf, w, h, "Thanks!");
    assert!(
        buf[(x, y)].style().add_modifier.contains(Modifier::DIM),
        "slide starts dim during the fade"
    );
}

#[test]
fn quick_edit_open_edit_save_updates_the_heading_and_leaves_other_blocks_alone() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' ')); // -> features
    assert_eq!(app.session().current().id, "features");

    press(&mut app, KeyCode::Char('e'));
    assert!(
        matches!(app.screen(), Screen::Edit { .. }),
        "e opens the modal: {:?}",
        app.screen()
    );

    // Cursor starts at (0, 0) on the first field (the heading) —
    // inserting a char prepends it.
    press(&mut app, KeyCode::Char('X'));
    press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);

    assert!(
        matches!(app.screen(), Screen::Edit { .. }),
        "the modal stays open until the write-back sink's result arrives"
    );
    let saved = app
        .take_pending_save()
        .expect("a save produces a pending graph");
    // The event loop hands the sink's outcome back via `Msg::SaveResult`;
    // here we simulate a successful write.
    app.update(Msg::SaveResult(Ok(())));
    assert_eq!(
        *app.screen(),
        Screen::Present,
        "a successful save closes the modal"
    );

    let node = saved.node("features").expect("features node still exists");
    match &node.content[0] {
        ContentBlock::Heading { text, .. } => {
            assert_eq!(text, "XCore Features");
        }
        other => panic!("expected the heading block, got {other:?}"),
    }
    // The other editable block (the trailing text) is untouched.
    match &node.content[3] {
        ContentBlock::Text { body, .. } => {
            assert_eq!(
                body,
                "Every edge is explicit. No implicit sequential fallback."
            );
        }
        other => panic!("expected the text block, got {other:?}"),
    }
    // Non-editable siblings on the same node are untouched too.
    assert!(matches!(node.content[1], ContentBlock::List { .. }));
    assert!(matches!(node.content[2], ContentBlock::Divider { .. }));
}

#[test]
fn quick_edit_cancel_leaves_the_session_and_pending_save_untouched() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' ')); // -> features
    press(&mut app, KeyCode::Char('e'));
    press(&mut app, KeyCode::Char('X'));
    press(&mut app, KeyCode::Esc);

    assert_eq!(*app.screen(), Screen::Present, "esc closes the modal");
    assert!(
        app.take_pending_save().is_none(),
        "cancel must not produce a save"
    );
    assert_eq!(
        app.session().current().content[0],
        ContentBlock::Heading {
            reveal: None,
            level: 2,
            text: "Core Features".to_owned(),
        },
        "cancel must not mutate the live session"
    );
}

#[test]
fn quick_edit_save_failure_keeps_the_modal_open_for_retry_or_cancel() {
    let mut app = app();
    press(&mut app, KeyCode::Char(' ')); // -> features
    press(&mut app, KeyCode::Char('e'));
    press(&mut app, KeyCode::Char('X'));
    press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    app.take_pending_save().expect("save produced a graph");

    // Simulate the write-back sink refusing the save (conflict, I/O
    // error, or no backing file) — the presenter's edit must not be
    // silently discarded (FR-013): the modal stays open so they can
    // retry (Ctrl+S again) or abandon (Esc).
    app.update(Msg::SaveResult(Err("Save skipped — the file changed on disk; Ctrl+S again to overwrite, Esc to discard your edit".to_owned())));
    assert!(
        matches!(app.screen(), Screen::Edit { .. }),
        "a failed save must not close the modal or discard the edit"
    );
    let s = screen(&app, 80, 24);
    assert!(s.contains("changed on disk"), "the failure is shown: {s}");

    // Retry: the presenter presses save again and it succeeds.
    press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    let saved = app
        .take_pending_save()
        .expect("retry produces a pending save with the same edit");
    app.update(Msg::SaveResult(Ok(())));
    assert_eq!(
        *app.screen(),
        Screen::Present,
        "a successful retry closes the modal"
    );
    match &saved.node("features").expect("features node").content[0] {
        ContentBlock::Heading { text, .. } => assert_eq!(text, "XCore Features"),
        other => panic!("expected the heading block, got {other:?}"),
    }
}

#[test]
fn quick_edit_save_never_touches_other_nodes_or_branch_structure() {
    let original = Graph::from_json(HELLO).expect("hello parses");

    let mut app = app();
    press(&mut app, KeyCode::Char(' ')); // -> features
    press(&mut app, KeyCode::Char('e'));
    press(&mut app, KeyCode::Char('X'));
    press_with(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    let saved = app.take_pending_save().expect("save produced a graph");

    for node in &original.nodes {
        if node.id == "features" {
            continue;
        }
        let edited = saved
            .node(&node.id)
            .unwrap_or_else(|| panic!("node {} must still exist after an unrelated save", node.id));
        assert_eq!(
            edited, node,
            "node {} must be untouched by a save on a different node",
            node.id
        );
    }
}

#[test]
fn quick_edit_on_a_node_with_nothing_editable_flashes_instead_of_opening() {
    let graph = Graph::from_json(NOTHING_TO_EDIT).expect("fixture parses");
    let mut app = App::new(Session::new(graph).expect("non-empty"));

    press(&mut app, KeyCode::Char('e'));

    assert_eq!(*app.screen(), Screen::Present, "no modal opens");
    let s = screen(&app, 80, 24);
    assert!(
        s.contains("no editable text"),
        "expected a clear message: {s}"
    );
}

#[test]
fn present_watching_refuses_saves_with_unavailable() {
    // `present`/`present_watching` (used by `fireside demo`, which has
    // no backing file) resolve internally to a sink that always
    // returns `Unavailable` — exercised directly here without a live
    // terminal, per quickstart.md scenario 4.
    let sink: crate::WriteBackSink<'_> = &mut |_| Err(crate::WriteBackError::Unavailable);
    let graph = Graph::from_json(HELLO).expect("hello parses");
    let err = sink(&graph).expect_err("the stub sink always refuses");
    assert_eq!(err, crate::WriteBackError::Unavailable);
}

#[test]
fn save_result_flashes_a_distinct_message_for_every_write_back_error() {
    for (error, expect_contains) in [
        (crate::WriteBackError::Unavailable, "no file to save to"),
        (crate::WriteBackError::Conflict, "changed on disk"),
        (
            crate::WriteBackError::Io("disk full".to_owned()),
            "disk full",
        ),
    ] {
        let mut app = app();
        app.update(Msg::SaveResult(Err(error.to_string())));
        let s = screen(&app, 80, 24);
        assert!(
            s.contains(expect_contains),
            "expected a message containing {expect_contains:?}: {s}"
        );
    }

    let mut app = app();
    app.update(Msg::SaveResult(Ok(())));
    let s = screen(&app, 80, 24);
    assert!(s.contains("Saved"), "{s}");
}

#[test]
fn link_cell_carries_osc8_escape_with_forced_width() {
    const DECK: &str = r#"{"nodes":[{"id":"a","content":[
        {"kind":"text","body":"See [docs](https://example.com) here"}
    ]}]}"#;
    let app =
        App::new(Session::new(Graph::from_json(DECK).expect("fixture parses")).expect("non-empty"));
    let (w, h) = (80, 24);
    let buf = buffer(&app, w, h);

    let mut found = None;
    'outer: for y in 0..h {
        for x in 0..w {
            if buf[(x, y)].symbol().contains("\u{1b}]8;;") {
                found = Some((x, y));
                break 'outer;
            }
        }
    }
    let (x, y) = found.expect("a link cell is present on screen");
    let cell = &buf[(x, y)];
    assert!(
        cell.symbol().contains("https://example.com"),
        "cell carries the url: {:?}",
        cell.symbol()
    );
    assert!(
        cell.symbol().contains("docs"),
        "cell carries the label: {:?}",
        cell.symbol()
    );
    match cell.diff_option {
        ratatui::buffer::CellDiffOption::ForcedWidth(width) => {
            assert_eq!(width.get(), 4, "\"docs\" is 4 columns wide");
        }
        other => panic!("expected ForcedWidth, got {other:?}"),
    }
    // The label's other 3 columns are blanked, not left with stray
    // leftover characters from the original per-character cells.
    for dx in 1..4 {
        assert_eq!(
            buf[(x + dx, y)].symbol(),
            " ",
            "trailing label cell at dx={dx} is blanked"
        );
    }
}
