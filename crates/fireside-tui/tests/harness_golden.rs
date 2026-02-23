mod harness;

use fireside_tui::Action;
use harness::AppHarness;

#[test]
fn hello_full_path_golden_ids() {
    let mut harness = AppHarness::for_hello(0);
    let mut visited = vec![harness.current_node_id()];

    for _ in 0..64 {
        let before = harness.app.session.current_node_index();
        if harness.app.session.current_node().branch_point().is_some() {
            harness.press(Action::ChooseBranch('a'));
        } else {
            harness.press(Action::NextNode);
        }

        let after = harness.app.session.current_node_index();
        if after == before {
            break;
        }
        visited.push(harness.current_node_id());
    }

    let joined = visited.join(" -> ");
    assert_eq!(
        joined,
        "title -> code-demo -> image-success -> image-fallback -> container-splits -> extension-known -> extension-unknown -> branch-demo -> themes -> thanks"
    );
}

#[test]
fn hello_branch_choose_golden() {
    let mut harness_a = AppHarness::for_hello(0);
    let mut harness_b = AppHarness::for_hello(0);

    let branch_index = harness_a
        .app
        .session
        .graph
        .index_of("branch-demo")
        .expect("branch-demo node should exist");

    harness_a.press(Action::GoToNode(branch_index));
    harness_b.press(Action::GoToNode(branch_index));

    harness_a.press(Action::ChooseBranch('a'));
    harness_b.press(Action::ChooseBranch('b'));

    assert_eq!(harness_a.current_node_id(), "themes");
    assert_eq!(harness_b.current_node_id(), "blocks");

    let rendered = harness_a.render_text();
    assert!(
        rendered.contains("themes"),
        "render should include node id/title"
    );
}
