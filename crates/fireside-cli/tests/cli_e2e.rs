//! End-to-end tests for the three CLI verbs.

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn fireside() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("fireside"))
}

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

#[test]
fn bare_invocation_teaches_the_three_verbs() {
    fireside()
        .assert()
        .success()
        .stdout(predicate::str::contains("fireside <file>"))
        .stdout(predicate::str::contains("validate"))
        .stdout(predicate::str::contains("new"));
}

#[test]
fn validate_hello_exits_zero() {
    fireside()
        .arg("validate")
        .arg(repo_root().join("docs/examples/hello.json"))
        .assert()
        .success()
        .stdout(predicate::str::contains("0 error(s)"));
}

#[test]
fn validate_missing_file_fails_with_readable_error() {
    fireside()
        .arg("validate")
        .arg("nonexistent.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not read"));
}

#[test]
fn validate_reports_dangling_targets_in_plain_language() {
    let temp = tempfile::tempdir().expect("temp dir");
    let deck = temp.path().join("broken.json");
    std::fs::write(
        &deck,
        r#"{"nodes":[{"id":"a","traversal":"ghost","content":[]}]}"#,
    )
    .expect("write fixture");

    fireside()
        .arg("validate")
        .arg(&deck)
        .assert()
        .failure()
        .stdout(predicate::str::contains("no node has that id"));
}

#[test]
fn present_refuses_a_broken_deck_before_taking_the_screen() {
    let temp = tempfile::tempdir().expect("temp dir");
    let deck = temp.path().join("broken.json");
    std::fs::write(
        &deck,
        r#"{"nodes":[{"id":"a","traversal":"ghost","content":[]}]}"#,
    )
    .expect("write fixture");

    fireside()
        .arg(&deck)
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be presented yet"));
}

#[test]
fn new_scaffolds_a_deck_that_validates_clean() {
    let temp = tempfile::tempdir().expect("temp dir");

    fireside()
        .current_dir(temp.path())
        .arg("new")
        .arg("Test Talk")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created test-talk.fireside.json"));

    let file = temp.path().join("test-talk.fireside.json");
    let contents = std::fs::read_to_string(&file).expect("scaffold is readable");
    let parsed: serde_json::Value = serde_json::from_str(&contents).expect("scaffold is JSON");
    assert!(parsed.get("nodes").is_some(), "expected nodes field");

    fireside()
        .arg("validate")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("0 error(s)"));
}

/// Kills and reaps the wrapped child even if an assertion panics, so a
/// failing test never leaves a `--watch` process running on the machine.
struct KillOnDrop(std::process::Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn validate_watch_prints_the_first_result_immediately() {
    let temp = tempfile::tempdir().expect("temp dir");
    let deck = temp.path().join("deck.json");
    std::fs::write(&deck, r#"{"nodes":[{"id":"a","content":[]}]}"#).expect("write fixture");

    let child = std::process::Command::new(assert_cmd::cargo::cargo_bin!("fireside"))
        .arg("validate")
        .arg("--watch")
        .arg(&deck)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn fireside validate --watch");
    let mut guard = KillOnDrop(child);

    let mut stdout = std::io::BufReader::new(guard.0.stdout.take().expect("piped stdout"));
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::io::BufRead;
        let mut first_line = String::new();
        let _ = stdout.read_line(&mut first_line);
        let _ = tx.send(first_line);
    });

    let first_line = rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("--watch prints its first result within 5s");
    assert!(
        first_line.contains("no problems found"),
        "expected the immediate first-check result: {first_line:?}"
    );
}

#[test]
fn new_refuses_to_overwrite() {
    let temp = tempfile::tempdir().expect("temp dir");
    fireside()
        .current_dir(temp.path())
        .arg("new")
        .arg("twice")
        .assert()
        .success();
    fireside()
        .current_dir(temp.path())
        .arg("new")
        .arg("twice")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}
